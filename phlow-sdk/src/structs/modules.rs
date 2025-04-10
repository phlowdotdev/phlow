use crate::context::Context;
use crossbeam::channel;
use std::collections::HashMap;
use tokio::sync::oneshot;
use valu3::value::Value;

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
    pub context: Context,
    pub sender: oneshot::Sender<ModuleResponse>,
}

impl ModulePackage {
    pub fn input(&self) -> Option<Value> {
        self.context.input.clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Modules {
    pub modules: HashMap<String, channel::Sender<ModulePackage>>,
}

impl Modules {
    pub fn extract(&self) -> Self {
        Self {
            modules: self.modules.clone(),
        }
    }

    pub fn register(&mut self, name: &str, sender: channel::Sender<ModulePackage>) {
        self.modules.insert(name.to_string(), sender);
    }

    pub async fn execute(
        &self,
        name: &str,
        context: &Context,
    ) -> Result<ModuleResponse, ModulesError> {
        if let Some(module_sender) = self.modules.get(name) {
            let (package_sender, package_receiver) = oneshot::channel();
            let package = ModulePackage {
                context: context.clone(),
                sender: package_sender,
            };

            let _ = module_sender.send(package);

            let value = package_receiver.await.unwrap_or(ModuleResponse::from_error(
                "Module response channel closed".to_string(),
            ));

            Ok(value)
        } else {
            Err(ModulesError::ModuleNotLoaded(name.to_string()))
        }
    }
}
