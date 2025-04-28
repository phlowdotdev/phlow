<p align="center">
  <img src="./docs/static/img/logo.svg" alt="Phlow logo" width="160"/>
  <h1 align="center">Phlow</h1>
</p>

<h2 align="center">Modular Flow Runtime for Composable Backends</h2>

**Phlow** is a **high-performance, scalable, and Low Code flow runtime** built with Rust — designed to revolutionize the way you build backends. With Phlow, you can **create APIs, automations, and event-driven systems using just YAML**, composing logic like building blocks.

Thanks to its modular architecture and clear separation between control and behavior, Phlow lets you **orchestrate complex flows without coding** — and when you need extra power, simply **plug in lightweight scripts or Rust modules**.


It comes with **built-in observability powered by OpenTelemetry**, giving you full visibility into your flows, modules, and executions. Easily export traces and metrics to **Jaeger**, **Grafana Tempo**, or **AWS X-Ray**, all with simple environment variables.

If you're looking for speed, flexibility, and full insight into your backend — **Phlow is the Low-Code revolution you’ve been waiting for**.

## 📚 Documentation

The complete documentation is available at [https://phlow.dev](https://phlow.dev).

## 🚀 Getting Started

### Installation
You can easily install or uninstall Phlow using our ready-to-use shell scripts.

#### Install via `curl`

```bash
curl -fsSL https://raw.githubusercontent.com/lowcarboncode/phlow/main/scripts/install-phlow.sh | bash
```

####  Install via `wget`

```bash
wget -qO- https://raw.githubusercontent.com/lowcarboncode/phlow/main/scripts/install-phlow.sh | bash
```
---

## 🛠️ Usage

### Run a Flow
```bash
phlow git@github.com:lowcarboncode/phlow-mirror-request.git
```

> Phlow mirror request is a simple example of a flow that mirrors requests to a given URL. It demonstrates how to use Phlow to create a flow that can handle incoming requests and forward them to another service.

## 📜 License

© 2025 — Built with ❤️ and Rust. Licensed under [MIT License](License).


