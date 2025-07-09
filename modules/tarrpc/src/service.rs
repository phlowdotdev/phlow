use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use crate::setup::{Config, RpcOutput};

// Estruturas simplificadas para o serviço
#[derive(Clone)]
pub struct PhlowServiceImpl {
    pub config: Arc<Config>,
    pub dispatch: Arc<Dispatch>,
    pub main_sender: Arc<MainRuntimeSender>,
    pub id: Arc<ModuleId>,
    pub method_handlers: Arc<HashMap<String, String>>,
}

impl PhlowServiceImpl {
    pub fn new(
        config: Config,
        dispatch: Dispatch,
        main_sender: MainRuntimeSender,
        id: ModuleId,
    ) -> Self {
        let method_handlers: HashMap<String, String> = config
            .methods
            .iter()
            .map(|method| (method.name.clone(), method.handler.clone()))
            .collect();

        Self {
            config: Arc::new(config),
            dispatch: Arc::new(dispatch),
            main_sender: Arc::new(main_sender),
            id: Arc::new(id),
            method_handlers: Arc::new(method_handlers),
        }
    }

    pub async fn execute(
        &self,
        method: String,
        args: String,
        context_data: Option<String>,
    ) -> String {
        let start_time = Instant::now();
        
        debug!("Executing RPC method: {} with args: {}", method, args);
        
        // Check if method exists
        if !self.method_handlers.contains_key(&method) {
            let error_msg = format!("Method '{}' not found", method);
            error!("{}", error_msg);
            return serde_json::to_string(&RpcOutput::error(error_msg)).unwrap_or_default();
        }

        // Parse arguments - para simplicidade, vamos usar string mesmo
        let args_value = args.to_value();

        // Parse context data if provided
        let context_value = context_data.map(|ctx| ctx.to_value());

        // Create input data combining args and context
        let mut input_data = std::collections::HashMap::new();
        input_data.insert("method".to_string(), method.to_value());
        input_data.insert("args".to_string(), args_value);
        if let Some(ctx) = context_value {
            input_data.insert("context".to_string(), ctx);
        }

        let input = Value::Object(input_data.into());

        // Execute the method through the main runtime
        let result = sender_package!(
            tracing::Span::current(),
            self.dispatch.as_ref().clone(),
            self.id.as_ref().clone(),
            self.main_sender.as_ref().clone(),
            Some(input)
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to execute method: {}", e);
            Value::Null
        });

        let execution_time = start_time.elapsed().as_millis() as f64;
        
        debug!("Method execution completed in {}ms", execution_time);
        
        let output = RpcOutput::success(result, execution_time);
        serde_json::to_string(&output).unwrap_or_default()
    }

    pub async fn health_check(&self) -> String {
        debug!("Health check requested");
        
        let mut health_data = std::collections::HashMap::new();
        health_data.insert("status".to_string(), "healthy".to_value());
        health_data.insert("service".to_string(), self.config.service_name.to_value());
        health_data.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339().to_value());
        
        let output = RpcOutput::success(
            Value::Object(health_data.into()),
            0.0,
        );
        
        serde_json::to_string(&output).unwrap_or_default()
    }

    pub async fn get_service_info(&self) -> String {
        debug!("Service info requested");
        
        let methods: Vec<String> = self.method_handlers.keys().cloned().collect();
        
        let mut info_data = std::collections::HashMap::new();
        info_data.insert("service_name".to_string(), self.config.service_name.to_value());
        info_data.insert("host".to_string(), self.config.host.to_value());
        info_data.insert("port".to_string(), (self.config.port as i64).to_value());
        info_data.insert("transport".to_string(), self.config.transport.to_value());
        info_data.insert("methods".to_string(), methods.to_value());
        info_data.insert("max_connections".to_string(), (self.config.max_connections as i64).to_value());
        info_data.insert("timeout".to_string(), (self.config.timeout as i64).to_value());
        
        let output = RpcOutput::success(
            Value::Object(info_data.into()),
            0.0,
        );
        
        serde_json::to_string(&output).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub args: Value,
    pub context: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub execution_time: f64,
}
