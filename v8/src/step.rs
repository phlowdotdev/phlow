use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;
use valu3::value::Value;

use crate::{condition::Condition, payload::Payload, pipeline::Context, Error};

pub type StepInnerId = String;
pub type Output = Value;

pub enum NextStep {
    Step(StepInnerId),
    Stop,
    Next,
}

pub struct StepOutput {
    pub(crate) next_step: NextStep,
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
    pub(crate) return_case: Option<String>,
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
            let output = Some(
                Payload::new(payload.to_string())
                    .evaluate(context)
                    .map_err(Error::PayloadError)?,
            );
            Ok(output)
        } else {
            Ok(None)
        }
    }

    fn evaluate_return(&self, context: &Context) -> Result<Option<Output>, Error> {
        if let Some(ref return_case) = self.return_case {
            let output = Some(
                Payload::new(return_case.to_string())
                    .evaluate(context)
                    .map_err(Error::PayloadError)?,
            );
            Ok(output)
        } else {
            Ok(None)
        }
    }

    pub fn execute(&self, context: &Context) -> Result<StepOutput, Error> {
        if let Some(ref condition) = self.condition {
            if condition.evaluate(context)? {
                let output = if self.evaluate_payload(context)?.is_some() {
                    self.evaluate_payload(context)?
                } else {
                    None
                };

                if let Some(ref then_case) = self.then_case {
                    return Ok(StepOutput {
                        next_step: NextStep::Step(then_case.clone()),
                        output,
                    });
                } else {
                    return Ok(StepOutput {
                        next_step: NextStep::Next,
                        output,
                    });
                }
            } else {
                if let Some(ref else_case) = self.else_case {
                    return Ok(StepOutput {
                        next_step: NextStep::Step(else_case.clone()),
                        output: None,
                    });
                } else {
                    return Ok(StepOutput {
                        next_step: NextStep::Stop,
                        output: None,
                    });
                }
            }
        }

        if let Some(return_case) = self.evaluate_return(context)? {
            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(return_case),
            });
        }

        if let Some(ref payload) = self.payload {
            let output = Some(
                Payload::new(payload.to_string())
                    .evaluate(context)
                    .map_err(Error::PayloadError)?,
            );

            return Ok(StepOutput {
                next_step: NextStep::Next,
                output,
            });
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            output: None,
        });
    }
}
