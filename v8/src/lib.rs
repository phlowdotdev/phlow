mod condition;
mod payload;
mod pipeline;
mod step;
mod transform;
mod v8;
mod variable;
use std::collections::HashMap;

use pipeline::{Pipeline, Step};
use step::{ID, StepWorker};
use valu3::Error as ValueError;

#[derive(Debug)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(ID),
    InvalidCondition,
    InvalidStep(ID),
    PayloadError(payload::PayloadError),
}

struct V8 {
    pipelines: HashMap<ID, Pipeline>,
    main: ID,
}

fn steps_to_pipelines(steps: Vec<Step>) -> Vec<Pipeline> {
    let mut pipelines = HashMap::new();

    for step in steps {
        let inner_step = StepWorker::from(step);
        let pipeline = Pipeline::new(ID::new(), vec![inner_step]);
        pipelines.insert(pipeline.id.clone(), pipeline);
    }

    pipelines
        .into_iter()
        .map(|(_, pipeline)| pipeline)
        .collect()
}
