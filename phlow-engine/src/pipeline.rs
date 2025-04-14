use crate::{
    context::Context,
    step_worker::{NextStep, StepOutput, StepWorker, StepWorkerError},
};

#[derive(Debug)]
pub enum PipelineError {
    StepWorkerError(StepWorkerError),
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
        for step in self.steps.iter().skip(skip) {
            println!("step: {:?}", step.label);
            println!("");

            match step.execute(&context).await {
                Ok(step_output) => {
                    context.add_step_payload(step_output.output.clone());

                    if step.get_id().is_some() {
                        if let Some(payload) = &step_output.output {
                            context.add_step_id_output(step.get_id().clone(), payload.clone());
                        }
                    }

                    if let NextStep::Pipeline(_) | NextStep::Stop = step_output.next_step {
                        return Ok(Some(step_output));
                    }

                    if let NextStep::GoToStep(to) = step_output.next_step {
                        return Ok(Some(StepOutput {
                            output: step_output.output,
                            next_step: NextStep::GoToStep(to),
                        }));
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
