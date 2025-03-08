use std::collections::HashMap;

use crate::{pipeline::Pipeline, step::InnerId, transform::transform_json};

struct V8 {
    pipelines: HashMap<InnerId, Pipeline>,
    main: InnerId,
}

impl V8 {
    fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            main: InnerId::new(),
        }
    }

    fn add_pipeline(&mut self, pipeline: Pipeline) {
        self.pipelines.insert(pipeline.id.clone(), pipeline);
    }

    fn get_pipeline(&self, id: InnerId) -> Option<&Pipeline> {
        self.pipelines.get(&id)
    }

    fn get_main_pipeline(&self) -> Option<&Pipeline> {
        self.pipelines.get(&self.main)
    }
}
