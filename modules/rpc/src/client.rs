use crate::service::{PhlowRpcClient, RpcRequest};
use crate::setup::Config;
use phlow_sdk::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tarpc::{client, context, tokio_serde::formats::Json};

pub struct RpcClient {
    config: Config,
}

impl RpcClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn execute_call(
        &self,
        input: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Executing RPC call to {}", self.config.server_address());

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        // Extract method and params from input
        let method = match input.get("method") {
            Some(Value::String(method)) => method.as_str(),
            _ => "call",
        }
        .to_string();

        // Convert phlow_sdk::Value to serde_json::Value
        let params = input.get("params").cloned().unwrap_or(input.clone());

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

        log::debug!("Sending RPC request: {:?}", request);

        let mut ctx = context::current();
        ctx.deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms);

        let response = client.call(ctx, request).await?;

        log::debug!("Received RPC response: {:?}", response);

        Ok(response.to_value())
    }

    pub async fn health_check(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!(
            "Performing health check on {}",
            self.config.server_address()
        );

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        let mut ctx = context::current();
        ctx.deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms);

        let is_healthy = client.health(ctx).await?;

        let mut result = HashMap::new();
        result.insert("healthy".to_string(), is_healthy.to_value());
        result.insert("service".to_string(), self.config.service_name.to_value());
        result.insert(
            "address".to_string(),
            self.config.server_address().to_value(),
        );

        Ok(result.to_value())
    }

    pub async fn get_service_info(
        &self,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Getting service info from {}", self.config.server_address());

        let server_addr: std::net::SocketAddr = self.config.server_address().parse()?;
        let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default).await?;
        let client = PhlowRpcClient::new(client::Config::default(), transport).spawn();

        let mut ctx = context::current();
        ctx.deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms);

        let info = client.info(ctx).await?;

        Ok(info.to_value())
    }
}
