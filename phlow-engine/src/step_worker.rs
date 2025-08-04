use crate::{
    condition::{Condition, ConditionError},
    context::Context,
    id::ID,
    script::Script,
};
use once_cell::sync::Lazy;
use phlow_sdk::{
    prelude::{log::debug, *},
    tracing::field,
};
use rhai::Engine;
use serde::Serialize;
use std::{fmt::Display, sync::Arc};

static PHLOW_TRUNCATE_SPAN_VALUE: Lazy<usize> =
    Lazy::new(|| match std::env::var("PHLOW_TRUNCATE_SPAN_VALUE") {
        Ok(value) => value.parse::<usize>().unwrap_or(100),
        Err(_) => 100,
    });

#[derive(Debug)]
pub enum StepWorkerError {
    ConditionError(ConditionError),
    PayloadError(phs::ScriptError),
    ModulesError(ModulesError),
    InputError(phs::ScriptError),
}

impl Display for StepWorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepWorkerError::ConditionError(err) => write!(f, "Condition error: {}", err),
            StepWorkerError::PayloadError(err) => write!(f, "Payload error: {}", err),
            StepWorkerError::ModulesError(err) => write!(f, "Modules error: {}", err),
            StepWorkerError::InputError(err) => write!(f, "Input error: {}", err),
        }
    }
}

impl std::error::Error for StepWorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StepWorkerError::ConditionError(err) => Some(err),
            StepWorkerError::PayloadError(_) => None, // ScriptError doesn't implement std::error::Error
            StepWorkerError::ModulesError(_) => None, // ModulesError doesn't implement std::error::Error
            StepWorkerError::InputError(_) => None, // ScriptError doesn't implement std::error::Error
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NextStep {
    Pipeline(usize),
    GoToStep(StepReference),
    Stop,
    Next,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, ToValue)]
pub struct StepReference {
    pub pipeline: usize,
    pub step: usize,
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
    pub(crate) to: Option<StepReference>,
}

impl StepWorker {
    pub fn try_from_value(
        engine: Arc<Engine>,
        modules: Arc<Modules>,
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
                if let Some(condition) = value.get("assert").map(|assert| {
                    Condition::try_build_with_assert(engine.clone(), assert.to_string())
                }) {
                    Some(condition.map_err(StepWorkerError::ConditionError)?)
                } else {
                    None
                }
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

        let to = match value.get("to") {
            Some(to_step) => match to_step.as_object() {
                Some(to_step) => {
                    let pipeline = to_step.get("pipeline").and_then(|v| v.to_u64());
                    let step = to_step.get("step").and_then(|v| v.to_u64());

                    if pipeline.is_some() && step.is_some() {
                        Some(StepReference {
                            pipeline: pipeline.unwrap() as usize,
                            step: step.unwrap() as usize,
                        })
                    } else {
                        None
                    }
                }
                None => None,
            },
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
            to,
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

            match self
                .modules
                .execute(module, &context.input, &context.payload)
                .await
            {
                Ok(response) => {
                    if let Some(err) = response.error {
                        return Err(StepWorkerError::ModulesError(ModulesError::ModuleError(
                            err,
                        )));
                    }

                    Ok(Some((Some(module.clone()), Some(response.data), context)))
                }
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
            context.main = field::Empty,
            context.params = field::Empty,
            context.payload = field::Empty,
            context.input = field::Empty,
            step.id = field::Empty,
            step.label = field::Empty,
            step.module = field::Empty,
            step.condition = field::Empty,
            step.payload = field::Empty,
            step.return = field::Empty,
        );
        let _guard = span.enter();

        {
            let step_name = self.label.clone().unwrap_or(self.id.to_string());
            span.record("otel.name", format!("step {}", step_name));

            if let Some(ref input) = context.input {
                span.record("context.input", input.to_string());
            }

            if let Some(ref payload) = context.payload {
                span.record("context.payload", truncate_string(&payload));
            }

            if let Some(ref main) = context.main {
                span.record("context.main", truncate_string(&main));
            }

            span.record("step.id", self.id.to_string());

            if let Some(ref label) = self.label {
                span.record("step.label", label.to_string());
            }
        }

        if let Some(output) = self.evaluate_return(context)? {
            {
                span.record("step.return", output.to_string());
            }

            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(output),
            });
        }

        if let Some((module, output, context)) = self.evaluate_module(context).await? {
            {
                span.record("step.module", module.clone());

                if let Some(ref output) = output {
                    span.record("context.payload", truncate_string(output));
                }
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

            {
                span.record("step.condition", condition.raw.to_string());

                if let Some(ref output) = output {
                    span.record("context.payload", truncate_string(output));
                }
            }

            return Ok(StepOutput { next_step, output });
        }

        let output = self.evaluate_payload(context, None)?;

        {
            if let Some(ref output) = output {
                span.record("context.payload", truncate_string(output));
            }
        }

        if let Some(to) = &self.to {
            debug!(
                "Define switching to step {} in pipeline {}",
                to.step, to.pipeline
            );
            return Ok(StepOutput {
                next_step: NextStep::GoToStep(to.clone()),
                output,
            });
        }

        return Ok(StepOutput {
            next_step: NextStep::Next,
            output,
        });
    }
}

fn truncate_string(string: &Value) -> String {
    let limit = *PHLOW_TRUNCATE_SPAN_VALUE;
    let string = string.to_string();
    if string.len() > limit {
        format!("{}...", &string[..limit])
    } else {
        string.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use phlow_sdk::valu3;
    use phs::build_engine;
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
        let engine = build_engine(None);
        let step = StepWorker {
            payload: Some(Script::try_build(engine, &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition() {
        let engine = build_engine(None);
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

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Next);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition_then_case() {
        let engine = build_engine(None);
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

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(0));
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_condition_else_case() {
        let engine = build_engine(None);
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

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Pipeline(1));
        assert_eq!(result.output, None);
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            return_case: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_payload() {
        let engine = build_engine(None);
        let step = StepWorker {
            id: ID::new(),
            payload: Some(Script::try_build(engine.clone(), &"10".to_value()).unwrap()),
            return_case: Some(Script::try_build(engine.clone(), &"20".to_value()).unwrap()),
            ..Default::default()
        };

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(20i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition() {
        let engine = build_engine(None);
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

        let context = Context::new();

        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from(10i64)));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition_then_case() {
        let engine = build_engine(None);
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

        let context = Context::new();
        let output = step.execute(&context).await.unwrap();

        assert_eq!(output.next_step, NextStep::Stop);
        assert_eq!(output.output, Some(Value::from("Ok")));
    }

    #[tokio::test]
    async fn test_step_execute_with_return_case_and_condition_else_case() {
        let engine = build_engine(None);
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

        let context = Context::new();
        let result = step.execute(&context).await.unwrap();

        assert_eq!(result.next_step, NextStep::Stop);
        assert_eq!(result.output, Some(Value::from("Ok")));
    }
}
