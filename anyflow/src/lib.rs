mod anyflow;
mod condition;
mod context;
mod id;
mod pipeline;
mod script;
mod step_worker;
mod transform;
mod variable;

pub use anyflow::AnyFlow;
pub use rhai::Engine;

#[macro_export]
macro_rules! anyflow {
    ($value:expr) => {
        let engine = $crate::Engine::new();
        anyflow::try_from_value(&engine, $value, None)
    };
    ($value:expr, $params:expr) => {
        let engine = $crate::Engine::new();
        anyflow::try_from_value(&engine, $value, Some($params))
    };
}
