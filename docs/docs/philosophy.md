---
sidebar_position: 2
title: Philosophy
---

###  1. Radical Modularity  
**Principle:** *Each piece must be independent, reusable, and pluggable.*

Phlow is designed as a set of decoupled modules. You connect functionalities like LEGO blocks, allowing you to replace or evolve parts without breaking the whole. This promotes maintainability and system flexibility.

---

### 2. Code-Free Composition (Low Code)  
**Principle:** *The flow matters more than the language.*

Business logic is declared using simple files like YAML. Instead of programming behavior, you **compose** it. This empowers both developers and analysts to build together, democratizing software creation.

---

### 3. High-Performance Runtime  
**Principle:** *Performance is not a detail — it's architecture.*

Phlow is built in **Rust**, ensuring memory safety, low resource consumption, and blazing speed. It runs anywhere — locally, on the edge, or in the cloud — with minimal latency and maximum scalability.

---

###  4. Automatic Module Installation  
**Principle:** *The user experience should be instant.*

Phlow detects the required modules and automatically downloads them from the official `phlow-packages` repository. Everything is installed locally under `./phlow-packages`, with no manual setup or external dependencies.

---

###  5. Observability by Design  
**Principle:** *You can only improve what you can observe.*

Every flow and module is traceable with **logs, metrics, and spans** via OpenTelemetry. Real-time tracking with Jaeger, Grafana, or Prometheus is built-in. Transparency and traceability are part of the system’s DNA.
