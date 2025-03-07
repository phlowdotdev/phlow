use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;
use valu3::value::Value;

use crate::{condition::Condition, payload::Payload, pipeline::Context, Error};

pub type StepInnerId = String;
pub type Output = Value;

pub struct StepOutput {
    pub(crate) next_step: Option<StepInnerId>,
    pub(crate) output: Option<Output>,
}

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
    pub(crate) inner_id: StepInnerId,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<String>,
    pub(crate) then_case: Option<StepInnerId>,
    pub(crate) else_case: Option<StepInnerId>,
    pub(crate) return_case: Option<Value>,
}

impl Step {
    pub(crate) fn new(
        id: Option<String>,
        name: Option<String>,
        step_type: StepType,
        condition: Option<Condition>,
        payload: Option<String>,
        then_case: Option<StepInnerId>,
        else_case: Option<StepInnerId>,
        return_case: Option<Value>,
    ) -> Self {
        Self {
            id,
            name,
            step_type,
            inner_id: Uuid::new_v4().to_string(),
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

    fn evaluate_payload(&self, context: &Context) -> Result<Option<Output>, Error> {
        if let Some(ref payload) = self.payload {
            let payload = Payload::new(payload.to_string());
            let output = Some(payload.evaluate(context).map_err(Error::PayloadError)?);
            Ok(output)
        } else {
            Ok(None)
        }
    }

    pub fn execute(&self, context: &Context) -> Result<StepOutput, Error> {
        if let Some(ref condition) = self.condition {
            let result = condition.evaluate(context)?;

            if result {
                if let Some(ref then_case) = self.then_case {
                    let output = if self.evaluate_payload(context)?.is_some() {
                        self.evaluate_payload(context)?
                    } else {
                        None
                    };

                    return Ok(StepOutput {
                        next_step: Some(then_case.clone()),
                        output,
                    });
                }
            } else {
                if let Some(ref else_case) = self.else_case {
                    return Ok(StepOutput {
                        next_step: Some(else_case.clone()),
                        output: None,
                    });
                }
            }
        }

        let output = if self.evaluate_payload(context)?.is_some() {
            self.evaluate_payload(context)?
        } else {
            None
        };

        Ok(StepOutput {
            next_step: None,
            output,
        })
    }
}
