use serde::Serialize;
use std::sync::mpsc::Sender;
use std::{collections::HashMap, fmt::Debug};
use valu3::prelude::*;

use crate::{condition::ConditionRaw, id::ID};

pub type ContextSender = Sender<Step>;

#[derive(Clone, Default, PartialEq, Serialize)]
pub struct Step {
    pub id: ID,
    pub label: Option<String>,
    pub condition: Option<ConditionRaw>,
    pub payload: Option<Value>,
    pub return_case: Option<Value>,
}

impl ToValueBehavior for Step {
    fn to_value(&self) -> Value {
        let mut value = HashMap::new();
        let mut condition = HashMap::new();

        if let Some(condition_raw) = &self.condition {
            condition.insert("left".to_string(), condition_raw.left.to_value());
            condition.insert("right".to_string(), condition_raw.right.to_value());
            condition.insert("operator".to_string(), condition_raw.operator.to_value());
        }

        value.insert("id", self.id.to_value());
        value.insert("label", self.label.to_value());
        value.insert("condition", condition.to_value());
        value.insert("payload", self.payload.to_value());
        value.insert("return_case", self.return_case.to_value());

        value.to_value()
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = self.to_value();
        let id = value.get("id").unwrap().clone();
        value.remove(&"id");
        value.insert("step_id", id);

        write!(f, "{}", value.to_json(valu3::prelude::JsonMode::Inline))
    }
}
