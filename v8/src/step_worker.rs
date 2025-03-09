use crate::{
    condition::{Condition, ConditionError},
    payload::Payload,
    pipeline::{Context, Step},
    Error,
};
use serde::Serialize;
use std::fmt::Display;
use valu3::{prelude::StringBehavior, value::Value};

#[derive(Debug, Clone, PartialEq, Serialize, Eq, Hash)]
pub struct ID(String);

impl ID {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl From<String> for ID {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&Value> for ID {
    fn from(id: &Value) -> Self {
        Self(id.to_string())
    }
}

impl From<Value> for ID {
    fn from(id: Value) -> Self {
        Self(id.to_string())
    }
}

impl From<&String> for ID {
    fn from(id: &String) -> Self {
        Self(id.clone())
    }
}

impl From<&str> for ID {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}

pub type Output = Value;

#[derive(Debug)]
pub enum StepWorkerError {
    ConditionError(ConditionError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NextStep {
    Step(ID),
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

impl Default for StepType {
    fn default() -> Self {
        StepType::Default
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct StepWorker {
    pub(crate) id: Option<ID>,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) worker_id: ID,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<ID>,
    pub(crate) else_case: Option<ID>,
    pub(crate) return_case: Option<Payload>,
}

impl TryFrom<&Value> for StepWorker {
    type Error = StepWorkerError;

    fn try_from(value: &Value) -> Result<Self, StepWorkerError> {
        let id: Option<ID> = match value.get("id") {
            Some(id) => Some(ID::from(id.to_string())),
            None => None,
        };
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
                Some(condition.map_err(StepWorkerError::ConditionError)?)
            } else {
                None
            }
        };

        let payload = value
            .get("payload")
            .map(|payload| Payload::new(payload.to_string()));
        let then_case = match value.get("then_case") {
            Some(then_case) => Some(ID::from(then_case.to_string())),
            None => None,
        };
        let else_case = match value.get("else_case") {
            Some(else_case) => Some(ID::from(else_case.to_string())),
            None => None,
        };
        let return_case = match value.get("return_case") {
            Some(return_case) => Some(Payload::new(return_case.to_string())),
            None => None,
        };

        Ok(Self {
            id,
            name,
            step_type,
            worker_id: ID::new(),
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        })
    }
}

impl From<Step> for StepWorker {
    fn from(step: Step) -> Self {
        Self {
            id: if step.id.is_some() {
                Some(ID::from(step.id.unwrap()))
            } else {
                None
            },
            name: step.name,
            step_type: step.step_type,
            worker_id: ID::new(),
            condition: step.condition,
            payload: step.payload,
            then_case: None,
            else_case: None,
            return_case: step.return_case,
        }
    }
}

impl StepWorker {
    pub fn new(
        id: Option<ID>,
        worker_id: ID,
        name: Option<String>,
        step_type: StepType,
        condition: Option<Condition>,
        payload: Option<Payload>,
        then_case: Option<ID>,
        else_case: Option<ID>,
        return_case: Option<Payload>,
    ) -> Self {
        Self {
            id,
            name,
            step_type,
            worker_id,
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        }
    }

    pub fn add_then_case(&mut self, then_case: ID) {
        self.then_case = Some(then_case);
    }

    pub fn add_else_case(&mut self, else_case: ID) {
        self.else_case = Some(else_case);
    }

    pub fn get_reference_id(&self) -> &ID {
        match self.id {
            Some(ref id) => id,
            None => &self.worker_id,
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
        let step = StepWorker::new(
            Some(ID::from("id")),
            ID::new(),
            Some("name".to_string()),
            StepType::Default,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(step.get_reference_id(), &ID::from("id"));
    }

    #[test]
    fn test_step_get_reference_id_without_id() {
        let step = StepWorker::new(
            None,
            ID::new(),
            Some("name".to_string()),
            StepType::Default,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(step.get_reference_id(), &step.worker_id);
    }

    #[test]
    fn test_step_execute() {
        let step = StepWorker::new(
            None,
            ID::new(),
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
        let step = StepWorker::new(
            None,
            ID::new(),
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
        let step = StepWorker::new(
            None,
            ID::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::NotEqual,
            )),
            Some(Payload::new("10".to_string())),
            Some(ID::from("then_case")),
            None,
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step(ID::from("then_case")));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_else_case() {
        let step = StepWorker::new(
            None,
            ID::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::Equal,
            )),
            Some(Payload::new("10".to_string())),
            None,
            Some(ID::from("else_case")),
            None,
        );

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step(ID::from("else_case")));
        assert_eq!(result.output, None);
    }

    #[test]
    fn test_step_execute_with_return_case() {
        let step = StepWorker::new(
            None,
            ID::new(),
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
        let step = StepWorker::new(
            None,
            ID::new(),
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
        let step = StepWorker::new(
            None,
            ID::new(),
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
        let step = StepWorker::new(
            None,
            ID::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            None,
            Some(ID::from("then_case")),
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
        let step = StepWorker::new(
            None,
            ID::new(),
            None,
            StepType::Default,
            Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::Equal,
            )),
            None,
            None,
            Some(ID::from("else_case")),
            Some(Payload::new(r#""Ok""#.to_string())),
        );

        let context = Context::new(Value::Null);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
