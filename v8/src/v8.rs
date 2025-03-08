use std::collections::HashMap;

use crate::{
    pipeline::{Pipeline, Step},
    step::{InnerId, InnerStep},
};

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

fn raw_steps_to_pipeline(steps: Vec<Step>) -> Pipeline {
    let mut steps_map = HashMap::new();
    let mut steps_order = Vec::new();

    for step in steps {
        let inner_step = InnerStep::from(step);
        steps_map.insert(inner_step.get_reference_id(), inner_step.clone());
        steps_order.push(inner_step.get_reference_id());
    }

    Pipeline {
        name: None,
        id: InnerId::new(),
        steps: steps_map,
        steps_order,
    }
}
