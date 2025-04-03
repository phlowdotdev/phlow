# 🌀 Phlow — Modular Flow Runtime for Composable Backends

**Phlow** is a high-performance and highly composable flow runtime built with Rust. It provides a new way to build backend systems using declarative configuration, modular logic, and pluggable runtime behavior — all driven by YAML, JSON, or TOML.

Whether you're creating APIs, consumers, automations, or event-driven systems, Phlow makes it easy to connect logic, transform data, and build applications like pipelines.

---

## 🎯 Philosophy

Phlow was built around the following principles:

### 1. **Flow over Frameworks**
Forget bulky frameworks. Phlow embraces flows. Each step is modular, each behavior is pluggable. You define what happens, when, and how — all through configuration and small, focused modules.

### 2. **Composability**
Phlow encourages building **small pieces** that fit together. Each module can:
- Run logic (`step module`)
- Start the system (`main module`)
- Interact via `input` and `output`
- Be swapped, reused, or extended easily.

### 3. **Extensibility with Scripts**
Need logic? Use `phs` (Phlow Script) or `rhai`. Define logic inline or in external files. You don't need to recompile to change behavior — just change the YAML.

### 4. **Observability First**
Every module, flow, and step can be traced using `tracing` and OpenTelemetry. You'll always know **where**, **why**, and **how** something happened.

### 5. **Separation of Control and Behavior**
Control lives in YAML (`steps`, `conditions`, `includes`). Behavior lives in modules and scripts. You can mix and match at will.

---

## 🔌 Module Types

| Type         | Purpose                                 |
|--------------|------------------------------------------|
| `main module`| Entry point. Starts the app (HTTP, CLI, AMQP, etc). |
| `step module`| Logic executed within a flow (log, fetch, transform, etc). |

---

## 🧱 Example: `main.yaml` for an HTTP Gateway

```yaml
main: restapi

modules:
  - name: restapi
    module: restapi
    with:
      host: 0.0.0.0
      port: 3000

  - name: request
    module: request

steps:
  - id: gateway
    module: request
    with:
      method: GET
      url: !eval main.path.replace("/proxy/", "https://")
      headers:
        x-forwarded-for: !eval main.client_ip
        x-original-path: !eval main.path
      query: !eval main.query_params
      body: !eval main.body
```
---

## 🧩 YAML Superpowers

Phlow extends YAML with:

- `!eval`: execute inline expressions using Phlow Script (phs).
- `!include`: include other YAML files into the flow tree.
- `!import`: import external script files (.phs or .rhai) and evaluate them with `!eval`.

---

## 🧠 Creating Your Own Module: `log`

Phlow modules are written in Rust and compiled as shared libraries. Here’s a real example of a simple **log module** that prints messages at various log levels.

### 🔧 Code (`src/lib.rs`)

```rust
use sdk::{
    crossbeam::channel,
    modules::ModulePackage,
    prelude::*,
    tracing::{debug, error, info, warn},
};

plugin!(log);

enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

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

pub fn log(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    setup.setup_sender.send(Some(tx)).ok();

    for package in rx {
        let log = match package.context.input {
            Some(value) => Log::from(&value),
            _ => Log {
                level: LogLevel::Info,
                message: "".to_string(),
            },
        };

        match log.level {
            LogLevel::Info => info!("{}", log.message),
            LogLevel::Debug => debug!("{}", log.message),
            LogLevel::Warn => warn!("{}", log.message),
            LogLevel::Error => error!("{}", log.message),
        }

        sender_safe!(package.sender, Value::Null);
    }

    Ok(())
}
```
---

### 🛠️ Example usage in a flow

```yaml
steps:
  - id: notify
    module: log
    with:
      level: info
      message: "Process started"

  - use: log
    with:
      level: error
      message: !eval "something went wrong: " + main.error
```
---

## 📦 Project Structure

```bash
phlow/
├── main.yaml
├── modules.yaml
├── assets/
│   └── body.yaml
├── scripts/
│   └── resolve_url.phs
├── phlow_modules/
│   ├── restapi/
│   │   └── module.so
│   ├── request/
│   │   └── module.so
│   └── log/
│       └── module.so
```
---

## 📡 Observability

Phlow integrates with:

- OpenTelemetry (OTLP)
- Tracing (Spans and Logs)
- Prometheus Metrics
- Jaeger, Grafana Tempo, AWS X-Ray

Enable it with:

```env
PHLOW_OTEL=true
PHLOW_LOG=DEBUG
PHLOW_SPAN=INFO
```
---

## 📜 License

MIT © 2025 — Built with ❤️ and Rust.
