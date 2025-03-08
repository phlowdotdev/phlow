use crate::{
    condition::{Condition, ConditionError},
    payload::Payload,
    pipeline::{Context, Step},
    Error,
};
use serde::Serialize;
use valu3::{prelude::StringBehavior, value::Value};

pub type InnerId = String;
pub type Output = Value;

pub enum InnerStepError {
    ConditionError(ConditionError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NextStep {
    Step(InnerId),
    Stop,
    Next,
}

pub struct StepOutput {
    pub next_step: NextStep,
    pub output: Option<Output>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StepType {
    Default,
    ThenCase,
    ElseCase,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InnerStep {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) inner_id: InnerId,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<InnerId>,
    pub(crate) else_case: Option<InnerId>,
    pub(crate) return_case: Option<Payload>,
}

impl TryFrom<&Value> for InnerStep {
    type Error = InnerStepError;

    fn try_from(value: &Value) -> Result<Self, InnerStepError> {
        let id = value.get("id").map(|id| id.to_string());
        let name = value.get("name").map(|name| name.to_string());
        let step_type = value
            .get("step_type")
            .map(|step_type| match step_type.as_str() {
                "default" => StepType::Default,
                "then_case" => StepType::ThenCase,
                "else_case" => StepType::ElseCase,
                _ => unreachable!(),
            })
            .unwrap_or(StepType::Default);

        let condition = {
            if let Some(condition) = value
                .get("condition")
                .map(|condition| Condition::try_from(condition))
            {
                Some(condition.map_err(InnerStepError::ConditionError)?)
            } else {
                None
            }
        };

        let payload = value
            .get("payload")
            .map(|payload| Payload::new(payload.to_string()));
        let then_case = value
            .get("then_case")
            .map(|then_case| then_case.to_string());
        let else_case = value
            .get("else_case")
            .map(|else_case| else_case.to_string());
        let return_case = value
            .get("return_case")
            .map(|return_case| Payload::new(return_case.to_string()));

        Ok(Self {
            id,
            name,
            step_type,
            inner_id: InnerId::new(),
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        })
    }
}

impl From<Step> for InnerStep {
    fn from(step: Step) -> Self {
        Self {
            id: step.id,
            name: step.name,
            step_type: step.step_type,
            inner_id: InnerId::new(),
            condition: step.condition,
            payload: step.payload,
            then_case: None,
            else_case: None,
            return_case: step.return_case,
        }
    }
}

impl InnerStep {
    pub fn new(
        id: Option<String>,
        inner_id: InnerId,
        name: Option<String>,
        step_type: StepType,
        condition: Option<Condition>,
        payload: Option<Payload>,
        then_case: Option<InnerId>,
        else_case: Option<InnerId>,
        return_case: Option<Payload>,
    ) -> Self {
        Self {
            id,
            name,
            step_type,
            inner_id,
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        }
    }

    pub fn add_then_case(&mut self, then_case: InnerId) {
        self.then_case = Some(then_case);
    }

    pub fn add_else_case(&mut self, else_case: InnerId) {
        self.else_case = Some(else_case);
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
        if let Some(return_case) = self.evaluate_return(context)? {
            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(return_case),
            });
        }

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
        let step = InnerStep::new(
            Some("id".to_string()),
            InnerId::new(),
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
        let step = InnerStep::new(
            None,
            InnerId::new(),
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
        let step = InnerStep::new(
            None,
            InnerId::new(),
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
        let step = InnerStep::new(
            None,
            InnerId::new(),
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
        let step = InnerStep::new(
            None,
            InnerId::new(),
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
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
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

    #[test]
    fn test_step_execute_with_return_case() {
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            None,
            None,
            None,
            None,
            Some(Payload::new("10".to_string())),
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_payload() {
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            None,
            Some(Payload::new("10".to_string())),
            None,
            None,
            Some(Payload::new("20".to_string())),
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(20i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition() {
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            None,
            None,
            None,
            Some(Payload::new("10".to_string())),
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_then_case() {
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            None,
            Some("then_case".to_string()),
            None,
            Some(Payload::new(r#""Ok""#.to_string())),
        );

        let context = Context::new(Value::Null);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_else_case() {
        let step = InnerStep::new(
            None,
            InnerId::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::Equal,
            )),
            None,
            None,
            Some("else_case".to_string()),
            Some(Payload::new(r#""Ok""#.to_string())),
        );

        let context = Context::new(Value::Null);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
