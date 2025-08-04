---
sidebar_position: 5
title: HTTP Server Module
hide_title: true
---

# Módulo HTTP Server

O módulo HTTP Server fornece um servidor web completo e de alta performance para construir APIs REST, webhooks e serviços web. Construído com Hyper e Tokio, oferece observabilidade completa com OpenTelemetry.

## 🚀 Funcionalidades

### Características Principais

- ✅ **Servidor HTTP/HTTPS** de alta performance
- ✅ **Suporte a todos os métodos HTTP** (GET, POST, PUT, PATCH, DELETE, OPTIONS, etc.)
- ✅ **Roteamento dinâmico** via fluxos Phlow
- ✅ **Headers customizados** para requisições e respostas
- ✅ **Parsing automático** de JSON, query parameters e headers
- ✅ **Health check** endpoint (`/health`)
- ✅ **Observabilidade completa** com OpenTelemetry tracing
- ✅ **Configuração flexível** de host e porta
- ✅ **Tratamento de authorization** com diferentes modos de span
- ✅ **Middleware de tracing** para todas as requisições
- ✅ **Suporte a keep-alive** para conexões persistentes

## 📋 Configuração

### Configuração Básica

```phlow
name: "my-api-server"
version: "1.0.0"
main: "api_server"

modules:
  - name: "api_server"
    module: "http_server"
    with:
      host: "0.0.0.0"
      port: 8080
```

### Configuração com Variáveis de Ambiente

```bash
# Controle de exibição do header Authorization nos spans
export PHLOW_AUTHORIZATION_SPAN_MODE="prefix"  # none, hidden, prefix, suffix, all
```

## 🔧 Parâmetros de Configuração

### Configuração do Módulo (with)
- `host` (string, opcional): Host para bind do servidor (padrão: "0.0.0.0")
- `port` (number, opcional): Porta para bind do servidor (padrão: 4000)

### Dados de Entrada do Request (output do módulo)
- `method` (string): Método HTTP (GET, POST, PUT, etc.)
- `path` (string): Caminho da requisição
- `headers` (object): Headers da requisição
- `body` (string): Corpo da requisição (parsed JSON quando aplicável)
- `query_string` (string): Query string completa
- `query_params` (object): Query parameters parseados
- `uri` (string): URI completa incluindo query string
- `client_ip` (string): IP do cliente
- `body_size` (number): Tamanho do corpo em bytes

### Dados de Saída para Response (input do step)
- `status_code` (number, opcional): Código de status HTTP (padrão: 200)
- `headers` (object, opcional): Headers da resposta
- `body` (string, opcional): Corpo da resposta

## 💻 Exemplos de Uso

### API REST Básica

```phlow
name: "users-api"
version: "1.0.0"
main: "web_server"

modules:
  - name: "web_server"
    module: "http_server"
    with:
      host: "0.0.0.0"
      port: 3000

steps:
  - name: "route_handler"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "GET"
    then:
      condition:
        left: "{{ $input.path }}"
        operator: "equals"
        right: "/users"
      then:
        # GET /users
        return:
          status_code: 200
          headers:
            "Content-Type": "application/json"
          body:
            users: [
              { id: 1, name: "Alice" },
              { id: 2, name: "Bob" }
            ]
      else:
        condition:
          left: "{{ $input.path }}"
          operator: "starts_with"
          right: "/users/"
        then:
          # GET /users/:id
          script: |
            let user_id = $input.path.replace("/users/", "");
            {
              status_code: 200,
              body: {
                id: parseInt(user_id),
                name: `User ${user_id}`
              }
            }
        else:
          # 404 Not Found
          return:
            status_code: 404
            body: { error: "Not Found" }
    else:
      condition:
        left: "{{ $input.method }}"
        operator: "equals"
        right: "POST"
      then:
        condition:
          left: "{{ $input.path }}"
          operator: "equals"
          right: "/users"
        then:
          # POST /users
          return:
            status_code: 201
            headers:
              "Content-Type": "application/json"
              "Location": "/users/123"
            body:
              id: 123
              name: "{{ $input.body.name }}"
              email: "{{ $input.body.email }}"
              created_at: "2024-01-01T00:00:00Z"
        else:
          # 405 Method Not Allowed
          return:
            status_code: 405
            body: { error: "Method Not Allowed" }
      else:
        # 405 Method Not Allowed
        return:
          status_code: 405
          body: { error: "Method Not Allowed" }
```

### API com Autenticação

```phlow
name: "secure-api"
version: "1.0.0"
main: "secure_server"

modules:
  - name: "secure_server"
    module: "http_server"
    with:
      port: 8443

steps:
  - name: "check_auth"
    condition:
      left: "{{ $input.headers.authorization }}"
      operator: "exists"
      right: true
    then:
      condition:
        left: "{{ $input.headers.authorization }}"
        operator: "starts_with"
        right: "Bearer "
      then:
        # Token válido, continuar processamento
        script: |
          let token = $input.headers.authorization.replace("Bearer ", "");
          // Aqui você validaria o token
          true
      else:
        # Token inválido
        return:
          status_code: 401
          body: { error: "Invalid token format" }
    else:
      # Sem autorização
      return:
        status_code: 401
        headers:
          "WWW-Authenticate": "Bearer"
        body: { error: "Authentication required" }

  - name: "handle_request"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "GET"
    then:
      condition:
        left: "{{ $input.path }}"
        operator: "equals"
        right: "/protected"
      then:
        return:
          status_code: 200
          body:
            message: "This is protected data"
            user: "{{ $input.headers.authorization }}"
            timestamp: "2024-01-01T00:00:00Z"
      else:
        return:
          status_code: 404
          body: { error: "Not Found" }
    else:
      return:
        status_code: 405
        body: { error: "Method Not Allowed" }
```

### Webhook Receiver

```phlow
name: "webhook-receiver"
version: "1.0.0"
main: "webhook_server"

modules:
  - name: "webhook_server"
    module: "http_server"
    with:
      port: 9000

  - name: "logger"
    module: "log"

steps:
  - name: "validate_webhook"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "POST"
    then:
      condition:
        left: "{{ $input.path }}"
        operator: "equals"
        right: "/webhook"
      then:
        # Validar signature se necessário
        condition:
          left: "{{ $input.headers['x-signature'] }}"
          operator: "exists"
          right: true
        then:
          # Processar webhook
          script: |
            {
              valid: true,
              payload: $input.body,
              source: $input.headers['x-source'] || 'unknown'
            }
        else:
          return:
            status_code: 400
            body: { error: "Missing signature" }
      else:
        return:
          status_code: 404
          body: { error: "Webhook endpoint not found" }
    else:
      return:
        status_code: 405
        body: { error: "Only POST method allowed" }

  - name: "log_webhook"
    use: "logger"
    input:
      level: "info"
      message: "Webhook received: {{ $validate_webhook.payload }}"

  - name: "process_webhook"
    script: |
      // Processar dados do webhook
      let processed = {
        received_at: new Date().toISOString(),
        source: $validate_webhook.source,
        data: $validate_webhook.payload,
        client_ip: $input.client_ip
      };
      
      processed

  - name: "webhook_response"
    return:
      status_code: 200
      headers:
        "Content-Type": "application/json"
      body:
        success: true
        message: "Webhook processed successfully"
        processed_at: "{{ $process_webhook.received_at }}"
```

### API com CORS

```phlow
name: "cors-api"
version: "1.0.0"
main: "cors_server"

modules:
  - name: "cors_server"
    module: "http_server"
    with:
      port: 5000

steps:
  - name: "handle_cors"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "OPTIONS"
    then:
      # Preflight CORS
      return:
        status_code: 200
        headers:
          "Access-Control-Allow-Origin": "*"
          "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS"
          "Access-Control-Allow-Headers": "Content-Type, Authorization"
          "Access-Control-Max-Age": "86400"
        body: ""
    else:
      # Processar requisição normal
      script: |
        // Lógica da API
        { message: "API response", data: $input.body }

  - name: "add_cors_headers"
    return:
      status_code: 200
      headers:
        "Access-Control-Allow-Origin": "*"
        "Content-Type": "application/json"
      body: "{{ $handle_cors }}"
```

## 🔍 Estrutura de Dados

### Request Data (disponível em `$input`)

```json
{
  "method": "POST",
  "path": "/users",
  "headers": {
    "content-type": "application/json",
    "authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user-agent": "Mozilla/5.0...",
    "host": "api.example.com"
  },
  "body": {
    "name": "João Silva",
    "email": "joao@example.com"
  },
  "query_string": "limit=10&offset=0",
  "query_params": {
    "limit": "10",
    "offset": "0"
  },
  "uri": "/users?limit=10&offset=0",
  "client_ip": "192.168.1.100",
  "body_size": 1024
}
```

### Response Data (retornado pelos steps)

```json
{
  "status_code": 201,
  "headers": {
    "Content-Type": "application/json",
    "Location": "/users/123",
    "X-Request-ID": "abc-123-def"
  },
  "body": {
    "id": 123,
    "name": "João Silva",
    "email": "joao@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

## 📊 Observabilidade

### OpenTelemetry Spans

O módulo gera spans detalhados para cada requisição:

```
http_request
├── otel.name: "POST /users"
├── http.request.method: "POST"
├── http.request.path: "/users"
├── http.request.size: 1024
├── http.request.body.size: 512
├── http.response.status_code: 201
├── http.response.body.size: 256
├── http.request.header.user-agent: "Mozilla/5.0..."
├── http.request.header.authorization: "Bearer eyJhbG..."
└── http.response.header.content-type: "application/json"
```

### Headers Capturados

O tracing captura automaticamente headers comuns:
- `user-agent`
- `host`
- `content-type`
- `authorization` (com controle de privacidade)
- `x-request-id`
- `x-trace-id`
- `origin`
- `referer`
- E muitos outros...

### Modos de Authorization

Configure como o header `Authorization` aparece nos spans:

```bash
# Não mostrar authorization
export PHLOW_AUTHORIZATION_SPAN_MODE="none"

# Mostrar mascarado
export PHLOW_AUTHORIZATION_SPAN_MODE="hidden"  # "xxxxxxxxxx"

# Mostrar prefixo
export PHLOW_AUTHORIZATION_SPAN_MODE="prefix"  # "Bearer eyJhbG..."

# Mostrar sufixo
export PHLOW_AUTHORIZATION_SPAN_MODE="suffix"  # "...xyz123"

# Mostrar completo
export PHLOW_AUTHORIZATION_SPAN_MODE="all"     # "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## 🌡️ Health Check

O servidor automaticamente expõe um endpoint de health check:

```bash
curl http://localhost:8080/health
# Resposta: "ok"
```

## 🚨 Tratamento de Erros

### Códigos de Status HTTP

```phlow
steps:
  - name: "error_handler"
    condition:
      left: "{{ $some_condition }}"
      operator: "equals"
      right: "error"
    then:
      # Bad Request
      return:
        status_code: 400
        body: { error: "Invalid request parameters" }
    else:
      condition:
        left: "{{ $auth_failed }}"
        operator: "equals"
        right: true
      then:
        # Unauthorized
        return:
          status_code: 401
          body: { error: "Authentication required" }
      else:
        condition:
          left: "{{ $resource_not_found }}"
          operator: "equals"
          right: true
        then:
          # Not Found
          return:
            status_code: 404
            body: { error: "Resource not found" }
        else:
          # Internal Server Error
          return:
            status_code: 500
            body: { error: "Internal server error" }
```

## 🔒 Segurança

### Headers de Segurança

```phlow
steps:
  - name: "secure_response"
    return:
      status_code: 200
      headers:
        "X-Content-Type-Options": "nosniff"
        "X-Frame-Options": "DENY"
        "X-XSS-Protection": "1; mode=block"
        "Strict-Transport-Security": "max-age=31536000; includeSubDomains"
        "Content-Security-Policy": "default-src 'self'"
      body: { message: "Secure response" }
```

### Validação de Input

```phlow
steps:
  - name: "validate_input"
    condition:
      left: "{{ $input.body.email }}"
      operator: "matches"
      right: "^[\\w\\.-]+@[\\w\\.-]+\\.[a-zA-Z]{2,}$"
    then:
      # Email válido
      script: "Valid email: {{ $input.body.email }}"
    else:
      return:
        status_code: 400
        body: { error: "Invalid email format" }
```

## 📈 Performance

- **Hyper**: Servidor HTTP de alta performance
- **Tokio**: Runtime assíncrono para máxima concorrência
- **Keep-alive**: Conexões persistentes para reduzir latência
- **Zero-copy**: Parsing eficiente de dados
- **Tracing otimizado**: Observabilidade sem impacto significativo

## 🏷️ Tags

- http
- rest
- api
- server
- web
- endpoint
- hyper
- tokio

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
