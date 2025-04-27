use std::fmt::Display;

use crate::{
    context::Context,
    step_worker::{NextStep, StepOutput, StepWorker, StepWorkerError},
};

#[derive(Debug)]
pub enum PipelineError {
    StepWorkerError(StepWorkerError),
}

impl Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::StepWorkerError(err) => write!(f, "Step worker error: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub(crate) steps: Vec<StepWorker>,
}

impl Pipeline {
    pub async fn execute(
        &self,
        context: &mut Context,
        skip: usize,
    ) -> Result<Option<StepOutput>, PipelineError> {
        for (i, step) in self.steps.iter().enumerate().skip(skip) {
            match step.execute(&context).await {
                Ok(step_output) => {
                    context.add_step_payload(step_output.output.clone());

                    if step.get_id().is_some() {
                        if let Some(payload) = &step_output.output {
                            context.add_step_id_output(step.get_id().clone(), payload.clone());
                        }
                    }

                    match step_output.next_step {
                        NextStep::Pipeline(_) | NextStep::Stop => return Ok(Some(step_output)),
                        NextStep::GoToStep(to) => {
                            return Ok(Some(StepOutput {
                                output: step_output.output,
                                next_step: NextStep::GoToStep(to),
                            }));
                        }
                        NextStep::Next => {
                            if i == self.steps.len() - 1 {
                                return Ok(Some(step_output));
                            }
                        }
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
