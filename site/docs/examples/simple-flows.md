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

```phlow
steps:
  - payload: "Hello, World!"
  - payload: !phs `Result: ${ payload }`
```

This flow will output: `Result: Hello, World!`

## With Metadata

You can add metadata to better describe your flow:

```phlow
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

```phlow
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

```phlow
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
phlow simple-flow.phlow

# With debug logging
PHLOW_LOG=debug phlow simple-flow.phlow
```

## Modular Flows with Include and Arguments

You can make your flows more modular by using `!include` with arguments to create reusable components:

**main.phlow:**
```phlow
name: Modular Processing
version: 1.0.0
description: Example of using !include with arguments
steps:
  !include ./process.phlow input_data='{"user": "john", "score": 85}' threshold=80
```

**process.phlow:**
```phlow
- payload: !phs JSON.parse('!arg input_data')
- assert: !phs payload.score >= !arg threshold
  then:
    payload:
      result: "passed"
      user: !phs payload.user
      score: !phs payload.score
      message: "User !arg input_data passed with score above !arg threshold"
  else:
    payload:
      result: "failed"
      user: !phs payload.user
      score: !phs payload.score
      message: "User failed to meet the threshold of !arg threshold"
```

This approach enables:
- **Reusability**: The same processing logic can be used with different parameters
- **Maintainability**: Changes to logic only need to be made in one place
- **Flexibility**: Different thresholds and input data can be easily provided
- **Organization**: Complex flows can be broken into smaller, focused files

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
