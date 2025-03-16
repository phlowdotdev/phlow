use crate::{
    context::Context,
    step_worker::{NextStep, StepOutput, StepWorker, StepWorkerError},
};

#[derive(Debug)]
pub enum PipelineError {
    StepWorkerError(StepWorkerError),
}

#[derive(Debug, Clone)]
pub struct Pipeline<'a> {
    pub(crate) steps: Vec<StepWorker<'a>>,
}

impl<'a> Pipeline<'a> {
    pub async fn execute(
        &self,
        context: &mut Context,
    ) -> Result<Option<StepOutput>, PipelineError> {
        for step in self.steps.iter() {
            match step.execute(&context).await {
                Ok(step_output) => {
                    if step.get_id().is_some() {
                        if let Some(payload) = &step_output.output {
                            context.add_step_output(step.get_id().clone(), payload.clone());
                        }
                    }

                    if let NextStep::Pipeline(_) | NextStep::Stop = step_output.next_step {
                        return Ok(Some(step_output));
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
