---
sidebar_position: 4
title: HTTP Request Module
hide_title: true
---

# HTTP Request Module

The HTTP Request module provides comprehensive functionality for making HTTP/HTTPS requests, supporting all standard HTTP methods, custom headers, SSL/TLS, timeouts, and comprehensive error handling.

## 🚀 Features

### Key Features

- ✅ **Complete HTTP methods**: GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE, CONNECT
- ✅ **Custom headers**: Full support for HTTP headers
- ✅ **Flexible body**: Support for text, JSON, and binary data
- ✅ **SSL/TLS**: Configurable certificate verification
- ✅ **Timeouts**: Customizable timeout configuration
- ✅ **Auto-detection**: Automatic Content-Type for JSON
- ✅ **User-Agent**: Configurable default User-Agent
- ✅ **Error handling**: Structured responses with status codes
- ✅ **Observability**: Complete tracing with OpenTelemetry

## 📋 Configuração

### Configuração Básica

```phlow
modules:
  - name: "http_client"
    module: "http_request"
    with:
      timeout: 30
      verify_ssl: true
```

### Configuração com Variáveis de Ambiente

```bash
# User-Agent customizado
export PHLOW_HTTP_REQUEST_USER_AGENT="MyApp/1.0.0"

# Desabilitar User-Agent padrão
export PHLOW_HTTP_REQUEST_USER_AGENT_DISABLE="true"
```

## 🔧 Parâmetros de Configuração

### Configuração do Módulo (with)
- `timeout` (number, opcional): Timeout em segundos (padrão: 29)
- `verify_ssl` (boolean, opcional): Verificar certificados SSL (padrão: true)

### Entrada (input)
- `method` (string, obrigatório): Método HTTP
- `url` (string, obrigatório): URL de destino
- `headers` (object, opcional): Headers HTTP
- `body` (string, opcional): Corpo da requisição

### Saída (output)
- `response` (object): Resposta HTTP completa
  - `status_code` (number): Código de status HTTP
  - `headers` (object): Headers da resposta
  - `body` (string): Corpo da resposta (parsed JSON se aplicável)
- `is_success` (boolean): Se a requisição foi bem-sucedida (200-299)
- `is_error` (boolean): Se houve erro (400-599)
- `message` (string): Mensagem de erro ou sucesso

## 💻 Exemplos de Uso

### Requisição GET Simples

```phlow
steps:
  - name: "get_users"
    use: "http_client"
    input:
      method: "GET"
      url: "https://jsonplaceholder.typicode.com/users"
```

### Requisição POST com JSON

```phlow
steps:
  - name: "create_user"
    use: "http_client"
    input:
      method: "POST"
      url: "https://api.example.com/users"
      headers:
        "Authorization": "Bearer {{ $auth_token }}"
        "Content-Type": "application/json"
      body: |
        {
          "name": "João Silva",
          "email": "joao@example.com",
          "age": 30
        }
```

### Requisição PUT com Headers Customizados

```phlow
steps:
  - name: "update_user"
    use: "http_client"
    input:
      method: "PUT"
      url: "https://api.example.com/users/{{ $user_id }}"
      headers:
        "Authorization": "Bearer {{ $auth_token }}"
        "Content-Type": "application/json"
        "X-Request-ID": "{{ $request_id }}"
        "X-User-Agent": "MyApp/1.0.0"
      body: |
        {
          "name": "João Silva Updated",
          "email": "joao.updated@example.com"
        }
```

### Requisição DELETE

```phlow
steps:
  - name: "delete_user"
    use: "http_client"
    input:
      method: "DELETE"
      url: "https://api.example.com/users/{{ $user_id }}"
      headers:
        "Authorization": "Bearer {{ $auth_token }}"
```

### Requisição com Timeout Customizado

```phlow
modules:
  - name: "slow_api_client"
    module: "http_request"
    with:
      timeout: 60  # 60 segundos
      verify_ssl: false  # Para APIs de desenvolvimento

steps:
  - name: "slow_operation"
    use: "slow_api_client"
    input:
      method: "POST"
      url: "https://slow-api.example.com/process"
      body: "{{ $large_data }}"
```

## 🔍 Métodos HTTP Suportados

### GET - Buscar Dados
```phlow
input:
  method: "GET"
  url: "https://api.example.com/users"
  headers:
    "Accept": "application/json"
```

### POST - Criar Recurso
```phlow
input:
  method: "POST"
  url: "https://api.example.com/users"
  headers:
    "Content-Type": "application/json"
  body: '{"name": "Novo Usuário"}'
```

### PUT - Atualizar Recurso Completo
```phlow
input:
  method: "PUT"
  url: "https://api.example.com/users/123"
  body: '{"name": "Nome Atualizado", "email": "novo@email.com"}'
```

### PATCH - Atualizar Recurso Parcial
```phlow
input:
  method: "PATCH"
  url: "https://api.example.com/users/123"
  body: '{"name": "Apenas Nome Atualizado"}'
```

### DELETE - Remover Recurso
```phlow
input:
  method: "DELETE"
  url: "https://api.example.com/users/123"
```

### OPTIONS - Verificar Opções
```phlow
input:
  method: "OPTIONS"
  url: "https://api.example.com/users"
```

### HEAD - Buscar Headers
```phlow
input:
  method: "HEAD"
  url: "https://api.example.com/users"
```

## 📊 Formato de Resposta

### Resposta de Sucesso

```json
{
  "response": {
    "status_code": 200,
    "headers": {
      "content-type": "application/json",
      "content-length": "1234",
      "date": "Mon, 01 Jan 2024 00:00:00 GMT"
    },
    "body": {
      "id": 1,
      "name": "João Silva",
      "email": "joao@example.com"
    }
  },
  "is_success": true,
  "is_error": false,
  "message": "Request successful"
}
```

### Resposta de Erro HTTP

```json
{
  "response": {
    "status_code": 404,
    "headers": {
      "content-type": "application/json"
    },
    "body": {
      "error": "User not found"
    }
  },
  "is_success": false,
  "is_error": true,
  "message": "Request successful"
}
```

### Erro de Conexão

```json
{
  "response": undefined,
  "is_success": false,
  "is_error": true,
  "message": "Request error: Connection timeout"
}
```

## 🌐 Exemplo Completo - API Client

```phlow
name: "user-api-client"
version: "1.0.0"
description: "Cliente completo para API de usuários"

modules:
  - name: "api_client"
    module: "http_request"
    with:
      timeout: 30
      verify_ssl: true

  - name: "secure_api_client"
    module: "http_request"
    with:
      timeout: 60
      verify_ssl: false  # Para desenvolvimento

steps:
  - name: "authenticate"
    use: "api_client"
    input:
      method: "POST"
      url: "https://api.example.com/auth/login"
      headers:
        "Content-Type": "application/json"
      body: |
        {
          "username": "{{ $username }}",
          "password": "{{ $password }}"
        }

  - name: "check_auth_success"
    condition:
      left: "{{ $authenticate.is_success }}"
      operator: "equals"
      right: true
    else:
      return: "Authentication failed: {{ $authenticate.message }}"

  - name: "get_user_profile"
    use: "api_client"
    input:
      method: "GET"
      url: "https://api.example.com/users/me"
      headers:
        "Authorization": "Bearer {{ $authenticate.response.body.access_token }}"
        "Accept": "application/json"

  - name: "update_user_profile"
    use: "api_client"
    input:
      method: "PUT"
      url: "https://api.example.com/users/me"
      headers:
        "Authorization": "Bearer {{ $authenticate.response.body.access_token }}"
        "Content-Type": "application/json"
      body: |
        {
          "name": "{{ $updated_name }}",
          "email": "{{ $updated_email }}",
          "preferences": {
            "theme": "dark",
            "notifications": true
          }
        }

  - name: "get_user_orders"
    use: "api_client"
    input:
      method: "GET"
      url: "https://api.example.com/users/me/orders"
      headers:
        "Authorization": "Bearer {{ $authenticate.response.body.access_token }}"

  - name: "create_order"
    use: "api_client"
    input:
      method: "POST"
      url: "https://api.example.com/orders"
      headers:
        "Authorization": "Bearer {{ $authenticate.response.body.access_token }}"
        "Content-Type": "application/json"
      body: |
        {
          "items": [
            {
              "product_id": "{{ $product_id }}",
              "quantity": {{ $quantity }},
              "price": {{ $price }}
            }
          ],
          "shipping_address": {
            "street": "{{ $address.street }}",
            "city": "{{ $address.city }}",
            "zipcode": "{{ $address.zipcode }}"
          }
        }

  - name: "final_result"
    script: |
      {
        profile: $get_user_profile.response.body,
        profile_updated: $update_user_profile.is_success,
        orders: $get_user_orders.response.body,
        new_order: $create_order.response.body
      }
```

## 🔒 Configuração de Segurança

### SSL/TLS
```phlow
modules:
  - name: "secure_client"
    module: "http_request"
    with:
      verify_ssl: true  # Produção
      
  - name: "dev_client"
    module: "http_request"
    with:
      verify_ssl: false  # Desenvolvimento
```

### Headers de Segurança
```phlow
input:
  headers:
    "Authorization": "Bearer {{ $jwt_token }}"
    "X-API-Key": "{{ $api_key }}"
    "X-Request-ID": "{{ $request_id }}"
    "X-Forwarded-For": "{{ $client_ip }}"
```

## 🚨 Tratamento de Erros

### Verificação de Status
```phlow
steps:
  - name: "api_call"
    use: "http_client"
    input:
      method: "GET"
      url: "https://api.example.com/data"
      
  - name: "handle_response"
    condition:
      left: "{{ $api_call.is_success }}"
      operator: "equals"
      right: true
    then:
      # Processar dados de sucesso
      script: "Success: {{ $api_call.response.body }}"
    else:
      # Tratar erro
      script: "Error {{ $api_call.response.status_code }}: {{ $api_call.message }}"
```

### Diferentes Tipos de Erro
```phlow
steps:
  - name: "check_error_type"
    condition:
      left: "{{ $api_call.response.status_code }}"
      operator: "equals"
      right: 404
    then:
      return: "Resource not found"
    else:
      condition:
        left: "{{ $api_call.response.status_code }}"
        operator: "greater_than_or_equal"
        right: 500
      then:
        return: "Server error"
      else:
        return: "Client error"
```

## 📈 Performance e Timeouts

### Configuração de Timeouts
```phlow
modules:
  - name: "fast_api"
    module: "http_request"
    with:
      timeout: 5  # 5 segundos para APIs rápidas
      
  - name: "slow_api"
    module: "http_request"
    with:
      timeout: 120  # 2 minutos para processamento longo
```

## 🏷️ Tags

- http
- https
- request
- api
- client
- rest
- web

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
