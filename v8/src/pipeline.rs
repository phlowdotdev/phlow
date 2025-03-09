use std::collections::HashMap;

use crate::{
    id::ID,
    step_worker::{NextStep, StepOutput, StepWorker, StepWorkerError},
    v8::Context,
};

#[derive(Debug)]
pub enum PipelineError {
    StepWorkerError(StepWorkerError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub(crate) id: usize,
    pub(crate) steps: Vec<StepWorker>,
}

impl Pipeline {
    pub fn new(id: usize, steps: Vec<StepWorker>) -> Self {
        Self { id, steps }
    }

    pub fn execute(&self, context: &mut Context) -> Result<Option<NextStep>, PipelineError> {
        for step in self.steps.iter() {
            match step.execute(&context) {
                Ok(step_output) => {
                    if step.get_id().is_some() {
                        if let Some(payload) = step_output.payload {
                            context.add_step_output(step.get_id().clone(), payload.clone());
                        }
                    }

                    if let NextStep::Pipeline(_) | NextStep::Stop = step_output.next_step {
                        return Ok(Some(step_output.next_step));
                    }
                }
                Err(err) => {
                    return Err(PipelineError::StepWorkerError(err));
                }
            }
        }

        Ok(None)
    }
}
