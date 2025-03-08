use std::collections::HashMap;

use crate::{pipeline::Pipeline, step_worker::ID};

struct V8 {
    pipelines: HashMap<ID, Pipeline>,
    main: ID,
}

impl V8 {
    fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            main: ID::new(),
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
