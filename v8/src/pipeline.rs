use std::collections::HashMap;

use crate::{
    condition::Condition,
    payload::Payload,
    step_worker::{Output, StepType},
    StepWorker, ID,
};
use serde::Serialize;
use valu3::value::Value;

pub enum PipelineError {}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) context: Value,
    pub(crate) steps: HashMap<ID, Output>,
}

impl Context {
    pub fn new(context: Value) -> Self {
        Self {
            context,
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, step: &StepWorker, output: Output) {
        self.steps.insert(step.get_id().clone(), output);
    }

    pub fn get_step_output(&self, step: &StepWorker) -> Option<&Output> {
        self.steps.get(&step.get_id())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) condition: Option<Condition>,
    pub(crate) echo: Option<String>,
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<Vec<Step>>,
    pub(crate) else_case: Option<Vec<Step>>,
    pub(crate) return_case: Option<Payload>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub(crate) name: Option<String>,
    pub(crate) id: ID,
    pub(crate) steps: Vec<StepWorker>,
}

impl Pipeline {
    pub fn new(id: ID, steps: Vec<StepWorker>) -> Self {
        Self {
            name: None,
            id,
            steps,
        }
    }
}
