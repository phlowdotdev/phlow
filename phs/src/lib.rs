pub mod functions;
mod repositories;
pub mod script;
pub mod variable;
use functions::build_functions;
pub use repositories::{Repositories, RepositoryFunction};
use rhai::serde::from_dynamic;
use rhai::{Dynamic, Engine};
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

            engine.register_raw_fn(&key, arg_types, move |_context, args| {
                let mut args_map = HashMap::new();

                for dynamic in args {
                    let value: Value = from_dynamic(&dynamic).unwrap_or(Value::Null);

                    if let Some(key) = repo.args.get(args_map.len()) {
                        args_map.insert(key.clone(), value);
                    }
                }

                let call = call.clone();
                let args_value = args_map.to_value();

                let result = tokio::task::block_in_place(move || {
                    let future = (call)(args_value);
                    tokio::runtime::Handle::current().block_on(future)
                });

                Ok(result)
            });
        }
    }

    Arc::new(engine)
}

pub fn resolve_function_name(name: &str) -> String {
    format!("__module_{}", name)
}

pub fn args_to_abstration(name: String, args: &Vec<String>) -> String {
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
        abstration: args_to_abstration(name, &args),
        args,
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
