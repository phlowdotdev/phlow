use std::collections::HashMap;

use serde::Serialize;
use valu3::value::Value;

use crate::{step::Output, Step, StepInnerId};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) context: Value,
    pub(crate) steps: HashMap<StepInnerId, Output>,
}

impl Context {
    pub fn new(context: Value) -> Self {
        Self {
            context,
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, step: &Step, output: Output) {
        self.steps.insert(step.get_reference_id(), output);
    }

    pub fn get_step_output(&self, step: &Step) -> Option<&Output> {
        self.steps.get(&step.get_reference_id())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    name: Option<String>,
    id: Option<String>,
    inner_id: StepInnerId,
    steps: Vec<Step>,
    context: Context,
}

impl Pipeline {
    pub fn new(
        name: Option<String>,
        id: Option<String>,
        inner_id: StepInnerId,
        context: Context,
    ) -> Self {
        Self {
            name,
            id,
            inner_id,
            steps: Vec::new(),
            context,
        }
    }
}
