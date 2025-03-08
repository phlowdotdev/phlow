use std::collections::HashMap;

use crate::{
    condition::Condition,
    payload::Payload,
    step::{Output, StepType},
    InnerStep, StepInnerId,
};
use serde::Serialize;
use valu3::value::Value;

pub enum PipelineError {}

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

    pub fn add_step_output(&mut self, step: &InnerStep, output: Output) {
        self.steps.insert(step.get_reference_id(), output);
    }

    pub fn get_step_output(&self, step: &InnerStep) -> Option<&Output> {
        self.steps.get(&step.get_reference_id())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<Vec<Step>>,
    pub(crate) else_case: Option<Vec<Step>>,
    pub(crate) return_case: Option<Payload>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    name: Option<String>,
    id: StepInnerId,
    steps: HashMap<String, InnerStep>,
    steps_order: Vec<StepInnerId>,
}
