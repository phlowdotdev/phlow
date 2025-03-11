use crate::{
    collector::{ContextSender, Step},
    condition::{Condition, ConditionError},
    context::Context,
    id::ID,
    script::{Script, ScriptError},
};
use rhai::Engine;
use serde::Serialize;
use valu3::prelude::NumberBehavior;
use valu3::{prelude::StringBehavior, value::Value};

#[derive(Debug)]
pub enum StepWorkerError {
    ConditionError(ConditionError),
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
    pub output: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct StepWorker<'a> {
    pub(crate) id: ID,
    pub(crate) label: Option<String>,
    pub(crate) condition: Option<Condition<'a>>,
    pub(crate) payload: Option<Script<'a>>,
    pub(crate) then_case: Option<usize>,
    pub(crate) else_case: Option<usize>,
    pub(crate) return_case: Option<Script<'a>>,
    pub(crate) sender: Option<ContextSender>,
}

impl<'a> StepWorker<'a> {
    pub fn try_from_value(
        engine: &'a Engine,
        sender: Option<ContextSender>,
        value: &Value,
    ) -> Result<Self, StepWorkerError> {
        let id = match value.get("id") {
            Some(id) => ID::from(id),
            None => ID::new(),
        };
        let label: Option<String> = match value.get("label") {
            Some(label) => Some(label.as_string()),
            None => None,
        };
        let condition = {
            if let Some(condition) = value
                .get("condition")
                .map(|condition| Condition::try_from_value(engine, condition))
            {
                Some(condition.map_err(StepWorkerError::ConditionError)?)
            } else {
                None
            }
        };
        let payload = match value.get("payload") {
            Some(payload) => Some(Script::new(&engine, payload)),
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
            Some(return_case) => Some(Script::new(&engine, return_case)),
            None => None,
        };

        Ok(Self {
            id,
            label,
            condition,
            payload,
            then_case,
            else_case,
            return_case,
            sender,
        })
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
            if let Some(sender) = &self.sender {
                sender
                    .send(Step {
                        id: self.id.clone(),
                        label: self.label.clone(),
                        condition: None,
                        payload: None,
                        return_case: Some(return_case.clone()),
                    })
                    .unwrap();
            }

            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(return_case),
            });
        }

        if let Some(condition) = &self.condition {
            let (next_step, output) = if condition
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

            if let Some(sender) = &self.sender {
                sender
                    .send(Step {
                        id: self.id.clone(),
                        label: self.label.clone(),
                        condition: Some(condition.raw.clone()),
                        payload: output.clone(),
                        return_case: None,
                    })
                    .unwrap();
            }

            return Ok(StepOutput { next_step, output });
        }

        let output = self.evaluate_payload(context)?;

        if let Some(sender) = &self.sender {
            sender
                .send(Step {
                    id: self.id.clone(),
                    label: self.label.clone(),
                    condition: None,
                    payload: output.clone(),
                    return_case: None,
                })
                .unwrap();
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            output,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::engine::build_engine;

    use super::*;
    use valu3::prelude::ToValueBehavior;
    use valu3::value::Value;

    #[test]
    fn test_step_get_reference_id() {
        let step = StepWorker {
            id: ID::from("id"),
            label: Some("label".to_string()),
            ..Default::default()
        };

        assert_eq!(step.get_id(), &ID::from("id"));
    }

    #[test]
    fn test_step_execute() {
        let engine = build_engine(None);
        let step = StepWorker {
            payload: Some(Script::new(&engine, &"10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::NotEqual,
            )),
            payload: Some(Script::new(&engine, &"10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_then_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::NotEqual,
            )),
            payload: Some(Script::new(&engine, &"10".to_value())),
            then_case: Some(0),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(0));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_condition_else_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            )),
            payload: Some(Script::new(&engine, &"10".to_value())),
            else_case: Some(1),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(1));
        assert_eq!(result.output, None);
    }

    #[test]
    fn test_step_execute_with_return_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            return_case: Some(Script::new(&engine, &"10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_payload() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            payload: Some(Script::new(&engine, &"10".to_value())),
            return_case: Some(Script::new(&engine, &"20".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(20i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            )),
            return_case: Some(Script::new(&engine, &"10".to_value())),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_then_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            )),
            then_case: Some(0),
            return_case: Some(Script::new(&engine, &r#""Ok""#.to_value())),
            ..Default::default()
        };

        let context = Context::new(None);
        let output = step.execute(&context).unwrap();

        assert_eq!(output.next_step, NextStep::Stop);
        assert_eq!(output.output, Some(Value::from("Ok")));
    }

    #[test]
    fn test_step_execute_with_return_case_and_condition_else_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(Condition::new(
                &engine,
                "10".to_string(),
                "20".to_string(),
                crate::condition::Operator::Equal,
            )),
            else_case: Some(0),
            return_case: Some(Script::new(&engine, &r#""Ok""#.to_value())),
            ..Default::default()
        };

        let context = Context::new(None);
        let result = step.execute(&context).unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
