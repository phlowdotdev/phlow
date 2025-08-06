# Phlow Module Development Guide

A comprehensive guide to creating custom modules for Phlow - the high-performance, low-code flow runtime built in Rust.

## Table of Contents

1. [Introduction](#introduction)
2. [Module Architecture Overview](#module-architecture-overview)
3. [Module Types](#module-types)
4. [Creating Step Modules](#creating-step-modules)
5. [Creating Main Modules](#creating-main-modules)
6. [Creating Hybrid Modules](#creating-hybrid-modules)
7. [The phlow.yaml Schema File](#the-phlowyaml-schema-file)
8. [Building and Testing Modules](#building-and-testing-modules)
9. [Best Practices](#best-practices)
10. [Complete Examples](#complete-examples)

## Introduction

Phlow is a modular flow runtime that enables you to build composable backends through reusable modules. Modules are the building blocks of Phlow applications, providing specific functionality that can be combined to create complex workflows.

There are two main types of modules in Phlow:

- **Official Modules**: Pre-compiled Rust modules from the official repository
- **Custom Modules**: User-created modules written in Rust and compiled as shared libraries

This guide focuses on creating custom Rust-based modules.

## Module Architecture Overview

Every Phlow module consists of three essential components:

```
my_module/
├── Cargo.toml          # Rust package configuration
├── phlow.yaml          # Module metadata and schema
└── src/
    └── lib.rs          # Main implementation
```

### Key Requirements

1. **Rust Library**: Must be compiled as a dynamic library (`cdylib`)
2. **Async Functions**: All module functions must be async
3. **Phlow SDK**: Must use the `phlow-sdk` crate
4. **Proper Macros**: Must use appropriate Phlow macros for registration
5. **Schema Definition**: Must have a complete `phlow.yaml` file

## Module Types

Phlow supports three types of modules:

### 1. Step Module (`type: step`)
- **Purpose**: Process data within a flow pipeline
- **Usage**: Called with `use: module_name` in steps
- **Examples**: logging, data transformation, external API calls

### 2. Main Module (`type: main`)
- **Purpose**: Serve as application entry points
- **Usage**: Defined with `main: module_name` in flow files
- **Examples**: HTTP servers, CLI applications, event consumers

### 3. Hybrid Module (`type: any`)
- **Purpose**: Can function as both main and step modules
- **Usage**: Flexible usage depending on context
- **Examples**: AMQP (consumer when main, producer when step)

## Creating Step Modules

Step modules process data within a flow. They receive input, perform operations, and return results.

There are two main patterns for step modules:

1. **Simple Step Module**: Performs a single operation
2. **Action-Based Step Module**: Supports multiple operations based on an `action` property

### Basic Step Module Structure

```rust
use phlow_sdk::prelude::*;

// Register the function as a step module
create_step!(my_step_function(rx));

// Main processing function
pub async fn my_step_function(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Step module started, waiting for messages");

    // Listen for incoming messages
    listen!(rx, move |package: ModulePackage| async {
        // Extract input from the package
        let input = package.input().unwrap_or(Value::Null);
        
        // Process the input
        let result = process_data(&input);
        
        // Send response back
        sender_safe!(package.sender, result.into());
    });

    Ok(())
}

fn process_data(input: &Value) -> Value {
    // Your processing logic here
    json!({
        "processed": true,
        "original_input": input
    }).to_value()
}
```

### Step Module Example: Enhanced Logger

```rust
use phlow_sdk::prelude::*;

create_step!(log(rx));

#[derive(Debug)]
enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

#[derive(Debug)]
struct LogMessage {
    level: LogLevel,
    message: String,
    timestamp: Option<String>,
}

impl From<&Value> for LogMessage {
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
        
        let timestamp = value.get("timestamp")
            .and_then(|v| v.as_string_b())
            .map(|s| s.as_string());

        Self { level, message, timestamp }
    }
}

pub async fn log(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Log module started");

    listen!(rx, move |package: ModulePackage| async {
        let value = package.input().unwrap_or(Value::Null);
        let log_msg = LogMessage::from(&value);

        // Add timestamp if not provided
        let timestamp = log_msg.timestamp.unwrap_or_else(|| {
            chrono::Utc::now().to_rfc3339()
        });

        // Format message with timestamp
        let formatted_message = format!("[{}] {}", timestamp, log_msg.message);

        // Log with appropriate level
        match log_msg.level {
            LogLevel::Info => log::info!("{}", formatted_message),
            LogLevel::Debug => log::debug!("{}", formatted_message),
            LogLevel::Warn => log::warn!("{}", formatted_message),
            LogLevel::Error => log::error!("{}", formatted_message),
        }

        // Return success response
        let response = json!({
            "success": true,
            "logged_at": timestamp,
            "level": format!("{:?}", log_msg.level).to_lowercase()
        });

        sender_safe!(package.sender, response.to_value().into());
    });

    Ok(())
}
```

### Action-Based Step Module Pattern

Some modules need to support multiple operations based on an `action` property in the input. This pattern is useful for modules that perform related but distinct operations, like the JWT module which can both create and verify tokens.

#### Action-Based Structure with Enums

```rust
use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

create_step!(jwt_handler(setup));

// Define input actions using Rust enums with serde
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]  // This tells serde to use the "action" field as discriminator
pub enum JwtInput {
    #[serde(rename = "create")]
    Create {
        data: Option<Value>,
        expires_in: Option<u64>,
    },
    #[serde(rename = "verify")]
    Verify { 
        token: String 
    },
}

// Configuration structure
#[derive(Debug, Clone)]
struct JwtConfig {
    secret: String,
}

impl TryFrom<Value> for JwtConfig {
    type Error = String;
    
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let secret = value.get("secret")
            .ok_or("Missing required 'secret' field")?.
            to_string();
            
        if secret.is_empty() {
            return Err("Secret cannot be empty".to_string());
        }
        
        Ok(Self { secret })
    }
}

// Custom input parsing
impl TryFrom<Option<Value>> for JwtInput {
    type Error = String;

    fn try_from(input_value: Option<Value>) -> Result<Self, Self::Error> {
        let input_value = input_value.ok_or("Missing input for JWT module")?;

        if !input_value.is_object() {
            return Err("JWT input must be an object".to_string());
        }

        // Extract action (required)
        let action = match input_value.get("action") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'action' field in JWT input".to_string()),
        };

        match action.as_str() {
            "create" => {
                let data = input_value.get("data").cloned();
                let expires_in = input_value.get("expires_in").and_then(|v| v.to_u64());
                Ok(JwtInput::Create { data, expires_in })
            }
            "verify" => {
                let token = match input_value.get("token") {
                    Some(Value::String(s)) => s.as_string(),
                    Some(v) => v.to_string(),
                    None => {
                        return Err("Missing required 'token' field for verify action".to_string())
                    }
                };

                if token.is_empty() {
                    return Err("Token cannot be empty for verify action".to_string());
                }

                Ok(JwtInput::Verify { token })
            }
            _ => Err(format!(
                "Invalid action '{}'. Must be 'create' or 'verify'",
                action
            )),
        }
    }
}

// Main module function
pub async fn jwt_handler(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse JWT configuration from 'with' parameters
    let config = JwtConfig::try_from(setup.with)
        .map_err(|e| format!("JWT configuration error: {}", e))?;

    log::debug!("JWT module started with config: {:?}", config);

    listen!(rx, move |package: ModulePackage| async {
        let config = config.clone();
        
        // Parse input based on action
        let input = match JwtInput::try_from(package.input.clone()) {
            Ok(input) => input,
            Err(e) => {
                log::error!("Invalid JWT input: {}", e);
                let response = json!({
                    "success": false,
                    "error": format!("Invalid input: {}", e)
                });
                sender_safe!(package.sender, response.to_value().into());
                return;
            }
        };

        log::debug!("JWT module received input: {:?}", input);

        // Process based on action
        let result = match input {
            JwtInput::Create { data, expires_in } => {
                create_jwt_token(&config.secret, data, expires_in).await
            }
            JwtInput::Verify { token } => {
                verify_jwt_token(&config.secret, &token).await
            }
        };

        match result {
            Ok(response_value) => {
                log::debug!("JWT operation successful");
                sender_safe!(package.sender, response_value.into());
            }
            Err(e) => {
                log::error!("JWT operation failed: {}", e);
                let response = json!({
                    "success": false,
                    "error": e.to_string()
                });
                sender_safe!(package.sender, response.to_value().into());
            }
        }
    });

    Ok(())
}

// Action implementation functions
async fn create_jwt_token(
    secret: &str, 
    data: Option<Value>, 
    expires_in: Option<u64>
) -> Result<Value, String> {
    // JWT creation logic here
    // This is a simplified example - use jsonwebtoken crate in real implementation
    
    let expires_in = expires_in.unwrap_or(3600); // Default 1 hour
    let issued_at = chrono::Utc::now();
    let expires_at = issued_at + chrono::Duration::seconds(expires_in as i64);
    
    // In a real implementation, you'd use the jsonwebtoken crate
    let token = format!("mock_jwt_token_for_{}", secret.len()); // Mock token
    
    Ok(json!({
        "token": token,
        "expires_at": expires_at.to_rfc3339(),
        "issued_at": issued_at.to_rfc3339(),
        "success": true
    }).to_value())
}

async fn verify_jwt_token(secret: &str, token: &str) -> Result<Value, String> {
    // JWT verification logic here
    // This is a simplified example - use jsonwebtoken crate in real implementation
    
    if token.starts_with("mock_jwt_token") {
        Ok(json!({
            "valid": true,
            "expired": false,
            "data": {
                "user_id": 12345,
                "role": "admin"
            },
            "success": true
        }).to_value())
    } else {
        Ok(json!({
            "valid": false,
            "expired": false,
            "error": "Invalid token format",
            "success": false
        }).to_value())
    }
}
```

#### Usage Example in Phlow Files

```yaml
# jwt-example.phlow
name: JWT Token Example
version: 1.0.0
description: Demonstrates JWT token creation and verification

modules:
  - module: jwt
    with:
      secret: "my-super-secret-key"
  - module: log

steps:
  # Create a JWT token
  - use: jwt
    input:
      action: create
      data:
        user_id: 12345
        role: admin
        email: user@example.com
      expires_in: 3600  # 1 hour
  
  - use: log
    input:
      level: info
      message: !phs `Token created: ${payload.token}`
  
  # Verify the token we just created
  - use: jwt
    input:
      action: verify
      token: !phs payload.token
  
  - use: log
    input:
      level: info
      message: !phs `Token verification result: ${payload.valid ? "VALID" : "INVALID"}`
  
  - return: !phs payload
```

#### Action-Based Schema Definition

When creating action-based modules, your `phlow.yaml` should define the action property and its possible values:

```yaml
name: jwt
description: |
  JSON Web Token (JWT) creation and verification module.
  
  **Actions:**
  - `create`: Generate a new JWT token with optional payload data
  - `verify`: Validate and decode an existing JWT token
  
  **Usage Examples:**
  
  Create a token:
  ```yaml
  - use: jwt
    input:
      action: create
      data:
        user_id: 123
        role: admin
      expires_in: 3600
  ```
  
  Verify a token:
  ```yaml
  - use: jwt
    input:
      action: verify
      token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
  ```
version: 1.0.0
author: Developer <dev@example.com>
type: step

tags:
  - jwt
  - auth
  - authentication
  - token
  - security

with:
  type: object
  required: true
  properties:
    secret:
      type: string
      description: "Secret key for signing and verifying JWT tokens"
      required: true

input:
  type: object
  required: true
  properties:
    action:
      type: string
      description: "Action to perform (create or verify)"
      required: true
      enum: ["create", "verify"]
    
    # Properties for create action
    data:
      type: object
      description: "Data to include in the token payload (for create action)"
      required: false
    expires_in:
      type: number
      description: "Token expiration time in seconds (for create action)"
      default: 3600
      required: false
    
    # Properties for verify action
    token:
      type: string
      description: "JWT token to verify (for verify action)"
      required: false

output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      description: "Whether the operation succeeded"
      required: true
    
    # Create action outputs
    token:
      type: string
      description: "Generated JWT token (for create action)"
      required: false
    expires_at:
      type: string
      description: "Token expiration timestamp in ISO 8601 format (for create action)"
      required: false
    issued_at:
      type: string
      description: "Token issue timestamp in ISO 8601 format (for create action)"
      required: false
    
    # Verify action outputs
    valid:
      type: boolean
      description: "Whether the token is valid (for verify action)"
      required: false
    data:
      type: object
      description: "Decoded token data (for verify action)"
      required: false
    expired:
      type: boolean
      description: "Whether the token has expired (for verify action)"
      required: false
    
    # Error outputs
    error:
      type: string
      description: "Error message if operation fails"
      required: false
```

### Benefits of Action-Based Modules

1. **Single Module, Multiple Operations**: Reduces the number of modules needed
2. **Related Functionality**: Groups logically related operations together
3. **Shared Configuration**: Common configuration (like secrets) shared across actions
4. **Type Safety**: Rust enums provide compile-time validation of action types
5. **Clear API**: Well-defined input/output schemas for each action
6. **Maintainability**: Easier to maintain related functionality in one place

### When to Use Action-Based Pattern

Use this pattern when:
- Operations are closely related (like create/verify, encode/decode, encrypt/decrypt)
- Operations share the same configuration
- You want to reduce module proliferation
- Operations have different input/output requirements

Avoid this pattern when:
- Operations are completely unrelated
- Each operation has vastly different configuration needs
- Operations are complex enough to warrant separate modules
- You want maximum modularity and separation of concerns

## Creating Main Modules

Main modules serve as application entry points. They initialize services and handle external inputs.

### Basic Main Module Structure

```rust
use phlow_sdk::prelude::*;

// Register the function as a main module
create_main!(start_application(setup));

pub async fn start_application(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // Check if this is actually the main module
    if !setup.is_main() {
        log::debug!("Not the main module, exiting");
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    // Skip actual service in test mode
    if setup.is_test_mode {
        log::debug!("Test mode detected, not starting service");
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    // Parse configuration from "with" section
    let config = parse_config(&setup.with);
    
    // Initialize your service
    start_service(config, setup).await?;

    Ok(())
}
```

### Main Module Example: Simple HTTP Server

```rust
use phlow_sdk::prelude::*;
use hyper::{server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use bytes::Bytes;
use std::{net::SocketAddr, sync::Arc};

create_main!(start_http_server(setup));

#[derive(Debug, Clone)]
struct ServerConfig {
    port: u16,
    host: String,
}

impl From<Value> for ServerConfig {
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

pub async fn start_http_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    if !setup.is_main() {
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    if setup.is_test_mode {
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    let config = ServerConfig::from(setup.with);
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("🚀 HTTP server listening on http://{}", addr);
    
    // Signal that setup is complete
    sender_safe!(setup.setup_sender, None);
    
    // Main server loop
    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        
        let main_sender = setup.main_sender.clone()
            .ok_or("Main sender is None")?;
        let module_id = setup.id;
        let dispatch = setup.dispatch.clone();
        
        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                handle_http_request(req, main_sender.clone(), module_id, dispatch.clone(), peer_addr)
            });
            
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                log::debug!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_http_request(
    req: Request<hyper::body::Incoming>,
    sender: MainRuntimeSender,
    id: ModuleId,
    dispatch: phlow_sdk::tracing::Dispatch,
    peer_addr: SocketAddr,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    
    // Extract request information
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let headers: std::collections::HashMap<String, String> = req.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Read body
    let body_bytes = hyper::body::to_bytes(req.into_body()).await
        .map_err(|_| hyper::Error::from(std::io::Error::new(std::io::ErrorKind::Other, "Body read error")))?;
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    
    // Create request data for Phlow pipeline
    let request_data = json!({
        "method": method,
        "path": path,
        "headers": headers,
        "body": body,
        "client_ip": peer_addr.ip().to_string(),
        "body_size": body_bytes.len()
    });
    
    log::info!("📥 {} {} from {}", method, path, peer_addr.ip());
    
    // Send to Phlow pipeline
    match sender_package!(dispatch, id, sender, Some(request_data.to_value())).await {
        Ok(response_value) => {
            // Parse Phlow response
            if let Some(obj) = response_value.as_object() {
                let status_code = obj.get("status_code")
                    .and_then(|v| v.to_i64().ok())
                    .unwrap_or(200) as u16;
                
                let response_body = obj.get("body")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "OK".to_string());
                
                let mut response_builder = Response::builder().status(status_code);
                
                // Add response headers
                if let Some(headers) = obj.get("headers").and_then(|v| v.as_object()) {
                    for (key, value) in headers {
                        response_builder = response_builder.header(key, value.to_string());
                    }
                }
                
                Ok(response_builder.body(Full::new(Bytes::from(response_body))).unwrap())
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

## Creating Hybrid Modules

Hybrid modules can function as both main and step modules, depending on the context.

### Hybrid Module Example: Message Queue Module

```rust
mod consumer;  // Consumer logic (main mode)
mod producer;  // Producer logic (step mode)
mod config;    // Shared configuration

use phlow_sdk::prelude::*;
use config::QueueConfig;

create_main!(start_queue_module(setup));

pub async fn start_queue_module(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let config = QueueConfig::try_from(&setup.with)?;
    
    if setup.is_main() {
        log::info!("🎯 Running in MAIN mode - Starting consumer");
        
        // Start consumer in background
        let consumer_setup = setup.clone();
        tokio::task::spawn(async move {
            if let Err(e) = consumer::start_consumer(consumer_setup, config.clone()).await {
                log::error!("Consumer error: {}", e);
            }
        });
        
        // Also start producer for step functionality
        producer::start_producer(setup.setup_sender, config).await?;
    } else {
        log::info!("📤 Running in STEP mode - Producer only");
        producer::start_producer(setup.setup_sender, config).await?;
    }
    
    Ok(())
}
```

## The phlow.yaml Schema File

The `phlow.yaml` file defines module metadata, configuration schema, and input/output specifications.

### Basic Schema Structure

```yaml
# Module identification
name: my_module
description: |
  A comprehensive description of what this module does.
  Can be multiline and include examples.
version: 1.0.0
author: Your Name <your.email@example.com>
repository: https://github.com/your-repo/phlow-modules
license: MIT

# Module type
type: step  # or "main" or "any"

# Tags for discovery and categorization
tags:
  - utility
  - data-processing
  - custom

# Configuration schema (used in "with" section)
with:
  type: object
  required: false  # or true if configuration is mandatory
  properties:
    host:
      type: string
      description: "Server hostname"
      default: "localhost"
      required: false
    port:
      type: number
      description: "Server port number"
      default: 3000
      required: false
    timeout:
      type: number
      description: "Timeout in milliseconds"
      default: 5000
      required: false

# Input schema (for step modules)
input:
  type: object
  required: true
  properties:
    data:
      type: any
      description: "Input data to process"
      required: true
    options:
      type: object
      description: "Processing options"
      required: false
      properties:
        format:
          type: string
          enum: ["json", "xml", "text"]
          default: "json"

# Output schema (what the module returns)
output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      description: "Whether the operation succeeded"
      required: true
    result:
      type: any
      description: "Processing result"
      required: false
    error_message:
      type: string
      description: "Error description if operation failed"
      required: false
```

### Schema for Step Module

```yaml
name: data_processor
description: |
  Processes various types of data with configurable transformations.
  Supports JSON, XML, and plain text formats.
version: 1.2.0
author: Developer <dev@example.com>
type: step

with:
  type: object
  required: false
  properties:
    default_format:
      type: string
      enum: ["json", "xml", "text"]
      default: "json"
      description: "Default output format"

input:
  type: object
  required: true
  properties:
    data:
      type: any
      required: true
      description: "Data to be processed"
    format:
      type: string
      enum: ["json", "xml", "text"]
      required: false
      description: "Desired output format"
    validate:
      type: boolean
      default: true
      required: false
      description: "Whether to validate the output"

output:
  type: object
  required: true
  properties:
    processed_data:
      type: any
      required: true
      description: "The processed data"
    format_used:
      type: string
      required: true
      description: "Format that was actually used"
    validation_passed:
      type: boolean
      required: true
      description: "Whether validation passed"
```

### Schema for Main Module

```yaml
name: web_server
description: |
  A flexible HTTP web server that can serve static files,
  handle API requests, and proxy to other services.
version: 2.0.0
author: Developer <dev@example.com>
type: main

with:
  type: object
  required: false
  properties:
    port:
      type: number
      default: 8080
      required: false
      description: "Port to listen on"
    host:
      type: string
      default: "0.0.0.0"
      required: false
      description: "Host address to bind to"
    static_dir:
      type: string
      required: false
      description: "Directory to serve static files from"
    ssl:
      type: object
      required: false
      properties:
        cert_file:
          type: string
          required: true
          description: "SSL certificate file path"
        key_file:
          type: string
          required: true
          description: "SSL private key file path"

output:
  type: object
  required: true
  properties:
    method:
      type: string
      required: true
      description: "HTTP method (GET, POST, etc.)"
    path:
      type: string
      required: true
      description: "Request path"
    headers:
      type: object
      required: true
      description: "Request headers"
    body:
      type: string
      required: true
      description: "Request body"
    query_params:
      type: object
      required: true
      description: "Query parameters"
    client_ip:
      type: string
      required: true
      description: "Client IP address"
```

### Schema for Hybrid Module

```yaml
name: message_queue
description: |
  Flexible message queue module that can act as both consumer and producer.
  
  **Main Mode**: Consumes messages from a queue and forwards to pipeline
  **Step Mode**: Publishes messages to a queue or exchange
version: 1.0.0
author: Developer <dev@example.com>
type: any  # Hybrid module

with:
  type: object
  required: true
  properties:
    connection_url:
      type: string
      required: true
      description: "Message queue connection URL"
    queue_name:
      type: string
      required: true
      description: "Queue name to consume from or publish to"
    exchange:
      type: string
      required: false
      description: "Exchange name (for advanced routing)"
    routing_key:
      type: string
      required: false
      description: "Routing key for message routing"
    consumer_tag:
      type: string
      default: "phlow_consumer"
      required: false
      description: "Consumer tag for identification"

# Input schema (for step mode)
input:
  type: object
  required: true
  properties:
    message:
      type: any
      required: true
      description: "Message to publish"
    headers:
      type: object
      required: false
      description: "Message headers"
    priority:
      type: number
      required: false
      description: "Message priority (0-255)"

# Output schemas (different for each mode)
output:
  type: object
  required: true
  properties:
    # For main mode (consumer)
    message:
      type: any
      required: false
      description: "Received message (consumer mode)"
    headers:
      type: object
      required: false
      description: "Message headers (consumer mode)"
    
    # For step mode (producer)
    published:
      type: boolean
      required: false
      description: "Whether message was published (producer mode)"
    message_id:
      type: string
      required: false
      description: "Published message ID (producer mode)"
```

## Building and Testing Modules

### Cargo.toml Configuration

```toml
[package]
name = "my_module"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A custom Phlow module"
license = "MIT"

[dependencies]
# Core Phlow SDK
phlow-sdk = { workspace = true }  # or version = "0.0.41"

# Additional dependencies as needed
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
log = "0.4"

[lib]
name = "my_module"              # Must match module name
crate-type = ["cdylib"]         # Compile as dynamic library
```

### Building the Module

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# The compiled module will be at:
# target/debug/libmy_module.so (Linux)
# target/debug/libmy_module.dylib (macOS)
# target/debug/my_module.dll (Windows)
```

### Testing Modules Locally

1. **Create a test Phlow file**:

```yaml
# test.phlow
name: Module Test
version: 1.0.0
main: my_module  # if it's a main module
modules:
  - module: my_module
    with:
      # your configuration
steps:
  - use: my_module
    input:
      # your test input
```

2. **Install module locally**:

```bash
# Create local module directory
mkdir -p phlow_packages/my_module

# Copy compiled module
cp target/debug/libmy_module.so phlow_packages/my_module/module.so

# Copy schema
cp phlow.yaml phlow_packages/my_module/
```

3. **Run the test**:

```bash
phlow test.phlow
```

## Best Practices

### 1. Error Handling

Always use proper error handling in your modules:

```rust
pub async fn my_function(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        
        match process_input(&input) {
            Ok(result) => {
                sender_safe!(package.sender, result.into());
            }
            Err(e) => {
                log::error!("Processing error: {}", e);
                let error_response = json!({
                    "success": false,
                    "error": e.to_string()
                });
                sender_safe!(package.sender, error_response.to_value().into());
            }
        }
    });
    
    Ok(())
}
```

### 2. Configuration Validation

Validate configuration early:

```rust
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
}

impl TryFrom<&Value> for Config {
    type Error = ConfigError;
    
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let port = value.get("port")
            .and_then(|v| v.to_i64().ok())
            .and_then(|p| if p > 0 && p <= 65535 { Some(p as u16) } else { None })
            .ok_or_else(|| ConfigError::InvalidPort)?;
        
        // ... validate other fields
        
        Ok(Self { host, port, timeout_ms })
    }
}
```

### 3. Proper Logging

Use structured logging throughout your module:

```rust
log::info!("Module started with config: {:?}", config);
log::debug!("Processing input: {:?}", input);
log::warn!("Retrying operation after error: {}", error);
log::error!("Fatal error in module: {}", error);
```

### 4. Resource Cleanup

Ensure proper cleanup of resources:

```rust
pub async fn start_server(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _cleanup_guard = CleanupGuard::new();
    
    // Your server logic here
    
    Ok(())
}

struct CleanupGuard;

impl CleanupGuard {
    fn new() -> Self {
        Self
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        log::debug!("Cleaning up resources");
        // Cleanup logic here
    }
}
```

### 5. Testing Support

Always handle test mode properly:

```rust
pub async fn start_server(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if setup.is_test_mode {
        log::debug!("Test mode detected, using mock services");
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }
    
    // Normal operation
}
```

## Complete Examples

### Example 1: File Processing Step Module

```rust
// src/lib.rs
use phlow_sdk::prelude::*;
use std::path::Path;
use tokio::fs;

create_step!(process_file(rx));

#[derive(Debug)]
struct FileOperation {
    path: String,
    operation: String,
    content: Option<String>,
}

impl From<&Value> for FileOperation {
    fn from(value: &Value) -> Self {
        let path = value.get("path").unwrap_or(&Value::Null).to_string();
        let operation = value.get("operation").unwrap_or(&Value::Null).to_string();
        let content = value.get("content").map(|v| v.to_string());
        
        Self { path, operation, content }
    }
}

pub async fn process_file(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("File processor module started");

    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        let file_op = FileOperation::from(&input);
        
        let result = match file_op.operation.as_str() {
            "read" => read_file(&file_op.path).await,
            "write" => write_file(&file_op.path, file_op.content.as_deref().unwrap_or("")).await,
            "exists" => check_exists(&file_op.path).await,
            "delete" => delete_file(&file_op.path).await,
            _ => Err(format!("Unknown operation: {}", file_op.operation)),
        };
        
        let response = match result {
            Ok(content) => json!({
                "success": true,
                "result": content
            }),
            Err(error) => json!({
                "success": false,
                "error": error
            })
        };
        
        sender_safe!(package.sender, response.to_value().into());
    });

    Ok(())
}

async fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).await
        .map_err(|e| format!("Failed to read file: {}", e))
}

async fn write_file(path: &str, content: &str) -> Result<String, String> {
    fs::write(path, content).await
        .map(|_| "File written successfully".to_string())
        .map_err(|e| format!("Failed to write file: {}", e))
}

async fn check_exists(path: &str) -> Result<String, String> {
    Ok(Path::new(path).exists().to_string())
}

async fn delete_file(path: &str) -> Result<String, String> {
    fs::remove_file(path).await
        .map(|_| "File deleted successfully".to_string())
        .map_err(|e| format!("Failed to delete file: {}", e))
}
```

With corresponding `phlow.yaml`:

```yaml
name: file_processor
description: |
  File processing module that can read, write, check existence, and delete files.
  Useful for file-based workflows and data processing pipelines.
version: 1.0.0
author: Developer <dev@example.com>
type: step

tags:
  - file
  - io
  - filesystem
  - utility

input:
  type: object
  required: true
  properties:
    path:
      type: string
      required: true
      description: "File path to operate on"
    operation:
      type: string
      required: true
      enum: ["read", "write", "exists", "delete"]
      description: "Operation to perform"
    content:
      type: string
      required: false
      description: "Content to write (required for write operation)"

output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      required: true
      description: "Whether the operation succeeded"
    result:
      type: string
      required: false
      description: "Operation result (file content, success message, etc.)"
    error:
      type: string
      required: false
      description: "Error message if operation failed"
```

### Example 2: CLI Application Main Module

```rust
// src/lib.rs
use phlow_sdk::prelude::*;
use std::env;

create_main!(start_cli(setup));

#[derive(Debug)]
struct CliConfig {
    args: Vec<String>,
    command: Option<String>,
}

impl From<Value> for CliConfig {
    fn from(value: Value) -> Self {
        let args = env::args().collect();
        let command = value.get("command").map(|v| v.to_string());
        
        Self { args, command }
    }
}

pub async fn start_cli(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    if !setup.is_main() {
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    if setup.is_test_mode {
        // In test mode, use mock arguments
        let test_data = json!({
            "command": "test",
            "args": ["phlow", "test", "--verbose"],
            "flags": {
                "verbose": true
            }
        });
        
        // Send test data to pipeline
        if let Some(sender) = setup.main_sender {
            let _ = sender_package!(setup.dispatch, setup.id, sender, Some(test_data.to_value())).await;
        }
        
        sender_safe!(setup.setup_sender, None);
        return Ok(());
    }

    let config = CliConfig::from(setup.with);
    
    // Parse command line arguments
    let cli_data = parse_cli_args(&config.args);
    
    log::info!("CLI started with command: {:?}", cli_data.get("command"));
    
    // Signal setup complete
    sender_safe!(setup.setup_sender, None);
    
    // Send CLI data to pipeline for processing
    if let Some(sender) = setup.main_sender {
        match sender_package!(setup.dispatch, setup.id, sender, Some(cli_data.to_value())).await {
            Ok(result) => {
                // Print result to stdout
                println!("{}", result.to_string());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

fn parse_cli_args(args: &[String]) -> Value {
    let mut command = None;
    let mut flags = std::collections::HashMap::new();
    let mut positional = Vec::new();
    
    let mut i = 1; // Skip program name
    while i < args.len() {
        let arg = &args[i];
        
        if arg.starts_with("--") {
            // Long flag
            let flag_name = &arg[2..];
            if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                flags.insert(flag_name.to_string(), Value::from(args[i + 1].clone()));
                i += 2;
            } else {
                flags.insert(flag_name.to_string(), Value::from(true));
                i += 1;
            }
        } else if arg.starts_with("-") {
            // Short flag
            let flag_name = &arg[1..];
            flags.insert(flag_name.to_string(), Value::from(true));
            i += 1;
        } else {
            // Positional argument
            if command.is_none() {
                command = Some(arg.clone());
            } else {
                positional.push(Value::from(arg.clone()));
            }
            i += 1;
        }
    }
    
    json!({
        "command": command.unwrap_or_else(|| "default".to_string()),
        "args": positional,
        "flags": flags,
        "raw_args": args
    }).to_value()
}
```

These examples demonstrate the key concepts and patterns for building robust Phlow modules. Each module should be self-contained, well-documented, and handle edge cases appropriately.

Remember to thoroughly test your modules and provide clear documentation for users who will integrate them into their Phlow workflows.
