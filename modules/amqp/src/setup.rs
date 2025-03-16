use sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct Setup {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub queue_name: Option<String>,
    pub message: Option<String>,
}

impl From<Value> for Setup {
    fn from(value: Value) -> Self {
        Setup {
            host: value.get("host").map(|v| v.as_string()),
            port: value.get("port").map(|v| v.to_u64().unwrap_or(5672) as u16),
            username: value.get("username").map(|v| v.as_string()),
            password: value.get("password").map(|v| v.as_string()),
            queue_name: value.get("queue_name").map(|v| v.as_string()),
            message: value.get("message").map(|v| v.as_string()),
        }
    }
}
