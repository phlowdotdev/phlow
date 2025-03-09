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

    fn add_pipeline(&mut self, pipeline: Pipeline) {
        self.pipelines.insert(pipeline.id.clone(), pipeline);
    }

    fn get_pipeline(&self, id: ID) -> Option<&Pipeline> {
        self.pipelines.get(&id)
    }

    fn get_main_pipeline(&self) -> Option<&Pipeline> {
        self.pipelines.get(&self.main)
    }
}

impl TryFrom<&str> for V8 {
    type Error = V8Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let pipelines = json_to_pipelines(value).map_err(V8Error::TransformError)?;
        Ok(Self::new(pipelines))
    }
}
