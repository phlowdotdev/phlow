use crate::id::ID;
use serde::Serialize;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) params: Option<Value>,
    pub(crate) steps: HashMap<ID, Value>,
    pub(crate) main: Option<Value>,
}

impl Context {
    pub fn new(params: Option<Value>) -> Self {
        Self {
            params,
            main: None,
            steps: HashMap::new(),
        }
    }

    pub fn from_main(main: Value) -> Self {
        Self {
            params: None,
            main: Some(main),
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, id: ID, output: Value) {
        self.steps.insert(id, output);
    }

    pub fn get_step_output(&self, id: &ID) -> Option<&Value> {
        self.steps.get(&id)
    }
}
