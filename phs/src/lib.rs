pub mod functions;
pub mod repositories;
use functions::build_functions;
use repositories::Repositories;
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
use std::future::Future;
use std::sync::Arc;
use valu3::value::Value;

pub fn build_engine(repositories: Option<Repositories>) -> Arc<Engine> {
    let mut engine = build_functions();

    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            let call = call.clone();

            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = from_dynamic(&dynamic).unwrap_or(Value::Null);

                // Wrapper para chamar async dentro do sync
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on((call)(value))
            });
        }
    }

    Arc::new(engine)
}

pub fn wrap_async_fn<F, Fut>(func: F) -> repositories::RepositoryFunction
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Value> + Send + 'static,
{
    Arc::new(move |value| Box::pin(func(value)))
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

        let mock_function = wrap_async_fn(|value: Value| async move {
            if let Value::String(s) = value {
                Value::from(format!("{}-processed", s))
            } else {
                Value::Null
            }
        });

        repositories.insert("process".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = build_engine(Some(repos));

        let result: Value = engine.eval(r#"process("data")"#).unwrap();

        assert_eq!(result, Value::from("data-processed"));
    }

    #[test]
    fn test_respository_log() {
        let mut repositories = HashMap::new();

        let mock_function = wrap_async_fn(|value: Value| async move {
            println!("Received value: {:?}", value);
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
        });

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
