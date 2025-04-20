use std::collections::HashMap;
use std::sync::Arc;
use valu3::value::Value;

pub type RepositoryFunction = Arc<dyn Fn(Value) -> Value + Send + Sync>;

#[derive(Clone)]
pub struct Repositories {
    pub repositories: HashMap<String, RepositoryFunction>,
}
