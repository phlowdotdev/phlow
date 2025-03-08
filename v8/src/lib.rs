mod condition;
mod payload;
mod pipeline;
mod step;
mod transform;
mod v8;
mod variable;
use std::collections::HashMap;

use pipeline::{Pipeline, Step};
use step::{InnerId, InnerStep};
use valu3::Error as ValueError;

#[derive(Debug)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(InnerId),
    InvalidCondition,
    InvalidStep(InnerId),
    PayloadError(payload::PayloadError),
}

struct V8 {
    pipelines: HashMap<InnerId, Pipeline>,
    main: InnerId,
}

fn steps_to_pipelines(steps: Vec<Step>) -> Vec<Pipeline> {
    let mut pipelines = HashMap::new();

    for step in steps {
        let inner_step = InnerStep::from(step);
        let pipeline = Pipeline::new(InnerId::new(), vec![inner_step]);
        pipelines.insert(pipeline.id.clone(), pipeline);
    }

    pipelines
        .into_iter()
        .map(|(_, pipeline)| pipeline)
        .collect()
}
