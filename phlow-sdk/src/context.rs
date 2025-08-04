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
    pub setup: Option<Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn from_payload(payload: Value) -> Self {
        Self {
            payload: Some(payload),
            ..Default::default()
        }
    }

    pub fn from_main(main: Value) -> Self {
        Self {
            main: Some(main),
            ..Default::default()
        }
    }

    pub fn from_input(input: Value) -> Self {
        Self {
            input: Some(input),
            ..Default::default()
        }
    }

    pub fn from_setup(setup: Value) -> Self {
        Self {
            setup: Some(setup),
            ..Default::default()
        }
    }

    pub fn add_module_input(&self, output: Value) -> Self {
        Self {
            main: self.main.clone(),
            steps: self.steps.clone(),
            payload: self.payload.clone(),
            input: Some(output),
            setup: self.setup.clone(),
        }
    }

    pub fn add_module_output(&self, output: Value) -> Self {
        Self {
            main: self.main.clone(),
            steps: self.steps.clone(),
            payload: Some(output),
            input: self.input.clone(),
            setup: self.setup.clone(),
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
