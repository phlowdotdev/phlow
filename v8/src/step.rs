use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;
use valu3::value::Value;

use crate::{condition::Condition, payload::Payload, pipeline::Context, Error};

pub type StepInnerId = String;
pub type Output = Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<StepInnerId>,
    pub(crate) else_case: Option<StepInnerId>,
    pub(crate) return_case: Option<Payload>,
}

impl Step {
    pub(crate) fn new(
        id: Option<String>,
        name: Option<String>,
        step_type: StepType,
        condition: Option<Condition>,
        payload: Option<Payload>,
        then_case: Option<StepInnerId>,
        else_case: Option<StepInnerId>,
        return_case: Option<Payload>,
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
            let output = Some(payload.evaluate(context).map_err(Error::PayloadError)?);
            Ok(output)
        } else {
            Ok(None)
        }
    }

    fn evaluate_return(&self, context: &Context) -> Result<Option<Output>, Error> {
        if let Some(ref return_case) = self.return_case {
            let output = Some(return_case.evaluate(context).map_err(Error::PayloadError)?);
            Ok(output)
        } else {
            Ok(None)
        }
    }

    pub fn execute(&self, context: &Context) -> Result<StepOutput, Error> {
        if let Some(condition) = &self.condition {
            let (next_step, output) = if condition.evaluate(context)? {
                let next_step = if let Some(ref then_case) = self.then_case {
                    NextStep::Step(then_case.clone())
                } else {
                    NextStep::Next
                };

                (next_step, self.evaluate_payload(context)?)
            } else {
                let next_step = if let Some(ref else_case) = self.else_case {
                    NextStep::Step(else_case.clone())
                } else {
                    NextStep::Stop
                };

                (next_step, None)
            };

            return Ok(StepOutput { next_step, output });
        }

        if let Some(return_case) = self.evaluate_return(context)? {
            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(return_case),
            });
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            output: self.evaluate_payload(context)?,
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use valu3::value::Value;

    #[test]
    fn test_step_get_reference_id() {
        let step = Step::new(
            Some("id".to_string()),
            Some("name".to_string()),
            StepType::Default,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(step.get_reference_id(), "id".to_string());
    }

    #[test]
    fn test_step_get_reference_id_without_id() {
        let step = Step::new(
            None,
            Some("name".to_string()),
            StepType::Default,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(step.get_reference_id(), step.inner_id);
    }

    #[test]
    fn test_step_execute() {
        let step = Step::new(
            None,
            None,
            StepType::Default,
            None,
            Some(Payload::new("10".to_string())),
            None,
            None,
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition() {
        let step = Step::new(
            None,
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::NotEqual,
            )),
            Some(Payload::new("10".to_string())),
            None,
            None,
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_then_case() {
        let step = Step::new(
            None,
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::NotEqual,
            )),
            Some(Payload::new("10".to_string())),
            Some("then_case".to_string()),
            None,
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step("then_case".to_string()));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_else_case() {
        let step = Step::new(
            None,
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            Some(Payload::new("10".to_string())),
            None,
            Some("else_case".to_string()),
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step("else_case".to_string()));
        assert_eq!(result.output, None);
    }
}
