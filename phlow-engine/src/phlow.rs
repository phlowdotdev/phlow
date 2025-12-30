use crate::{
    context::Context,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{TransformError, value_to_pipelines},
};
use phlow_sdk::prelude::{log::error, *};
use phs::build_engine;
use std::{collections::HashMap, fmt::Display, sync::Arc};

#[derive(Debug)]
pub enum PhlowError {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
    ParentError,
}

impl Display for PhlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhlowError::TransformError(err) => write!(f, "Transform error: {}", err),
            PhlowError::PipelineError(err) => write!(f, "Pipeline error: {}", err),
            PhlowError::PipelineNotFound => write!(f, "Pipeline not found"),
            PhlowError::ParentError => write!(f, "Parent error"),
        }
    }
}

impl std::error::Error for PhlowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PhlowError::TransformError(err) => Some(err),
            PhlowError::PipelineError(err) => Some(err),
            PhlowError::PipelineNotFound => None,
            PhlowError::ParentError => None,
        }
    }
}

pub type PipelineMap = HashMap<usize, Pipeline>;

#[derive(Debug, Default)]
pub struct Phlow {
    pipelines: PipelineMap,
    script: Value,
}

impl Phlow {
    pub fn try_from_value(
        value: &Value,
        modules: Option<Arc<Modules>>,
    ) -> Result<Self, PhlowError> {
        let engine = match &modules {
            Some(modules) => {
                let repositories = modules.extract_repositories();
                build_engine(Some(repositories))
            }
            None => build_engine(None),
        };

        let modules = if let Some(modules) = modules {
            modules
        } else {
            Arc::new(Modules::default())
        };

        let pipelines =
            value_to_pipelines(engine, modules, value).map_err(PhlowError::TransformError)?;

        Ok(Self {
            pipelines,
            script: value.clone(),
        })
    }

    pub async fn execute(&self, context: &mut Context) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current_pipeline = self.pipelines.len() - 1;
        let mut current_step = 0;

        loop {
            log::debug!(
                "Executing pipeline {} step {}",
                current_pipeline,
                current_step
            );
            let pipeline = self
                .pipelines
                .get(&current_pipeline)
                .ok_or(PhlowError::PipelineNotFound)?;

            match pipeline.execute(context, current_step).await {
                Ok(step_output) => match step_output {
                    Some(step_output) => {
                        log::debug!(
                            "Next step decision: {:?}, payload: {:?}",
                            step_output.next_step,
                            step_output.output
                        );
                        match step_output.next_step {
                            NextStep::Stop => {
                                log::debug!("NextStep::Stop - terminating execution");
                                return Ok(step_output.output);
                            }
                            NextStep::Next => {
                                log::debug!(
                                    "NextStep::Next - checking if sub-pipeline needs to return to parent"
                                );
                                // Check if this is the main pipeline (highest index)
                                let main_pipeline = self.pipelines.len() - 1;
                                if current_pipeline == main_pipeline {
                                    log::debug!(
                                        "NextStep::Next - terminating execution (main pipeline completed)"
                                    );
                                    return Ok(step_output.output);
                                } else {
                                    log::debug!(
                                        "NextStep::Next - sub-pipeline completed, checking for parent return"
                                    );
                                    // This is a sub-pipeline that completed - we should return to parent
                                    // For now, terminate execution but this needs proper parent tracking
                                    return Ok(step_output.output);
                                }
                            }
                            NextStep::Pipeline(id) => {
                                log::debug!("NextStep::Pipeline({}) - jumping to pipeline", id);
                                current_pipeline = id;
                                current_step = 0;
                            }
                            NextStep::GoToStep(to) => {
                                log::debug!("NextStep::GoToStep({:?}) - jumping to step", to);
                                current_pipeline = to.pipeline;
                                current_step = to.step;
                            }
                        }
                    }
                    None => {
                        return Ok(None);
                    }
                },
                Err(err) => {
                    error!("Error executing step: {:?}", err);
                    return Err(PhlowError::PipelineError(err));
                }
            }
        }
    }

    pub fn script(&self) -> Value {
        self.script.clone()
    }
}
