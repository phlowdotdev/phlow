use std::collections::HashMap;
use std::sync::Arc;
use valu3::value::Value;

pub type PluginFunction = Arc<dyn Fn(Value) -> Value + Send + Sync>;

#[derive(Clone)]
pub struct Plugins {
    pub plugins: HashMap<String, PluginFunction>,
}

#[macro_export]
macro_rules! plugin {
    ($call:expr) => {
        Arc::new($call) as PluginFunction
    };
}
