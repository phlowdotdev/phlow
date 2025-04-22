use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use valu3::value::Value;

pub type Function = Arc<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Value> + Send>> + Send + Sync>;

#[derive(Clone)]
pub struct RepositoryFunction {
    pub function: Function,
    pub abstration: String,
}

#[derive(Clone)]
pub struct Repositories {
    pub repositories: HashMap<String, RepositoryFunction>,
}
