use crossbeam::channel;
use phs::{wrap_async_fn, Repositories};
use std::{collections::HashMap, fmt::Display};
use tokio::sync::oneshot::{self, Receiver};
use valu3::{prelude::*, value::Value};

pub enum Error {
    VersionNotFound(String),
    ModuleLoaderError(String),
    ModuleNotFound(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VersionNotFound(module) => write!(f, "Version not found for module: {}", module),
            Error::ModuleLoaderError(err) => write!(f, "Module loader error: {}", err),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VersionNotFound(module) => write!(f, "Version not found for module: {}", module),
            Error::ModuleLoaderError(err) => write!(f, "Module loader error: {}", err),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
        }
    }
}

#[derive(ToValue, FromValue, Clone, Debug)]
pub struct ModuleData {
    pub version: String,
    pub repository: Option<String>,
    pub repository_path: Option<String>,
    pub repository_raw_content: Option<String>,
    pub module: String,
    pub name: String,
    pub with: Value,
    pub input: Value,
    pub output: Value,
    pub input_order: Value,
}

impl TryFrom<Value> for ModuleData {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Error> {
        let module = match value.get("module") {
            Some(module) => module.to_string(),
            None => return Err(Error::ModuleLoaderError("Module not found".to_string())),
        };
        let repository = value.get("repository").map(|v| v.to_string());

        let repository_path = if repository.is_none() {
            let mut padded = module.to_string();
            while padded.len() < 4 {
                padded.push('_');
            }

            let prefix = &padded[0..2];
            let middle = &padded[2..4];

            let repository = format!("{}/{}/{}", prefix, middle, module);
            Some(repository)
        } else {
            None
        };

        let repository_raw_content = value.get("repository_raw_content").map(|v| v.to_string());

        let version = match value.get("version") {
            Some(version) => version.to_string(),
            None => return Err(Error::VersionNotFound(module.clone())),
        };

        let name = match value.get("name") {
            Some(name) => name.to_string(),
            None => module.clone(),
        };

        let with = match value.get("with") {
            Some(with) => with.clone(),
            None => Value::Null,
        };

        let (input, output, input_order) = if let Some(info) = value.get("info") {
            let input = match info.get("input") {
                Some(input) => {
                    if let Value::Object(obj) = input {
                        if let Some(obj_type) = obj.get("type") {
                            if obj_type.to_string() == "object" {
                                obj.get("properties").unwrap_or(&Value::Null).clone()
                            } else {
                                input.clone()
                            }
                        } else {
                            input.clone()
                        }
                    } else {
                        input.clone()
                    }
                }
                None => Value::Null,
            };

            let output = match info.get("output") {
                Some(output) => output.clone(),
                None => Value::Null,
            };

            let input_order = match info.get("input_order") {
                Some(input_order) => input_order.clone(),
                None => Value::Null,
            };

            (input, output, input_order)
        } else {
            (Value::Null, Value::Null, Value::Null)
        };

        Ok(ModuleData {
            module,
            repository,
            version,
            name,
            with,
            input,
            output,
            repository_path,
            repository_raw_content,
            input_order,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ModulesError {
    ModuleNotFound(String),
    ModuleNotLoaded(String),
    ModuleError(String),
}

#[derive(Debug, Clone)]
pub struct ModuleResponse {
    pub error: Option<String>,
    pub data: Value,
}

impl Into<ModuleResponse> for Value {
    fn into(self) -> ModuleResponse {
        ModuleResponse {
            error: None,
            data: self,
        }
    }
}

impl ModuleResponse {
    pub fn from_error(error: String) -> Self {
        Self {
            error: Some(error),
            data: Value::Null,
        }
    }

    pub fn from_success(value: Value) -> Self {
        Self {
            error: None,
            data: value,
        }
    }
}

#[derive(Debug)]
pub struct ModulePackage {
    pub input: Option<Value>,
    pub sender: oneshot::Sender<ModuleResponse>,
}

impl ModulePackage {
    pub fn input(&self) -> Option<Value> {
        self.input.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModuleParams {
    pub with: Value,
    pub input: Value,
    pub output: Value,
    pub input_order: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub sender: channel::Sender<ModulePackage>,
    pub params: ModuleParams,
}

impl Module {
    pub fn send(&self, input: Option<Value>) -> Receiver<ModuleResponse> {
        let (package_sender, package_receiver) = oneshot::channel();
        let package = ModulePackage {
            input,
            sender: package_sender,
        };

        let _ = self.sender.send(package);

        package_receiver
    }
}

#[derive(Debug, Default, Clone)]
pub struct Modules {
    pub modules: HashMap<String, Module>,
}

impl Modules {
    pub fn extract(&self) -> Self {
        Self {
            modules: self.modules.clone(),
        }
    }

    pub fn register(&mut self, module_data: ModuleData, sender: channel::Sender<ModulePackage>) {
        let input_order = if let Value::Array(arr) = module_data.input_order {
            arr.into_iter().map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        };

        let module = Module {
            sender,
            params: ModuleParams {
                with: module_data.with,
                input: module_data.input,
                output: module_data.output,
                input_order,
            },
        };

        self.modules.insert(module_data.name.to_string(), module);
    }

    pub async fn execute(
        &self,
        name: &str,
        input: &Option<Value>,
    ) -> Result<ModuleResponse, ModulesError> {
        if let Some(module) = self.modules.get(name) {
            let package_receiver = module.send(input.clone());

            let value = package_receiver.await.unwrap_or(ModuleResponse::from_error(
                "Module response channel closed".to_string(),
            ));

            Ok(value)
        } else {
            Err(ModulesError::ModuleNotLoaded(name.to_string()))
        }
    }

    pub fn extract_repositories(&self) -> Repositories {
        let mut repositories = HashMap::new();

        for (name, module) in self.modules.clone() {
            let args = module.params.input_order.clone();
            let func = move |value: Value| {
                let package_receiver = module.send(Some(value));

                async move {
                    let result = package_receiver.await.unwrap_or(ModuleResponse::from_error(
                        "Module response channel closed".to_string(),
                    ));

                    if let Some(error) = result.error {
                        Value::from(format!("Error: {}", error))
                    } else {
                        result.data
                    }
                }
            };

            let repository_function = wrap_async_fn(name.clone(), func, args);

            repositories.insert(name, repository_function);
        }

        Repositories { repositories }
    }
}
