use crate::{
    pipeline::Pipeline,
    step_worker::ID,
    transform::{json_to_pipelines, TransformError},
};
use std::collections::HashMap;

pub enum V8Error {
    PipelineNotFound,
    TransformError(TransformError),
}

struct V8 {
    pipelines: HashMap<ID, Pipeline>,
    main: ID,
}

impl V8 {
    fn new(pipelines: HashMap<ID, Pipeline>) -> Self {
        Self {
            pipelines,
            main: ID::from("pipeline_id_0"),
        }
    }
}

impl TryFrom<&str> for V8 {
    type Error = V8Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let pipelines = json_to_pipelines(value).map_err(V8Error::TransformError)?;
        Ok(Self::new(pipelines))
    }
}
