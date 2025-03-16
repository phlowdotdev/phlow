use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

use tokio::sync::oneshot;
use valu3::value::Value;

use crate::Context;

#[derive(Debug, Clone)]
pub enum ModulesError {
    ModuleNotFound(String),
}

#[derive(Debug)]
pub struct ModulePackage {
    pub context: Context,
    pub sender: oneshot::Sender<Value>,
}

#[derive(Debug, Default)]
pub struct Modules {
    modules: HashMap<String, Sender<ModulePackage>>,
    modules_receivers: HashMap<String, Receiver<ModulePackage>>,
}

impl Modules {
    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn register(&mut self, name: &str) {
        let (sender, receiver) = channel();
        self.modules_receivers.insert(name.to_string(), receiver);
        self.modules.insert(name.to_string(), sender);
    }

    pub fn execute(&self, name: &str, context: &Context) -> Result<Value, ModulesError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(self.execute_async(name, context)).unwrap();
        Ok(res)
    }

    pub async fn execute_async(
        &self,
        name: &str,
        context: &Context,
    ) -> Result<Value, ModulesError> {
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
            Err(ModulesError::ModuleNotFound(name.to_string()))
        }
    }
}
