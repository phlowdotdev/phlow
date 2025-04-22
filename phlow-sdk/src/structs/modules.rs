use crossbeam::channel;
use phs::{repositories::Repositories, wrap_async_fn};
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

        let input = match value.get("input") {
            Some(input) => input.clone(),
            None => Value::Null,
        };

        let output = match value.get("output") {
            Some(output) => output.clone(),
            None => Value::Null,
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
pub struct ModuleValidator {
    pub with: Value,
    pub input: Value,
    pub output: Value,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub sender: channel::Sender<ModulePackage>,
    pub validator: ModuleValidator,
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
        let module = Module {
            sender,
            validator: ModuleValidator {
                with: module_data.with,
                input: module_data.input,
                output: module_data.output,
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
            let args = {
                match &module.validator.input {
                    Value::Object(obj) => {
                        let mut args = Vec::new();

                        for (key, _) in obj.iter() {
                            args.push(key.to_string());
                        }

                        args
                    }
                    _ => {
                        vec!["input".to_string()]
                    }
                }
            };

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
