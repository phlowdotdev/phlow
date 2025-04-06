use crate::{
    collector::{ContextSender, Step},
    condition::{Condition, ConditionError},
    context::Context,
    id::ID,
    script::{Script, ScriptError},
};
use phlow_sdk::{prelude::*, valu3};
use rhai::Engine;
use serde::Serialize;
use std::sync::Arc;
use valu3::prelude::NumberBehavior;
use valu3::{prelude::StringBehavior, value::Value};

#[derive(Debug)]
pub enum StepWorkerError {
    ConditionError(ConditionError),
    PayloadError(ScriptError),
    ModulesError(ModulesError),
    InputError(ScriptError),
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
pub struct StepWorker {
    pub(crate) id: ID,
    pub(crate) label: Option<String>,
    pub(crate) module: Option<String>,
    pub(crate) condition: Option<Condition>,
    pub(crate) input: Option<Script>,
    pub(crate) payload: Option<Script>,
    pub(crate) then_case: Option<usize>,
    pub(crate) else_case: Option<usize>,
    pub(crate) modules: Arc<Modules>,
    pub(crate) return_case: Option<Script>,
    pub(crate) trace_sender: Option<ContextSender>,
}

impl StepWorker {
    pub fn try_from_value(
        engine: Arc<Engine>,
        modules: Arc<Modules>,
        trace_sender: Option<ContextSender>,
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
                .map(|condition| Condition::try_from_value(engine.clone(), condition))
            {
                Some(condition.map_err(StepWorkerError::ConditionError)?)
            } else {
                None
            }
        };
        let payload = match value.get("payload") {
            Some(payload) => match Script::try_build(engine.clone(), payload) {
                Ok(payload) => Some(payload),
                Err(err) => return Err(StepWorkerError::PayloadError(err)),
            },
            None => None,
        };
        let input = match value.get("input") {
            Some(input) => match Script::try_build(engine.clone(), input) {
                Ok(input) => Some(input),
                Err(err) => return Err(StepWorkerError::InputError(err)),
            },
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
            Some(return_case) => match Script::try_build(engine, return_case) {
                Ok(return_case) => Some(return_case),
                Err(err) => return Err(StepWorkerError::PayloadError(err)),
            },
            None => None,
        };
        let module = match value.get("use") {
            Some(module) => Some(module.to_string()),
            None => None,
        };

        Ok(Self {
            id,
            label,
            module,
            input,
            condition,
            payload,
            then_case,
            else_case,
            modules,
            return_case,
            trace_sender,
        })
    }

    pub fn get_id(&self) -> &ID {
        &self.id
    }

    fn evaluate_payload(
        &self,
        context: &Context,
        default: Option<Value>,
    ) -> Result<Option<Value>, StepWorkerError> {
        if let Some(ref payload) = self.payload {
            let value = Some(
                payload
                    .evaluate(context)
                    .map_err(StepWorkerError::PayloadError)?,
            );
            Ok(value)
        } else {
            Ok(default)
        }
    }

    fn evaluate_input(&self, context: &Context) -> Result<Option<Value>, StepWorkerError> {
        if let Some(ref input) = self.input {
            let value = Some(
                input
                    .evaluate(context)
                    .map_err(StepWorkerError::InputError)?,
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

    async fn evaluate_module(
        &self,
        context: &Context,
    ) -> Result<Option<(Option<String>, Option<Value>, Context)>, StepWorkerError> {
        if let Some(ref module) = self.module {
            let input = self.evaluate_input(context)?;

            let context = if let Some(input) = &input {
                context.add_module_input(input.clone())
            } else {
                context.clone()
            };

            match self.modules.execute(module, &context).await {
                Ok(value) => Ok(Some((Some(module.clone()), Some(value), context))),
                Err(err) => Err(StepWorkerError::ModulesError(err)),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn execute(&self, context: &Context) -> Result<StepOutput, StepWorkerError> {
        let span = tracing::info_span!(
            "step",
            otel.name = field::Empty,
            params = field::Empty,
            id = field::Empty,
            payload = field::Empty,
            input = field::Empty,
        );
        let _guard = span.enter();

        {
            let step_name = self.label.clone().unwrap_or(self.id.to_string());
            span.record("otel.name", format!("step {}", step_name));
        }

        if let Some(output) = self.evaluate_return(context)? {
            if let Some(sender) = &self.trace_sender {
                sender_safe!(
                    sender,
                    Step {
                        id: self.id.clone(),
                        label: self.label.clone(),
                        input: None,
                        module: None,
                        condition: None,
                        payload: None,
                        return_case: Some(output.clone()),
                    }
                );
            }

            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(output),
            });
        }

        if let Ok(Some((module, output, context))) = self.evaluate_module(context).await {
            if let Some(sender) = &self.trace_sender {
                sender_safe!(
                    sender,
                    Step {
                        id: self.id.clone(),
                        label: self.label.clone(),
                        input: context.input.clone(),
                        module,
                        condition: None,
                        payload: output.clone(),
                        return_case: None,
                    }
                );
            }

            let context = if let Some(output) = output.clone() {
                context.add_module_output(output)
            } else {
                context.clone()
            };

            return Ok(StepOutput {
                next_step: NextStep::Next,
                output: self.evaluate_payload(&context, output)?,
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

                (next_step, self.evaluate_payload(context, None)?)
            } else {
                let next_step = if let Some(ref else_case) = self.else_case {
                    NextStep::Pipeline(*else_case)
                } else {
                    NextStep::Next
                };

                (next_step, None)
            };

            if let Some(sender) = &self.trace_sender {
                sender_safe!(
                    sender,
                    Step {
                        id: self.id.clone(),
                        label: self.label.clone(),
                        module: None,
                        input: None,
                        condition: Some(condition.raw.clone()),
                        payload: output.clone(),
                        return_case: None,
                    }
                );
            }

            return Ok(StepOutput { next_step, output });
        }

        let output = self.evaluate_payload(context, None)?;

        if let Some(sender) = &self.trace_sender {
            sender_safe!(
                sender,
                Step {
                    id: self.id.clone(),
                    label: self.label.clone(),
                    module: None,
                    input: None,
                    condition: None,
                    payload: output.clone(),
                    return_case: None,
                }
            );
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            output,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::engine::build_engine_async;

    use super::*;
    use valu3::prelude::ToValueBehavior;
    use valu3::value::Value;

    #[tokio::test]
    async fn test_step_get_reference_id() {
        let step = StepWorker {
            id: ID::from("id"),
            label: Some("label".to_string()),
            ..Default::default()
        };

        assert_eq!(step.get_id(), &ID::from("id"));
    }

    #[tokio::test]
    async fn test_step_execute() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            payload: Some(Script::try_build(engine, &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::NotEqual,
                )
                .unwrap(),
            ),
            payload: Some(Script::try_build(engine, &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition_then_case() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::NotEqual,
                )
                .unwrap(),
            ),
            payload: Some(Script::try_build(engine, &"10".to_value()).unwrap()),
            then_case: Some(0),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(0));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition_else_case() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::Equal,
                )
                .unwrap(),
            ),
            payload: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            else_case: Some(1),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(1));
        assert_eq!(result.output, None);
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            return_case: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_payload() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            payload: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            return_case: Some(Script::try_build(engine.clone(), &"20".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(20i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::Equal,
                )
                .unwrap(),
            ),
            return_case: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition_then_case() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::Equal,
                )
                .unwrap(),
            ),
            then_case: Some(0),
            return_case: Some(Script::try_build(engine.clone(), &"Ok".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);
        let output = step.execute(&context).await.unwrap();

        assert_eq!(output.next_step, NextStep::Stop);
        assert_eq!(output.output, Some(Value::from("Ok")));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition_else_case() {
        let engine = build_engine_async(None);
        let step = StepWorker {
            id: ID::new(),
            condition: Some(
                Condition::try_build_with_operator(
                    engine.clone(),
                    "10".to_string(),
                    "20".to_string(),
                    crate::condition::Operator::Equal,
                )
                .unwrap(),
            ),
            else_case: Some(0),
            return_case: Some(Script::try_build(engine.clone(), &"Ok".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new(None);
        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
