---
sidebar_position: 1
title: Simple Flows
---

# Simple Flows

Phlow supports simple flows that don't require a main module or complex module definitions. These flows are perfect for:

- Data processing and transformation
- Simple automation tasks
- Batch processing
- Testing and prototyping
- Quick scripts

## Basic Example

The simplest Phlow flow contains only `steps`:

```yaml
steps:
  - payload: "Hello, World!"
  - payload: !phs `Result: ${ payload }`
```

This flow will output: `Result: Hello, World!`

## With Metadata

You can add metadata to better describe your flow:

```yaml
name: Simple Greeting
version: 1.0.0
description: A simple greeting flow
author: Your Name
steps:
  - payload:
      message: "Hello, Phlow!"
      timestamp: !phs `new Date().toISOString()`
  - payload: !phs `${ payload.message } - Generated at ${ payload.timestamp }`
```

## Data Processing Example

Here's a more complex example that processes data:

```yaml
name: Data Processor
version: 1.0.0
description: Process and transform data
steps:
  - payload:
      users:
        - name: "John"
          age: 25
        - name: "Alice"
          age: 30
        - name: "Bob"
          age: 20
  - payload: !phs `
      payload.users
        .filter(user => user.age >= 25)
        .map(user => \`\${user.name} is \${user.age} years old\`)
        .join(', ')
    `
```

## Using Optional Modules

You can still use modules without specifying versions:

```yaml
name: Simple Logging
description: Process data with logging
modules:
  - module: log
    # version omitted - will use 'latest'
    with:
      level: info
steps:
  - payload: "Processing important data"
  - log:
      message: !phs `Starting process: ${ payload }`
  - payload: !phs `Processed: ${ payload }`
  - log:
      message: !phs `Completed process: ${ payload }`
```

## Running Simple Flows

All simple flows are executed the same way:

```bash
# Run the flow
phlow simple-flow.yaml

# With debug logging
PHLOW_LOG=debug phlow simple-flow.yaml
```

## Benefits of Simple Flows

- **Quick to write**: No need to define modules or main entry points
- **Easy to understand**: Clear, linear execution
- **Perfect for automation**: Ideal for batch processing and simple tasks
- **Backward compatible**: Works alongside complex flows
- **Flexible**: Can be expanded with modules when needed

## When to Use Simple Flows

Use simple flows when you need:
- Quick data transformations
- Simple automation without external dependencies
- Testing and prototyping
- Batch processing tasks
- Educational examples

For more complex scenarios with user input, web servers, or database connections, consider using flows with main modules and explicit module definitions.
