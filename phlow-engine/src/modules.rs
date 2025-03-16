use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc},
};

use valu3::value::Value;

use crate::Context;

#[derive(Debug, Clone)]
pub enum ModulesError {
    ModuleNotFound(String),
}

#[derive(Debug, Clone, Default)]
pub struct Modules {
    modules: HashMap<String, Sender<Context>>,
}

impl Modules {
    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn register(&mut self, name: &str, sender: Sender<Context>) {
        self.modules.insert(name.to_string(), sender);
    }

    pub fn execute(&self, name: &str, context: &Context) -> Result<Value, ModulesError> {
        match self.modules.get(name) {
            Some(sender) => {
                sender.send(context.clone()).unwrap();
                Ok(Value::Null)
            }
            None => Err(ModulesError::ModuleNotFound(name.to_string())),
        }
    }
}
