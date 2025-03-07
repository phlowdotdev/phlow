mod condition;
mod payload;
mod pipeline;
mod step;
mod variable;
use serde::Serialize;
use std::collections::HashMap;
use step::Step;
use valu3::{prelude::*, Error as ValueError};

pub type InnerId = u32;

#[derive(Debug)]
pub enum Error {
    JsonParseError(ValueError),
    InvalidPipeline(InnerId),
    InvalidCondition,
    InvalidStep(InnerId),
    PayloadError(payload::PayloadError),
}
