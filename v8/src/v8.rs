use crate::{
    id::ID,
    payload::PayloadError,
    pipeline::Pipeline,
    transform::{json_to_pipelines, TransformError},
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
}

struct V8 {
    pipelines: HashMap<ID, Pipeline>,
    main: ID,
    params: Option<Value>,
}

impl V8 {
    fn new(pipelines: HashMap<ID, Pipeline>, params: Option<Value>) -> Self {
        Self {
            pipelines,
            main: ID::from("pipeline_id_0"),
            params,
        }
    }
}

impl TryFrom<&str> for V8 {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (pipelines, params) = json_to_pipelines(value).map_err(Error::TransformError)?;
        Ok(Self::new(pipelines, params))
    }
}
