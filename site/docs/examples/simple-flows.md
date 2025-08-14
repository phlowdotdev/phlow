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

You can use modules with both the new recommended syntax and legacy syntax. Phlow automatically transforms the legacy syntax during processing:

### New Syntax (Recommended)

```phlow
name: Simple Logging
description: Process data with logging using new syntax
modules:
  - module: log
    # version omitted - will use 'latest'
    with:
      level: info
steps:
  - payload: "Processing important data"
  - use: log
    input:
      message: !phs `Starting process: ${payload}`
  - payload: !phs `Processed: ${payload}`
  - use: log
    input:
      message: !phs `Completed process: ${payload}`
```

### Legacy Syntax (Still Supported)

```phlow
name: Simple Logging
description: Process data with logging using legacy syntax
modules:
  - module: log
    with:
      level: info
steps:
  - payload: "Processing important data"
  - log:
      message: !phs `Starting process: ${payload}`
  - payload: !phs `Processed: ${payload}`
  - log:
      message: !phs `Completed process: ${payload}`
```

### Mixed Syntax

```phlow
name: Mixed Syntax Example
modules:
  - module: log
  - module: cache
steps:
  # New syntax
  - use: log
    input:
      message: "Starting with new syntax"
      
  # Legacy syntax (auto-transformed)
  - cache:
      action: set
      key: "status"
      value: "processing"
      
  # New syntax
  - use: log
    input:
      message: "Mixed syntax works perfectly"
```

## Code Blocks in Simple Flows

You can use multi-line code blocks with `!phs` for complex logic:

```phlow
name: Advanced Data Processing
description: Using code blocks for complex calculations
steps:
  - payload: !phs {
      let users = [
        { name: "John", age: 25, score: 85 },
        { name: "Alice", age: 30, score: 92 },
        { name: "Bob", age: 20, score: 78 }
      ];
      
      let processedUsers = users
        .filter(user => user.age >= 21)
        .map(user => {
          let grade = user.score >= 90 ? "A" : 
                      user.score >= 80 ? "B" : "C";
          return {
            name: user.name,
            age: user.age,
            score: user.score,
            grade: grade,
            status: "processed"
          };
        });
        
      {
        total: users.length,
        processed: processedUsers.length,
        users: processedUsers
      }
    }
    
  - payload: !phs {
      let result = payload;
      let summary = {
        totalUsers: result.total,
        processedUsers: result.processed,
        averageScore: result.users.reduce((sum, user) => sum + user.score, 0) / result.users.length,
        topPerformer: result.users.reduce((top, user) => 
          user.score > top.score ? user : top, result.users[0]
        )
      };
      
      summary
    }
```

## Complex Example with Code Blocks and Modules

```phlow
name: E-commerce Order Processing
description: Complex flow with code blocks and modules
modules:
  - module: log
  - module: cache

steps:
  - payload: !phs {
      // Simulate incoming order data
      let order = {
        id: "ORD-" + Math.random().toString(36).substr(2, 9),
        customer: {
          name: "John Doe",
          email: "john@example.com",
          tier: "premium"
        },
        items: [
          { id: "ITEM001", name: "Widget A", price: 29.99, quantity: 2 },
          { id: "ITEM002", name: "Widget B", price: 49.99, quantity: 1 }
        ],
        timestamp: new Date().toISOString()
      };
      
      order
    }
    
  - use: log
    input:
      level: "info"
      message: !phs {
        let order = payload;
        `Processing order ${order.id} for customer ${order.customer.name}`
      }
      
  - payload: !phs {
      let order = payload;
      let subtotal = order.items.reduce((sum, item) => 
        sum + (item.price * item.quantity), 0
      );
      
      let discount = order.customer.tier === "premium" ? 0.1 : 0.05;
      let discountAmount = subtotal * discount;
      let tax = (subtotal - discountAmount) * 0.08;
      let total = subtotal - discountAmount + tax;
      
      {
        ...order,
        pricing: {
          subtotal: Math.round(subtotal * 100) / 100,
          discount: Math.round(discountAmount * 100) / 100,
          tax: Math.round(tax * 100) / 100,
          total: Math.round(total * 100) / 100
        },
        status: "calculated"
      }
    }
    
  - use: cache
    input:
      action: set
      key: !phs `order_${payload.id}`
      value: !phs payload
      ttl: 3600
      
  - use: log
    input:
      level: "info" 
      message: !phs {
        let order = payload;
        let pricing = order.pricing;
        
        `Order ${order.id} calculated: Total $${pricing.total} (Discount: $${pricing.discount}, Tax: $${pricing.tax})`
      }
      
  - assert: !phs payload.pricing.total > 0
    then:
      - payload: !phs {
          let order = payload;
          {
            ...order,
            status: "approved",
            approvedAt: new Date().toISOString()
          }
        }
      - use: log
        input:
          level: "info"
          message: !phs `Order ${payload.id} approved successfully`
    else:
      - payload: !phs {
          let order = payload;
          {
            ...order,
            status: "rejected",
            rejectedAt: new Date().toISOString(),
            reason: "Invalid total amount"
          }
        }
      - use: log
        input:
          level: "error"
          message: !phs `Order ${payload.id} rejected: ${payload.reason}`
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

## New Features Summary

### Module Syntax Options

- **New Syntax**: `use` + `input` (recommended for new projects)
- **Legacy Syntax**: Direct module properties (still supported, auto-transformed)
- **Mixed Usage**: Both syntaxes can be used in the same flow

### Code Blocks with `!phs`

- **Multi-line support**: Use `{}` for complex code blocks
- **Automatic formatting**: Code is unified to single line during processing
- **Variable scope**: Access to `main`, `payload`, and other flow variables
- **Limitations**: No function declarations (use `!import` for that)

### When to Use What

**Use Code Blocks (`!phs {}`) for:**
- Complex calculations and data transformations
- Conditional logic with multiple branches
- Array/object manipulations
- Inline data processing

**Use `!import` for:**
- Reusable function definitions
- Complex algorithms
- Shared utility functions
- Logic that spans multiple files

**Use New Module Syntax for:**
- New projects and flows
- Better tooling support
- Consistent code style
- Future-proof implementations

**Use Legacy Module Syntax when:**
- Migrating existing flows gradually
- Working with legacy codebases
- Quick prototyping with familiar syntax

## When to Use Simple Flows

Use simple flows when you need:
- Quick data transformations
- Simple automation without external dependencies
- Testing and prototyping
- Batch processing tasks
- Educational examples

For more complex scenarios with user input, web servers, or database connections, consider using flows with main modules and explicit module definitions.
