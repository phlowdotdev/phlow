use crate::{
    context::Context,
    debug::debug_controller,
    pipeline::{Pipeline, PipelineError},
    step_worker::{NextStep, StepReference},
    transform::{TransformError, value_to_pipelines},
};
use phlow_sdk::prelude::{log::error, *};
use phs::build_engine;
use std::{collections::HashMap, fmt::Display, sync::Arc};
use uuid::Uuid;

#[derive(Debug)]
pub enum PhlowError {
    TransformError(TransformError),
    PipelineError(PipelineError),
    PipelineNotFound,
    InvalidStartStep { pipeline: usize, step: usize },
    ParentError,
}

impl Display for PhlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhlowError::TransformError(err) => write!(f, "Transform error: {}", err),
            PhlowError::PipelineError(err) => write!(f, "Pipeline error: {}", err),
            PhlowError::PipelineNotFound => write!(f, "Pipeline not found"),
            PhlowError::InvalidStartStep { pipeline, step } => {
                write!(f, "Invalid start step: pipeline {} step {}", pipeline, step)
            }
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
            PhlowError::InvalidStartStep { .. } => None,
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

        let script = if should_add_uuid() {
            let in_steps = value.is_array();
            add_uuids(value, in_steps)
        } else {
            value.clone()
        };

        let pipelines =
            value_to_pipelines(engine, modules, &script).map_err(PhlowError::TransformError)?;

        Ok(Self {
            pipelines,
            script,
        })
    }

    pub async fn execute(&self, context: &mut Context) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let main_pipeline = self.pipelines.len() - 1;
        let start = StepReference {
            pipeline: main_pipeline,
            step: 0,
        };

        self.execute_from(context, start).await
    }

    pub fn find_step_reference(&self, id: &str) -> Option<StepReference> {
        for pipeline in self.pipelines.values() {
            let pipeline_id = pipeline.get_id();
            for (step_index, step) in pipeline.steps.iter().enumerate() {
                let step_id = step.get_id();
                if step_id.is_some() && step_id.to_string() == id {
                    return Some(StepReference {
                        pipeline: pipeline_id,
                        step: step_index,
                    });
                }
            }
        }

        None
    }

    pub async fn execute_from(
        &self,
        context: &mut Context,
        start: StepReference,
    ) -> Result<Option<Value>, PhlowError> {
        if self.pipelines.is_empty() {
            return Ok(None);
        }

        let mut current_pipeline = start.pipeline;
        let mut current_step = start.step;
        let main_pipeline = self.pipelines.len() - 1;

        {
            let pipeline = self
                .pipelines
                .get(&current_pipeline)
                .ok_or(PhlowError::PipelineNotFound)?;
            if current_step >= pipeline.steps.len() {
                return Err(PhlowError::InvalidStartStep {
                    pipeline: current_pipeline,
                    step: current_step,
                });
            }
        }

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

fn should_add_uuid() -> bool {
    if debug_controller().is_some() {
        return true;
    }
    std::env::var("PHLOW_DEBUG")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn add_uuids(value: &Value, in_steps: bool) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = HashMap::new();
            for (key, value) in map.iter() {
                let key_str = key.to_string();
                let is_pipeline = matches!(key_str.as_str(), "then" | "else")
                    && (value.is_object() || value.is_array());
                let next_in_steps = key_str == "steps" || is_pipeline;
                new_map.insert(key_str, add_uuids(value, next_in_steps));
            }
            if in_steps && !map.contains_key(&"#uuid".to_string()) {
                new_map.insert(
                    "#uuid".to_string(),
                    Uuid::new_v4().to_string().to_value(),
                );
            }
            Value::from(new_map)
        }
        Value::Array(array) => {
            let mut new_array = Vec::new();
            for value in array.values.iter() {
                new_array.push(add_uuids(value, in_steps));
            }
            Value::from(new_array)
        }
        _ => value.clone(),
    }
}
