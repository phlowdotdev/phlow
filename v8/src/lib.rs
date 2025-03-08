mod condition;
mod payload;
mod pipeline;
mod step;
mod variable;
use step::{InnerStep, StepInnerId};
use valu3::Error as ValueError;

#[derive(Debug)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(StepInnerId),
    InvalidCondition,
    InvalidStep(StepInnerId),
    PayloadError(payload::PayloadError),
}
