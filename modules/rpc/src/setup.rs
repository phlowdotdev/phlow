use phlow_sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidPort,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "Invalid address format"),
            Self::InvalidPort => write!(f, "Invalid port number"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
    pub max_connections: usize,
    pub service_name: String,
}

impl Config {
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let host = value
            .get("host")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let port = value
            .get("port")
            .and_then(|v| v.to_i64())
            .map(|v| v as u16)
            .unwrap_or(8080);

        let timeout_ms = value
            .get("timeout_ms")
            .and_then(|v| v.to_i64())
            .map(|v| v as u64)
            .unwrap_or(5000);

        let max_connections = value
            .get("max_connections")
            .and_then(|v| v.to_i64())
            .map(|v| v as usize)
            .unwrap_or(100);

        let service_name = value
            .get("service_name")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "default".to_string());

        Ok(Self {
            host,
            port,
            timeout_ms,
            max_connections,
            service_name,
        })
    }
}
