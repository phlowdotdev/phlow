use crate::{
    context::Context,
    debug::{debug_controller, DebugContext, DebugSnapshot},
    step_worker::{NextStep, StepOutput, StepWorker, StepWorkerError},
};
use phlow_sdk::prelude::Value;
use std::fmt::Display;

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

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PipelineError::StepWorkerError(err) => Some(err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub(crate) steps: Vec<StepWorker>,
    pub(crate) id: usize,
}

impl Pipeline {
    pub fn get_id(&self) -> usize {
        self.id
    }

    pub async fn execute(
        &self,
        context: &mut Context,
        skip: usize,
    ) -> Result<Option<StepOutput>, PipelineError> {
        for (step_index, step) in self.steps.iter().enumerate().skip(skip) {
            let controller = debug_controller().cloned();
            if let Some(controller) = &controller {
                let snapshot = DebugSnapshot {
                    context: DebugContext {
                        payload: context.get_payload(),
                        main: context.get_main(),
                    },
                    step: step.step_value.clone().unwrap_or(Value::Null),
                    pipeline: self.id + 1,
                };
                controller.before_step(snapshot).await;
            }

            let result = step.execute(&context).await;
            if let Some(controller) = &controller {
                controller.finish_step().await;
            }

            match result {
                Ok(step_output) => {
                    context.add_step_payload(step_output.output.clone());

                    if step.get_id().is_some() {
                        if let Some(payload) = &step_output.output {
                            context.add_step_id_output(step.get_id().clone(), payload.clone());
                        }
                    }

                    match step_output.next_step {
                        NextStep::Pipeline(_) | NextStep::Stop => {
                            return Ok(Some(step_output));
                        }
                        NextStep::GoToStep(to) => {
                            return Ok(Some(StepOutput {
                                output: step_output.output,
                                next_step: NextStep::GoToStep(to),
                            }));
                        }
                        NextStep::Next => {
                            if step_index == self.steps.len() - 1 {
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

        Ok(Some(StepOutput {
            output: context.get_payload().clone(),
            next_step: NextStep::Next,
        }))
    }
}
