pub mod functions;
pub mod repositories;
use functions::build_functions;
use repositories::{Repositories, RepositoryFunction};
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
use std::future::Future;
use std::sync::Arc;
use valu3::value::Value;

pub fn build_engine(repositories: Option<Repositories>) -> Arc<Engine> {
    let mut engine = build_functions();

    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            engine.register_fn(resolve_function_name(&key), move |dynamic: Dynamic| {
                let call = call.function.clone();
                let value: Value = from_dynamic(&dynamic).unwrap_or(Value::Null);

                tokio::task::block_in_place(move || {
                    let future = (call)(value);
                    tokio::runtime::Handle::current().block_on(future)
                })
            });
        }
    }

    Arc::new(engine)
}

pub fn resolve_function_name(name: &str) -> String {
    format!("__module_{}", name)
}

pub fn args_to_abstration(name: String, args: Vec<String>) -> String {
    let args = args.join(", ");
    let target = resolve_function_name(&name);

    format!("{}({}){{{}([{}])}}", name, args, target, args)
}

pub fn wrap_async_fn<F, Fut>(
    name: String,
    func: F,
    args: Vec<String>,
) -> repositories::RepositoryFunction
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Value> + Send + 'static,
{
    RepositoryFunction {
        function: Arc::new(move |value| Box::pin(func(value))),
        abstration: args_to_abstration(name, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use valu3::traits::ToValueBehavior;
    use valu3::value::Value;

    #[test]
    fn test_repository_function() {
        let mut repositories = HashMap::new();

        let mock_function = wrap_async_fn(
            "process".to_string(),
            |value: Value| async move {
                if let Value::String(s) = value {
                    Value::from(format!("{}-processed", s))
                } else {
                    Value::Null
                }
            },
            vec!["input".into()],
        );

        repositories.insert("process".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = build_engine(Some(repos));

        let result: Value = engine.eval(r#"process("data")"#).unwrap();

        assert_eq!(result, Value::from("data-processed"));
    }

    #[test]
    fn test_respository_log() {
        let mut repositories = HashMap::new();

        let mock_function = wrap_async_fn(
            "log".into(),
            |value: Value| async move {
                if let Value::Object(log) = value {
                    let level = match log.get("level") {
                        Some(Value::String(s)) => s.to_string(),
                        _ => "info".to_string(),
                    };

                    let message = match log.get("message") {
                        Some(Value::String(s)) => s.to_string(),
                        _ => "No message".to_string(),
                    };

                    Value::from(format!("Log Level: {}, Message: {}", level, message))
                } else {
                    Value::Null
                }
            },
            vec!["message".into(), "level".into()],
        );

        repositories.insert("log".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = build_engine(Some(repos));

        let result: Value = engine
            .eval(r#"log(#{"level": "warn", "message": "data" })"#)
            .unwrap();

        let expected = "Log Level: warn, Message: data".to_value();

        assert_eq!(result, expected);
    }
}
