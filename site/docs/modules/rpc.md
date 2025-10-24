---
sidebar_position: 8
title: RPC Module
hide_title: true
---

---
sidebar_position: 8
title: RPC Module
hide_title: true
---

# RPC Module

The RPC module provides Remote Procedure Call (RPC) functionality using tarpc for high-performance client-server communication.

## 🚀 Features

### Key Features

- ✅ **RPC Server**: Receives RPC calls and processes via steps
- ✅ **RPC Client**: Makes calls to remote RPC servers
- ✅ **High performance**: Using tarpc with TCP transport
- ✅ **JSON serialization**: Cross-language compatibility
- ✅ **Health checks**: Health verification endpoint
- ✅ **Connection pool**: Automatic connection management
- ✅ **Observability**: Complete tracing with OpenTelemetry

## 📋 Configuration

### RPC Server (Main)

```phlow
name: "rpc-server"
main: "rpc_server"

modules:
  - name: "rpc_server"
    module: "rpc"
    with:
      host: "0.0.0.0"
      port: 8090
      service_name: "user-service"
      max_connections: 100

steps:
  - name: "handle_rpc_call"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "get_user"
    then:
      return:
        id: "{{ $input.params.user_id }}"
        name: "User {{ $input.params.user_id }}"
        active: true
```

### RPC Client (Steps)

```phlow
modules:
  - name: "rpc_client"
    module: "rpc"
    with:
      host: "rpc-server.example.com"
      port: 8090
      timeout_ms: 5000

steps:
  - name: "call_remote_service"
    use: "rpc_client"
    input:
      method: "process_data"
      params:
        data: "some value"
        count: 42
      headers:
        "Content-Type": "application/json"
        "X-Request-ID": "123"
```

## 🔧 Parameters

### Configuration (with)
- `host` (string): IP or hostname (default: "127.0.0.1")
- `port` (integer): Port (default: 8080)
- `timeout_ms` (integer): Timeout in ms (default: 5000)
- `max_connections` (integer): Maximum connections (default: 100)
- `service_name` (string): Service name (default: "default")

### Client Input (input)
- `action` (string): Special action ["health", "info"]
- `method` (string): Remote method to call
- `params` (any): Method parameters
- `headers` (object): Call headers

### Output (output)
- `result` (any): Result of the remote method
- `error` (string): Error message if any
- `headers` (object): Response headers
- `healthy` (boolean): Health status (action="health")
- `service_name` (string): Service name (action="info")

## 💻 Usage Examples

### Full RPC Server

```phlow
name: "user-rpc-server"
version: "1.0.0"
main: "rpc_server"

modules:
  - name: "rpc_server"
    module: "rpc"
    with:
      host: "0.0.0.0"
      port: 8090
      service_name: "user-service"
      max_connections: 200

  - name: "logger"
    module: "log"

  - name: "db"
    module: "postgres"
    with:
      host: "localhost"
      database: "users"
      user: "app"
      password: "secret"

steps:
  - name: "log_rpc_call"
    use: "logger"
    input:
      level: "info"
      message: "RPC call: {{ $input.method }} with params: {{ $input.params }}"

  - name: "route_method"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "get_user"
    then:
      use: "db"
      input:
        query: "SELECT id, name, email FROM users WHERE id = $1"
        params: ["{{ $input.params.user_id }}"]
    else:
      condition:
        left: "{{ $input.method }}"
        operator: "equals"
        right: "create_user"
      then:
        use: "db"
        input:
          query: "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id"
          params: ["{{ $input.params.name }}", "{{ $input.params.email }}"]
      else:
        return:
          error: "Method not found"
          available_methods: ["get_user", "create_user"]
```

### RPC Client with Health Check

```phlow
name: "rpc-client-example"
version: "1.0.0"

modules:
  - name: "user_service"
    module: "rpc"
    with:
      host: "user-service.company.com"
      port: 8090
      timeout_ms: 3000

steps:
  - name: "health_check"
    use: "user_service"
    input:
      action: "health"

  - name: "service_info"
    use: "user_service"
    input:
      action: "info"

  - name: "check_health_status"
    condition:
      left: "{{ $health_check.healthy }}"
      operator: "equals"
      right: true
    then:
      # Service is healthy, perform call
      use: "user_service"
      input:
        method: "get_user"
        params:
          user_id: 123
          include_details: true
        headers:
          "X-Request-ID": "req-123"
    else:
      return:
        error: "Service unavailable"
        health_status: "{{ $health_check }}"
```

### Microservices with RPC

```phlow
name: "order-processing"
version: "1.0.0"

modules:
  - name: "user_service"
    module: "rpc"
    with:
      host: "user-service"
      port: 8090

  - name: "payment_service"
    module: "rpc"
    with:
      host: "payment-service"
      port: 8091

  - name: "inventory_service"
    module: "rpc"
    with:
      host: "inventory-service"
      port: 8092

steps:
  - name: "validate_user"
    use: "user_service"
    input:
      method: "validate_user"
      params:
        user_id: "{{ $order.user_id }}"
      headers:
        "Authorization": "Bearer {{ $auth_token }}"

  - name: "check_inventory"
    use: "inventory_service"
    input:
      method: "check_availability"
      params:
        product_id: "{{ $order.product_id }}"
        quantity: "{{ $order.quantity }}"

  - name: "process_payment"
    condition:
      left: "{{ $validate_user.result.valid }}"
      operator: "equals"
      right: true
    then:
      condition:
        left: "{{ $check_inventory.result.available }}"
        operator: "equals"
        right: true
      then:
        use: "payment_service"
        input:
          method: "process_payment"
          params:
            user_id: "{{ $order.user_id }}"
            amount: "{{ $order.amount }}"
            payment_method: "{{ $order.payment_method }}"
          headers:
            "Authorization": "Bearer {{ $auth_token }}"
      else:
        return:
          error: "Product not available"
          inventory_status: "{{ $check_inventory.result }}"
    else:
      return:
        error: "Invalid user"
        user_validation: "{{ $validate_user.result }}"

  - name: "final_result"
    script: |
      {
        order_id: $order.id,
        user_valid: $validate_user.result.valid,
        inventory_available: $check_inventory.result.available,
        payment_processed: $process_payment.result.success,
        transaction_id: $process_payment.result.transaction_id
      }
```

## 📊 Data Structures

### Server Data (main input)

```json
{
  "method": "get_user",
  "params": {
    "user_id": 123,
    "include_details": true
  },
  "headers": {
    "Content-Type": "application/json",
    "Authorization": "Bearer token123"
  }
}
```

### Client Response

```json
{
  "result": {
    "id": 123,
    "name": "João Silva",
    "email": "joao@example.com",
    "active": true
  },
  "error": null,
  "headers": {
    "Content-Type": "application/json",
    "X-Response-Time": "150ms"
  }
}
```

### Health Check Response

```json
{
  "healthy": true,
  "service": "user-service",
  "address": "127.0.0.1:8090",
  "result": "OK"
}
```

## 🏷️ Tags

- rpc
- tarpc
- communication
- client-server
- remote-procedure-call
- tcp
- json
- microservices

---

**Version**: 0.0.1  
**Author**: Philippe Assis `<codephilippe@gmail.com>`  
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
