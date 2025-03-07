use serde::Serialize;
use valu3::value::Value;

use crate::{condition::Condition, InnerId};

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
    pub(crate) output: Option<Value>,
    pub(crate) then_case: Option<InnerId>,
    pub(crate) else_case: Option<InnerId>,
    pub(crate) return_case: Option<Value>,
}
