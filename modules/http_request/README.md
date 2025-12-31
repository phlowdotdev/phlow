# Módulo HTTP Request

O módulo HTTP Request fornece funcionalidades completas para realizar requisições HTTP/HTTPS, suportando todos os métodos HTTP padrão, headers customizados, SSL/TLS, timeouts e tratamento abrangente de erros.

## 🚀 Funcionalidades

### Características Principais

- ✅ **Métodos HTTP completos**: GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, TRACE, CONNECT
- ✅ **Headers customizados**: Suporte completo a headers HTTP
- ✅ **Body flexível**: Suporte a texto, JSON e dados binários
- ✅ **SSL/TLS**: Verificação de certificados configurável
- ✅ **Timeouts**: Configuração de timeout personalizável
- ✅ **Auto-detecção**: Content-Type automático para JSON
- ✅ **User-Agent**: User-Agent padrão configurável
- ✅ **Tratamento de erros**: Respostas estruturadas com códigos de status
- ✅ **Observabilidade**: Tracing completo com OpenTelemetry

## 📋 Configuração

### Configuração Básica

```yaml
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

```yaml
steps:
  - name: "get_users"
    use: "http_client"
    input:
      method: "GET"
      url: "https://jsonplaceholder.typicode.com/users"
```

### Requisição POST com JSON

```yaml
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

```yaml
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

```yaml
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

```yaml
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
```yaml
input:
  method: "GET"
  url: "https://api.example.com/users"
  headers:
    "Accept": "application/json"
```

### POST - Criar Recurso
```yaml
input:
  method: "POST"
  url: "https://api.example.com/users"
  headers:
    "Content-Type": "application/json"
  body: '{"name": "Novo Usuário"}'
```

### PUT - Atualizar Recurso Completo
```yaml
input:
  method: "PUT"
  url: "https://api.example.com/users/123"
  body: '{"name": "Nome Atualizado", "email": "novo@email.com"}'
```

### PATCH - Atualizar Recurso Parcial
```yaml
input:
  method: "PATCH"
  url: "https://api.example.com/users/123"
  body: '{"name": "Apenas Nome Atualizado"}'
```

### DELETE - Remover Recurso
```yaml
input:
  method: "DELETE"
  url: "https://api.example.com/users/123"
```

### OPTIONS - Verificar Opções
```yaml
input:
  method: "OPTIONS"
  url: "https://api.example.com/users"
```

### HEAD - Buscar Headers
```yaml
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

```yaml
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
    assert: "{{ $authenticate.is_success == true }}"
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
```yaml
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
```yaml
input:
  headers:
    "Authorization": "Bearer {{ $jwt_token }}"
    "X-API-Key": "{{ $api_key }}"
    "X-Request-ID": "{{ $request_id }}"
    "X-Forwarded-For": "{{ $client_ip }}"
```

## 🚨 Tratamento de Erros

### Verificação de Status
```yaml
steps:
  - name: "api_call"
    use: "http_client"
    input:
      method: "GET"
      url: "https://api.example.com/data"
      
  - name: "handle_response"
    assert: "{{ $api_call.is_success == true }}"
    then:
      # Processar dados de sucesso
      script: "Success: {{ $api_call.response.body }}"
    else:
      # Tratar erro
      script: "Error {{ $api_call.response.status_code }}: {{ $api_call.message }}"
```

### Diferentes Tipos de Erro
```yaml
steps:
  - name: "check_error_type"
    assert: "{{ $api_call.response.status_code == 404 }}"
    then:
      return: "Resource not found"
    else:
      assert: "{{ $api_call.response.status_code >= 500 }}"
      then:
        return: "Server error"
      else:
        return: "Client error"
```

## 📈 Performance e Timeouts

### Configuração de Timeouts
```yaml
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
**Autor**: Philippe Assis <codephilippe@gmail.com>  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
