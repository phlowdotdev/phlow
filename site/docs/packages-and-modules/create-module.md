---
sidebar_position: 3
title: Creating Your Own Module - Complete Guide
---

# Creating Your Own Phlow Module - Complete Developer Guide

Phlow modules are written in **Rust** and compiled as **shared libraries (cdylib)**. This comprehensive guide will walk you through creating three types of modules:

1. **Step Module** - Executes logic within a flow
2. **Main Module** - Entry point of applications 
3. **Hybrid Module** - Can act as both main and step (like AMQP)

## 📋 Prerequisites

Before starting, ensure you have:
- **Rust** installed (latest stable version)
- **Cargo** package manager
- Basic knowledge of async Rust programming
- Understanding of Phlow concepts

## 🏗️ Module Architecture Overview

Every Phlow module consists of:
```
my_module/
├── Cargo.toml          # Rust package configuration
├── phlow.yaml          # Module metadata and schema
└── src/
    └── lib.rs          # Main implementation
```

---

## 🔧 Part 1: Creating a Step Module

Step modules process data within a flow. Let's create an improved **log module**.

### Step 1: Create the Project Structure

```bash
# Create module directory
mkdir log_module && cd log_module

# Initialize Cargo project as library
cargo init --lib
```

### Step 2: Configure Cargo.toml

```toml
[package]
name = "log"
version = "0.0.2"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A logging module for Phlow"
license = "MIT"

[dependencies]
# Core Phlow SDK - provides all necessary macros and types
phlow-sdk = { workspace = true }  # or version = "0.0.41"

# Logging dependencies
log = { version = "0.4" }
env_logger = { version = "0.11" }

[lib]
name = "log"                    # Library name (must match module name)
crate-type = ["cdylib"]         # Compile as dynamic library
```

### Step 3: Implement the Step Module (src/lib.rs)

```rust
use phlow_sdk::prelude::*;

// 🎯 STEP MODULE MACRO - This registers the function as a step module
create_step!(log(rx));

// Define log levels
#[derive(Debug)]
enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

// Data structure for log entries
#[derive(Debug)]
struct Log {
    level: LogLevel,
    message: String,
}

// Convert Phlow Value to our Log struct
impl From<&Value> for Log {
    fn from(value: &Value) -> Self {
        let level = match value.get("level") {
            Some(level) => match level.to_string().as_str() {
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => LogLevel::Info,
            },
            _ => LogLevel::Info,
        };

        let message = value.get("message").unwrap_or(&Value::Null).to_string();

        Self { level, message }
    }
}

// 🚀 MAIN STEP FUNCTION
// This function will be called for each message sent to this module
pub async fn log(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Log module started, waiting for messages");

    // 🎯 LISTEN MACRO - Handles incoming messages
    listen!(rx, move |package: ModulePackage| async {
        // Extract input from the package
        let value = package.input().unwrap_or(Value::Null);
        log::debug!("Log module received input: {:?}", value);

        // Convert to our log structure
        let log_value = Log::from(&value);
        log::debug!("Parsed log: {:?}", log_value);

        // Execute the actual logging
        match log_value.level {
            LogLevel::Info => log::info!("{}", log_value.message),
            LogLevel::Debug => log::debug!("{}", log_value.message),
            LogLevel::Warn => log::warn!("{}", log_value.message),
            LogLevel::Error => log::error!("{}", log_value.message),
        }

        // 🎯 RETURN RESULT - Send response back
        let payload = package.payload().unwrap_or(Value::Null);
        sender_safe!(package.sender, payload.into());
    });

    Ok(())
}
```

### Step 4: Create Module Metadata (phlow.yaml)

```yaml
# Module identification
name: log
description: |
  Advanced logging module that supports multiple log levels (info, debug, warn, error).
  Designed for debugging and monitoring Phlow applications.
version: 0.0.2
author: Your Name <your.email@example.com>
repository: https://github.com/your-repo/phlow-modules
license: MIT

# 🎯 MODULE TYPE
type: step                      # This is a step module

# Tags for discovery
tags:
  - log
  - debug
  - monitoring
  - step

# 📋 INPUT SCHEMA - What this module expects as input
input:
  type: object
  required: true
  properties:
    level:
      type: string
      description: "Log level: info, debug, warn, or error"
      default: info
      required: false
      enum: ["info", "debug", "warn", "error"]
    message:
      type: string
      description: "Message to be logged"
      required: true

# 📤 OUTPUT SCHEMA - What this module returns
output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      description: "Whether the logging operation succeeded"
      required: true
```

---

## 🌟 Part 2: Creating a Main Module

Main modules serve as application entry points. Let's create a **simple HTTP server**.

### Step 1: Create Project Structure

```bash
mkdir simple_server && cd simple_server
cargo init --lib
```

### Step 2: Configure Cargo.toml

```toml
[package]
name = "simple_server"
version = "0.1.0"
edition = "2021"

[dependencies]
phlow-sdk = { workspace = true }

# HTTP server dependencies
hyper = { version = "1", features = ["full"] }
http-body-util = "0.1"
hyper-util = { version = "0.1", features = ["full"] }
bytes = "1.10.1"
futures-util = "0.3.31"
tokio = { version = "1", features = ["full"] }

[lib]
name = "simple_server"
crate-type = ["cdylib"]
```

### Step 3: Implement the Main Module (src/lib.rs)

```rust
use phlow_sdk::prelude::*;
use hyper::{server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use bytes::Bytes;
use std::{net::SocketAddr, sync::Arc};

// 🎯 MAIN MODULE MACRO - This registers the function as a main module
create_main!(start_server(setup));

// Configuration structure
#[derive(Debug, Clone)]
struct Config {
    port: u16,
    host: String,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        let port = value.get("port")
            .and_then(|v| v.to_i64().ok())
            .unwrap_or(3000) as u16;
        
        let host = value.get("host")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string());
        
        Self { port, host }
    }
}

// 🚀 MAIN FUNCTION - Entry point of the application
pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // 🔍 CHECK IF THIS IS THE MAIN MODULE
    if !setup.is_main() {
        log::debug!("This module is not the main module, exiting");
        // Notify setup completion and exit
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    // 🧪 CHECK TEST MODE
    if setup.is_test_mode {
        log::debug!("Test mode detected, not starting HTTP server");
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    // Parse configuration from "with" section
    let config = Config::from(setup.with);
    
    // Create server address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    
    // Start TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("🚀 Simple server listening on http://{}", addr);
    
    // Notify that setup is complete
    sender_safe!(setup.setup_sender, None);
    
    // 🔄 MAIN SERVER LOOP
    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        
        // Clone sender for this connection
        let main_sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                log::error!("Main sender is None");
                return Err("Main sender is None".into());
            }
        };
        
        let module_id = setup.id;
        let dispatch = setup.dispatch.clone();
        
        // Handle connection in a separate task
        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                handle_request(req, main_sender.clone(), module_id, dispatch.clone(), peer_addr)
            });
            
            if let Err(e) = http1::Builder::new()
                .keep_alive(true)
                .serve_connection(io, service)
                .await
            {
                log::debug!("Error serving connection: {}", e);
            }
        });
    }
}

// Handle individual HTTP requests
async fn handle_request(
    req: Request<hyper::body::Incoming>,
    sender: MainRuntimeSender,
    id: ModuleId,
    dispatch: phlow_sdk::tracing::Dispatch,
    peer_addr: SocketAddr,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    
    // Extract request information
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers = req.headers().clone();
    
    // Create request data
    let request_data = json!({
        "method": method,
        "path": path,
        "headers": {}, // Simplified
        "body": {},
        "client_ip": peer_addr.to_string()
    });
    
    log::info!("📥 {} {}", method, path);
    
    // Send to Phlow pipeline for processing
    match sender_package!(dispatch, id, sender, Some(request_data.to_value())).await {
        Ok(response_value) => {
            // Parse response
            if let Some(obj) = response_value.as_object() {
                let status_code = obj.get("status_code")
                    .and_then(|v| v.to_i64().ok())
                    .unwrap_or(200) as u16;
                
                let body = obj.get("body")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "OK".to_string());
                
                // Build response
                let mut response = Response::builder().status(status_code);
                
                if let Some(headers_obj) = obj.get("headers").and_then(|v| v.as_object()) {
                    for (key, value) in headers_obj.iter() {
                        response = response.header(key.to_string(), value.to_string());
                    }
                }
                
                Ok(response.body(Full::new(Bytes::from(body))).unwrap())
            } else {
                Ok(Response::new(Full::new(Bytes::from("OK"))))
            }
        }
        Err(e) => {
            log::error!("Pipeline error: {}", e);
            Ok(Response::builder()
                .status(500)
                .body(Full::new(Bytes::from("Internal Server Error")))
                .unwrap())
        }
    }
}
```

### Step 4: Create Main Module Metadata (phlow.yaml)

```yaml
name: simple_server
description: |
  Simple HTTP server that can serve as the main entry point for Phlow applications.
  Handles HTTP requests and forwards them to the Phlow pipeline for processing.
version: 0.1.0
author: Your Name <your.email@example.com>
repository: https://github.com/your-repo/phlow-modules
license: MIT

# 🎯 MAIN MODULE TYPE
type: main                      # This is a main module

tags:
  - http
  - server
  - main
  - web
  - api

# ⚙️ CONFIGURATION SCHEMA - Used in "with" section
with:
  type: object
  required: false
  properties:
    port:
      type: number
      description: "Port number to listen on"
      default: 3000
      required: false
    host:
      type: string
      description: "Host address to bind to"
      default: "0.0.0.0"
      required: false

# 📤 OUTPUT SCHEMA - What this module provides to steps
output:
  type: object
  required: true
  properties:
    method:
      type: string
      description: "HTTP method (GET, POST, etc.)"
      required: true
    path:
      type: string
      description: "Request path"
      required: true
    headers:
      type: object
      description: "Request headers"
      required: true
    body:
      type: object
      description: "Request body"
      required: true
    client_ip:
      type: string
      description: "Client IP address"
      required: true
```

---

## 🔄 Part 3: Creating a Hybrid Module (Main + Step)

Hybrid modules can act as both main and step modules. The **AMQP module** is a perfect example.

### Step 1: Create Project Structure

```bash
mkdir messaging_module && cd messaging_module
cargo init --lib
```

### Step 2: Configure Cargo.toml

```toml
[package]
name = "messaging"
version = "0.1.0"
edition = "2021"

[dependencies]
phlow-sdk = { workspace = true }

# Messaging dependencies
lapin = "2.5.1"                # AMQP client
futures-lite = "2.6.0"         # Async utilities
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"             # JSON handling

[lib]
name = "messaging"
crate-type = ["cdylib"]
```

### Step 3: Implement the Hybrid Module (src/lib.rs)

```rust
mod consumer;           // Consumer logic (main mode)
mod producer;           // Producer logic (step mode)
mod setup;              // Configuration

use phlow_sdk::prelude::*;
use lapin::{Connection, ConnectionProperties};
use setup::Config;

// 🎯 HYBRID MODULE MACRO - Can act as both main and step
create_main!(start_server(setup));

// 🚀 MAIN ENTRY POINT - Handles both main and step modes
pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Messaging module starting...");
    
    // Parse configuration
    let config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;
    log::debug!("Config parsed: {:?}", config);
    
    // Establish AMQP connection
    let uri = match config.uri.clone() {
        Some(uri) => uri,
        None => config.to_connection_string(),
    };
    
    log::debug!("Connecting to AMQP at {}", uri);
    let conn = Connection::connect(&uri, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    
    // 🔍 DETERMINE MODE: Main or Step
    if setup.is_main() {
        log::info!("🎯 Running in MAIN mode - Starting consumer");
        
        // Main mode: Start consumer
        let dispatch = setup.dispatch.clone();
        let consumer_channel = conn.create_channel().await?;
        let main_sender = setup.main_sender.clone()
            .ok_or("Main sender is None")?;
        let id = setup.id.clone();
        let config = config.clone();
        
        // Start consumer in background
        tokio::task::spawn(async move {
            if let Err(e) = consumer::start_consumer(id, main_sender, config, consumer_channel, dispatch).await {
                log::error!("Consumer error: {}", e);
            }
        });
    } else {
        log::info!("📤 Running in STEP mode - Producer only");
    }
    
    // Always start producer (for step functionality)
    producer::start_producer(setup.setup_sender, config, channel).await?;
    
    Ok(())
}
```

### Step 4: Implement Consumer (src/consumer.rs)

```rust
use super::Config;
use phlow_sdk::prelude::*;
use lapin::{options::*, types::FieldTable, BasicProperties, Channel};
use lapin::message::DeliveryResult;

pub async fn start_consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: Channel,
    dispatch: phlow_sdk::tracing::Dispatch,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Starting AMQP consumer on queue: {}", config.queue_name);
    
    // Declare queue
    let _queue = channel
        .queue_declare(
            &config.queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Start consuming
    let consumer = channel
        .basic_consume(
            &config.queue_name,
            &config.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    // Handle messages
    consumer.set_delegate(move |delivery: DeliveryResult| {
        let sender = main_sender.clone();
        let module_id = id.clone();
        let dispatch = dispatch.clone();
        
        Box::pin(async move {
            match delivery {
                Ok(Some(delivery)) => {
                    // Parse message
                    let message = String::from_utf8_lossy(&delivery.data).to_string();
                    let data = json!({"message": message}).to_value();
                    
                    // Send to pipeline
                    match sender_package!(dispatch, module_id, sender, Some(data)).await {
                        Ok(_) => {
                            // Acknowledge message
                            let _ = delivery.ack(BasicAckOptions::default()).await;
                            log::debug!("Message processed and acknowledged");
                        }
                        Err(e) => {
                            log::error!("Pipeline processing error: {}", e);
                            let _ = delivery.nack(BasicNackOptions::default()).await;
                        }
                    }
                }
                Ok(None) => {
                    log::debug!("No message received");
                }
                Err(e) => {
                    log::error!("Consumer error: {}", e);
                }
            }
        })
    });
    
    Ok(())
}
```

### Step 5: Implement Producer (src/producer.rs)

```rust
use super::Config;
use phlow_sdk::prelude::*;
use lapin::{options::*, BasicProperties, Channel};

pub async fn start_producer(
    setup_sender: SetupSender,
    config: Config,
    channel: Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Producer ready for step operations");
    
    // Create module channel for step operations
    let (tx, rx) = module_channel();
    
    // Notify setup completion
    sender_safe!(setup_sender, Some(tx));
    
    // Listen for step operations
    listen!(rx, move |package: ModulePackage| async {
        let channel = channel.clone();
        let config = config.clone();
        
        // Get input message
        let input = package.input().unwrap_or(Value::Null);
        let message = input.to_string();
        
        // Publish message
        match channel
            .basic_publish(
                &config.exchange,
                &config.routing_key,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
        {
            Ok(_) => {
                log::debug!("Message published successfully");
                let response = json!({
                    "success": true,
                    "error_message": null
                });
                sender_safe!(package.sender, response.to_value().into());
            }
            Err(e) => {
                log::error!("Failed to publish message: {}", e);
                let response = json!({
                    "success": false,
                    "error_message": e.to_string()
                });
                sender_safe!(package.sender, response.to_value().into());
            }
        }
    });
    
    Ok(())
}
```

### Step 6: Implement Configuration (src/setup.rs)

```rust
use phlow_sdk::prelude::*;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub vhost: String,
    pub routing_key: String,
    pub exchange: String,
    pub queue_name: String,
    pub consumer_tag: String,
    pub uri: Option<String>,
}

impl Config {
    pub fn to_connection_string(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.vhost
        )
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingField(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl TryFrom<&Value> for Config {
    type Error = ConfigError;
    
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let host = value.get("host")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "localhost".to_string());
        
        let port = value.get("port")
            .and_then(|v| v.to_i64().ok())
            .unwrap_or(5672) as u16;
        
        let username = value.get("username")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "guest".to_string());
        
        let password = value.get("password")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "guest".to_string());
        
        let vhost = value.get("vhost")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "/".to_string());
        
        let routing_key = value.get("routing_key")
            .map(|v| v.to_string())
            .ok_or_else(|| ConfigError::MissingField("routing_key".to_string()))?;
        
        let exchange = value.get("exchange")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string());
        
        let queue_name = value.get("queue_name")
            .map(|v| v.to_string())
            .unwrap_or_else(|| routing_key.clone());
        
        let consumer_tag = value.get("consumer_tag")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "phlow_consumer".to_string());
        
        let uri = value.get("uri")
            .and_then(|v| v.as_string_b())
            .map(|s| s.as_string());
        
        Ok(Self {
            host,
            port,
            username,
            password,
            vhost,
            routing_key,
            exchange,
            queue_name,
            consumer_tag,
            uri,
        })
    }
}
```

### Step 7: Create Hybrid Module Metadata (phlow.yaml)

```yaml
name: messaging
description: |
  Hybrid AMQP messaging module that can act as both main and step module.
  
  - **Main Mode**: Consumes messages from AMQP queue and forwards to pipeline
  - **Step Mode**: Publishes messages to AMQP exchange/queue
  
  Perfect for microservices and event-driven architectures.
version: 0.1.0
author: Your Name <your.email@example.com>
repository: https://github.com/your-repo/phlow-modules
license: MIT

# 🎯 HYBRID MODULE TYPE
type: any                       # Can be both main and step

tags:
  - messaging
  - amqp
  - rabbitmq
  - queue
  - hybrid
  - main
  - step

# ⚙️ CONFIGURATION SCHEMA
with:
  type: object
  required: true
  properties:
    uri:
      type: string
      description: "Full AMQP URI (e.g., amqp://user:pass@host:port/vhost)"
      required: false
    host:
      type: string
      description: "AMQP host"
      default: "localhost"
      required: false
    port:
      type: number
      description: "AMQP port"
      default: 5672
      required: false
    username:
      type: string
      description: "AMQP username"
      default: "guest"
      required: false
    password:
      type: string
      description: "AMQP password"
      default: "guest"
      required: false
    vhost:
      type: string
      description: "AMQP virtual host"
      default: "/"
      required: false
    routing_key:
      type: string
      description: "AMQP routing key"
      required: true
    exchange:
      type: string
      description: "AMQP exchange name"
      required: false
    queue_name:
      type: string
      description: "AMQP queue name"
      required: false
    consumer_tag:
      type: string
      description: "Consumer tag for identification"
      default: "phlow_consumer"
      required: false

# 📋 INPUT SCHEMA (for step mode)
input:
  type: object
  required: true
  properties:
    message:
      type: any
      description: "Message to publish to AMQP"
      required: true
    headers:
      type: object
      description: "Optional AMQP message headers"
      required: false

# 📤 OUTPUT SCHEMAS
output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      description: "Operation success status"
      required: true
    error_message:
      type: string
      description: "Error message if operation failed"
      required: false
```

---

## 🏗️ Building and Testing Modules

### Building a Module

```bash
# Build in development mode
cargo build

# Build optimized release
cargo build --release

# The compiled module will be at:
# target/debug/lib<module_name>.so (Linux)
# target/debug/lib<module_name>.dylib (macOS)
# target/debug/<module_name>.dll (Windows)
```

### Testing Modules Locally

1. **Create a test Phlow file**:

```yaml
# test.phlow
main: your_module
modules:
  - module: your_module
    with:
      # your configuration
steps:
  - use: your_module
    input:
      # your test input
```

2. **Install module locally**:

```bash
# Copy module to phlow_packages
mkdir -p phlow_packages/your_module
cp target/debug/libyour_module.so phlow_packages/your_module/module.so
cp phlow.yaml phlow_packages/your_module/
```

3. **Run test**:

```bash
phlow test.phlow
```

---

## 📚 Key Concepts Summary

### Module Types

| Type | Purpose | Macro | Function Signature |
|------|---------|-------|--------------------|
| **Step** | Process data in pipeline | `create_step!(fn_name(rx))` | `async fn(ModuleReceiver) -> Result<(), Error>` |
| **Main** | Application entry point | `create_main!(fn_name(setup))` | `async fn(ModuleSetup) -> Result<(), Error>` |
| **Hybrid** | Both main and step | `create_main!(fn_name(setup))` | Handles both modes internally |

### Key Macros

| Macro | Purpose | Usage |
|-------|---------|-------|
| `create_step!()` | Register step function | `create_step!(my_function(rx));` |
| `create_main!()` | Register main function | `create_main!(start_app(setup));` |
| `listen!()` | Handle incoming messages | `listen!(rx, \|pkg\| async { ... });` |
| `sender_safe!()` | Send response safely | `sender_safe!(sender, response);` |
| `sender_package!()` | Send to pipeline | `sender_package!(dispatch, id, sender, data);` |

### Module Structure Checklist

- ✅ **Cargo.toml**: Correct `crate-type = ["cdylib"]`
- ✅ **phlow.yaml**: Complete metadata and schemas
- ✅ **src/lib.rs**: Proper macro usage and error handling
- ✅ **Dependencies**: Include `phlow-sdk` in workspace or specific version
- ✅ **Error Handling**: Use `Result<(), Box<dyn std::error::Error + Send + Sync>>`
- ✅ **Async/Await**: All functions must be async
- ✅ **Logging**: Use `log::info!()`, `log::debug!()`, etc.

This guide provides a comprehensive foundation for creating any type of Phlow module. Each pattern can be adapted and extended based on your specific needs!
