use phlow_sdk::prelude::*;
use crate::{router::Router, openapi::OpenAPIValidator};

#[derive(Clone, Debug)]
pub struct Config {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub router: Router,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        if value.is_null() {
            return Config {
                port: Some(3000),
                host: Some("0.0.0.0".to_string()),
                router: Router::from(Value::Null),
            };
        }

        let port = match value.get("port") {
            Some(port) => Some(port.to_u64().unwrap_or(3000) as u16),
            None => Some(3000),
        };

        let host = match value.get("host") {
            Some(host) => Some(host.as_string()),
            None => Some("0.0.0.0".to_string()),
        };

        let mut router = Router::from(value.clone());
        
        // Try to load OpenAPI validator
        if let Ok(Some(validator)) = OpenAPIValidator::from_value(value.clone()) {
            router.openapi_validator = Some(validator);
            log::info!("OpenAPI validator loaded successfully");
        }

        Config { port, host, router }
    }
}
