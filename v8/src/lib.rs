mod condition;
mod payload;
mod pipeline;
mod step_worker;
mod transform;
mod v8;
mod variable;
use std::collections::HashMap;

use pipeline::Pipeline;
use step_worker::{StepWorker, ID};
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
