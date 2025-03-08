mod condition;
mod payload;
mod pipeline;
mod step;
mod variable;
use std::collections::HashMap;

use pipeline::Pipeline;
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
