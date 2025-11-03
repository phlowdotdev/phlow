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

/// Configuration for the cache module
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub capacity: usize,
    pub default_ttl: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,      // Default capacity: 1000 items
            default_ttl: None,   // No default TTL
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

**Usage in .phlow file:**

```yaml
modules:
  - module: cache
    with:
      capacity: 5000        # Maximum 5000 items
      default_ttl: 3600     # 1 hour default
```

### 2. Input Definitions (input.rs)

The `input.rs` defines all possible actions using Rust enums with serde.

```rust
use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Cache input actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]  // Use "action" field as discriminator
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

**Custom parsing implementation:**

```rust
impl TryFrom<Option<Value>> for CacheInput {
    type Error = String;

    fn try_from(input_value: Option<Value>) -> Result<Self, Self::Error> {
        let input_value = input_value.ok_or("Missing input for cache module")?;

        if !input_value.is_object() {
            return Err("Cache input must be an object".to_string());
        }

        // Extract action (required)
        let action = match input_value.get("action") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'action' field in cache input".to_string()),
        };

        // Match based on action
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
            
            // ... other actions ...
            
            _ => Err(format!(
                "Invalid action '{}'. Must be one of: set, get, remove, clear, exists, list, cleanup, stats",
                action
            )),
        }
    }
}
```

**Benefits of this pattern:**

- ✅ **Type Safety**: Compile-time validation
- ✅ **Clear Documentation**: Enums document possible actions
- ✅ **Robust Validation**: Clear errors for invalid inputs
- ✅ **Maintainability**: Easy to add new actions

### 3. Statistics (stats.rs)

The `stats.rs` tracks cache operation metrics.

```rust
/// Statistics tracker for cache operations
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

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.total_gets += 1;
        self.total_hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.total_gets += 1;
    }

    /// Calculate hit rate as percentage
    pub fn get_hit_rate(&self) -> f64 {
        if self.total_gets == 0 {
            0.0
        } else {
            (self.total_hits as f64 / self.total_gets as f64) * 100.0
        }
    }

    // ... other methods ...
}
```

**Included tests:**

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

### 4. Main Implementation (lib.rs)

The `lib.rs` orchestrates everything and implements the business logic.

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

// Register the function as a step module
create_step!(cache_handler(setup));

/// Global cache instance with thread safety
type CacheInstance = Arc<Mutex<Quickleaf>>;

/// Handler that manages a QuickLeaf instance
pub async fn cache_handler(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse cache configuration
    let config = CacheConfig::try_from(&setup.with)?;
    log::debug!("Cache module started with config: {:?}", config);

    // Initialize cache instance
    let cache = if let Some(default_ttl) = config.default_ttl {
        Arc::new(Mutex::new(Quickleaf::with_default_ttl(
            config.capacity,
            Duration::from_secs(default_ttl),
        )))
    } else {
        Arc::new(Mutex::new(Quickleaf::new(config.capacity)))
    };

    // Initialize statistics
    let stats = Arc::new(Mutex::new(CacheStats::new()));

    // Message processing loop
    for package in rx {
        let cache = cache.clone();
        let stats = stats.clone();

        // Parse input based on action
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

        // Process based on action
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

        // Send response
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

**Handler Example: Get**

```rust
/// Handle get action
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

**Handler Example: List with Filters**

```rust
/// Handle list action
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

    // Determine filter
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

    // Determine order
    let list_order = match order.as_str() {
        "desc" => Order::Desc,
        _ => Order::Asc,
    };

    // Build list properties
    let list_props = ListProps::default().filter(filter).order(list_order);

    // Get items from cache
    let items = cache_guard
        .list(list_props)
        .map_err(|e| format!("List operation failed: {:?}", e))?;

    // Apply pagination
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

## phlow.yaml Schema

The `phlow.yaml` file defines metadata, configuration, and input/output schema of the module.

### Complete Cache Schema

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

# Configuration via 'with'
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

# Input schema
input:
  type: object
  required: true
  properties:
    action:
      type: string
      description: "Action to perform"
      required: true
      enum: ["set", "get", "remove", "clear", "exists", "list", "cleanup", "stats"]

    # Properties for set action
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

    # Properties for list action
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

# Output schema
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

### Main Schema Sections

#### 1. Metadata
```yaml
name: cache                    # Unique module name
version: 0.1.0                 # Semantic versioning
author: Philippe Assis         # Author
type: step                     # Module type
tags: [cache, memory, ...]     # Tags for discovery
```

#### 2. Configuration (with)
Defines options that can be configured when declaring the module:

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
Defines the expected input structure:

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
Defines the response structure:

```yaml
output:
  type: object
  properties:
    success:
      type: boolean
      required: true
```

---

## Tests and Examples

### Unit Tests

The Cache module includes tests in each file:

**input.rs - Parsing Tests:**

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

**stats.rs - Statistics Tests:**

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

### Integration Example

**simple-test.phlow - Basic Tests:**

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

### Real Use Case Example

**user-sessions.phlow - Session Management:**

```yaml
name: User Session Cache Example
version: 1.0.0

modules:
  - module: cache
    with:
      capacity: 1000
      default_ttl: 1800  # 30 minutes

steps:
  # Create user session
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
      ttl: 3600  # 1 hour

  # Validate session exists
  - use: cache
    input:
      action: exists
      key: "session:12345"

  # Retrieve session data
  - use: cache
    input:
      action: get
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          message: !phs `User ${payload.value.username} authenticated`

  # List active sessions
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "session:"
      limit: 10

  # Get statistics
  - use: cache
    input:
      action: stats

  - use: log
    input:
      message: !phs `Cache stats - Size: ${payload.stats.size}, Hit rate: ${payload.stats.hit_rate}%`
```

---

## Build and Deploy

### Compile the Module

```bash
# Development build
cd modules/cache
cargo build

# Optimized production build
cargo build --release

# The compiled module will be at:
# target/debug/libcache.so     (Linux)
# target/debug/libcache.dylib  (macOS)
# target/debug/cache.dll       (Windows)
```

### Local Installation

```bash
# Create package directory
mkdir -p phlow_packages/cache

# Copy compiled module
cp target/release/libcache.so phlow_packages/cache/module.so

# Copy schema
cp phlow.yaml phlow_packages/cache/

# Final structure:
# phlow_packages/
#   cache/
#     module.so
#     phlow.yaml
```

### Test the Module

```bash
# Run example file
phlow examples/cache/simple-test.phlow

# Run with detailed logging
RUST_LOG=debug phlow examples/cache/user-sessions.phlow

# Run tests
phlow test examples/cache/simple-test.phlow
```

### Automated Build

For modules in the official repository, use cargo-make:

```bash
# Build all modules
cargo make build-modules

# Build a specific module
cargo make build-module cache

# Build and package
cargo make packages
```

---

## Best Practices

### 1. Code Organization

```rust
// ✅ GOOD: Separate into logical modules
mod config;    // Configuration
mod input;     // Input parsing
mod stats;     // Statistics
mod handlers;  // Business logic

// ❌ BAD: Everything in lib.rs
// 2000 lines in a single file
```

### 2. Error Handling

```rust
// ✅ GOOD: Descriptive errors
Err(format!("Invalid capacity: must be > 0, got {}", cap))

// ❌ BAD: Generic errors
Err("Invalid input".to_string())
```

### 3. Configuration Validation

```rust
// ✅ GOOD: Validate early
impl TryFrom<&Value> for Config {
    fn try_from(value: &Value) -> Result<Self, String> {
        if capacity <= 0 {
            return Err("Capacity must be positive".to_string());
        }
        // ...
    }
}

// ❌ BAD: Accept any value
impl From<&Value> for Config {
    fn from(value: &Value) -> Self {
        // No validation
    }
}
```

### 4. Structured Logging

```rust
// ✅ GOOD: Informative logs at different levels
log::debug!("Cache operation: action={}, key={}", action, key);
log::info!("Cache hit rate: {:.2}%", stats.hit_rate());
log::warn!("Cache nearing capacity: {}/{}", size, capacity);
log::error!("Cache operation failed: {}", error);

// ❌ BAD: Vague logs
log::info!("Operation completed");
```

### 5. Thread Safety

```rust
// ✅ GOOD: Use Arc<Mutex<T>> for shared state
type CacheInstance = Arc<Mutex<Quickleaf>>;
let cache = Arc::new(Mutex::new(Quickleaf::new(1000)));

// ❌ BAD: Mutable state without protection
static mut CACHE: Option<Quickleaf> = None;
```

### 6. Comprehensive Tests

```rust
// ✅ GOOD: Test success AND failure cases
#[test]
fn test_valid_input() { /* ... */ }

#[test]
fn test_invalid_action() { /* ... */ }

#[test]
fn test_missing_required_field() { /* ... */ }

#[test]
fn test_edge_cases() { /* ... */ }
```

### 7. Clear Documentation

```rust
// ✅ GOOD: Document public functions
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

### 8. Semantic Versioning

```toml
# ✅ GOOD: Follow SemVer
version = "0.1.0"  # MAJOR.MINOR.PATCH

# 0.1.0 → 0.1.1 : Bug fix
# 0.1.1 → 0.2.0 : New feature
# 0.2.0 → 1.0.0 : Breaking change
```

### 9. Performance

```rust
// ✅ GOOD: O(1) operations when possible
cache_guard.get(&key)  // O(1) lookup

// ✅ GOOD: Pagination in listings
let items = all_items
    .skip(offset)
    .take(limit)
    .collect();

// ❌ BAD: Return everything without pagination
let items = all_items.collect();
```

### 10. Complete Schema

```yaml
# ✅ GOOD: Document all properties
input:
  properties:
    key:
      type: string
      description: "Cache key for the operation"
      required: true
      
# ❌ BAD: Incomplete schema
input:
  properties:
    key:
      type: string
```

---

## Development Checklist

Use this checklist when creating a new module:

### Structure
- [ ] Create directory `modules/my_module/`
- [ ] Create `Cargo.toml` with `crate-type = ["cdylib"]`
- [ ] Create `phlow.yaml` with complete schema
- [ ] Create `src/lib.rs` with appropriate macro

### Configuration
- [ ] Define configuration struct in `config.rs`
- [ ] Implement `TryFrom<&Value>` with validation
- [ ] Define sensible default values
- [ ] Document all options

### Input/Output
- [ ] Define action enum in `input.rs`
- [ ] Implement robust parsing
- [ ] Validate all required fields
- [ ] Return descriptive errors

### Implementation
- [ ] Use `Arc<Mutex<T>>` for shared state
- [ ] Implement handlers for each action
- [ ] Add appropriate logging
- [ ] Handle all errors

### Tests
- [ ] Add unit tests
- [ ] Create simple usage example
- [ ] Create real use case example
- [ ] Test error cases

### Documentation
- [ ] Document public functions
- [ ] Add examples in `phlow.yaml`
- [ ] Create README if needed
- [ ] Document actions and parameters

### Build
- [ ] Compile without warnings
- [ ] Test in local environment
- [ ] Check binary size
- [ ] Test with phlow runtime

---

## Conclusion

This guide used the Cache module as a real example to demonstrate all aspects of Phlow module development. The main takeaways are:

1. **Modular Organization**: Separate code into logical files (`config.rs`, `input.rs`, `stats.rs`)
2. **Type Safety**: Use Rust enums and traits for compile-time validation
3. **Action-Based Pattern**: Multiple operations in a single module using tagged enums
4. **Thread Safety**: Use `Arc<Mutex<T>>` for shared state
5. **Robust Validation**: Validate input early and return clear errors
6. **Comprehensive Tests**: Test success, failure, and edge cases
7. **Clear Documentation**: Complete schema and usage examples

The Cache module demonstrates a mature and robust pattern that can be adapted to create high-quality Phlow modules.

### Next Steps

1. Explore the complete source code in `modules/cache/`
2. Try the examples in `examples/cache/`
3. Use this pattern as a base for your own modules
4. Contribute improvements and new modules to the Phlow ecosystem

**Happy coding! 🚀**
