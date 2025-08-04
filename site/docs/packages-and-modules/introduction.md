---
sidebar_position: 1
title: Introduction
hide_title: true
---
---
#  Packages and Modules

Phlow has a powerful package management system that supports two types of modules:

1. **Official Modules**: Pre-compiled Rust modules from the official repository
2. **Phlow Modules**: Custom modules written in pure Phlow syntax (`.phlow` files)

This makes it easier to reuse code, integrate with external libraries, and create custom business logic without requiring Rust knowledge.

## Module Types

### Official Modules (Rust-based)
These are pre-compiled, high-performance modules available from the official repository:
- **http_server**: HTTP server functionality
- **postgres**: Database operations
- **amqp**: Message queue integration
- **jwt**: Authentication tokens
- **log**: Structured logging
- And many more...

### Phlow Modules (`.phlow` files)
**New Feature**: Custom modules written entirely in Phlow syntax:
- **No compilation required**: Pure Phlow declarative syntax
- **Rapid development**: Changes take effect immediately  
- **Schema validation**: Automatic input/output validation
- **Full integration**: Access to PHS, includes, and all Phlow features

## Automatic Module Download

Phlow automatically downloads official modules specified in your flow configuration.

The official module repository is [phlow-packages](https://github.com/phlowdotdev/phlow-packages), which contains all official Phlow modules precompiled for Linux.

When you run Phlow, it will automatically fetch and install the required modules into a local `phlow-packages/` folder at the root of your project execution.
You don't need to worry about building or installing them manually — just describe the modules in your Phlow files, and Phlow takes care of the rest.

Phlow has a powerful package management system that allows you to import and use third-party modules in your workflows. This makes it easier to reuse code and integrate with external libraries.

## Automatic Module Download

Phlow automatically downloads the modules specified in your flow configuration.

The official module repository is [phlow-packages](https://github.com/phlowdotdev/phlow-packages), which contains all official Phlow modules precompiled for Linux.

When you run Phlow, it will automatically fetch and install the required modules into a local `phlow-packages/` folder at the root of your project execution.

You don’t need to worry about building or installing them manually — just describe the modules in your YAML, and Phlow takes care of the rest.

## Using modules

To use modules in your flow, declare them under the `modules` section and reference them in your `steps`.

### Example: Using Official Modules

Here's a minimal working example using the official `log` module:

```phlow
main: log_example
modules:
  - module: log
    version: latest
steps:
  - use: log
    input:
      level: info
      message: "📥 Starting process..."
  - use: log
    input:
      level: debug
      message: !phs "'Current time: ' + timestamp()"
  - use: log
    input:
      level: error
      message: "❌ Something went wrong"
```

### Example: Using Phlow Modules

Here's an example using a custom Phlow module for HTTP routing:

```phlow
main: http_server
modules:
  - module: http_server
  - module: ./route          # Custom Phlow module
    name: users_route
    with:
      path: "/users"
      method: "GET"
      default_response:
        status_code: 200
        body: { users: [] }

steps:
  - use: users_route
    input: !phs main
  - assert: !phs payload.matched
    then:
      return: !phs payload.response
    else:
      return:
        status_code: 404
        body: { error: "Not Found" }
```

### Example: Combining Both Types

You can seamlessly combine official and custom modules:

```phlow
main: http_server
modules:
  # Official modules
  - module: log
  - module: http_server
  - module: postgres
    with:
      host: localhost
      user: postgres
      password: !phs envs.DB_PASSWORD
  
  # Custom Phlow modules
  - module: ./auth
    name: jwt_auth
    with:
      secret: !phs envs.JWT_SECRET
  - module: ./validator
    name: user_validator
    with:
      required_fields: ["name", "email"]

steps:
  - use: log
    input:
      message: "🚀 API server starting"
  
  - use: jwt_auth
    input: !phs main.headers
  
  - use: user_validator
    input: !phs main.body
  
  - use: postgres
    input:
      query: "INSERT INTO users (name, email) VALUES ($1, $2)"
      params: !phs [main.body.name, main.body.email]
```