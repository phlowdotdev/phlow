use crate::{openapi::OpenAPIValidator, router::Router};
use phlow_sdk::prelude::*;

#[derive(Clone, Debug)]
pub struct CorsConfig {
    pub origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub credentials: bool,
    pub max_age: u32,
}

impl From<Value> for CorsConfig {
    fn from(value: Value) -> Self {
        let origins = if let Some(origins) = value.get("origins").and_then(|v| v.as_array()) {
            let parsed_origins: Vec<String> = origins.values.iter()
                .map(|v| v.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed_origins.is_empty() {
                parsed_origins
            } else {
                vec!["*".to_string()]
            }
        } else {
            vec!["*".to_string()]
        };

        let methods = if let Some(methods) = value.get("methods").and_then(|v| v.as_array()) {
            let parsed_methods: Vec<String> = methods.values.iter()
                .map(|v| v.to_string().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed_methods.is_empty() {
                parsed_methods
            } else {
                vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), 
                     "PATCH".to_string(), "DELETE".to_string(), "OPTIONS".to_string()]
            }
        } else {
            vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), 
                 "PATCH".to_string(), "DELETE".to_string(), "OPTIONS".to_string()]
        };

        let headers = if let Some(headers) = value.get("headers").and_then(|v| v.as_array()) {
            let parsed_headers: Vec<String> = headers.values.iter()
                .map(|v| v.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed_headers.is_empty() {
                parsed_headers
            } else {
                vec!["Content-Type".to_string(), "Authorization".to_string(), "X-Requested-With".to_string()]
            }
        } else {
            vec!["Content-Type".to_string(), "Authorization".to_string(), "X-Requested-With".to_string()]
        };

        let credentials = value.get("credentials")
            .and_then(|v| v.as_bool())
            .map(|b| *b)
            .unwrap_or(true);

        let max_age = value.get("max_age")
            .and_then(|v| v.to_u64())
            .unwrap_or(86400) as u32;

        let mut config = Self {
            origins,
            methods,
            headers,
            credentials,
            max_age,
        };

        // Security validation: cannot use wildcard origins with credentials
        if config.credentials && config.origins.contains(&"*".to_string()) {
            log::warn!(
                "CORS Security Warning: Cannot use wildcard origins (*) with credentials=true. \
                This violates CORS specification and may be rejected by browsers. \
                Setting credentials=false to allow wildcard origins."
            );
            config.credentials = false;
        }

        config
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub router: Router,
    pub cors: Option<CorsConfig>,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        if value.is_null() {
            return Config {
                port: Some(3000),
                host: Some("0.0.0.0".to_string()),
                router: Router::from(Value::Null),
                cors: None,
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
        if let Ok(Some(validator)) = OpenAPIValidator::from_value(value.clone()).map_err(|e| {
            log::error!("Failed to load OpenAPI validator: {:?}", e);
        }) {
            router.openapi_validator = Some(validator);
            log::info!("OpenAPI validator loaded successfully");
        }

        // Load CORS configuration only if present
        let cors = value.get("cors").map(|cors_value| {
            CorsConfig::from(cors_value.clone())
        });

        Config { port, host, router, cors }
    }
}
