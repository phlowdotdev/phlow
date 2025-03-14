use serde::Serialize;
use std::sync::mpsc::Sender;
use valu3::value::Value;

use crate::{condition::ConditionRaw, id::ID};

pub type ContextSender = Sender<Step>;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Step {
    pub id: ID,
    pub label: Option<String>,
    pub condition: Option<ConditionRaw>,
    pub payload: Option<Value>,
    pub return_case: Option<Value>,
}
