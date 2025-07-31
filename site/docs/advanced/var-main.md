---
sidebar_position: 1
title: Main Variable Simulation (--var-main)
---

# Main Variable Simulation

The `--var-main` parameter allows you to simulate the output of a main module (like CLI) without actually executing it. This is particularly useful for testing, debugging, or running flows that depend on data from interactive modules.

## Usage

```bash
phlow --var-main VALUE [file]
```

The `VALUE` must be valid JSON and will be parsed using the same mechanism as other values in Phlow.

## How it Works

When `--var-main` is specified:

1. **Main modules are prevented from executing** - Modules marked as main (like CLI) won't run
2. **The provided value becomes available as the `main` variable** - Your flow steps can access it via `!phs main`
3. **Flow execution starts immediately** - No waiting for user input or module initialization

## Examples

### Simple Values

```bash
# String value
phlow --var-main '"hello world"'

# Number value  
phlow --var-main '42'

# Boolean value
phlow --var-main 'true'

# Array value
phlow --var-main '[1, 2, 3]'
```

### Object Values

```bash
# Simple object
phlow --var-main '{"user_id": 123, "name": "John"}'

# Complex nested object
phlow --var-main '{
  "user": {
    "id": 123,
    "name": "John Doe", 
    "email": "john@example.com"
  },
  "settings": {
    "theme": "dark",
    "notifications": true
  }
}'
```

## Practical Example

Consider a flow that expects CLI input:

```yaml
# user-processor.phflow
name: user-processor
version: 1.0.0
description: Process user data

modules:
  - module: cli
    with:
      args:
        - name: user_id
          type: number
          required: true
        - name: action
          type: string
          required: true

steps:
  - step: process-user
    run: !phs |
      console.log("Processing user:", main.user_id);
      console.log("Action:", main.action);
      return {
        user_id: main.user_id,
        action: main.action,
        processed_at: new Date().toISOString()
      };

  - return: !phs payload
```

Instead of running it interactively:

```bash
# This would require user input
phlow user-processor.phflow
```

You can simulate the CLI input:

```bash
# Test with simulated data
phlow user-processor.phflow --var-main '{"user_id": 42, "action": "update"}'
```

Output:
```
Processing user: 42
Action: update
{"user_id":42,"action":"update","processed_at":"2024-01-15T10:30:00.000Z"}
```

## Testing Scenarios

The `--var-main` parameter is especially useful for testing different scenarios:

```bash
# Test with valid user
phlow user-processor.phflow --var-main '{"user_id": 1, "action": "create"}'

# Test with different action
phlow user-processor.phflow --var-main '{"user_id": 1, "action": "delete"}'

# Test with edge case
phlow user-processor.phflow --var-main '{"user_id": 999999, "action": "archive"}'
```

## Integration with Testing

You can combine `--var-main` with the `--test` parameter for comprehensive testing:

```yaml
# user-processor.phlow
tests:
  - description: "Should process valid user data"
    main:
      user_id: 123
      action: "update"
    expect: !phs |
      return payload.user_id === 123 && 
             payload.action === "update" &&
             payload.processed_at !== undefined;

  - description: "Should handle delete action"
    main:
      user_id: 456  
      action: "delete"
    expect: !phs payload.action === "delete"
```

Run tests normally:

```bash
phlow user-processor.phlow --test
```

Or test individual scenarios:

```bash
# Test specific scenario manually
phlow user-processor.phlow --var-main '{"user_id": 789, "action": "create"}'
```

## Error Handling

If the JSON value is invalid, Phlow will show an error:

```bash
phlow --var-main '{"invalid": json}'
# Error: Failed to parse --var-main value: expected `,` or `}` at line 1 column 18
```

Make sure to properly escape quotes in your shell:

```bash
# Good - properly quoted
phlow --var-main '{"name": "John Doe"}'

# Good - using double quotes with escaping
phlow --var-main "{\"name\": \"John Doe\"}"
```

## Use Cases

- **Development**: Test flows without going through CLI prompts
- **CI/CD**: Automate testing with predetermined input values  
- **Debugging**: Isolate flow logic from input collection
- **Integration Testing**: Simulate various input scenarios
- **Documentation**: Provide reproducible examples

The `--var-main` parameter makes Phlow flows more testable and automation-friendly while maintaining the flexibility of interactive execution when needed.
