---
sidebar_position: 2
title: HTTP API Examples
---

# HTTP API Examples

This section demonstrates how to create HTTP APIs using Phlow's `http_server` module and make HTTP requests using the `http_request` module.

## Simple HTTP Server

Create a basic HTTP server that responds to requests:

```yaml title="simple-server.phlow"
name: Simple HTTP Server
version: 1.0.0
description: A basic HTTP server example

main: http_server
modules:
  - module: http_server
    version: latest
    with:
      port: 8080
      host: "0.0.0.0"

steps:
  - assert: !phs main.path == "/"
    then:
      return:
        status_code: 200
        body:
          message: "Welcome to Phlow API"
          version: "1.0.0"
        headers:
          Content-Type: application/json
  
  - assert: !phs main.path == "/health"
    then:
      return:
        status_code: 200
        body:
          status: "healthy"
          timestamp: !phs new Date().toISOString()
        headers:
          Content-Type: application/json
  
  - assert: !phs main.path == "/api/users" && main.method == "GET"
    then:
      return:
        status_code: 200
        body:
          users:
            - id: 1
              name: "John Doe"
              email: "john@example.com"
            - id: 2
              name: "Jane Smith"
              email: "jane@example.com"
        headers:
          Content-Type: application/json
  
  - assert: !phs main.path == "/api/users" && main.method == "POST"
    then:
      return:
        status_code: 201
        body:
          message: "User created successfully"
          user: !phs JSON.parse(main.body)
          id: !phs Math.floor(Math.random() * 1000)
        headers:
          Content-Type: application/json
  
  # Default response for unknown routes
  - return:
      status_code: 404
      body:
        error: "Route not found"
        path: !phs main.path
        method: !phs main.method
      headers:
        Content-Type: application/json
```

### Running the Server

```bash
phlow simple-server.phlow
```

### Testing the API

```bash
# Test the root endpoint
curl http://localhost:8080/

# Test the health endpoint
curl http://localhost:8080/health

# Test GET users
curl http://localhost:8080/api/users

# Test POST users
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice Johnson", "email": "alice@example.com"}'
```

## HTTP Client Example

Make HTTP requests to external APIs:

```yaml title="weather-client.phlow"
name: Weather Client
version: 1.0.0
description: Fetch weather information from an external API

modules:
  - module: http_request
    version: latest
  - module: log
    version: latest

steps:
  - log:
      message: "Fetching weather data..."
  
  - http_request:
      url: "https://httpbin.org/json"
      method: GET
      headers:
        User-Agent: "Phlow Weather Client 1.0"
  
  - log:
      message: !phs `Response received: ${JSON.stringify(payload, null, 2)}`
  
  - payload: !phs {
      slideshow: payload.slideshow,
      processed_at: new Date().toISOString(),
      status: "success"
    }
```

## REST API with CRUD Operations

A more complete REST API example:

```yaml title="crud-api.phlow"
name: CRUD API Example
version: 1.0.0
description: Complete CRUD API with in-memory storage

main: http_server
modules:
  - module: http_server
    version: latest
    with:
      port: 3000
      host: "0.0.0.0"

steps:
  # Initialize in-memory storage
  - payload: !phs {
      items: [
        { id: 1, name: "Item 1", description: "First item" },
        { id: 2, name: "Item 2", description: "Second item" }
      ]
    }
  
  # GET /api/items - List all items
  - assert: !phs main.path == "/api/items" && main.method == "GET"
    then:
      return:
        status_code: 200
        body:
          items: !phs payload.items
          total: !phs payload.items.length
        headers:
          Content-Type: application/json
  
  # POST /api/items - Create new item
  - assert: !phs main.path == "/api/items" && main.method == "POST"
    then:
      - payload: !phs {
          ...payload,
          newItem: {
            id: Math.max(...payload.items.map(item => item.id)) + 1,
            ...JSON.parse(main.body),
            created_at: new Date().toISOString()
          }
        }
      - payload: !phs {
          ...payload,
          items: [...payload.items, payload.newItem]
        }
      - return:
          status_code: 201
          body:
            message: "Item created successfully"
            item: !phs payload.newItem
          headers:
            Content-Type: application/json
  
  # GET /api/items/{id} - Get specific item
  - assert: !phs main.path.startsWith("/api/items/") && main.method == "GET" && main.path.split("/").length == 4
    then:
      - payload: !phs {
          ...payload,
          requestedId: parseInt(main.path.split("/")[3])
        }
      - payload: !phs {
          ...payload,
          foundItem: payload.items.find(item => item.id === payload.requestedId)
        }
      - assert: !phs payload.foundItem
        then:
          return:
            status_code: 200
            body: !phs payload.foundItem
            headers:
              Content-Type: application/json
        else:
          return:
            status_code: 404
            body:
              error: "Item not found"
            headers:
              Content-Type: application/json
  
  # PUT /api/items/{id} - Update item
  - assert: !phs main.path.startsWith("/api/items/") && main.method == "PUT" && main.path.split("/").length == 4
    then:
      - payload: !phs {
          ...payload,
          requestedId: parseInt(main.path.split("/")[3])
        }
      - payload: !phs {
          ...payload,
          items: payload.items.map(item => 
            item.id === payload.requestedId 
              ? { ...item, ...JSON.parse(main.body), updated_at: new Date().toISOString() }
              : item
          )
        }
      - return:
          status_code: 200
          body:
            message: "Item updated successfully"
            item: !phs payload.items.find(item => item.id === payload.requestedId)
          headers:
            Content-Type: application/json
  
  # DELETE /api/items/{id} - Delete item
  - assert: !phs main.path.startsWith("/api/items/") && main.method == "DELETE" && main.path.split("/").length == 4
    then:
      - payload: !phs {
          ...payload,
          requestedId: parseInt(main.path.split("/")[3])
        }
      - payload: !phs {
          ...payload,
          items: payload.items.filter(item => item.id !== payload.requestedId)
        }
      - return:
          status_code: 200
          body:
            message: "Item deleted successfully"
            deletedId: !phs payload.requestedId
          headers:
            Content-Type: application/json
  
  # Default response for unknown routes
  - return:
      status_code: 404
      body:
        error: "Route not found"
        path: !phs main.path
        method: !phs main.method
      headers:
        Content-Type: application/json
```

## API Proxy Example

Create a proxy that forwards requests to another API:

```yaml title="api-proxy.phlow"
name: API Proxy
version: 1.0.0
description: Proxy requests to external APIs

main: http_server
modules:
  - module: http_server
    version: latest
    with:
      port: 8081
      host: "0.0.0.0"
  - module: http_request
    version: latest
  - module: log
    version: latest

steps:
  - log:
      message: !phs `Proxying ${main.method} request to: ${main.path}`
  
  # Only proxy requests that start with /proxy
  - assert: !phs main.path.startsWith('/proxy')
    then:
      - payload: !phs {
          targetUrl: `https://httpbin.org${main.path.replace('/proxy', '')}`,
          method: main.method,
          headers: main.headers || {},
          body: main.body
        }
      
      - http_request:
          url: !phs payload.targetUrl
          method: !phs payload.method
          headers: !phs payload.headers
          body: !phs payload.body
      
      - log:
          message: !phs `Proxy response received`
      
      - return:
          status_code: 200
          body: !phs payload
          headers:
            Content-Type: application/json
    else:
      - return:
          status_code: 404
          body:
            error: "Proxy endpoint not found"
            message: "Use /proxy/{path} to proxy requests"
          headers:
            Content-Type: application/json
```

## Testing HTTP APIs

Create tests for your HTTP APIs:

```yaml title="api-test.phlow"
name: API Test Suite
version: 1.0.0
description: Testing HTTP API endpoints

modules:
  - module: http_request
    version: latest

tests:
  - main:
      base_url: "http://localhost:8080"
    payload: null
    assert: !phs payload.message == "Welcome to Phlow API"
  
  - main:
      base_url: "http://localhost:8080"
    payload: null
    assert: !phs payload.status == "healthy"
  
  - main:
      base_url: "http://localhost:8080"
    payload: null
    assert: !phs payload.users.length >= 2

steps:
  - assert: !phs main.base_url.includes("localhost:8080")
    then:
      # Test root endpoint
      - http_request:
          url: !phs `${main.base_url}/`
          method: GET
      - assert: !phs payload.message
        then:
          return: !phs payload
      
      # Test health endpoint
      - http_request:
          url: !phs `${main.base_url}/health`
          method: GET
      - assert: !phs payload.status
        then:
          return: !phs payload
      
      # Test users endpoint
      - http_request:
          url: !phs `${main.base_url}/api/users`
          method: GET
      - return: !phs payload
```

## Key Features Demonstrated

1. **HTTP Server Setup**: Configure host and port for your server
2. **Manual Routing**: Process different HTTP methods and paths using assertions
3. **Response Formatting**: Return proper HTTP responses with status codes, headers, and body
4. **HTTP Client**: Make requests to external APIs
5. **CRUD Operations**: Complete Create, Read, Update, Delete functionality
6. **API Proxy**: Forward requests to external services
7. **Testing**: Automated testing of API endpoints
8. **Error Handling**: Proper error responses and status codes
9. **Path Parameters**: Extract parameters from URL paths
10. **Request Body Processing**: Parse JSON request bodies

## Usage Examples

### Testing the CRUD API

```bash
# List all items
curl http://localhost:3000/api/items

# Create a new item
curl -X POST http://localhost:3000/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "New Item", "description": "A new item"}'

# Get item by ID
curl http://localhost:3000/api/items/1

# Update item
curl -X PUT http://localhost:3000/api/items/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Item", "description": "Updated description"}'

# Delete item
curl -X DELETE http://localhost:3000/api/items/1
```

### Testing the Proxy

```bash
# Proxy a GET request
curl http://localhost:8081/proxy/get

# Proxy a POST request
curl -X POST http://localhost:8081/proxy/post \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

These examples show how to build production-ready HTTP APIs using Phlow's http_server module with manual routing and proper response handling.
