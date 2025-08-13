---
sidebar_position: 5
title: HTTP Server Module
hide_title: true
---

# HTTP Server Module

The HTTP Server module provides a complete, high-performance web server for building REST APIs, webhooks, and web services. Built with Hyper and Tokio, it offers complete observability with OpenTelemetry and comprehensive OpenAPI 3.0 validation.

## 🚀 Features

### Key Features

- ✅ **High-performance HTTP/HTTPS server**
- ✅ **Support for all HTTP methods** (GET, POST, PUT, PATCH, DELETE, OPTIONS, etc.)
- ✅ **Dynamic routing** via Phlow flows
- ✅ **OpenAPI 3.0 specification support** with automatic validation
- ✅ **Request validation** (parameters, body, headers, Content-Type)
- ✅ **Schema validation** (string formats, numeric constraints, arrays, objects)
- ✅ **Custom headers** for requests and responses
- ✅ **Automatic parsing** of JSON, query parameters, and headers
- ✅ **Health check** endpoint (`/health`)
- ✅ **Complete observability** with OpenTelemetry tracing
- ✅ **Flexible configuration** of host and port
- ✅ **Authorization handling** with different span modes
- ✅ **Tracing middleware** for all requests
- ✅ **Keep-alive support** for persistent connections

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

### Configuração com OpenAPI

Para APIs que seguem especificação OpenAPI 3.0, configure o caminho para o arquivo de especificação:

```phlow
name: "openapi-server"
version: "1.0.0"
main: "api_server"

modules:
  - name: "api_server"
    module: "http_server"
    with:
      host: "0.0.0.0"
      port: 8080
      openapi_spec: "./openapi.yaml"  # Caminho para a especificação OpenAPI
```

## 🔍 OpenAPI 3.0 e Validação

### Recursos de Validação

O módulo HTTP Server oferece validação completa baseada em especificação OpenAPI 3.0:

#### ✅ Validação de Parâmetros de Rota
- Validação de tipos (`integer`, `string`, `number`, `boolean`)
- Validação de formatos (`email`, `date`, `uuid`, etc.)
- Validação de constraints (`minimum`, `maximum`, `minLength`, `maxLength`)
- Validação de patterns regex
- Parâmetros obrigatórios e opcionais

#### ✅ Validação do Corpo da Requisição
- Validação de schemas JSON complexos
- Campos obrigatórios e opcionais
- Validação de tipos de dados
- Validação de formatos especiais
- Validação de arrays e objetos aninhados
- Propriedades adicionais (`additionalProperties`)

#### ✅ Validação de Headers
- Validação do Content-Type
- Headers obrigatórios
- Formato de headers customizados

#### ✅ Validação de Query Parameters
- Todos os tipos de validação suportados
- Arrays de parâmetros
- Parâmetros opcionais com valores padrão

### Exemplo de Especificação OpenAPI

```yaml
# openapi.yaml
openapi: 3.0.0
info:
  title: Users API
  version: 1.0.0
  description: API para gerenciamento de usuários

paths:
  /users:
    post:
      summary: Criar usuário
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/NewUser'
      responses:
        '201':
          description: Usuário criado com sucesso
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
        '400':
          description: Dados inválidos
    get:
      summary: Listar usuários
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 10
        - name: offset
          in: query
          schema:
            type: integer
            minimum: 0
            default: 0
      responses:
        '200':
          description: Lista de usuários

  /users/{id}:
    parameters:
      - name: id
        in: path
        required: true
        schema:
          type: integer
          minimum: 1
    get:
      summary: Buscar usuário por ID
      responses:
        '200':
          description: Usuário encontrado
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
        '404':
          description: Usuário não encontrado
    put:
      summary: Atualizar usuário
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateUser'
      responses:
        '200':
          description: Usuário atualizado
        '404':
          description: Usuário não encontrado
    delete:
      summary: Remover usuário
      responses:
        '204':
          description: Usuário removido
        '404':
          description: Usuário não encontrado

components:
  schemas:
    User:
      type: object
      required:
        - id
        - name
        - email
      properties:
        id:
          type: integer
          example: 123
        name:
          type: string
          pattern: '^[a-zA-ZÀ-ÿ ]+$'
          minLength: 2
          maxLength: 100
          example: "João Silva"
        email:
          type: string
          format: email
          example: "joao@example.com"
        phone:
          type: string
          pattern: '^\+?[1-9]\d{1,14}$'
          example: "+5511999999999"
        age:
          type: integer
          minimum: 18
          maximum: 120
          example: 30
        created_at:
          type: string
          format: date-time
          example: "2024-01-01T00:00:00Z"

    NewUser:
      type: object
      required:
        - name
        - email
      properties:
        name:
          type: string
          pattern: '^[a-zA-ZÀ-ÿ ]+$'
          minLength: 2
          maxLength: 100
        email:
          type: string
          format: email
        phone:
          type: string
          pattern: '^\+?[1-9]\d{1,14}$'
        age:
          type: integer
          minimum: 18
          maximum: 120

    UpdateUser:
      type: object
      properties:
        name:
          type: string
          pattern: '^[a-zA-ZÀ-ÿ ]+$'
          minLength: 2
          maxLength: 100
        email:
          type: string
          format: email
        phone:
          type: string
          pattern: '^\+?[1-9]\d{1,14}$'
        age:
          type: integer
          minimum: 18
          maximum: 120
```

### Validações Automáticas

Com a especificação OpenAPI configurada, o servidor automaticamente:

#### 🔍 **Valida Parâmetros de Rota**
```bash
# ✅ Válido
curl -X GET "http://localhost:8080/users/123"

# ❌ Inválido - ID deve ser um número positivo
curl -X GET "http://localhost:8080/users/-1"
# Resposta: 400 Bad Request
```

#### 🔍 **Valida Query Parameters**
```bash
# ✅ Válido
curl -X GET "http://localhost:8080/users?limit=10&offset=0"

# ❌ Inválido - limit deve ser entre 1 e 100
curl -X GET "http://localhost:8080/users?limit=200"
# Resposta: 400 Bad Request
```

#### 🔍 **Valida Content-Type**
```bash
# ✅ Válido
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João", "email": "joao@example.com"}'

# ❌ Inválido - Content-Type incorreto
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: text/plain" \
  -d '{"name": "João", "email": "joao@example.com"}'
# Resposta: 400 Bad Request
```

#### 🔍 **Valida Corpo da Requisição**
```bash
# ✅ Válido
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João Silva", "email": "joao@example.com", "age": 30}'

# ❌ Inválido - campos obrigatórios ausentes
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"age": 30}'
# Resposta: 400 Bad Request

# ❌ Inválido - formato de email incorreto
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João", "email": "email-inválido"}'
# Resposta: 400 Bad Request

# ❌ Inválido - idade fora do range permitido
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João", "email": "joao@example.com", "age": 15}'
# Resposta: 400 Bad Request
```

### Tipos de Validação Suportados

#### 📝 **String**
- `minLength` / `maxLength`: Comprimento mínimo e máximo
- `pattern`: Validação por regex
- `format`: Formatos especiais (email, date, uuid, etc.)
- `enum`: Lista de valores permitidos

#### 🔢 **Números (integer/number)**
- `minimum` / `maximum`: Valor mínimo e máximo
- `exclusiveMinimum` / `exclusiveMaximum`: Valores exclusivos
- `multipleOf`: Múltiplo de um valor específico

#### 📋 **Arrays**
- `minItems` / `maxItems`: Número mínimo e máximo de itens
- `uniqueItems`: Itens únicos no array
- Validação de itens individuais

#### 🏗️ **Objects**
- `required`: Propriedades obrigatórias
- `additionalProperties`: Controle de propriedades extras
- `minProperties` / `maxProperties`: Número de propriedades
- Validação de propriedades aninhadas

#### ✅ **Boolean**
- Validação de tipo estrita

### Formatos Especiais Suportados

- `email`: Validação de email
- `date`: Formato de data (YYYY-MM-DD)
- `date-time`: Formato ISO 8601
- `uuid`: UUID válido
- `uri`: URI válida
- `hostname`: Nome de host válido
- `ipv4` / `ipv6`: Endereços IP

### Content-Types Suportados

Por padrão, o servidor aceita os seguintes Content-Types:
- `application/json` (padrão)
- `application/octet-stream`

Para adicionar outros tipos, configure no código:

```rust
// No arquivo resolver.rs, modificar o array ACCEPTED_CONTENT_TYPES
const ACCEPTED_CONTENT_TYPES: &[&str] = &[
    "application/json",
    "application/octet-stream",
    "application/xml",  // Exemplo de tipo adicional
    "text/csv",         // Exemplo de tipo adicional
];
```

### Mensagens de Erro de Validação

O servidor retorna mensagens de erro detalhadas para facilitar debugging:

```json
{
  "error": "Validation failed",
  "details": {
    "field": "email",
    "message": "Invalid email format",
    "value": "email-invalido",
    "expected": "Valid email address (e.g., user@example.com)"
  }
}
```

### Exemplo Completo de Flow com OpenAPI

```phlow
name: "openapi-users-api"
version: "1.0.0"
main: "users_server"

modules:
  - name: "users_server"
    module: "http_server"
    with:
      host: "0.0.0.0"
      port: 8080
      openapi_spec: "./openapi.yaml"

  - name: "db"
    module: "postgres"
    with:
      host: "localhost"
      database: "users_db"
      user: "postgres"
      password: "password"

steps:
  # Com OpenAPI, a validação é automática antes de chegar aos steps
  
  - name: "handle_users"
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
        # POST /users - dados já validados pelo OpenAPI
        use: "db"
        input:
          query: |
            INSERT INTO users (name, email, phone, age) 
            VALUES ($1, $2, $3, $4) 
            RETURNING id, name, email, phone, age, created_at
          params:
            - "{{ $input.body.name }}"
            - "{{ $input.body.email }}"
            - "{{ $input.body.phone }}"
            - "{{ $input.body.age }}"
      else:
        # GET /users
        condition:
          left: "{{ $input.method }}"
          operator: "equals"
          right: "GET"
        then:
          use: "db"
          input:
            query: |
              SELECT id, name, email, phone, age, created_at 
              FROM users 
              ORDER BY created_at DESC 
              LIMIT $1 OFFSET $2
            params:
              - "{{ $input.query_params.limit || 10 }}"
              - "{{ $input.query_params.offset || 0 }}"
        else:
          return:
            status_code: 405
            body: { error: "Method not allowed" }
    else:
      condition:
        left: "{{ $input.method }}"
        operator: "equals"
        right: "GET"
      then:
        condition:
          left: "{{ $input.path }}"
          operator: "matches"
          right: "^/users/\\d+$"
        then:
          # GET /users/:id - parâmetro já validado pelo OpenAPI
          script: |
            let user_id = parseInt($input.path.split('/')[2]);
            { user_id: user_id }
        else:
          return:
            status_code: 404
            body: { error: "Not found" }
      else:
        return:
          status_code: 405
          body: { error: "Method not allowed" }

  - name: "get_user_by_id"
    condition:
      left: "{{ $handle_users.user_id }}"
      operator: "exists"
      right: true
    then:
      use: "db"
      input:
        query: "SELECT id, name, email, phone, age, created_at FROM users WHERE id = $1"
        params:
          - "{{ $handle_users.user_id }}"

  - name: "format_response"
    condition:
      left: "{{ $input.method }}"
      operator: "equals"
      right: "POST"
    then:
      return:
        status_code: 201
        headers:
          "Content-Type": "application/json"
          "Location": "/users/{{ $handle_users.rows[0].id }}"
        body: "{{ $handle_users.rows[0] }}"
    else:
      condition:
        left: "{{ $get_user_by_id.rows }}"
        operator: "length_gt"
        right: 0
      then:
        return:
          status_code: 200
          headers:
            "Content-Type": "application/json"
          body: "{{ $get_user_by_id.rows[0] }}"
      else:
        condition:
          left: "{{ $handle_users }}"
          operator: "exists"
          right: true
        then:
          return:
            status_code: 404
            body: { error: "User not found" }
        else:
          return:
            status_code: 200
            headers:
              "Content-Type": "application/json"
            body:
              users: "{{ $handle_users.rows }}"
              total: "{{ $handle_users.rows.length }}"
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

## 🧪 Testes e Debugging

### Scripts de Teste Automatizados

Para APIs com OpenAPI, recomenda-se criar scripts de teste abrangentes:

```bash
#!/bin/bash
# run_integration_tests.sh

API_URL="http://localhost:8080"
TOTAL_TESTS=0
PASSED_TESTS=0

# Função para executar teste
test_request() {
    local description="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="$4"
    local data="$5"
    local content_type="${6:-application/json}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ -n "$data" ]; then
        response=$(curl -s -X "$method" "$API_URL$endpoint" \
            -H "Content-Type: $content_type" \
            -d "$data" \
            -w "HTTP_STATUS:%{http_code}")
    else
        response=$(curl -s -X "$method" "$API_URL$endpoint" \
            -w "HTTP_STATUS:%{http_code}")
    fi
    
    http_status=$(echo "$response" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    
    if [ "$http_status" = "$expected_status" ]; then
        echo "✅ Test $TOTAL_TESTS: $description"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo "❌ Test $TOTAL_TESTS: $description (Expected: $expected_status, Got: $http_status)"
    fi
}

echo "🚀 Executando testes de integração..."
echo "URL da API: $API_URL"
echo

# Testes de validação
test_request "POST /users - Válido" "POST" "/users" "201" \
    '{"name": "João Silva", "email": "joao@example.com", "age": 30}'
    
test_request "POST /users - Email inválido" "POST" "/users" "400" \
    '{"name": "João Silva", "email": "email-invalido", "age": 30}'
    
test_request "POST /users - Campo obrigatório ausente" "POST" "/users" "400" \
    '{"age": 30}'
    
test_request "POST /users - Content-Type inválido" "POST" "/users" "400" \
    '{"name": "João", "email": "joao@example.com"}' "text/plain"
    
test_request "GET /users - Válido" "GET" "/users" "200"
test_request "GET /users - Query param inválido" "GET" "/users?limit=200" "400"
test_request "GET /users/123 - Válido" "GET" "/users/123" "200"
test_request "GET /users/-1 - ID inválido" "GET" "/users/-1" "400"

# Relatório final
echo
echo "📊 Relatório de Testes:"
echo "Total: $TOTAL_TESTS"
echo "Passou: $PASSED_TESTS"
echo "Falhou: $((TOTAL_TESTS - PASSED_TESTS))"
echo "Taxa de sucesso: $((PASSED_TESTS * 100 / TOTAL_TESTS))%"
```

### Debugging de Validação

Para debugar problemas de validação, use logs detalhados:

```bash
# Ativar logs de debug
export RUST_LOG=debug

# Executar o servidor
cargo run --example api-openapi
```

Os logs mostrarão:
```
[DEBUG] openapi: Validating request: POST /users
[DEBUG] openapi: Content-Type: application/json
[DEBUG] openapi: Request body: {"name": "João", "email": "invalid-email"}
[DEBUG] openapi: Validation failed: Invalid email format
[DEBUG] openapi: Field 'email' failed validation with value 'invalid-email'
```

### Teste Manual com curl

```bash
# Teste básico de saúde
curl http://localhost:8080/health

# Teste de criação válida
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João Silva", "email": "joao@example.com", "phone": "+5511999999999", "age": 30}' \
  -v

# Teste de validação de email
curl -X POST "http://localhost:8080/users" \
  -H "Content-Type: application/json" \
  -d '{"name": "João", "email": "email-invalido"}' \
  -v

# Teste de query parameters
curl "http://localhost:8080/users?limit=10&offset=0" -v

# Teste de parâmetro de rota
curl "http://localhost:8080/users/123" -v
```

### Monitoramento de Performance

Para monitorar performance em produção:

```bash
# Métricas básicas com curl
curl -w "@curl-format.txt" -s -o /dev/null "http://localhost:8080/users"

# Onde curl-format.txt contém:
# time_namelookup:  %{time_namelookup}\n
# time_connect:     %{time_connect}\n
# time_appconnect:  %{time_appconnect}\n
# time_pretransfer: %{time_pretransfer}\n
# time_redirect:    %{time_redirect}\n
# time_starttransfer: %{time_starttransfer}\n
# ----------\n
# time_total:       %{time_total}\n
```

## 📈 Performance

- **Hyper**: Servidor HTTP de alta performance
- **Tokio**: Runtime assíncrono para máxima concorrência
- **Keep-alive**: Conexões persistentes para reduzir latência
- **Zero-copy**: Parsing eficiente de dados
- **Tracing otimizado**: Observabilidade sem impacto significativo
- **Validação otimizada**: Cache de schemas OpenAPI para máxima performance

## 🏷️ Tags

- http
- rest
- api
- server
- web
- endpoint
- hyper
- tokio
- openapi
- validation
- json-schema
- swagger
- content-type
- request-validation
- schema-validation
- api-documentation

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
