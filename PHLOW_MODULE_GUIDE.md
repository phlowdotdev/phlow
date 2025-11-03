# Complete Phlow Module Development Guide

> A practical and detailed guide for creating custom modules for Phlow, using the Cache module as a real implementation example.

## 📑 Table of Contents

1. [Introduction](#introduction)
2. [Module Architecture](#module-architecture)
3. [Module Types](#module-types)
4. [Anatomy of a Step Module: Cache](#anatomy-of-a-step-module-cache)
5. [File Structure](#file-structure)
6. [Cargo.toml Configuration](#cargotoml-configuration)
7. [Detailed Implementation](#detailed-implementation)
8. [phlow.yaml Schema](#phlowyaml-schema)
9. [Tests and Examples](#tests-and-examples)
10. [Build and Deploy](#build-and-deploy)
11. [Best Practices](#best-practices)

---

## Introduction

Phlow is a high-performance modular runtime built in Rust for creating composable backends. Modules are the fundamental building blocks that provide specific functionalities that can be combined to create complex workflows.

### Why use Cache as an example?

The Cache module is an excellent example because it demonstrates:
- ✅ **Action-Based Pattern**: Multiple operations in a single module
- ✅ **State Management**: Use of thread-safe shared structures
- ✅ **Flexible Configuration**: Options via `with` section
- ✅ **Input Validation**: Robust parsing with Rust enums
- ✅ **Statistics**: Operation metrics tracking
- ✅ **Modular Organization**: Separation of concerns across multiple files

---

## Module Architecture

Each Phlow module consists of three essential components:

```
my_module/
├── Cargo.toml          # Rust package configuration
├── phlow.yaml          # Module schema and metadata
└── src/
    ├── lib.rs          # Main entry point
    ├── config.rs       # Module configuration
    ├── input.rs        # Input definitions
    └── stats.rs        # Statistics (optional)
```

### Fundamental Requirements

1. **Rust Library**: Must be compiled as a dynamic library (`cdylib`)
2. **Async Functions**: All module functions must be asynchronous
3. **Phlow SDK**: Must use the `phlow-sdk` crate
4. **Appropriate Macros**: Use `create_step!`, `create_main!` or both
5. **Complete Schema**: Have a well-defined `phlow.yaml` file

---

## Module Types

### 1. Step Module (`type: step`)
- **Purpose**: Process data within a pipeline
- **Usage**: `use: module_name` in steps
- **Examples**: cache, log, data transformation

### 2. Main Module (`type: main`)
- **Purpose**: Serve as application entry point
- **Usage**: `main: module_name` in flow file
- **Examples**: HTTP server, CLI, message consumer

### 3. Hybrid Module (`type: any`)
- **Purpose**: Function as both main AND step
- **Usage**: Flexible depending on context
- **Examples**: AMQP (consumer when main, producer when step)

---

## Anatomy of a Step Module: Cache

The Cache module is a **Step Module** that implements high-performance in-memory caching using the QuickLeaf library. Let's explore each aspect of its implementation.

### Cache Module Overview

```yaml
Features:
  - In-memory key-value storage
  - Automatic TTL (Time To Live)
  - LRU (Least Recently Used) eviction
  - Advanced filtering (prefix, suffix, pattern)
  - Real-time statistics
  - O(1) operations for get/set

Supported Actions:
  - set      # Store data
  - get      # Retrieve data
  - remove   # Remove entry
  - clear    # Clear cache
  - exists   # Check existence
  - list     # List entries with filters
  - cleanup  # Clean up expired items
  - stats    # Get statistics
```

---

## File Structure

### Cache Module Structure

```
modules/cache/
├── Cargo.toml          # Dependencies and configuration
├── phlow.yaml          # Module schema
└── src/
    ├── lib.rs          # Main implementation (531 lines)
    ├── config.rs       # Cache configuration (58 lines)
    ├── input.rs        # Input parsing (219 lines)
    └── stats.rs        # Statistics tracking (95 lines)
```

### Why separate into multiple files?

```rust
// ❌ Everything in lib.rs = hard to maintain
// ✅ Clear separation = easy to understand and modify

lib.rs      → Business logic and handlers
config.rs   → Configuration validation
input.rs    → Input parsing and validation
stats.rs    → Metrics and statistics
```

---

## Cargo.toml Configuration

### Cache Module Cargo.toml

```toml
[package]
name = "cache"
version = "0.1.0"
edition = { workspace = true }
authors = ["Philippe Assis <codephilippe@gmail.com>"]
description = "Phlow cache module using QuickLeaf for high-performance in-memory caching"
license = "MIT"

[dependencies]
# Core Phlow SDK (required)
phlow-sdk = { workspace = true }

# Cache implementation
quickleaf = "0.4.10"

# Auxiliary dependencies
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
log = "0.4"

[lib]
name = "cache"              # Module name
crate-type = ["cdylib"]     # CRITICAL: Compile as dynamic library
```

### Key Points

1. **`crate-type = ["cdylib"]`**: Essential for Phlow to load the module
2. **`phlow-sdk`**: Always use workspace = true in official modules
3. **Consistent naming**: The name in `[lib]` must match the name in `phlow.yaml`

---

## Detailed Implementation

### 1. Configuration File (config.rs)

The `config.rs` defines how the module is configured via the `with` section in the `.phlow` file.

```rust
use phlow_sdk::prelude::*;

/// Configuração para o módulo cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub capacity: usize,
    pub default_ttl: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,      // Capacidade padrão: 1000 itens
            default_ttl: None,   // Sem TTL padrão
        }
    }
}

impl TryFrom<&Value> for CacheConfig {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let mut config = CacheConfig::default();

        // Parse capacity
        if let Some(capacity) = value.get("capacity") {
            match capacity.to_i64() {
                Some(cap) if cap > 0 => {
                    config.capacity = cap as usize;
                }
                Some(_) => {
                    return Err("Capacity must be a positive number".to_string());
                }
                None => {
                    return Err("Invalid capacity value".to_string());
                }
            }
        }

        // Parse default_ttl
        if let Some(ttl) = value.get("default_ttl") {
            match ttl.to_i64() {
                Some(ttl_value) if ttl_value > 0 => {
                    config.default_ttl = Some(ttl_value as u64);
                }
                Some(_) => {
                    return Err("Default TTL must be a positive number".to_string());
                }
                None => {
                    return Err("Invalid default_ttl value".to_string());
                }
            }
        }

        Ok(config)
    }
}
```

**Uso no arquivo .phlow:**

```yaml
modules:
  - module: cache
    with:
      capacity: 5000        # Máximo 5000 itens
      default_ttl: 3600     # 1 hora padrão
```

### 2. Definições de Entrada (input.rs)

O `input.rs` define todas as ações possíveis usando enums do Rust com serde.

```rust
use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Ações de entrada do cache
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]  // Usa campo "action" como discriminador
pub enum CacheInput {
    #[serde(rename = "set")]
    Set {
        key: String,
        value: Value,
        ttl: Option<u64>,
    },
    
    #[serde(rename = "get")]
    Get { 
        key: String 
    },
    
    #[serde(rename = "remove")]
    Remove { 
        key: String 
    },
    
    #[serde(rename = "clear")]
    Clear,
    
    #[serde(rename = "exists")]
    Exists { 
        key: String 
    },
    
    #[serde(rename = "list")]
    List {
        filter_type: String,
        filter_value: Option<String>,
        filter_prefix: Option<String>,
        filter_suffix: Option<String>,
        order: String,
        limit: Option<u64>,
        offset: u64,
    },
    
    #[serde(rename = "cleanup")]
    Cleanup,
    
    #[serde(rename = "stats")]
    Stats,
}
```

**Implementação do parsing customizado:**

```rust
impl TryFrom<Option<Value>> for CacheInput {
    type Error = String;

    fn try_from(input_value: Option<Value>) -> Result<Self, Self::Error> {
        let input_value = input_value.ok_or("Missing input for cache module")?;

        if !input_value.is_object() {
            return Err("Cache input must be an object".to_string());
        }

        // Extrair action (obrigatório)
        let action = match input_value.get("action") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'action' field in cache input".to_string()),
        };

        // Match baseado na action
        match action.as_str() {
            "set" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for set action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for set action".to_string());
                }

                let value = input_value
                    .get("value")
                    .ok_or("Missing 'value' field for set action")?
                    .clone();

                let ttl = input_value.get("ttl").and_then(|v| v.to_u64());

                Ok(CacheInput::Set { key, value, ttl })
            }
            
            "get" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for get action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for get action".to_string());
                }

                Ok(CacheInput::Get { key })
            }
            
            // ... outras actions ...
            
            _ => Err(format!(
                "Invalid action '{}'. Must be one of: set, get, remove, clear, exists, list, cleanup, stats",
                action
            )),
        }
    }
}
```

**Benefícios deste padrão:**

- ✅ **Type Safety**: Validação em tempo de compilação
- ✅ **Documentação Clara**: Enums documentam as ações possíveis
- ✅ **Validação Robusta**: Erros claros para entradas inválidas
- ✅ **Manutenibilidade**: Fácil adicionar novas ações

### 3. Estatísticas (stats.rs)

O `stats.rs` rastreia métricas de operação do cache.

```rust
/// Rastreador de estatísticas para operações de cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    total_gets: u64,
    total_hits: u64,
    total_sets: u64,
    total_removes: u64,
    total_clears: u64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            total_gets: 0,
            total_hits: 0,
            total_sets: 0,
            total_removes: 0,
            total_clears: 0,
        }
    }

    /// Registrar um cache hit
    pub fn record_hit(&mut self) {
        self.total_gets += 1;
        self.total_hits += 1;
    }

    /// Registrar um cache miss
    pub fn record_miss(&mut self) {
        self.total_gets += 1;
    }

    /// Calcular hit rate como porcentagem
    pub fn get_hit_rate(&self) -> f64 {
        if self.total_gets == 0 {
            0.0
        } else {
            (self.total_hits as f64 / self.total_gets as f64) * 100.0
        }
    }

    // ... outros métodos ...
}
```

**Testes incluídos:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_rate_calculation() {
        let mut stats = CacheStats::new();

        // 100% hit rate
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.get_hit_rate(), 100.0);

        // 50% hit rate
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.get_hit_rate(), 50.0);
    }
}
```

### 4. Implementação Principal (lib.rs)

O `lib.rs` orquestra tudo e implementa a lógica de negócio.

```rust
mod config;
mod input;
mod stats;

use config::CacheConfig;
use input::CacheInput;
use stats::CacheStats;
use phlow_sdk::prelude::*;
use quickleaf::{Quickleaf, Filter, ListProps, Order, Duration};
use std::sync::{Arc, Mutex};

// Registrar a função como step module
create_step!(cache_handler(setup));

/// Instância global do cache com thread safety
type CacheInstance = Arc<Mutex<Quickleaf>>;

/// Handler que gerencia uma instância QuickLeaf
pub async fn cache_handler(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse da configuração do cache
    let config = CacheConfig::try_from(&setup.with)?;
    log::debug!("Cache module started with config: {:?}", config);

    // Inicializar instância do cache
    let cache = if let Some(default_ttl) = config.default_ttl {
        Arc::new(Mutex::new(Quickleaf::with_default_ttl(
            config.capacity,
            Duration::from_secs(default_ttl),
        )))
    } else {
        Arc::new(Mutex::new(Quickleaf::new(config.capacity)))
    };

    // Inicializar estatísticas
    let stats = Arc::new(Mutex::new(CacheStats::new()));

    // Loop de processamento de mensagens
    for package in rx {
        let cache = cache.clone();
        let stats = stats.clone();

        // Parse da entrada baseado na action
        let input = match CacheInput::try_from(package.input.clone()) {
            Ok(input) => input,
            Err(e) => {
                log::error!("Invalid cache input: {}", e);
                let response = std::collections::HashMap::from([
                    ("success", false.to_value()),
                    ("error", format!("Invalid input: {}", e).to_value()),
                ])
                .to_value();
                sender_safe!(package.sender, response.into());
                continue;
            }
        };

        log::debug!("Cache module received input: {:?}", input);

        // Processar baseado na action
        let result = match input {
            CacheInput::Set { key, value, ttl } => {
                handle_set(cache, stats, key, value, ttl).await
            }
            CacheInput::Get { key } => {
                handle_get(cache, stats, key).await
            }
            CacheInput::Remove { key } => {
                handle_remove(cache, stats, key).await
            }
            CacheInput::Clear => {
                handle_clear(cache, stats).await
            }
            CacheInput::Exists { key } => {
                handle_exists(cache, stats, key).await
            }
            CacheInput::List {
                filter_type,
                filter_value,
                filter_prefix,
                filter_suffix,
                order,
                limit,
                offset,
            } => {
                handle_list(
                    cache,
                    filter_type,
                    filter_value,
                    filter_prefix,
                    filter_suffix,
                    order,
                    limit,
                    offset,
                )
                .await
            }
            CacheInput::Cleanup => {
                handle_cleanup(cache).await
            }
            CacheInput::Stats => {
                handle_stats(cache, stats).await
            }
        };

        // Enviar resposta
        match result {
            Ok(response_value) => {
                log::debug!("Cache operation successful");
                sender_safe!(package.sender, response_value.into());
            }
            Err(e) => {
                log::error!("Cache operation failed: {}", e);
                let response = std::collections::HashMap::from([
                    ("success", false.to_value()),
                    ("error", e.to_string().to_value()),
                ])
                .to_value();
                sender_safe!(package.sender, response.into());
            }
        }
    }

    Ok(())
}
```

**Exemplo de Handler: Get**

```rust
/// Handle da action get
async fn handle_get(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    match cache_guard.get(&key) {
        Some(value) => {
            // Cache hit
            if let Ok(mut stats_guard) = stats.lock() {
                stats_guard.record_hit();
            }

            log::debug!("Cache hit for key '{}'", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("found", true.to_value()),
                ("key", key.to_value()),
                ("value", value.clone()),
            ])
            .to_value())
        }
        None => {
            // Cache miss
            if let Ok(mut stats_guard) = stats.lock() {
                stats_guard.record_miss();
            }

            log::debug!("Cache miss for key '{}'", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("found", false.to_value()),
                ("key", key.to_value()),
                ("value", Value::Null),
            ])
            .to_value())
        }
    }
}
```

**Exemplo de Handler: List com Filtros**

```rust
/// Handle da action list
async fn handle_list(
    cache: CacheInstance,
    filter_type: String,
    filter_value: Option<String>,
    filter_prefix: Option<String>,
    filter_suffix: Option<String>,
    order: String,
    limit: Option<u64>,
    offset: u64,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    // Determinar filtro
    let filter = match filter_type.as_str() {
        "prefix" => {
            if let Some(prefix) = filter_prefix.or(filter_value) {
                Filter::StartWith(prefix)
            } else {
                Filter::None
            }
        }
        "suffix" => {
            if let Some(suffix) = filter_suffix.or(filter_value) {
                Filter::EndWith(suffix)
            } else {
                Filter::None
            }
        }
        "pattern" => {
            if let (Some(prefix), Some(suffix)) = (filter_prefix.as_ref(), filter_suffix.as_ref()) {
                Filter::StartAndEndWith(prefix.clone(), suffix.clone())
            } else {
                Filter::None
            }
        }
        _ => Filter::None,
    };

    // Determinar ordem
    let list_order = match order.as_str() {
        "desc" => Order::Desc,
        _ => Order::Asc,
    };

    // Construir propriedades da lista
    let list_props = ListProps::default().filter(filter).order(list_order);

    // Obter itens do cache
    let items = cache_guard
        .list(list_props)
        .map_err(|e| format!("List operation failed: {:?}", e))?;

    // Aplicar paginação
    let total_count = items.len();
    let start_idx = offset as usize;
    let end_idx = if let Some(limit) = limit {
        std::cmp::min(start_idx + (limit as usize), total_count)
    } else {
        total_count
    };

    let paginated_items: Vec<_> = items
        .iter()
        .skip(start_idx)
        .take(end_idx.saturating_sub(start_idx))
        .map(|(key, value)| {
            std::collections::HashMap::from([
                ("key", key.to_value()),
                ("value", (*value).clone()),
            ])
            .to_value()
        })
        .collect();

    let has_more = end_idx < total_count;

    log::debug!(
        "Listed {} items (total: {}, offset: {}, limit: {:?})",
        paginated_items.len(),
        total_count,
        offset,
        limit
    );

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("items", paginated_items.to_value()),
        ("total_count", total_count.to_value()),
        ("offset", offset.to_value()),
        ("limit", limit.to_value()),
        ("has_more", has_more.to_value()),
    ])
    .to_value())
}
```

---

## Schema phlow.yaml

O arquivo `phlow.yaml` define metadados, configuração e schema de entrada/saída do módulo.

### Schema Completo do Cache

```yaml
name: cache
description: |
  High-performance in-memory cache module powered by QuickLeaf.

  **Actions:**
  - `set`: Store a key-value pair in cache with optional TTL
  - `get`: Retrieve a value by key
  - `remove`: Remove a key-value pair from cache
  - `clear`: Clear all items from cache
  - `exists`: Check if a key exists in cache
  - `list`: List cache entries with filtering and ordering
  - `cleanup`: Manually clean up expired items
  - `stats`: Get cache statistics and information

  **Features:**
  - O(1) access time for get/set operations
  - TTL (Time To Live) support with automatic expiration
  - LRU (Least Recently Used) eviction
  - Advanced filtering (prefix, suffix, pattern matching)
  - Ordered listing with pagination support
  - Real-time statistics

version: 0.1.0
author: Philippe Assis <codephilippe@gmail.com>
repository: https://github.com/phlowdotdev/phlow
license: MIT
type: step

tags:
  - cache
  - memory
  - storage
  - performance
  - ttl
  - lru

# Configuração via 'with'
with:
  type: object
  required: false
  properties:
    capacity:
      type: number
      description: "Maximum number of items in cache"
      default: 1000
      required: false
    default_ttl:
      type: number
      description: "Default TTL in seconds for all cached items"
      required: false

# Schema de entrada
input:
  type: object
  required: true
  properties:
    action:
      type: string
      description: "Action to perform"
      required: true
      enum: ["set", "get", "remove", "clear", "exists", "list", "cleanup", "stats"]

    # Propriedades para set action
    key:
      type: string
      description: "Cache key (for set, get, remove, exists actions)"
      required: false
    value:
      type: any
      description: "Value to cache (for set action)"
      required: false
    ttl:
      type: number
      description: "TTL in seconds (for set action)"
      required: false

    # Propriedades para list action
    filter_type:
      type: string
      enum: ["none", "prefix", "suffix", "pattern"]
      description: "Type of filter to apply (for list action)"
      default: "none"
      required: false
    filter_value:
      type: string
      description: "Filter value (for list action)"
      required: false
    filter_prefix:
      type: string
      description: "Filter by key prefix (for list action)"
      required: false
    filter_suffix:
      type: string
      description: "Filter by key suffix (for list action)"
      required: false
    order:
      type: string
      enum: ["asc", "desc"]
      description: "Ordering for results (for list action)"
      default: "asc"
      required: false
    limit:
      type: number
      description: "Maximum number of results to return (for list action)"
      required: false
    offset:
      type: number
      description: "Number of results to skip (for list action)"
      default: 0
      required: false

# Schema de saída
output:
  type: object
  required: true
  properties:
    success:
      type: boolean
      description: "Whether the operation succeeded"
      required: true
    error:
      type: string
      description: "Error message if operation failed"
      required: false
    
    # Get action output
    value:
      type: any
      description: "Retrieved value (for get action)"
      required: false
    found:
      type: boolean
      description: "Whether key was found (for get/exists actions)"
      required: false
    
    # List action output
    items:
      type: array
      description: "List of cache items (for list action)"
      required: false
    total_count:
      type: number
      description: "Total number of items matching filter (for list action)"
      required: false
    has_more:
      type: boolean
      description: "Whether there are more results (for list action)"
      required: false
    
    # Stats action output
    stats:
      type: object
      description: "Cache statistics (for stats action)"
      required: false
      properties:
        size:
          type: number
          description: "Current number of items in cache"
        capacity:
          type: number
          description: "Maximum cache capacity"
        hit_rate:
          type: number
          description: "Cache hit rate percentage"
        memory_usage:
          type: number
          description: "Estimated memory usage in bytes"
```

### Seções Principais do Schema

#### 1. Metadados
```yaml
name: cache                    # Nome único do módulo
version: 0.1.0                 # Versionamento semântico
author: Philippe Assis         # Autor
type: step                     # Tipo do módulo
tags: [cache, memory, ...]     # Tags para descoberta
```

#### 2. Configuração (with)
Define opções que podem ser configuradas ao declarar o módulo:

```yaml
with:
  type: object
  required: false
  properties:
    capacity:
      type: number
      default: 1000
    default_ttl:
      type: number
```

#### 3. Input
Define a estrutura de entrada esperada:

```yaml
input:
  type: object
  required: true
  properties:
    action:
      type: string
      enum: ["set", "get", ...]
```

#### 4. Output
Define a estrutura de resposta:

```yaml
output:
  type: object
  properties:
    success:
      type: boolean
      required: true
```

---

## Testes e Exemplos

### Testes Unitários

O módulo Cache inclui testes em cada arquivo:

**input.rs - Testes de Parsing:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_input_set() {
        let value = json!({
            "action": "set",
            "key": "test_key",
            "value": "test_value",
            "ttl": 3600
        });

        let input = CacheInput::try_from(Some(value)).unwrap();
        match input {
            CacheInput::Set { key, value, ttl } => {
                assert_eq!(key, "test_key");
                assert_eq!(value.to_string(), "test_value");
                assert_eq!(ttl, Some(3600));
            }
            _ => panic!("Expected Set variant"),
        }
    }

    #[test]
    fn test_cache_input_invalid_action() {
        let value = json!({
            "action": "invalid",
            "key": "test_key"
        });

        let result = CacheInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid action 'invalid'"));
    }
}
```

**stats.rs - Testes de Estatísticas:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_rate_calculation() {
        let mut stats = CacheStats::new();
        
        // 100% hit rate
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.get_hit_rate(), 100.0);
        
        // 50% hit rate
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.get_hit_rate(), 50.0);
    }
}
```

### Exemplo de Integração

**simple-test.phlow - Testes Básicos:**

```yaml
name: Cache Module Simple Tests
version: 1.0.0

modules:
  - module: cache
    with:
      capacity: 10
      default_ttl: 300

tests:
  - describe: "Set and get string value"
    main:
      test_key: "simple:string"
      test_value: "Hello Cache!"
    assert: !phs payload.success && payload.key == "simple:string"

  - describe: "Retrieve stored string value"
    main:
      test_key: "simple:string"
    assert: !phs payload.found && payload.value == "Hello Cache!"

steps:
  - assert: !phs main.test_key == "simple:string" && main.test_value != null
    then:
      - use: cache
        input:
          action: set
          key: !phs main.test_key
          value: !phs main.test_value
```

### Exemplo de Caso de Uso Real

**user-sessions.phlow - Gerenciamento de Sessões:**

```yaml
name: User Session Cache Example
version: 1.0.0

modules:
  - module: cache
    with:
      capacity: 1000
      default_ttl: 1800  # 30 minutos

steps:
  # Criar sessão de usuário
  - use: cache
    input:
      action: set
      key: "session:12345"
      value:
        user_id: 12345
        username: "john.doe"
        email: "john.doe@example.com"
        login_time: "2025-08-06T23:10:00Z"
        permissions: ["read", "write"]
      ttl: 3600  # 1 hora

  # Validar sessão existe
  - use: cache
    input:
      action: exists
      key: "session:12345"

  # Recuperar dados da sessão
  - use: cache
    input:
      action: get
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          message: !phs `User ${payload.value.username} authenticated`

  # Listar sessões ativas
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "session:"
      limit: 10

  # Obter estatísticas
  - use: cache
    input:
      action: stats

  - use: log
    input:
      message: !phs `Cache stats - Size: ${payload.stats.size}, Hit rate: ${payload.stats.hit_rate}%`
```

---

## Build e Deploy

### Compilar o Módulo

```bash
# Build de desenvolvimento
cd modules/cache
cargo build

# Build otimizado para produção
cargo build --release

# O módulo compilado estará em:
# target/debug/libcache.so     (Linux)
# target/debug/libcache.dylib  (macOS)
# target/debug/cache.dll       (Windows)
```

### Instalação Local

```bash
# Criar diretório de pacotes
mkdir -p phlow_packages/cache

# Copiar módulo compilado
cp target/release/libcache.so phlow_packages/cache/module.so

# Copiar schema
cp phlow.yaml phlow_packages/cache/

# Estrutura final:
# phlow_packages/
#   cache/
#     module.so
#     phlow.yaml
```

### Testar o Módulo

```bash
# Executar arquivo de exemplo
phlow examples/cache/simple-test.phlow

# Executar com log detalhado
RUST_LOG=debug phlow examples/cache/user-sessions.phlow

# Executar testes
phlow test examples/cache/simple-test.phlow
```

### Build Automatizado

Para módulos no repositório oficial, use o cargo-make:

```bash
# Build de todos os módulos
cargo make build-modules

# Build de um módulo específico
cargo make build-module cache

# Build e empacotamento
cargo make packages
```

---

## Melhores Práticas

### 1. Organização de Código

```rust
// ✅ BOM: Separar em módulos lógicos
mod config;    // Configuração
mod input;     // Parsing de entrada
mod stats;     // Estatísticas
mod handlers;  // Lógica de negócio

// ❌ RUIM: Tudo em lib.rs
// 2000 linhas em um único arquivo
```

### 2. Tratamento de Erros

```rust
// ✅ BOM: Erros descritivos
Err(format!("Invalid capacity: must be > 0, got {}", cap))

// ❌ RUIM: Erros genéricos
Err("Invalid input".to_string())
```

### 3. Validação de Configuração

```rust
// ✅ BOM: Validar cedo
impl TryFrom<&Value> for Config {
    fn try_from(value: &Value) -> Result<Self, String> {
        if capacity <= 0 {
            return Err("Capacity must be positive".to_string());
        }
        // ...
    }
}

// ❌ RUIM: Aceitar qualquer valor
impl From<&Value> for Config {
    fn from(value: &Value) -> Self {
        // Sem validação
    }
}
```

### 4. Logging Estruturado

```rust
// ✅ BOM: Logs informativos em diferentes níveis
log::debug!("Cache operation: action={}, key={}", action, key);
log::info!("Cache hit rate: {:.2}%", stats.hit_rate());
log::warn!("Cache nearing capacity: {}/{}", size, capacity);
log::error!("Cache operation failed: {}", error);

// ❌ RUIM: Logs vagos
log::info!("Operation completed");
```

### 5. Thread Safety

```rust
// ✅ BOM: Usar Arc<Mutex<T>> para estado compartilhado
type CacheInstance = Arc<Mutex<Quickleaf>>;
let cache = Arc::new(Mutex::new(Quickleaf::new(1000)));

// ❌ RUIM: Estado mutável sem proteção
static mut CACHE: Option<Quickleaf> = None;
```

### 6. Testes Abrangentes

```rust
// ✅ BOM: Testar casos de sucesso E falha
#[test]
fn test_valid_input() { /* ... */ }

#[test]
fn test_invalid_action() { /* ... */ }

#[test]
fn test_missing_required_field() { /* ... */ }

#[test]
fn test_edge_cases() { /* ... */ }
```

### 7. Documentação Clara

```rust
// ✅ BOM: Documentar funções públicas
/// Handle get action from cache
///
/// # Arguments
/// * `cache` - Thread-safe cache instance
/// * `stats` - Statistics tracker
/// * `key` - Key to retrieve
///
/// # Returns
/// * `Ok(Value)` - Success response with value or null
/// * `Err(String)` - Error message
async fn handle_get(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
) -> Result<Value, String>
```

### 8. Versionamento Semântico

```toml
# ✅ BOM: Seguir SemVer
version = "0.1.0"  # MAJOR.MINOR.PATCH

# 0.1.0 → 0.1.1 : Bug fix
# 0.1.1 → 0.2.0 : Nova funcionalidade
# 0.2.0 → 1.0.0 : Breaking change
```

### 9. Performance

```rust
// ✅ BOM: Operações O(1) quando possível
cache_guard.get(&key)  // O(1) lookup

// ✅ BOM: Paginação em listagens
let items = all_items
    .skip(offset)
    .take(limit)
    .collect();

// ❌ RUIM: Retornar tudo sem paginação
let items = all_items.collect();
```

### 10. Schema Completo

```yaml
# ✅ BOM: Documentar todas as propriedades
input:
  properties:
    key:
      type: string
      description: "Cache key for the operation"
      required: true
      
# ❌ RUIM: Schema incompleto
input:
  properties:
    key:
      type: string
```

---

## Checklist de Desenvolvimento

Use este checklist ao criar um novo módulo:

### Estrutura
- [ ] Criar diretório `modules/my_module/`
- [ ] Criar `Cargo.toml` com `crate-type = ["cdylib"]`
- [ ] Criar `phlow.yaml` com schema completo
- [ ] Criar `src/lib.rs` com macro apropriada

### Configuração
- [ ] Definir struct de configuração em `config.rs`
- [ ] Implementar `TryFrom<&Value>` com validação
- [ ] Definir valores padrão sensatos
- [ ] Documentar todas as opções

### Entrada/Saída
- [ ] Definir enum de ações em `input.rs`
- [ ] Implementar parsing robusto
- [ ] Validar todos os campos obrigatórios
- [ ] Retornar erros descritivos

### Implementação
- [ ] Usar `Arc<Mutex<T>>` para estado compartilhado
- [ ] Implementar handlers para cada ação
- [ ] Adicionar logging apropriado
- [ ] Tratar todos os erros

### Testes
- [ ] Adicionar testes unitários
- [ ] Criar exemplo de uso simples
- [ ] Criar exemplo de caso de uso real
- [ ] Testar casos de erro

### Documentação
- [ ] Documentar funções públicas
- [ ] Adicionar exemplos no `phlow.yaml`
- [ ] Criar README se necessário
- [ ] Documentar ações e parâmetros

### Build
- [ ] Compilar sem warnings
- [ ] Testar em ambiente local
- [ ] Verificar tamanho do binário
- [ ] Testar com phlow runtime

---

## Conclusão

Este guia usou o módulo Cache como exemplo real para demonstrar todos os aspectos do desenvolvimento de módulos Phlow. Os principais takeaways são:

1. **Organização Modular**: Separar código em arquivos lógicos (`config.rs`, `input.rs`, `stats.rs`)
2. **Type Safety**: Usar enums e traits do Rust para validação em compile-time
3. **Padrão Action-Based**: Múltiplas operações em um único módulo usando enums tagged
4. **Thread Safety**: Usar `Arc<Mutex<T>>` para estado compartilhado
5. **Validação Robusta**: Validar entrada cedo e retornar erros claros
6. **Testes Abrangentes**: Testar sucesso, falha e casos extremos
7. **Documentação Clara**: Schema completo e exemplos de uso

O módulo Cache demonstra um padrão maduro e robusto que pode ser adaptado para criar novos módulos Phlow de alta qualidade.

### Próximos Passos

1. Explore o código-fonte completo em `modules/cache/`
2. Experimente os exemplos em `examples/cache/`
3. Use este padrão como base para seus próprios módulos
4. Contribua com melhorias e novos módulos para o ecossistema Phlow

**Happy coding! 🚀**
