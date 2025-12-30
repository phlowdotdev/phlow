use crate::{
    condition::{Condition, ConditionError},
    context::Context,
    debug::debug_controller,
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
use uuid::Uuid;

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
    LogError(phs::ScriptError),
}

impl Display for StepWorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepWorkerError::ConditionError(err) => write!(f, "Condition error: {}", err),
            StepWorkerError::PayloadError(err) => write!(f, "Payload error: {}", err),
            StepWorkerError::ModulesError(err) => write!(f, "Modules error: {}", err),
            StepWorkerError::InputError(err) => write!(f, "Input error: {}", err),
            StepWorkerError::LogError(err) => write!(f, "Log error: {}", err),
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
            StepWorkerError::LogError(_) => None, // ScriptError doesn't implement std::error::Error
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
    Trace,
}

impl LogLevel {
    fn from_str(level: &str) -> Self {
        match level.to_ascii_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }

    fn log(self, message: &str) {
        match self {
            LogLevel::Info => log::info!("{}", message),
            LogLevel::Debug => log::debug!("{}", message),
            LogLevel::Warn => log::warn!("{}", message),
            LogLevel::Error => log::error!("{}", message),
            LogLevel::Trace => log::trace!("{}", message),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LogStep {
    level: LogLevel,
    message: Option<Script>,
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
    pub(crate) log: Option<LogStep>,
    pub(crate) step_value: Option<Value>,
    #[cfg(debug_assertions)]
    pub(crate) step_raw: String,
}

impl StepWorker {
    pub fn try_from_value(
        engine: Arc<Engine>,
        modules: Arc<Modules>,
        value: &Value,
    ) -> Result<Self, StepWorkerError> {
        log::debug!(
            "Parsing step worker from value: {}",
            value.to_json(JsonMode::Indented)
        );

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
            Some(return_case) => match Script::try_build(engine.clone(), return_case) {
                Ok(return_case) => Some(return_case),
                Err(err) => return Err(StepWorkerError::PayloadError(err)),
            },
            None => None,
        };
        let module = match value.get("use") {
            Some(module) => Some(module.to_string()),
            None => None,
        };
        let log = build_log_step(engine.clone(), value, module.as_deref())?;

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

        let mut step_value = value.clone();
        if Self::should_add_uuid() {
            if let Some(obj) = step_value.as_object_mut() {
                if !obj.contains_key(&"#uuid".to_string()) {
                    obj.insert("#uuid".to_string(), Uuid::new_v4().to_string().to_value());
                }
            }
        }
        #[cfg(debug_assertions)]
        let step_raw = value.to_string();

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
            log,
            step_value: Some(step_value),
            #[cfg(debug_assertions)]
            step_raw,
        })
    }

    pub fn get_id(&self) -> &ID {
        &self.id
    }

    pub(crate) fn compiled_debug(&self) -> Value {
        let mut map = std::collections::HashMap::new();
        if let Some(payload) = &self.payload {
            map.insert("payload".to_string(), payload.compiled_debug());
        }
        if let Some(input) = &self.input {
            map.insert("input".to_string(), input.compiled_debug());
        }
        if let Some(return_case) = &self.return_case {
            map.insert("return".to_string(), return_case.compiled_debug());
        }
        if let Some(condition) = &self.condition {
            map.insert("condition".to_string(), condition.expression.compiled_debug());
        }
        if let Some(log_step) = &self.log {
            if let Some(message) = &log_step.message {
                map.insert("log".to_string(), message.compiled_debug());
            }
        }
        map.to_value()
    }

    fn should_add_uuid() -> bool {
        if debug_controller().is_some() {
            return true;
        }
        std::env::var("PHLOW_DEBUG")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
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

            #[cfg(debug_assertions)]
            log::debug!(
                "Evaluating return case for step {}: {}",
                self.id,
                value.as_ref().map_or("None".to_string(), |v| v.to_string())
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
        if self.module.as_deref() == Some("log") {
            return Ok(None);
        }

        if let Some(ref module) = self.module {
            let input = self.evaluate_input(context)?;

            let context = if let Some(input) = &input {
                context.clone_with_input(input.clone())
            } else {
                context.clone()
            };

            match self
                .modules
                .execute(module, &context.get_input(), &context.get_payload())
                .await
            {
                Ok(response) => {
                    #[cfg(debug_assertions)]
                    log::debug!("Module response for step {}: {:?}", self.id, response);

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
        #[cfg(debug_assertions)]
        log::debug!(
            "Entering step: {}, with: \n\tmain={:?}\n\tpayload={:?}\n\tsetup={:?}",
            self.step_raw,
            &context.get_main().to_value().to_string(),
            &context.get_payload().to_value().to_string(),
            &context.get_setup().to_value().to_string()
        );

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

            if let Some(ref input) = context.get_input() {
                span.record("context.input", input.to_string());
            }

            if let Some(ref payload) = context.get_payload() {
                span.record("context.payload", truncate_string(&payload));
            }

            if let Some(ref main) = context.get_main() {
                span.record("context.main", truncate_string(&main));
            }

            span.record("step.id", self.id.to_string());

            if let Some(ref label) = self.label {
                span.record("step.label", label.to_string());
            }
        }

        if let Some(log_step) = &self.log {
            let message = match &log_step.message {
                Some(script) => script
                    .evaluate(context)
                    .map_err(StepWorkerError::LogError)?
                    .to_string(),
                None => String::new(),
            };
            log_step.level.log(&message);
        }

        if let Some(output) = self.evaluate_return(context)? {
            debug!(
                "[step {}] return case acionado (condicional de parada)",
                self.id
            );
            {
                span.record("step.return", output.to_string());
            }

            return Ok(StepOutput {
                next_step: NextStep::Stop,
                output: Some(output),
            });
        }

        if let Some((module, output, context)) = self.evaluate_module(context).await? {
            debug!(
                "[step {}] módulo '{}' executado; output inicial {:?}",
                self.id,
                module.as_deref().unwrap_or("<none>"),
                output
            );
            {
                span.record("step.module", module.clone());

                if let Some(ref output) = output {
                    span.record("context.payload", truncate_string(output));
                }
            }

            let context = if let Some(output) = output.clone() {
                debug!(
                    "[step {}] definindo output no contexto após execução do módulo",
                    self.id
                );
                context.clone_with_output(output)
            } else {
                context.clone()
            };

            let output = self.evaluate_payload(&context, output)?;

            if let Some(to) = &self.to {
                debug!(
                    "[step {}] condição 'to' detectada após módulo -> pipeline {}, step {}",
                    self.id, to.pipeline, to.step
                );
                debug!(
                    "Define switching to step {} in pipeline {}",
                    to.step, to.pipeline
                );
                return Ok(StepOutput {
                    next_step: NextStep::GoToStep(to.clone()),
                    output,
                });
            }

            debug!("[step {}] seguindo para próximo step após módulo", self.id);
            return Ok(StepOutput {
                next_step: NextStep::Next,
                output,
            });
        }

        if let Some(condition) = &self.condition {
            debug!("[step {}] avaliando condição", self.id);
            let (next_step, output) = if condition
                .evaluate(context)
                .map_err(StepWorkerError::ConditionError)?
            {
                debug!("[step {}] condição verdadeira", self.id);
                let next_step = if let Some(ref then_case) = self.then_case {
                    debug!("[step {}] then_case -> pipeline {}", self.id, then_case);
                    NextStep::Pipeline(*then_case)
                } else {
                    debug!("[step {}] then_case não definido -> Next", self.id);
                    NextStep::Next
                };

                (next_step, self.evaluate_payload(context, None)?)
            } else {
                debug!("[step {}] condição falsa", self.id);
                let next_step = if let Some(ref else_case) = self.else_case {
                    debug!("[step {}] else_case -> pipeline {}", self.id, else_case);
                    NextStep::Pipeline(*else_case)
                } else {
                    debug!("[step {}] else_case não definido -> Next", self.id);
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

        let default_output = if self.log.is_some() {
            context.get_payload()
        } else {
            None
        };
        let output = self.evaluate_payload(context, default_output)?;

        {
            if let Some(ref output) = output {
                span.record("context.payload", truncate_string(output));
            }
        }

        if let Some(to) = &self.to {
            debug!(
                "[step {}] condição 'to' detectada (sem módulo) -> pipeline {}, step {}",
                self.id, to.pipeline, to.step
            );
            debug!(
                "Define switching to step {} in pipeline {}",
                to.step, to.pipeline
            );
            return Ok(StepOutput {
                next_step: NextStep::GoToStep(to.clone()),
                output,
            });
        }

        debug!("[step {}] nenhuma condição especial -> Next", self.id);
        return Ok(StepOutput {
            next_step: NextStep::Next,
            output,
        });
    }
}

fn build_log_step(
    engine: Arc<Engine>,
    value: &Value,
    module: Option<&str>,
) -> Result<Option<LogStep>, StepWorkerError> {
    if let Some(log_step) = extract_log_from_key(engine.clone(), value)? {
        return Ok(Some(log_step));
    }

    if module == Some("log") {
        let input_value = value.get("input");
        let log_step = build_log_from_input(engine, input_value)?;
        return Ok(Some(log_step));
    }

    Ok(None)
}

fn extract_log_from_key(
    engine: Arc<Engine>,
    value: &Value,
) -> Result<Option<LogStep>, StepWorkerError> {
    let Some(obj) = value.as_object() else {
        return Ok(None);
    };

    for (key, value) in obj.iter() {
        let key_str = key.to_string();
        let Some(level_key) = key_str.strip_prefix("log.") else {
            continue;
        };
        let level = level_key.split('.').next().unwrap_or(level_key);
        let level = LogLevel::from_str(level);
        let message_value = if let Some(obj) = value.as_object() {
            obj.get("message").cloned().unwrap_or_else(|| value.clone())
        } else {
            value.clone()
        };
        let message =
            Script::try_build(engine.clone(), &message_value).map_err(StepWorkerError::LogError)?;

        return Ok(Some(LogStep {
            level,
            message: Some(message),
        }));
    }

    Ok(None)
}

fn build_log_from_input(
    engine: Arc<Engine>,
    input_value: Option<&Value>,
) -> Result<LogStep, StepWorkerError> {
    let mut level = LogLevel::Info;
    let mut message_value: Option<Value> = None;

    if let Some(input_value) = input_value {
        if let Some(obj) = input_value.as_object() {
            if let Some(level_value) = obj.get("action").or_else(|| obj.get("level")) {
                level = LogLevel::from_str(level_value.as_string().as_str());
            }

            message_value = obj.get("message").cloned();
        } else {
            message_value = Some(input_value.clone());
        }
    }

    let message = if let Some(message_value) = message_value {
        Some(
            Script::try_build(engine, &message_value).map_err(StepWorkerError::LogError)?,
        )
    } else {
        None
    };

    Ok(LogStep { level, message })
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
}
