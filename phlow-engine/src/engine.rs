use crate::repositories::{Repositories, RepositoryFunction};
use regex::Regex;
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use valu3::value::Value;

fn build_engine() -> Engine {
    let mut engine = Engine::new();

    // Define operadores personalizados
    match engine.register_custom_operator("starts_with", 80) {
        Ok(engine) => engine.register_fn("start_withs", |x: String, y: String| x.starts_with(&y)),
        Err(_) => {
            panic!("Error on register custom operator starts_with");
        }
    };

    match engine.register_custom_operator("ends_with", 81) {
        Ok(engine) => engine.register_fn("ends_with", |x: String, y: String| x.ends_with(&y)),
        Err(_) => {
            panic!("Error on register custom operator ends_with");
        }
    };

    match engine.register_custom_operator("search", 82) {
        Ok(engine) => engine.register_fn("search", |x: String, y: String| match Regex::new(&x) {
            Ok(re) => re.is_match(&y),
            Err(_) => false,
        }),
        Err(_) => {
            panic!("Error on register custom operator search");
        }
    };

    engine
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn build_engine_sync(repositories: Option<Repositories>) -> Engine {
    let mut engine = build_engine();
    let rt = RUNTIME.get_or_init(|| match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => panic!("Error creating runtime: {:?}", e),
    });

    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            let call: RepositoryFunction = Arc::new(move |value: Value| -> Value {
                let call_clone = Arc::clone(&call);
                let (tx, rx) = oneshot::channel();

                rt.spawn(async move {
                    let result = (call_clone)(value);
                    let _ = tx.send(result);
                });

                rx.blocking_recv().unwrap_or(Value::Null)
            }) as RepositoryFunction;

            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = match from_dynamic(&dynamic) {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                call(value)
            });
        }
    }

    engine
}

pub fn build_engine_async(repositories: Option<Repositories>) -> Arc<Engine> {
    let mut engine = build_engine();

    if let Some(repositories) = repositories {
        for (key, call) in repositories.repositories {
            let call: RepositoryFunction = Arc::new(move |value: Value| -> Value {
                let call_clone = Arc::clone(&call);
                let (tx, rx) = oneshot::channel();

                // Executa a chamada assíncrona corretamente sem criar um novo runtime
                tokio::task::spawn(async move {
                    let result = (call_clone)(value);
                    let _ = tx.send(result);
                });

                // Usa tokio::runtime::Handle::current() para evitar erro de runtime
                rx.blocking_recv().unwrap_or(Value::Null) // Aguarda sem criar outro runtime
            }) as RepositoryFunction;

            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = match from_dynamic(&dynamic) {
                    Ok(value) => value,
                    Err(_) => Value::Null,
                };
                call(value)
            });
        }
    }

    Arc::new(engine)
}

#[cfg(test)]
mod tests {
    use crate::plugin;

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use valu3::value::Value;

    #[test]
    fn test_custom_operators() {
        let engine = build_engine_async(None);

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

        let mock_function: Arc<dyn Fn(Value) -> Value + Send + Sync> = plugin!(|value| {
            if let Value::String(s) = value {
                Value::from(format!("{}-processed", s))
            } else {
                Value::Null
            }
        });

        repositories.insert("process".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = build_engine_sync(Some(repos));

        let result: Value = engine.eval(r#"process("data")"#).unwrap();

        assert_eq!(result, Value::from("data-processed"));
    }
}
