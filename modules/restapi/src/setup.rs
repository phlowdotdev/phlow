use sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: Option<u16>,
    pub host: Option<String>,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        if value.is_null() {
            return Config {
                port: Some(3000),
                host: Some("0.0.0.0".to_string()),
            };
        }

        let port = match value.get("port") {
            Some(port) => Some(port.to_u64().unwrap() as u16),
            None => Some(3000),
        };

        let host = match value.get("host") {
            Some(host) => Some(host.as_string()),
            None => Some("0.0.0.0".to_string()),
        };

        Config { port, host }
    }
}
