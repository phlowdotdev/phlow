use phlow_sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub server_addr: String,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        if value.is_null() {
            return Config {
                port: 31451,
                server_addr: "127.0.0.1".to_string(),
            };
        }

        let port = match value.get("port") {
            Some(port) => port.to_u64().unwrap_or(3000) as u16,
            None => 31451,
        };

        let server_addr = match value.get("server_addr") {
            Some(addr) => addr.to_string(),
            None => "127.0.0.1".to_string(),
        };

        Config { port, server_addr }
    }
}
