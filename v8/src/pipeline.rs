use crate::{id::ID, step_worker::StepWorker};

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
