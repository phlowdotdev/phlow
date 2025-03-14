use std::collections::HashMap;
use std::sync::Arc;
use valu3::value::Value;

pub type ModuleFunction = Arc<dyn Fn(Value) -> Value + Send + Sync>;

#[derive(Clone)]
pub struct Module {
    pub plugins: HashMap<String, ModuleFunction>,
}

#[macro_export]
macro_rules! plugin {
    ($call:expr) => {
        Arc::new($call) as ModuleFunction
    };
}
