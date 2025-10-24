---
sidebar_position: 6
title: Log Module
hide_title: true
---

---
sidebar_position: 6
title: Log Module
hide_title: true
---

# Log Module

The Log module provides structured logging functionality for Phlow applications, allowing you to record messages with different severity levels.

## 🚀 Features

### Key Features

- ✅ **Multiple log levels**: info, debug, warn, error
- ✅ **Structured logging**: Compatible with env_logger
- ✅ **Flexible configuration**: Via PHLOW_LOG environment variable
- ✅ **Observability**: Integration with OpenTelemetry
- ✅ **Performance**: Asynchronous logging without blocking

## 📋 Configuration

### Basic Configuration (Recommended Syntax)

```phlow
steps:
  - use: log
    input:
      level: "info"
      message: "Application started successfully"
```

### Basic Configuration (Legacy Syntax - Still Supported)

```phlow
steps:
  - log:
      level: "info"
      message: "Application started successfully"
```

**Note:** Both syntaxes are supported. The legacy syntax is automatically transformed to the new syntax during processing.

### Configuration with Environment Variables

```bash
# Default log level
export PHLOW_LOG="debug"  # info, debug, warn, error
```

## 🔧 Parameters

### Input
- `level` (string, optional): Log level [info, debug, warn, error] (default: "info")
- `message` (string, required): Message to be logged

### Output
- Returns `null` after processing the log

## 💻 Usage Examples

### Different Log Levels (New Syntax)

```phlow
steps:
  - use: log
    input:
      level: "info"
      message: "Processing started"
      
  - use: log
    input:
      level: "debug"
      message: !phs `Variable x = ${main.x}`
      
  - use: log
    input:
      level: "warn"
      message: "Configuration not found, using default"
      
  - use: log
    input:
      level: "error"
      message: "Database connection failed"
```

### Different Log Levels (Legacy Syntax - Automatically Transformed)

```phlow
steps:
  - log:
      level: "info"
      message: "Processing started"
      
  - log:
      level: "debug"
      message: !phs `Variable x = ${main.x}`
      
  - log:
      level: "warn"
      message: "Configuration not found, using default"
      
  - log:
      level: "error"
      message: "Database connection failed"
```

### Logging with Code Blocks

```phlow
steps:
  - payload: !phs {
      let user = main.user;
      let timestamp = new Date().toISOString();
      
      #{
        id: user.id,
        name: user.name,
        loginTime: timestamp,
        sessionId: Math.random().toString(36)
      }
    }
    
  - use: log
    input:
      level: "info"
      message: !phs {
        let session = payload;
        let status = session.id ? "success" : "failed";
        
        `User login ${status}: ${session.name} (ID: ${session.id}) at ${session.loginTime}`
      }
```

### Pipeline Logging

```phlow
steps:
  - use: log
    input:
      message: !phs `Starting user processing ${main.user_id}`
      
  - payload: !phs {
      let userId = main.user_id;
      let processedAt = new Date().toISOString();
      
      #{
        id: userId,
        status: "processed",
        timestamp: processedAt,
        result: `User ${userId} processed successfully`
      }
    }
      
  - use: log
    input:
      level: "info"
      message: !phs `User ${payload.id} processed successfully`
      
  - use: log
    input:
      level: "debug"
      message: !phs {
        let data = JSON.stringify(payload, null, 2);
        `Processed user data: ${data}`
      }
```

## 🌐 Complete Example

```phlow
name: "logging-example"
version: "1.0.0"
description: "Example using the Log module with new features"

modules:
  - module: log
    version: latest

steps:
  - use: log
    input:
      level: "info"
      message: !phs {
        let timestamp = new Date().toISOString();
        `Application started at ${timestamp}`
      }
      
  - payload: !phs {
      // Simulate configuration loading
      let config = {
        database: "postgresql://localhost:5432/mydb",
        port: 3000,
        debug: true,
        version: "1.0.0"
      };
      
      config
    }
      
  - use: log
    input:
      level: "debug"
      message: !phs {
        let configStr = JSON.stringify(payload, null, 2);
        `Configuration loaded: ${configStr}`
      }
      
  - assert: !phs payload.database != null
    then:
      - use: log
        input:
          level: "info"
          message: "Database configuration valid"
    else:
      - use: log
        input:
          level: "error"
          message: "Database configuration missing"
        
  - assert: !phs payload.debug === true
    then:
      - use: log
        input:
          level: "warn"
          message: !phs {
            let version = payload.version;
            `Debug mode enabled in version ${version} - performance may be affected`
          }
        
  - use: log
    input:
      level: "info"
      message: !phs {
        let port = payload.port;
        let dbHost = payload.database.split("://")[1].split("/")[0];
        
        `Application configured - Port: ${port}, DB: ${dbHost}`
      }
```

### Example with Mixed Syntax (Legacy + New)

```phlow
modules:
  - module: log

steps:
  # New syntax
  - use: log
    input:
      message: "Starting with new syntax"
      
  # Legacy syntax (will be automatically transformed)
  - log:
      level: "debug"
      message: "This is legacy syntax"
      
  # New syntax with code block
  - use: log
    input:
      level: "info"
      message: !phs {
        let mode = "mixed";
        let timestamp = new Date().toISOString();
        
        `Mode ${mode} active at ${timestamp}`
      }
```

## 🔧 Advanced Configuration

### Log Levels

```bash
# Errors only
export PHLOW_LOG="error"

# Warnings and errors
export PHLOW_LOG="warn"

# Info, warnings and errors
export PHLOW_LOG="info"

# All logs including debug
export PHLOW_LOG="debug"
```

### Log Formatting

The module uses env_logger, which can be configured:

```bash
# Custom format
export RUST_LOG_STYLE="always"
export PHLOW_LOG="debug"
```

## 📊 Sample Output

```
[2024-01-01T00:00:00Z INFO  phlow] Application started successfully
[2024-01-01T00:00:01Z DEBUG phlow] Variable x = 42
[2024-01-01T00:00:02Z WARN  phlow] Configuration not found, using default
[2024-01-01T00:00:03Z ERROR phlow] Database connection failed
```

## 🏷️ Tags

- log
- echo
- print
- logging
- debug

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
