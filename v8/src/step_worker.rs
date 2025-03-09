use crate::{
    condition::{Condition, ConditionError},
    payload::Payload,
    Error,
};
use serde::Serialize;
use std::{collections::HashMap, fmt::Display};
use valu3::{prelude::StringBehavior, value::Value};

#[derive(Debug, Clone, PartialEq, Serialize, Eq, Hash)]
pub struct ID(Option<String>);

impl ID {
    pub fn new() -> Self {
        Self(None)
    }
}

impl From<String> for ID {
    fn from(id: String) -> Self {
        Self(Some(id))
    }
}

impl From<&Value> for ID {
    fn from(value: &Value) -> Self {
        Self::from(value.to_string())
    }
}

impl From<Value> for ID {
    fn from(value: Value) -> Self {
        Self::from(value.to_string())
    }
}

impl From<&String> for ID {
    fn from(id: &String) -> Self {
        Self::from(id.to_string())
    }
}

impl From<&str> for ID {
    fn from(id: &str) -> Self {
        Self::from(id.to_string())
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.0.is_some() {
                self.0.as_ref().unwrap()
            } else {
                ""
            }
        )
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Context {
    pub(crate) params: Value,
    pub(crate) steps: HashMap<ID, Output>,
}

impl Context {
    pub fn new(params: Value) -> Self {
        Self {
            params,
            steps: HashMap::new(),
        }
    }

    pub fn add_step_output(&mut self, step: &StepWorker, output: Output) {
        self.steps.insert(step.get_id().clone(), output);
    }

    pub fn get_step_output(&self, step: &StepWorker) -> Option<&Output> {
        self.steps.get(&step.get_id())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct StepWorker {
    pub(crate) id: ID,
    pub(crate) params: Option<Value>,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<Payload>,
    pub(crate) then_case: Option<ID>,
    pub(crate) else_case: Option<ID>,
    pub(crate) return_case: Option<Payload>,
}

impl StepWorker {
    pub fn add_then_case(&mut self, then_case: ID) {
        self.then_case = Some(then_case);
    }

    pub fn add_else_case(&mut self, else_case: ID) {
        self.else_case = Some(else_case);
    }

    pub fn get_id(&self) -> &ID {
        &self.id
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

impl TryFrom<&Value> for StepWorker {
    type Error = StepWorkerError;

    fn try_from(value: &Value) -> Result<Self, StepWorkerError> {
        let id = match value.get("id") {
            Some(id) => ID::from(id),
            None => ID::new(),
        };
        let name = match value.get("name") {
            Some(name) => Some(name.as_string()),
            None => None,
        };
        let request = value.get("request").cloned();
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
        let payload = match value.get("payload") {
            Some(payload) => Some(Payload::new(payload.to_string())),
            None => None,
        };
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

        let step_type = if then_case.is_some() {
            StepType::ThenCase
        } else if else_case.is_some() {
            StepType::ElseCase
        } else {
            StepType::Default
        };

        Ok(Self {
            id,
            params: request,
            name,
            step_type,
            condition,
            payload,
            then_case,
            else_case,
            return_case,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use valu3::value::Value;

    #[test]
    fn test_step_get_reference_id() {
        let step = StepWorker {
            id: ID::from("id"),
            name: Some("name".to_string()),
            ..Default::default()
        };

        assert_eq!(step.get_id(), &ID::from("id"));
    }

    #[test]
    fn test_step_execute() {
        let step = StepWorker {
            payload: Some(Payload::new("10".to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::NotEqual,
            )),
            payload: Some(Payload::new("10".to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_then_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::NotEqual,
            )),
            payload: Some(Payload::new("10".to_string())),
            then_case: Some(ID::from("then_case")),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step(ID::from("then_case")));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_else_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::Equal,
            )),
            payload: Some(Payload::new("10".to_string())),
            else_case: Some(ID::from("else_case")),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Step(ID::from("else_case")));
        assert_eq!(result.output, None);
    }

    #[test]
    fn test_step_execute_with_return_case() {
        let step = StepWorker {
            id: ID::new(),
            return_case: Some(Payload::new("10".to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_payload() {
        let step = StepWorker {
            id: ID::new(),
            payload: Some(Payload::new("10".to_string())),
            return_case: Some(Payload::new("20".to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(20i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            return_case: Some(Payload::new("10".to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_then_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("10".to_string()),
                crate::condition::Operator::Equal,
            )),
            then_case: Some(ID::from("then_case")),
            return_case: Some(Payload::new(r#""Ok""#.to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_else_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                Payload::new("10".to_string()),
                Payload::new("20".to_string()),
                crate::condition::Operator::Equal,
            )),
            else_case: Some(ID::from("else_case")),
            return_case: Some(Payload::new(r#""Ok""#.to_string())),
            ..Default::default()
        };

        let context = Context::new(Value::Null);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
