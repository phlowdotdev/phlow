use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;
use valu3::value::Value;

use crate::{condition::Condition, InnerId};

pub type Output = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum StepType {
    Default,
    ThenCase,
    ElseCase,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Step {
    pub(crate) id: Option<String>, // id do json enviado pelo cliente
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) inner_id: InnerId,
    pub(crate) echo: Option<String>,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<String>,
    pub(crate) then_case: Option<InnerId>,
    pub(crate) else_case: Option<InnerId>,
    pub(crate) return_case: Option<Value>,
}

impl Step {
    pub(crate) fn new(
        id: Option<String>,
        name: Option<String>,
        step_type: StepType,
        echo: Option<String>,
        condition: Option<Condition>,
        payload: Option<String>,
        then_case: Option<InnerId>,
        else_case: Option<InnerId>,
        return_case: Option<Value>,
    ) -> Self {
        Self {
            id,
            name,
            step_type,
            inner_id: Uuid::new_v4().to_string(),
            echo,
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        }
    }

    pub fn get_reference_id(&self) -> String {
        match self.id {
            Some(ref id) => id.clone(),
            None => self.inner_id.clone(),
        }
    }
}
