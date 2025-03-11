mod condition;
mod context;
mod id;
mod pipeline;
mod script;
mod step_worker;
mod transform;
mod v8;
mod variable;

pub use rhai::Engine;
pub use v8::V8;

#[macro_export]
macro_rules! v8 {
    ($value:expr) => {
        let engine = $crate::Engine::new();
        V8::try_from_value(&engine, $value, None)
    };
    ($value:expr, $params:expr) => {
        let engine = $crate::Engine::new();
        V8::try_from_value(&engine, $value, Some($params))
    };
}
