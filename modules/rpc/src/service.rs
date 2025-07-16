use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tarpc::context;

// Use serde_json::Value for serialization with tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub headers: HashMap<String, String>,
}

#[tarpc::service]
pub trait PhlowRpc {
    /// Execute a remote procedure call
    async fn call(request: RpcRequest) -> RpcResponse;

    /// Health check endpoint
    async fn health() -> bool;

    /// Get service information
    async fn info() -> HashMap<String, String>;
}

#[derive(Clone)]
pub struct PhlowRpcServer {
    #[allow(dead_code)] // Used in tracing and sender_package! macro
    pub dispatch: Dispatch,
    pub service_name: String,
    pub main_sender: MainRuntimeSender,
    pub id: ModuleId,
}

impl PhlowRpc for PhlowRpcServer {
    async fn call(self, _: context::Context, request: RpcRequest) -> RpcResponse {
        log::debug!(
            "Received RPC call: method={}, params={:?}",
            request.method,
            request.params
        );

        // Convert RPC request to phlow_sdk::Value format
        let mut internal_request = serde_json::Map::new();
        internal_request.insert(
            "method".to_string(),
            serde_json::Value::String(request.method.clone()),
        );
        internal_request.insert("params".to_string(), request.params.clone());
        internal_request.insert(
            "headers".to_string(),
            serde_json::to_value(request.headers.clone()).unwrap_or(serde_json::Value::Null),
        );

        let request_json =
            serde_json::to_string(&internal_request).unwrap_or_else(|_| "{}".to_string());
        let request_value = request_json.to_value();

        log::debug!("Sending RPC request to steps: {:?}", request_value);

        // Execute the request through the phlow pipeline system
        // This integrates with the steps defined in the YAML configuration
        let response_value =
            phlow_sdk::tracing::dispatcher::with_default(&self.dispatch.clone(), || {
                let span = tracing::span!(
                    Level::INFO,
                    "rpc_call",
                    "rpc.method" = request.method.clone(),
                    "rpc.service" = self.service_name.clone(),
                );

                span_enter!(span);

                Box::pin(async move {
                    let response_value = sender_package!(
                        span.clone(),
                        self.dispatch.clone(),
                        self.id.clone(),
                        self.main_sender.clone(),
                        Some(request_value)
                    )
                    .await
                    .unwrap_or(Value::Null);

                    log::debug!("Received response from steps: {:?}", response_value);
                    response_value
                })
            })
            .await;

        log::debug!("Final response from steps: {:?}", response_value);

        // Convert response back to RPC format
        let result_json = response_value.to_string();
        let result: serde_json::Value = serde_json::from_str(&result_json)
            .unwrap_or_else(|_| serde_json::Value::String(result_json));

        let response = RpcResponse {
            result,
            error: None,
            headers: HashMap::new(),
        };

        log::debug!("RPC response: {:?}", response);
        response
    }

    async fn health(self, _: context::Context) -> bool {
        log::debug!("Health check requested");
        true
    }

    async fn info(self, _: context::Context) -> HashMap<String, String> {
        log::debug!("Service info requested");
        let mut info = HashMap::new();
        info.insert("service_name".to_string(), self.service_name.clone());
        info.insert("version".to_string(), "0.1.0".to_string());
        info.insert("status".to_string(), "running".to_string());

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        info.insert("hostname".to_string(), hostname);

        info
    }
}
