use serde::Serialize;
use std::sync::mpsc::Sender;
use valu3::value::Value;

use crate::id::ID;

pub type ContextSender = Sender<CollectorStep>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CollectorStep {
    pub id: ID,
    pub label: Option<String>,
    pub condition: Option<Value>,
    pub payload: Option<Value>,
    pub return_case: Option<Value>,
}
