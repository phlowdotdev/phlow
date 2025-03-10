use crate::{
    condition::{Condition, ConditionError},
    id::ID,
    script::{Script, ScriptError},
    v8::Context,
};
use serde::Serialize;
use valu3::prelude::NumberBehavior;
use valu3::{prelude::StringBehavior, value::Value, Error as ValueError};

#[derive(Debug)]
pub enum StepWorkerError {
    ConditionError(ConditionError),
    PipelineNotFound,
    JsonParseError(ValueError),
    InvalidPipeline(ID),
    InvalidCondition,
    InvalidStep(ID),
    PayloadError(ScriptError),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NextStep {
    Pipeline(usize),
    Stop,
    Next,
}

#[derive(Debug)]
pub struct StepOutput {
    pub next_step: NextStep,
    pub payload: Option<Value>,
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
    pub(crate) id: ID,
    pub(crate) name: Option<String>,
    pub(crate) step_type: StepType,
    pub(crate) condition: Option<Condition>,
    pub(crate) payload: Option<Script>,
    pub(crate) then_case: Option<usize>,
    pub(crate) else_case: Option<usize>,
    pub(crate) return_case: Option<Script>,
}

impl StepWorker {
    pub fn add_then_case(&mut self, then_case: usize) {
        self.then_case = Some(then_case);
    }

    pub fn add_else_case(&mut self, else_case: usize) {
        self.else_case = Some(else_case);
    }

    pub fn get_id(&self) -> &ID {
        &self.id
    }

    fn evaluate_payload(&self, context: &Context) -> Result<Option<Value>, StepWorkerError> {
        if let Some(ref payload) = self.payload {
            let value = Some(
                payload
                    .evaluate(context)
                    .map_err(StepWorkerError::PayloadError)?,
            );
            Ok(value)
        } else {
            Ok(None)
        }
    }

    fn evaluate_return(&self, context: &Context) -> Result<Option<Value>, StepWorkerError> {
        if let Some(ref return_case) = self.return_case {
            let value = Some(
                return_case
                    .evaluate(context)
                    .map_err(StepWorkerError::PayloadError)?,
            );
            Ok(value)
        } else {
            Ok(None)
        }
    }

    pub fn execute(&self, context: &Context) -> Result<StepOutput, StepWorkerError> {
        if let Some(return_case) = self.evaluate_return(context)? {
            return Ok(StepOutput {
                next_step: NextStep::Stop,
                payload: Some(return_case),
            });
        }

        if let Some(condition) = &self.condition {
            let (next_step, payload) = if condition
                .evaluate(context)
                .map_err(StepWorkerError::ConditionError)?
            {
                let next_step = if let Some(ref then_case) = self.then_case {
                    NextStep::Pipeline(*then_case)
                } else {
                    NextStep::Next
                };

                (next_step, self.evaluate_payload(context)?)
            } else {
                let next_step = if let Some(ref else_case) = self.else_case {
                    NextStep::Pipeline(*else_case)
                } else {
                    NextStep::Stop
                };

                (next_step, None)
            };

            return Ok(StepOutput { next_step, payload });
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            payload: self.evaluate_payload(context)?,
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
            Some(payload) => Some(Script::from(payload)),
            None => None,
        };
        let then_case = match value.get("then") {
            Some(then_case) => match then_case.to_u64() {
                Some(then_case) => Some(then_case as usize),
                None => None,
            },
            None => None,
        };
        let else_case = match value.get("else") {
            Some(else_case) => match else_case.to_u64() {
                Some(else_case) => Some(else_case as usize),
                None => None,
            },
            None => None,
        };
        let return_case = match value.get("return") {
            Some(return_case) => Some(Script::from(return_case)),
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
    use valu3::prelude::ToValueBehavior;
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
            payload: Some(Script::from("10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.payload, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::NotEqual,
            ))),
            payload: Some(Script::from("10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.payload, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_then_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::NotEqual,
            ))),
            payload: Some(Script::from("10".to_value())),
            then_case: Some(0),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(0));
        assert_eq!(result.payload, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_else_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            ))),
            payload: Some(Script::from("10".to_value())),
            else_case: Some(1),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(1));
        assert_eq!(result.payload, None);
    }

    #[test]
    fn test_step_execute_with_return_case() {
        let step = StepWorker {
            id: ID::new(),
            return_case: Some(Script::from("10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.payload, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_payload() {
        let step = StepWorker {
            id: ID::new(),
            payload: Some(Script::from("10".to_value())),
            return_case: Some(Script::from("20".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.payload, Some(Value::from(20i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            ))),
            return_case: Some(Script::from("10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.payload, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_then_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            ))),
            then_case: Some(0),
            return_case: Some(Script::from(r#""Ok""#.to_value())),
            ..Default::default()
        };

        let context = Context::new(None);
        let output = step.execute(&context).unwrap();

        assert_eq!(output.next_step, NextStep::Stop);
        assert_eq!(output.payload, Some(Value::from("Ok")));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_else_case() {
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::from((
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            ))),
            else_case: Some(0),
            return_case: Some(Script::from(r#""Ok""#.to_value())),
            ..Default::default()
        };

        let context = Context::new(None);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.payload, Some(Value::from("Ok")));
    }
}
