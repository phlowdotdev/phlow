use crate::id::ID;
use serde::Serialize;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct Context {
    pub steps: HashMap<ID, Value>,
    pub main: Option<Value>,
    pub payload: Option<Value>,
    pub input: Option<Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            main: None,
            steps: HashMap::new(),
            payload: None,
            input: None,
        }
    }

    pub fn from_payload(payload: Value) -> Self {
        Self {
            main: None,
            steps: HashMap::new(),
            payload: Some(payload),
            input: None,
        }
    }

    pub fn from_main(main: Value) -> Self {
        Self {
            main: Some(main),
            steps: HashMap::new(),
            payload: None,
            input: None,
        }
    }

    pub fn add_module_input(&self, output: Value) -> Self {
        Self {
            main: self.main.clone(),
            steps: self.steps.clone(),
            payload: self.payload.clone(),
            input: Some(output),
        }
    }

    pub fn add_module_output(&self, output: Value) -> Self {
        Self {
            main: self.main.clone(),
            steps: self.steps.clone(),
            payload: Some(output),
            input: self.input.clone(),
        }
    }

    pub fn add_step_payload(&mut self, output: Option<Value>) {
        self.payload = output;
    }

    pub fn add_step_id_output(&mut self, id: ID, output: Value) {
        self.steps.insert(id, output.clone());
    }

    pub fn get_step_output(&self, id: &ID) -> Option<&Value> {
        self.steps.get(&id)
    }
}
