use crate::id::ID;
use crossbeam::channel;
use phlow_sdk::{crossbeam, valu3};
use serde::Serialize;
use std::{collections::HashMap, fmt::Debug};
use valu3::prelude::*;

pub type ContextSender = channel::Sender<Step>;

#[derive(Clone, Default, PartialEq, Serialize)]
pub struct Step {
    pub id: ID,
    pub label: Option<String>,
    pub module: Option<String>,
    pub input: Option<Value>,
    pub condition: Option<Value>,
    pub payload: Option<Value>,
    pub return_case: Option<Value>,
}

impl ToValueBehavior for Step {
    fn to_value(&self) -> Value {
        let mut value = HashMap::new();

        value.insert("id", self.id.to_value());
        value.insert("label", self.label.to_value());
        value.insert("condition", self.condition.to_value());
        value.insert("payload", self.payload.to_value());
        value.insert("return_case", self.return_case.to_value());

        value.to_value()
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = self.to_value();
        let id = match value.get("id") {
            Some(id) => id.to_string(),
            None => "unknown".to_string(),
        };
        value.remove(&"id");
        value.insert("step_id", id);

        write!(f, "{}", value.to_json(valu3::prelude::JsonMode::Inline))
    }
}
