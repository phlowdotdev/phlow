use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ServiceName,
    Methods,
    InvalidMethod,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceName => write!(f, "service_name is required"),
            Self::Methods => write!(f, "methods configuration is required for server mode"),
            Self::InvalidMethod => write!(f, "invalid method configuration"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcMethod {
    pub name: String,
    pub handler: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub transport: String,
    pub timeout: u64,
    pub methods: Vec<RpcMethod>,
    pub max_connections: u16,
    pub retry_attempts: u8,
}

impl Config {
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn tcp_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl TryFrom<&Value> for Config {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
        let host = value
            .get("host")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "localhost".to_string());

        let port = value
            .get("port")
            .map(|v| v.to_i64().unwrap_or(8080) as u16)
            .unwrap_or(8080);

        let service_name = value
            .get("service_name")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "phlow_service".to_string());

        let transport = value
            .get("transport")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "tcp".to_string());

        let timeout = value
            .get("timeout")
            .map(|v| v.to_i64().unwrap_or(30) as u64)
            .unwrap_or(30);

        let max_connections = value
            .get("max_connections")
            .map(|v| v.to_i64().unwrap_or(100) as u16)
            .unwrap_or(100);

        let retry_attempts = value
            .get("retry_attempts")
            .map(|v| v.to_i64().unwrap_or(3) as u8)
            .unwrap_or(3);

        let methods = if let Some(methods_value) = value.get("methods") {
            if let Value::Array(methods_array) = methods_value {
                let mut methods = Vec::new();
                for method_value in methods_array {
                    if let Value::Object(method_obj) = method_value {
                        let name = method_obj
                            .get("name")
                            .ok_or(Error::InvalidMethod)?
                            .to_string();
                        let handler = method_obj
                            .get("handler")
                            .ok_or(Error::InvalidMethod)?
                            .to_string();
                        
                        methods.push(RpcMethod { name, handler });
                    } else {
                        return Err(Error::InvalidMethod);
                    }
                }
                methods
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            host,
            port,
            service_name,
            transport,
            timeout,
            methods,
            max_connections,
            retry_attempts,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcInput {
    pub method: String,
    pub args: Value,
    pub timeout: Option<u64>,
    pub context: Option<Value>,
}

impl From<&Value> for RpcInput {
    fn from(value: &Value) -> Self {
        let method = value
            .get("method")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let args = value
            .get("args")
            .cloned()
            .unwrap_or(Value::Null);

        let timeout = value
            .get("timeout")
            .map(|v| v.to_i64().unwrap_or(30) as u64);

        let context = value
            .get("context")
            .cloned();

        Self {
            method,
            args,
            timeout,
            context,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcOutput {
    pub success: bool,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time: Option<f64>,
}

impl RpcOutput {
    pub fn success(result: Value, execution_time: f64) -> Self {
        Self {
            success: true,
            result: Some(result),
            error_message: None,
            execution_time: Some(execution_time),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            result: None,
            error_message: Some(message),
            execution_time: None,
        }
    }
}

impl From<RpcOutput> for Value {
    fn from(output: RpcOutput) -> Self {
        let mut map = std::collections::HashMap::new();
        map.insert("success".to_string(), output.success.to_value());
        
        if let Some(result) = output.result {
            map.insert("result".to_string(), result);
        }
        
        if let Some(error_message) = output.error_message {
            map.insert("error_message".to_string(), error_message.to_value());
        }
        
        if let Some(execution_time) = output.execution_time {
            map.insert("execution_time".to_string(), execution_time.to_value());
        }
        
        Value::Object(map.into())
    }
}
