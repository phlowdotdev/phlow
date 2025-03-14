use regex::Regex;
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use tokio::sync::oneshot;
use valu3::value::Value;

use crate::plugins::{PluginFunction, Plugins};

use std::sync::OnceLock;
use tokio::runtime::Runtime;
fn build_engine() -> Engine {
    let mut engine = Engine::new();

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

    engine
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn build_engine_sync(plugins: Option<Plugins>) -> Engine {
    let mut engine = build_engine();
    let rt = RUNTIME.get_or_init(|| Runtime::new().unwrap());

    if let Some(repositories) = plugins {
        for (key, call) in repositories.plugins {
            let call: PluginFunction = Arc::new(move |value: Value| -> Value {
                let call_clone = Arc::clone(&call);
                let (tx, rx) = oneshot::channel();

                rt.spawn(async move {
                    let result = (call_clone)(value);
                    let _ = tx.send(result);
                });

                rx.blocking_recv().unwrap_or(Value::Null)
            }) as PluginFunction;

            engine.register_fn(key.clone(), move |dynamic: Dynamic| {
                let value: Value = from_dynamic(&dynamic).unwrap();
                call(value)
            });
        }
    }

    engine
}

pub fn build_engine_async(plugins: Option<Plugins>) -> Engine {
    let mut engine = build_engine();

    if let Some(repositories) = plugins {
        for (key, call) in repositories.plugins {
            let call: PluginFunction = Arc::new(move |value: Value| -> Value {
                let call_clone = Arc::clone(&call);
                let (tx, rx) = oneshot::channel();

                // Executa a chamada assíncrona corretamente sem criar um novo runtime
                tokio::task::spawn(async move {
                    let result = (call_clone)(value);
                    let _ = tx.send(result);
                });

                // Usa tokio::runtime::Handle::current() para evitar erro de runtime
                rx.blocking_recv().unwrap_or(Value::Null) // Aguarda sem criar outro runtime
            }) as PluginFunction;

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

        let repos = Plugins {
            plugins: repositories,
        };
        let engine = build_engine_sync(Some(repos));

        let result: Value = engine.eval(r#"process("data")"#).unwrap();

        assert_eq!(result, Value::from("data-processed"));
    }
}
