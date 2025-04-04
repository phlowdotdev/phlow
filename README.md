<p align="center">
  <img src="./docs/phlow.svg" alt="Phlow logo" width="100"/>
  <h1 align="center">Phlow</h1>
</p>

## Modular Flow Runtime for Composable Backends

**Phlow** is a high-performance and highly composable flow runtime built with Rust. It provides a new way to build backend systems using declarative configuration, modular logic, and pluggable runtime behavior — all driven by YAML, JSON, or TOML.

Phlow comes with **native support for OpenTelemetry**, enabling full observability of flows, modules, and runtime execution. You can export traces and metrics to systems like **Jaeger**, **Grafana Tempo**, or **AWS X-Ray** with minimal configuration using environment variables.

Whether you're building APIs, consumers, automations, or event-driven systems, Phlow makes it easy to connect logic, transform data, and monitor everything — all in a composable and developer-friendly way.


---

## 📚 Table of Contents

- [🎯 Philosophy](#-philosophy)
- [🔌 Module Types](#-module-types)
- [🧱 Example: `main.yaml` for an HTTP Gateway](#-example-mainyaml-for-an-http-gateway)
- [🧩 YAML Superpowers](#-yaml-superpowers)
- [⚙️ Install & Usage](#%EF%B8%8F-installation--usage)
- [🧠 Creating Your Own Module: `log`](#-creating-your-own-module-log)
- [📦 Project Structure](#-project-structure)
- [📡 Observability](#-observability)
- [🧪 OpenTelemetry + Jaeger (Local Dev Setup)](#-opentelemetry--jaeger-local-dev-setup)
- [🌍 Environment Settings](#-environment-settings)
- [📜 License](#-license)

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
main: gateway

modules:
    - name: gateway
        module: rest_api
        with:
            host: 0.0.0.0
            port: 3000

    - name: request
        module: http_request
        with:
            timeout: 29000 # 29s

steps:
    - condition:
        assert: !eval main.path.start_with("/public")
        then:
            module: request
            input:
                method: !eval main.method
                url: !eval `public-service.local${main.uri}?` 
                headers:
                    x-forwarded-for: !eval main.client_ip
                    x-original-path: !eval main.path   
                body: !eval main.body
    - use: authorization
        id: auth
        input:
            api_key: !eval main.header.authorization
    - condition:
        assert: !eval steps.auth.authorized == true          
        then:
            module: request
            with:
                method: !eval main.method
                url: !eval `private-service.local${main.uri}?` 
                headers:
                    x-forwarded-for: !eval main.client_ip
                    x-original-path: !eval main.path   
                body: !eval main.body
    - return:
        status_code: 401
        body: {
            "message": "unauthorized",
            "code": 401
        }
```
---

## 🧩 YAML Superpowers

Phlow extends YAML with:

- `!eval`: execute inline expressions using Phlow Script (phs).
- `!include`: include other YAML files into the flow tree.
- `!import`: import external script files (.phs or .rhai) and evaluate them with `!eval`.

---

## ⚙️ Installation & Usage

Install Phlow globally using Cargo:

```bash
cargo install phlow-runtime
```

### 🔧 Running a Flow

By default, Phlow will look for a \`main.yaml\` in the current directory:

```bash
phlow
```

To run a specific file:

```bash
phlow path/to/your-flow.yaml
```

If you provide a directory path and it contains a \`main.yaml\`, Phlow will automatically run that:

```bash
phlow path/to/directory
# → runs path/to/directory/main.yaml
```

### 🆘 Help

For all available options and usage info:

```bash
phlow -h
# or
phlow --help
```
---

## 🧠 Creating Your Own Module: `log`

Phlow modules are written in Rust and compiled as shared libraries. Here’s a real example of a simple **log module** that prints messages at various log levels.

### 🔧 Code (`src/lib.rs`)

```rust
use phlow_sdk::{
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
you_project/
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
All compiled `.so` modules **must be placed inside the `phlow_modules/` directory**.

To build all modules at once, this project includes a utility script:
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

## 🧪 OpenTelemetry + Jaeger (Local Dev Setup)

To enable observability with **Jaeger** during development, you can run a full OpenTelemetry-compatible collector locally in seconds.

### 🔄 1. Run Jaeger with OTLP support

```bash
docker run -d \
  -p4318:4318 \  # OTLP HTTP
  -p4317:4317 \  # OTLP gRPC
  -p16686:16686 \  # Jaeger UI
  jaegertracing/all-in-one:latest
```
This container supports OTLP over HTTP and gRPC, which are both compatible with Phlow's OpenTelemetry output.

---

### ⚙️ 2. Configure environment variables

Set the following environment variables in your shell or `.env` file:

```bash
export OTEL_RESOURCE_ATTRIBUTES="service.name=phlow-dev,service.version=0.1.0"
export OTEL_SERVICE_NAME="phlow-dev"
```
---

### 🔍 3. Open the Jaeger UI

Once running, access the Jaeger web interface at:

[http://localhost:16686](http://localhost:16686)

Search for your service using the name defined in `OTEL_SERVICE_NAME`.

---

### ✅ Tips

- Combine this with `PHLOW_OTEL=true`, `PHLOW_SPAN=INFO`, and `PHLOW_LOG=DEBUG` for full observability.
- You can also integrate with **Grafana Tempo** or **AWS X-Ray** by replacing the collector backend.


---
## 🌍 Environment Settings

Below is a list of **all** environment variables used by the application, combining those defined in both files, along with their descriptions, default values, and types.

### Environment Variables Table

| Variable                                        | Description                                                                                                                       | Default Value | Type    |
|-------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------|----------------|---------|
| **PHLOW_PACKAGE_CONSUMERS_COUNT**               | **Number of package consumers**<br>Defines how many threads will be used to process packages.                                     | `10`           | `i32`   |
| **PHLOW_MIN_ALLOCATED_MEMORY_MB**               | **Minimum allocated memory (MB)**<br>Defines the minimum amount of memory, in MB, allocated to the process.                       | `10`           | `usize` |
| **PHLOW_GARBAGE_COLLECTION_ENABLED**            | **Enable garbage collection**<br>Enables or disables garbage collection (GC).                                                     | `true`         | `bool`  |
| **PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS**   | **Garbage collection interval (seconds)**<br>Defines the interval at which garbage collection will be performed.                  | `60`           | `u64`   |
| **PHLOW_LOG**                                   | **Log level**<br>Defines the log verbosity for standard logging output. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `WARN` | `str`   |
| **PHLOW_SPAN**                                  | **Span level**<br>Defines the verbosity level for span (OpenTelemetry) tracing. Possible values typically include `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. | `INFO` | `str`   |
| **PHLOW_OTEL**                                  | **Enable OpenTelemetry**<br>Enables or disables OpenTelemetry tracing and metrics.                                                | `true`         | `bool`  |

---

### Notes

- If an environment variable is not set, the default value indicated in the table above will be used.
- Set the corresponding environment variables before running the application to override the defaults.
- The **log level** (`PHLOW_LOG`) and **span level** (`PHLOW_SPAN`) control different layers of logging:
  - `PHLOW_LOG`: Affects standard logging (e.g., error, warning, info messages).
  - `PHLOW_SPAN`: Affects tracing spans (useful for deeper telemetry insights with OpenTelemetry).
- The `PHLOW_OTEL` variable controls whether or not OpenTelemetry providers (for both tracing and metrics) are initialized.

---

## 📜 License

MIT © 2025 — Built with ❤️ and Rust.
