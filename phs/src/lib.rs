pub mod functions;
pub mod preprocessor;
mod repositories;
pub mod script;
pub mod variable;
use functions::build_functions;
pub use repositories::{Repositories, RepositoryFunction};
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine, EvalAltResult, NativeCallContext};
pub use script::{Script, ScriptError};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use valu3::prelude::*;

pub fn build_engine(repositories: Option<Repositories>) -> Arc<Engine> {
    let mut engine = build_functions();

    if let Some(repositories) = repositories {
        for (key, repo) in repositories.repositories {
            let call: Arc<
                dyn Fn(Value) -> std::pin::Pin<Box<dyn Future<Output = Value> + Send>>
                    + Send
                    + Sync,
            > = repo.function.clone();

            let arg_types: Vec<std::any::TypeId> =
                vec![std::any::TypeId::of::<Dynamic>(); repo.args.len()];
            let repo_args = repo.args.clone();
            let call_clone = call.clone();

            let handler_arc: Arc<
                dyn Fn(&NativeCallContext, &mut [&mut Dynamic]) -> Result<Value, Box<EvalAltResult>>
                    + Send
                    + Sync,
            > = Arc::new(move |_context, args| {
                let mut args_map = HashMap::new();

                for dynamic in args.iter() {
                    // dynamic: &mut Dynamic
                    let value: Value = from_dynamic(&*dynamic).unwrap_or(Value::Null);

                    if let Some(key) = repo_args.get(args_map.len()) {
                        args_map.insert(key.clone(), value);
                    }
                }

                // Se o repositório espera múltiplos argumentos, mas recebemos
                // um único argumento que é um objeto, desembrulhe esse objeto
                // e use-o como `args_map`. Isso permite chamadas como
                // fn(a, b) onde o usuário passou um único objeto {a:.., b:..}.
                if repo_args.len() > 1 && args_map.len() == 1 {
                    if let Some((_only_key, only_value_ref)) = args_map.iter().next() {
                        // clone para obter um `Value` owned e poder mover o Object
                        let only_value = only_value_ref.clone();
                        if let Value::Object(obj) = only_value {
                            // substituir args_map pelo conteúdo do objeto
                            args_map = obj
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.clone()))
                                .collect();
                        }
                    }
                }

                let call = call_clone.clone();
                let args_value = args_map.to_value();

                // Try to use the current Tokio runtime if present. If there is no
                // runtime (e.g. running in a synchronous unit test), create a
                // temporary runtime to execute the future. When inside a runtime
                // we use `block_in_place` + `Handle::block_on` to block safely.
                let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let call = call.clone();
                    let args_value = args_value.clone();
                    tokio::task::block_in_place(move || {
                        let future = (call)(args_value);
                        handle.block_on(future)
                    })
                } else {
                    // No runtime available — create a new temporary runtime.
                    tokio::runtime::Runtime::new()
                        .expect("failed to create runtime")
                        .block_on((call)(args_value))
                };

                Ok(result)
            });

            // Register using the full arg list
            {
                let handler_clone = handler_arc.clone();
                engine.register_raw_fn(&key, arg_types, move |c, a| (handler_clone)(&c, a));
            }

            // Register using a single Dynamic argument (fallback)
            {
                let handler_clone = handler_arc.clone();
                engine.register_raw_fn(&key, &[std::any::TypeId::of::<Dynamic>()], move |c, a| {
                    (handler_clone)(&c, a)
                });
            }
        }
    }

    Arc::new(engine)
}

fn args_to_abstration(name: String, args: &Vec<String>) -> String {
    let args = args.join(", ");
    let target = format!("__module_{}", &name);

    format!("{}({}){{{}([{}])}}", name, args, target, args) // name(arg1, arg2){__module_name([arg1, arg2])}
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
        abstration: args_to_abstration(name, &args),
        args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use valu3::value::Value;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_repository_function() {
        let mut repositories = HashMap::new();

        let mock_function = wrap_async_fn(
            "process".to_string(),
            |value: Value| async move {
                // O valor recebido é um objeto com os argumentos mapeados
                if let Value::Object(obj) = value {
                    if let Some(Value::String(s)) = obj.get("input") {
                        Value::from(format!("{}-processed", s))
                    } else {
                        Value::Null
                    }
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
                // `build_engine` maps named args into an object, e.g. {"message": "..."}
                if let Value::Object(obj) = value {
                    let level = obj
                        .get("level")
                        .and_then(|v| Some(v.as_str()))
                        .unwrap_or("info");
                    let message = obj
                        .get("message")
                        .and_then(|v| Some(v.as_str()))
                        .unwrap_or("no message");

                    Value::from(format!("Logged [{}]: {}", level, message))
                } else if let Value::String(s) = value {
                    // fallback: accept a plain string too
                    Value::from(format!("Logged: {}", s))
                } else {
                    Value::from("Logged: unknown")
                }
            },
            vec!["level".into(), "message".into()],
        );

        repositories.insert("log".to_string(), mock_function);

        let repos = Repositories { repositories };
        let engine = build_engine(Some(repos));

        let result: Value = engine.eval(r#"log("info", "message")"#).unwrap();
        assert_eq!(result, Value::from("Logged [info]: message"));

        let result: Value = engine
            .eval(r#"log(#{"level": "warn", "message": "message"})"#)
            .unwrap();
        assert_eq!(result, Value::from("Logged [warn]: message"));
    }
}
