use regex::Regex;
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine, Map};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use valu3::value::Value;

pub type RepositoryFunction = Arc<dyn Fn(Value) -> Value + Send + Sync>;

pub struct Repositories {
    pub repositories: HashMap<String, RepositoryFunction>,
}

pub fn create_engine(repositories: Option<Repositories>) -> Engine {
    let mut engine = Engine::new();
    let rt = Arc::new(Runtime::new().unwrap()); // Compartilha o runtime

    // Define operadores personalizados
    engine
        .register_custom_operator("starts_with", 80)
        .unwrap()
        .register_fn("start_withs", |x: String, y: String| x.starts_with(&y));

    engine
        .register_custom_operator("ends_with", 81)
        .unwrap()
        .register_fn("ends_with", |x: String, y: String| x.ends_with(&y));

    engine
        .register_custom_operator("search", 82)
        .unwrap()
        .register_fn("search", |x: String, y: String| {
            Regex::new(&x).unwrap().is_match(&y)
        });

    // Registra funções assíncronas dos repositórios
    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            let rt_clone = Arc::clone(&rt); // Clona o runtime
            let call: RepositoryFunction = Arc::new(move |value: Value| -> Value {
                let call_clone = Arc::clone(&call);
                let (tx, rx) = oneshot::channel();

                rt_clone.spawn(async move {
                    let result = (call_clone)(value);
                    let _ = tx.send(result);
                });

                rx.blocking_recv().unwrap_or(Value::Null)
            }) as RepositoryFunction;

            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = from_dynamic(&dynamic).unwrap();
                call(value)
            });
        }
    }

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use valu3::value::Value;

    #[test]
    fn test_custom_operators() {
        let engine = create_engine(None);

        let result: bool = engine.eval(r#""hello" starts_with "he""#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#""world" ends_with "ld""#).unwrap();
        assert!(result);

        let result: bool = engine.eval(r#""\\d+" search "123""#).unwrap();
        assert!(result);
    }

    #[test]
    fn test_repository_function() {
        let mut repositories = HashMap::new();

        let mock_function = Arc::new(|value: Value| -> Value {
            if let Value::String(s) = value {
                Value::from(format!("{}-processed", s))
            } else {
                Value::Null
            }
        }) as RepositoryFunction;

        repositories.insert("process".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = create_engine(Some(repos));

        let result: Value = engine.eval(r#"process("data")"#).unwrap();

        assert_eq!(result, Value::from("data-processed"));
    }
}
