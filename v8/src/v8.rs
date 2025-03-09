use crate::{
    id::ID,
    payload::PayloadError,
    pipeline::{Pipeline, PipelineError},
    step_worker::NextStep,
    transform::{get_pipeline_id, json_to_pipelines, TransformError},
};
use serde::Serialize;
use std::collections::HashMap;
use valu3::value::Value;
use valu3::Error as ValueError;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) params: Value,
    pub(crate) steps: HashMap<ID, Value>,
}

impl Context {
    pub fn new(params: Value) -> Self {
        Self {
            params,
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, id: ID, output: Value) {
        self.steps.insert(id, output);
    }

    pub fn get_step_output(&self, id: &ID) -> Option<&Value> {
        self.steps.get(&id)
    }
}
#[derive(Debug)]
pub enum Error {
    PipelineNotFound,
    JsonParseError(ValueError),
    TransformError(TransformError),
    InvalidPipeline(ID),
    InvalidCondition,
    InvalidStep(ID),
    PayloadError(PayloadError),
    PipelineError(PipelineError),
}

pub type PipelineMap = HashMap<usize, Pipeline>;

struct V8 {
    pipelines: PipelineMap,
    main: usize,
    params: Option<Value>,
}

impl V8 {
    pub fn execute(&self) -> Result<Context, Error> {
        let mut context = Context::new(self.params.clone().unwrap_or_default());

        let mut current = self.main;
        loop {
            let pipeline = self
                .pipelines
                .get(&current)
                .ok_or(Error::PipelineNotFound)?;

            match pipeline.execute(&mut context) {
                Ok(next_step) => match next_step {
                    NextStep::Next => {
                        return Ok(context);
                    }
                    NextStep::Pipeline(id) => {
                        current = id;
                    }
                    NextStep::Stop => {
                        return Ok(context);
                    }
                },
                Err(err) => {
                    return Err(Error::PipelineError(err));
                }
            }
        }
    }
}

impl TryFrom<&str> for V8 {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (pipelines, params) = json_to_pipelines(value).map_err(Error::TransformError)?;
        Ok(Self {
            pipelines,
            main: 0,
            params,
        })
    }
}
