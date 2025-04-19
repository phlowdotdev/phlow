# 🧪 Phlow Examples

This folder contains ready-to-run examples to help you learn how to use **Phlow** in different scenarios.

Each example demonstrates how to compose flows using YAML, modules, conditions, scripting, and more.

---

## 🚀 How to Run an Example

First, make sure you have Phlow installed:

```bash
cargo install phlow-runtime
```

Then, run an example like this:

```bash
phlow examples/EXAMPLE_NAME
```

Example:

```bash
phlow examples/api_proxy
```

Phlow will automatically detect the `main.yaml` inside the specified folder.

---

## 📂 Available Examples

- `api_proxy` – A simple API gateway that proxies requests
- `api-postgres` – Example of API interacting with a PostgreSQL database
- `cli_to_steps` – CLI-based trigger that executes a flow
- `macros` – Demonstrates usage of YAML macros or includes
- `restapi-ampq` – Connects REST APIs to AMQP message queues
- `restapi-echo` – Echo service for testing HTTP input/output
- `restapi-ping` – Lightweight ping endpoint
- `restapi-sleep` – Simulates latency and delay with a sleep module

---

Feel free to explore, modify, and build your own flows based on these examples!

---

MIT © 2025 — Built with ❤️ and Rust.
