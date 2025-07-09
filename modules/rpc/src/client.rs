use crate::service::{PhlowRpcClient, RpcRequest};
use crate::setup::Config;
use phlow_sdk::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use tarpc::{client, context, tokio_serde::formats::Json};

pub struct RpcClient {
    config: Config,
}

impl RpcClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn execute_call(&self, input: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing RPC call to {}", self.config.server_address());

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        // Extract method and params from input
        let method = match input.get("method") {
            Some(Value::String(method)) => method.as_str(),
            _ => "call"
        }.to_string();

        // Convert phlow_sdk::Value to serde_json::Value
        let params_str = input.get("params").cloned().unwrap_or(input.clone()).to_string();
        let params: serde_json::Value = serde_json::from_str(&params_str)
            .unwrap_or_else(|_| serde_json::Value::String(params_str));

        let headers: HashMap<String, String> = input
            .get("headers")
            .and_then(|v| {
                if let Value::Object(obj) = v {
                    let mut map = HashMap::new();
                    for (k, v) in obj.iter() {
                        map.insert(k.to_string(), v.to_string());
                    }
                    Some(map)
                } else {
                    None
                }
            })
            .unwrap_or_else(HashMap::new);

        let request = RpcRequest {
            method,
            params,
            headers,
        };

        debug!("Sending RPC request: {:?}", request);

        let mut ctx = context::current();
        ctx.deadline = std::time::SystemTime::now() + Duration::from_millis(self.config.timeout_ms);

        let response = client.call(ctx, request).await?;

        debug!("Received RPC response: {:?}", response);

        // Convert response back to phlow_sdk::Value
        let mut final_result = serde_json::Map::new();
        final_result.insert("result".to_string(), response.result);
        
        if let Some(error) = response.error {
            final_result.insert("error".to_string(), serde_json::Value::String(error));
        }
        
        if !response.headers.is_empty() {
            final_result.insert("headers".to_string(), serde_json::to_value(response.headers)?);
        }

        let final_json = serde_json::to_string(&final_result)?;
        Ok(final_json.to_value())
    }

    pub async fn health_check(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Performing health check on {}", self.config.server_address());

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        let mut ctx = context::current();
        ctx.deadline = std::time::SystemTime::now() + Duration::from_millis(self.config.timeout_ms);

        let is_healthy = client.health(ctx).await?;

        let mut result = serde_json::Map::new();
        result.insert("healthy".to_string(), serde_json::Value::Bool(is_healthy));
        result.insert("service".to_string(), serde_json::Value::String(self.config.service_name.clone()));
        result.insert("address".to_string(), serde_json::Value::String(self.config.server_address()));

        let result_json = serde_json::to_string(&result)?;
        Ok(result_json.to_value())
    }

    pub async fn get_service_info(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Getting service info from {}", self.config.server_address());

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        let mut ctx = context::current();
        ctx.deadline = std::time::SystemTime::now() + Duration::from_millis(self.config.timeout_ms);

        let info = client.info(ctx).await?;

        let info_json = serde_json::to_string(&info)?;
        Ok(info_json.to_value())
    }
}
