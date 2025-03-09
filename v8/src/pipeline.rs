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
