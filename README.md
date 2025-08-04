<p align="center">
  <img src="./site/static/img/logo.svg" alt="Phlow logo" width="160"/>
  <h1 align="center">Phlow</h1>
</p>

<h2 align="center">Modular Flow Runtime for Composable Backends</h2>

**Phlow** is a **high-performance**, **low-code** flow runtime built in **Rust** — crafted to transform the way you build backends.
With Phlow, you design APIs, automations, and event-driven systems by composing YAML flows, treating logic as modular building blocks.

Its radically **modular** architecture separates control from behavior, empowering you to orchestrate complex workflows without writing traditional code.
Need more flexibility? Simply extend with lightweight scripts or Rust-based modules — no rewrites, no complexity.

**Observability** is built-in by design. Every flow and module emits traces, logs, and metrics through **OpenTelemetry**, integrating seamlessly with Jaeger, Grafana Tempo, Prometheus, or AWS X-Ray — all via simple environment variables.

Whether you’re running locally, on the edge, or across the cloud, Phlow delivers extreme speed, effortless scalability, and full-stack visibility.
If you're ready to rethink how backends are built — **Phlow is the low-code revolution you’ve been waiting for**.

> ⚠️ **Warning:** Phlow is currently under active development and is not recommended for use in production environments at this time.

## 📚 Documentation

The complete documentation is available at [https://phlow.dev](https://phlow.dev).

## 🎨 VS Code Extension

Get syntax highlighting, IntelliSense, and enhanced development experience for Phlow files:

[![Install VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/phlow.phlow?style=flat-square&label=VS%20Code%20Extension)](https://marketplace.visualstudio.com/items?itemName=phlow.phlow)

- **Marketplace**: [phlow.phlow](https://marketplace.visualstudio.com/items?itemName=phlow.phlow)
- **Repository**: [phlowdotdev/vscode](https://github.com/phlowdotdev/vscode)

## Quick Start

### Install Phlow
```bash
curl -fsSL https://raw.githubusercontent.com/phlowdotdev/phlow/main/scripts/install-phlow.sh | bash
```

### Run a demo
```bash
phlow git@github.com:phlowdotdev/phlow-mirror-request.git
```

### Try it now on GitHub Codespaces

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?repo=phlowdotdev/phlow-mirror-request)

## 📜 License

© 2025 — Built with ❤️ and Rust. Licensed under [MIT License](License).


