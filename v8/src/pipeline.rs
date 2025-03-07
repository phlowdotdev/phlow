use std::collections::HashMap;

use serde::Serialize;
use valu3::value::Value;

use crate::{InnerId, Step};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) context: Value,
    pub(crate) steps: HashMap<InnerId, Step>,
}

impl Context {
    pub fn new(context: Value) -> Self {
        Self {
            context,
            steps: HashMap::new(),
        }
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.insert(step.inner_id, step);
    }

    pub fn get_step(&self, inner_id: InnerId) -> Option<&Step> {
        self.steps.get(&inner_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    name: Option<String>,
    id: Option<String>,
    inner_id: InnerId,
    steps: Vec<Step>,
    context: Context,
}

impl Pipeline {
    pub fn new(
        name: Option<String>,
        id: Option<String>,
        inner_id: InnerId,
        context: Context,
    ) -> Self {
        Self {
            name,
            id,
            inner_id,
            steps: Vec::new(),
            context,
        }
    }
}
