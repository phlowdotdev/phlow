---
sidebar_position: 3
title: Creating Your Own Module
---

Phlow modules are written in Rust and compiled as shared libraries. Here’s a real example of a simple **log module** that prints messages at various log levels.

### Cargo.toml

This file defines the module's metadata and dependencies. [`phlow-sdk`](https://crates.io/crates/phlow-sdk) is the core library for building Phlow modules, while `log` and `env_logger` are used for logging functionality.

```toml
[package]
name = "log"
version = "0.1.0"
edition = "2021"

[dependencies]
phlow-sdk = "0.1"
log = { version = "0.4" }
env_logger = { version = "0.11" }

[lib]
name = "log"
crate-type = ["cdylib"]

```

### src/lib.rs

This file contains the main logic of the module. It listens for incoming messages and logs them based on their severity level. The `log` module is defined with a `log` function that takes a `ModuleReceiver` as an argument. It initializes the logger and listens for incoming messages, logging them according to their specified log level.

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
struct Log {
    level: LogLevel,
    message: String,
}

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

pub async fn log(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .default_filter_or("info")
            .filter_or("PHLOW_LOG", "info"),
    )
    .init();

    debug!("PHLOW_OTEL is set to false, using default subscriber");

    listen!(rx, move |package: ModulePackage| async {
        let value = package.input.unwrap_or(Value::Null);
        let log = Log::from(&value);

        match log.level {
            LogLevel::Info => info!("{}", log.message),
            LogLevel::Debug => debug!("{}", log.message),
            LogLevel::Warn => warn!("{}", log.message),
            LogLevel::Error => error!("{}", log.message),
        }

        sender_safe!(package.sender, Value::Null.into());
    });

    Ok(())
}
```

### phlow.yaml

This file describes the module's metadata, including its name, description, version, author, repository, license, type, and tags. It also defines the input schema for the module, specifying the required properties and their types.

**phlow.yaml** is required for Phlow to recognize the module. The file must contain the following information:

| Field       | Description                                                              | Example | Type |
|-------------|--------------------------------------------------------------------------|-------|------|
| name        | Name of the module.                                                      | my_module | string |
| description | Description of the module.                                               | This is my module | string |
| version     | Version of the module.                                                  | 0.1.0 (major.minor.patch)| string|
| author      | Author of the module.                                                   | John Doe \<email@email.c>  |string |
| repository  | URL of the module's repository.                                         | github.com/my_repo/my_phlow| string |
| license     | License of the module.                                                  | MIT ou ./LICENSE | string ou path |
| type        | Type of the module                                                      | step, main or any |
| tags        | Tags associated with the module.                                        | [http, web, request] | array |
| input       | Input schema of the module, defining the properties and their types.    | [object schema](#object-schema) | object |
| output      | Output schema of the module, defining the properties and their types.   | [object schema](#object-schema) | object |
| with        | Additional configurations for the module.                               | `{database: postgres, username: postgres}` | object |


```yaml
name: log
description: Log a message to the console.
version: 0.0.1
author: Philippe Assis <codephilippe@gmail.com>
repository: https://github.com/lowcarboncode/phlow
license: MIT
type: step
tags:
  - log
  - echo
  - print
input: 
  type: object
  required: true
  properties:
    level:
      type: string
      description: The log level (e.g., info, debug, warn, error).
      default: info
      required: false
    message:
      type: string
      description: The message to log.
      required: true
```
### Object Schema

The input and output schemas are defined using a format similar to [JSON Schema Draft 7](https://json-schema.org/draft-07), with some additional properties such as `default`. The `input` schema specifies the expected input format for the module, while the `output` schema defines the format of the output produced by the module.


| Field       | Description                                                              | Value | Type | 
|-------------|--------------------------------------------------------------------------|-------|----|
| type        | Type of the schema.                                                     | object, array, number, string, boolean, null| string |
| properties  | Properties of the object schema.                                        | object schema | object |
| required    | Required properties of the object schema.                               | array of strings | array |
| items       | Items of the array schema.                                             | object schema | object |
| enum        | Enum values of the schema.                                             | array of strings | array |
| default     | Default value of the schema.                                           | any type | any |
| description | Description of the schema.                                             | string | string |
| properties | Properties of the object schema.                                        | object schema | object |