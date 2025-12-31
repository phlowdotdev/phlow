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

## 📋 Configuration

### Basic Configuration

```phlow
modules:
  - name: "http_client"
    module: "http_request"
    with:
      timeout: 30
      verify_ssl: true
```

### Configuration with Environment Variables

```bash
# Custom User-Agent
export PHLOW_HTTP_REQUEST_USER_AGENT="MyApp/1.0.0"

# Disable default User-Agent
export PHLOW_HTTP_REQUEST_USER_AGENT_DISABLE="true"
```

## 🔧 Configuration Parameters

### Module Configuration (with)
- `timeout` (number, optional): Timeout in seconds (default: 29)
- `verify_ssl` (boolean, optional): Verify SSL certificates (default: true)

### Input
- `method` (string, required): HTTP method
- `url` (string, required): Target URL
- `headers` (object, optional): HTTP headers
- `body` (string, optional): Request body

### Output
- `response` (object): Complete HTTP response
  - `status_code` (number): HTTP status code
  - `headers` (object): Response headers
  - `body` (string): Response body (parsed JSON if applicable)
- `is_success` (boolean): Whether the request was successful (200-299)
- `is_error` (boolean): Whether there was an error (400-599)
- `message` (string): Error or success message

## 💻 Usage Examples

### Simple GET Request

```phlow
steps:
  - name: "get_users"
    use: "http_client"
    input:
      method: "GET"
      url: "https://jsonplaceholder.typicode.com/users"
```

### POST Request with JSON

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
          "name": "John Smith",
          "email": "john@example.com",
          "age": 30
        }
```

### PUT Request with Custom Headers

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
          "name": "John Smith Updated",
          "email": "john.updated@example.com"
        }
```

### DELETE Request

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

### Request with Custom Timeout

```phlow
modules:
  - name: "slow_api_client"
    module: "http_request"
    with:
      timeout: 60  # 60 seconds
      verify_ssl: false  # For development APIs

steps:
  - name: "slow_operation"
    use: "slow_api_client"
    input:
      method: "POST"
      url: "https://slow-api.example.com/process"
      body: "{{ $large_data }}"
```

## 🔍 Supported HTTP Methods

### GET - Retrieve Data
```phlow
input:
  method: "GET"
  url: "https://api.example.com/users"
  headers:
    "Accept": "application/json"
```

### POST - Create Resource
```phlow
input:
  method: "POST"
  url: "https://api.example.com/users"
  headers:
    "Content-Type": "application/json"
  body: '{"name": "New User"}'
```

### PUT - Update Complete Resource
```phlow
input:
  method: "PUT"
  url: "https://api.example.com/users/123"
  body: '{"name": "Updated Name", "email": "new@email.com"}'
```

### PATCH - Update Partial Resource
```phlow
input:
  method: "PATCH"
  url: "https://api.example.com/users/123"
  body: '{"name": "Only Name Updated"}'
```

### DELETE - Remove Resource
```phlow
input:
  method: "DELETE"
  url: "https://api.example.com/users/123"
```

### OPTIONS - Check Options
```phlow
input:
  method: "OPTIONS"
  url: "https://api.example.com/users"
```

### HEAD - Retrieve Headers
```phlow
input:
  method: "HEAD"
  url: "https://api.example.com/users"
```

## 📊 Response Format

### Success Response

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
      "name": "John Smith",
      "email": "john@example.com"
    }
  },
  "is_success": true,
  "is_error": false,
  "message": "Request successful"
}
```

### HTTP Error Response

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

### Connection Error

```json
{
  "response": undefined,
  "is_success": false,
  "is_error": true,
  "message": "Request error: Connection timeout"
}
```

## 🌐 Complete Example - API Client

```phlow
name: "user-api-client"
version: "1.0.0"
description: "Complete client for user API"

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
      verify_ssl: false  # For development

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

## 🔒 Security Configuration

### SSL/TLS
```phlow
modules:
  - name: "secure_client"
    module: "http_request"
    with:
      verify_ssl: true  # Production
      
  - name: "dev_client"
    module: "http_request"
    with:
      verify_ssl: false  # Development
```

### Security Headers
```phlow
input:
  headers:
    "Authorization": "Bearer {{ $jwt_token }}"
    "X-API-Key": "{{ $api_key }}"
    "X-Request-ID": "{{ $request_id }}"
    "X-Forwarded-For": "{{ $client_ip }}"
```

## 🚨 Error Handling

### Status Verification
```phlow
steps:
  - name: "api_call"
    use: "http_client"
    input:
      method: "GET"
      url: "https://api.example.com/data"
      
  - name: "handle_response"
    assert: "{{ $api_call.is_success == true }}"
    then:
      # Process success data
      script: "Success: {{ $api_call.response.body }}"
    else:
      # Handle error
      script: "Error {{ $api_call.response.status_code }}: {{ $api_call.message }}"
```

### Different Error Types
```phlow
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
```phlow
modules:
  - name: "fast_api"
    module: "http_request"
    with:
      timeout: 5  # 5 seconds for fast APIs
      
  - name: "slow_api"
    module: "http_request"
    with:
      timeout: 120  # 2 minutes for long processing
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

**Version**: 0.0.1  
**Author**: Philippe Assis `<codephilippe@gmail.com>`
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
