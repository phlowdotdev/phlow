use crate::id::ID;
use serde::Serialize;
use std::collections::HashMap;
use valu3::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct Context {
    steps: HashMap<ID, Value>,
    main: Option<Value>,
    payload: Option<Value>,
    input: Option<Value>,
    setup: Option<Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn from_main(main: Value) -> Self {
        Self {
            main: Some(main),
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
        if let Some(output) = output {
            self.payload = Some(output);
        }
    }

    pub fn get_payload(&self) -> Option<Value> {
        self.payload.clone()
    }

    pub fn get_main(&self) -> Option<Value> {
        self.main.clone()
    }

    pub fn get_setup(&self) -> Option<Value> {
        self.setup.clone()
    }

    pub fn get_input(&self) -> Option<Value> {
        self.input.clone()
    }

    pub fn get_steps(&self) -> &HashMap<ID, Value> {
        &self.steps
    }

    pub fn set_main(&mut self, main: Value) {
        self.main = Some(main);
    }

    pub fn add_step_id_output(&mut self, id: ID, output: Value) {
        self.steps.insert(id, output.clone());
    }

    pub fn get_step_output(&self, id: &ID) -> Option<&Value> {
        self.steps.get(&id)
    }
}
