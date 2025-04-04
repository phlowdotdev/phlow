use crossbeam::channel;
use std::collections::HashMap;
use tokio::sync::oneshot;
use valu3::value::Value;

use crate::Context;

#[derive(Debug, Clone)]
pub enum ModulesError {
    ModuleNotFound(String),
    ModuleNotLoaded(String),
}

#[derive(Debug)]
pub struct ModulePackage {
    pub context: Context,
    pub sender: oneshot::Sender<Value>,
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

    pub async fn execute(&self, name: &str, context: &Context) -> Result<Value, ModulesError> {
        if let Some(module_sender) = self.modules.get(name) {
            let (package_sender, package_receiver) = oneshot::channel();
            let package = ModulePackage {
                context: context.clone(),
                sender: package_sender,
            };

            let _ = module_sender.send(package);

            let value = package_receiver.await.unwrap_or(Value::Null);

            Ok(value)
        } else {
            Err(ModulesError::ModuleNotLoaded(name.to_string()))
        }
    }
}
