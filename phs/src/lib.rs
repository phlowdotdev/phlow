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
}
