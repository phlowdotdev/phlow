---
sidebar_position: 8
title: RPC Module
hide_title: true
---

# Módulo RPC

O módulo RPC fornece funcionalidade de Remote Procedure Call (RPC) usando tarpc para comunicação cliente-servidor de alta performance.

## 🚀 Funcionalidades

### Características Principais

- ✅ **Servidor RPC**: Recebe chamadas RPC e processa via steps
- ✅ **Cliente RPC**: Faz chamadas para servidores RPC remotos
- ✅ **Alta performance**: Usando tarpc com transporte TCP
- ✅ **Serialização JSON**: Compatibilidade cross-language
- ✅ **Health checks**: Endpoint de verificação de saúde
- ✅ **Pool de conexões**: Gerenciamento automático de conexões
- ✅ **Observabilidade**: Tracing completo com OpenTelemetry

## 📋 Configuração

### Servidor RPC (Main)

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

### Cliente RPC (Steps)

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

## 🔧 Parâmetros

### Configuração (with)
- `host` (string): IP ou hostname (padrão: "127.0.0.1")
- `port` (integer): Porta (padrão: 8080)
- `timeout_ms` (integer): Timeout em ms (padrão: 5000)
- `max_connections` (integer): Conexões máximas (padrão: 100)
- `service_name` (string): Nome do serviço (padrão: "default")

### Entrada Cliente (input)
- `action` (string): Ação especial ["health", "info"]
- `method` (string): Método remoto a chamar
- `params` (any): Parâmetros do método
- `headers` (object): Headers da chamada

### Saída (output)
- `result` (any): Resultado do método remoto
- `error` (string): Mensagem de erro se houver
- `headers` (object): Headers de resposta
- `healthy` (boolean): Status de saúde (action="health")
- `service_name` (string): Nome do serviço (action="info")

## 💻 Exemplos de Uso

### Servidor RPC Completo

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

### Cliente RPC com Health Check

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
      # Serviço saudável, fazer chamada
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

### Microserviços com RPC

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

## 📊 Estrutura de Dados

### Dados do Servidor (main input)

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

### Resposta do Cliente

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

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
