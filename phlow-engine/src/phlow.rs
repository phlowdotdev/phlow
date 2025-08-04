use crate::{
    context::Context,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{value_to_pipelines, TransformError},
};
use phlow_sdk::{
    prelude::{log::debug, *},
    tracing_subscriber::field::debug,
};
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

        Ok(Self { pipelines })
    }

    pub async fn execute(&self, context: &mut Context) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current_pipeline = self.pipelines.len() - 1;
        let mut current_step = 0;

        debug!(
            "Starting execution with {} pipelines. In pipeline {} and step {}",
            self.pipelines.len(),
            current_pipeline,
            current_step
        );

        // let mut total = 0;

        loop {
            // if total >= self.pipelines.len() {
            //     debug!("Reached the end of all pipelines.");
            //     return Ok(None);
            // }
            // total += 1;

            let pipeline = self
                .pipelines
                .get(&current_pipeline)
                .ok_or(PhlowError::PipelineNotFound)?;

            match pipeline.execute(context, current_step).await {
                Ok(step_output) => match step_output {
                    Some(step_output) => match step_output.next_step {
                        NextStep::Next | NextStep::Stop => {
                            debug!("Step output: {:?}", step_output.output);
                            return Ok(step_output.output);
                        }
                        NextStep::Pipeline(id) => {
                            debug!("Switching to pipeline {} and step 0", id);
                            current_pipeline = id;
                            current_step = 0;
                        }
                        NextStep::GoToStep(to) => {
                            debug!("Switching to pipeline {} and step {}", to.pipeline, to.step);
                            current_pipeline = to.pipeline;
                            current_step = to.step;
                        }
                    },
                    None => {
                        debug!("No step output, continuing to next step");
                        return Ok(None);
                    }
                },
                Err(err) => {
                    debug!("Error executing step: {:?}", err);
                    return Err(PhlowError::PipelineError(err));
                }
            }
        }
    }
}
