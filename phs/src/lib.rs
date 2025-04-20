mod functions;
mod repositories;
use functions::build_functions;
use repositories::Repositories;
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use valu3::value::Value;

pub fn build_engine(repositories: Option<Repositories>) -> Arc<Engine> {
    let mut engine = build_functions();

    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = match from_dynamic(&dynamic) {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                (call)(value)
            });
        }
    }

    Arc::new(engine)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use valu3::traits::ToValueBehavior;
    use valu3::value::Value;

    #[test]
    fn test_repository_function() {
        let mut repositories = HashMap::new();

        let mock_function: Arc<dyn Fn(Value) -> Value + Send + Sync> = Arc::new(|value| {
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
    fn test_respository_logic() {
        let mut repositories = HashMap::new();

        let mock_function: Arc<dyn Fn(Value) -> Value + Send + Sync> = Arc::new(|value| {
            println!("Received value: {:?}", value);
            if let Value::Object(log) = value {
                let level = if let Some(level) = log.get("level") {
                    if let Value::String(level_str) = level {
                        level_str.to_string()
                    } else {
                        "info".to_string()
                    }
                } else {
                    "info".to_string()
                };

                let message = if let Some(message) = log.get("message") {
                    if let Value::String(message_str) = message {
                        message_str.to_string()
                    } else {
                        "No message".to_string()
                    }
                } else {
                    "No message".to_string()
                };

                format!("Log Level: {}, Message: {}", level, message).to_value()
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
