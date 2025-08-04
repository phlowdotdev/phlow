---
sidebar_position: 4
title: Creating Phlow Modules (.phlow)
---

# Creating Phlow Modules (.phlow)

**New Feature**: Phlow now supports creating modules using pure Phlow syntax. These modules are defined as `.phlow` files and provide a simple way to create reusable logic without requiring Rust knowledge or compilation.

## Overview

Phlow modules (`.phlow` files) are lightweight, declarative modules that can encapsulate reusable logic and be shared across different flows. They offer:

- **No compilation required**: Pure Phlow syntax
- **Schema validation**: Automatic input/output validation
- **Rapid development**: Changes take effect immediately
- **Full Phlow integration**: Access to PHS, includes, and all Phlow features

## Module Structure

A Phlow module consists of four main sections:

```phlow
# 1. Configuration Schema (with)
with:
  type: object
  properties:
    # Define what configuration parameters the module accepts

# 2. Input Schema (input) 
input:
  type: object
  properties:
    # Define the structure of runtime input data

# 3. Output Schema (output)
output:
  type: object
  properties:
    # Define what the module returns

# 4. Module Logic (steps)
steps:
  # Implementation of the module behavior
```

## Example: Route Module

Let's create a simple HTTP route matching module:

**File: `route.phlow`**

```phlow
# Configuration schema - parameters passed via 'with'
with:
  type: object
  required: true
  properties:
    path:
      type: string
      description: "HTTP path to match (e.g., /users)"
      required: true
    method:
      type: enum
      description: "HTTP method to match"
      enum: [GET, POST, DELETE, PUT, PATCH, OPTIONS]
      required: true
    default_response:
      type: object
      description: "Default response when route matches"
      required: false
      properties:
        status_code:
          type: number
          default: 200
        body:
          type: object
          default: {}
        headers:
          type: object
          default: {}

# Input schema - runtime data structure
input:
  type: object
  required: true
  properties:
    request:
      type: object
      required: true
      properties:
        path:
          type: string
          description: "Incoming request path"
          required: true
        method:
          type: string
          description: "Incoming request method"
          required: true
        headers:
          type: object
          description: "Request headers"
          required: false
        body:
          type: object
          description: "Request body"
          required: false

# Output schema - what the module returns
output:
  type: object
  required: true
  properties:
    matched:
      type: boolean
      description: "Whether the route matched"
      required: true
    response:
      type: object
      description: "HTTP response data"
      required: false
      properties:
        status_code:
          type: number
          required: true
        body:
          type: object
          required: false
        headers:
          type: object
          required: false

# Module implementation
steps:
  - assert: !phs setup.path == main.request.path && setup.method == main.request.method
    then:
      payload:
        matched: true
        response: !phs setup.default_response || { status_code: 200, body: {} }
    else:
      payload:
        matched: false
        response: null
```

## Using the Module

**File: `main.phlow`**

```phlow
modules:
  - module: log
  - module: http_server
  # Import our custom route module
  - module: ./route
    name: users_get_route
    with:
      path: "/users"
      method: "GET"
      default_response:
        status_code: 200
        body:
          users: []
  
  - module: ./route
    name: users_post_route
    with:
      path: "/users"
      method: "POST"
      default_response:
        status_code: 201
        body:
          id: 1
          name: "New User"

main: http_server

steps:
  # Use the route modules
  - use: users_get_route
    input: !phs main
  - assert: !phs payload.matched
    then:
      return: !phs payload.response
  
  - use: users_post_route
    input: !phs main
  - assert: !phs payload.matched
    then:
      return: !phs payload.response
  
  # Default 404 response
  - return:
      status_code: 404
      body:
        error: "Not Found"
```

## Module Variables

Inside a Phlow module, you have access to special variables:

### `setup`
Contains the configuration passed via the `with` section:

```phlow
# In main.phlow
- module: ./route
  name: my_route
  with:
    path: "/api/users"
    method: "GET"

# In route.phlow, 'setup' contains:
# {
#   "path": "/api/users", 
#   "method": "GET"
# }
```

### `main`
Contains the runtime input data passed to the module:

```phlow
# In main.phlow
- use: my_route
  input: 
    request:
      path: "/api/users"
      method: "GET"

# In route.phlow, 'main' contains:
# {
#   "request": {
#     "path": "/api/users",
#     "method": "GET"
#   }
# }
```

### `payload`
Standard Phlow payload variable for passing data between steps within the module.

## Advanced Example: Data Transformation Module

**File: `transform.phlow`**

```phlow
with:
  type: object
  required: true
  properties:
    operations:
      type: array
      description: "List of transformation operations"
      required: true
      items:
        type: object
        properties:
          type:
            type: enum
            enum: ["filter", "map", "sort"]
          field:
            type: string
          condition:
            type: string

input:
  type: object
  required: true
  properties:
    data:
      type: array
      description: "Array of objects to transform"
      required: true

output:
  type: object
  required: true
  properties:
    transformed_data:
      type: array
      required: true

steps:
  - payload: !phs main.data
  
  # Apply each transformation operation
  - assert: !phs setup.operations && setup.operations.length > 0
    then:
      steps:
        # This is a simplified example - in practice you'd iterate through operations
        - assert: !phs setup.operations[0].type == "filter"
          then:
            payload: !phs `
              payload.filter(item => 
                item[setup.operations[0].field] === setup.operations[0].condition
              )
            `
        
        - assert: !phs setup.operations[0].type == "map"
          then:
            payload: !phs `
              payload.map(item => ({
                ...item,
                [setup.operations[0].field]: item[setup.operations[0].field].toUpperCase()
              }))
            `
        
        - return:
            transformed_data: !phs payload
    else:
      return:
        transformed_data: !phs payload
```

## Best Practices

### 1. **Clear Schemas**
Always define comprehensive input/output schemas:

```phlow
input:
  type: object
  required: true
  properties:
    field_name:
      type: string
      description: "Clear description of what this field represents"
      required: true
      default: "default_value"
```

### 2. **Error Handling**
Include proper error handling in your modules:

```phlow
steps:
  - assert: !phs main && main.required_field
    then:
      # Normal processing
      payload: !phs process_data(main)
    else:
      # Error response
      return:
        error: "Missing required field: required_field"
        status: "error"
```

### 3. **Descriptive Configuration**
Use clear, descriptive configuration parameters:

```phlow
with:
  type: object
  properties:
    database_url:
      type: string
      description: "PostgreSQL connection string"
      required: true
    timeout_seconds:
      type: number
      description: "Query timeout in seconds"
      default: 30
      required: false
```

### 4. **Modular Design**
Keep modules focused on a single responsibility:

```phlow
# Good: Focused on authentication
auth.phlow

# Good: Focused on data validation  
validator.phlow

# Avoid: Too many responsibilities
auth_validator_logger_cache.phlow
```

## Comparison: Phlow vs Rust Modules

| Feature | Phlow Modules | Rust Modules |
|---------|---------------|--------------|
| **Development Speed** | ⚡ Instant | 🔨 Compilation required |
| **Learning Curve** | 📚 Phlow syntax only | 🦀 Rust knowledge needed |
| **Performance** | 🚀 Fast | ⚡ Maximum performance |
| **Debugging** | 🔍 Easy with Phlow tools | 🛠️ Rust debugging tools |
| **Portability** | 🌍 Platform independent | 🏗️ Platform specific |
| **System Access** | 📋 Limited to Phlow features | 🔧 Full system access |
| **Schema Validation** | ✅ Automatic | 🔨 Manual implementation |

## When to Use Phlow Modules

**Choose Phlow modules when:**
- ✅ Building business logic and data transformations
- ✅ Creating application-specific utilities  
- ✅ Rapid prototyping and iteration
- ✅ Team has limited Rust experience
- ✅ Need platform-independent modules
- ✅ Schema validation is important

**Choose Rust modules when:**
- ✅ Maximum performance is critical
- ✅ Need system-level integrations
- ✅ Complex algorithms and computations
- ✅ Existing Rust libraries to leverage
- ✅ Team has Rust expertise

## Module Distribution

Phlow modules can be distributed just like any other file:

### Version Control
```bash
# Include in your repository
git add route.phlow
git commit -m "Add custom route module"
```

### Package as Archive
```bash
# Create a module package
tar -czf my-modules.tar.gz *.phlow
```

### Share via URL
```phlow
# Reference remote Phlow modules
modules:
  - module: https://example.com/modules/auth.phlow
    name: auth_module
    with:
      secret: !phs envs.SECRET_KEY
```

Phlow modules represent a significant evolution in the platform, making it accessible to a broader range of developers while maintaining the performance and reliability that makes Phlow powerful.
