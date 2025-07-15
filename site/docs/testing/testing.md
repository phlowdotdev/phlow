---
sidebar_position: 8
title: Testing
---

# Testing in Phlow

Phlow provides a built-in testing framework that allows you to write and run tests directly in your `.phlow` files. This documentation covers how to write, run, and understand tests in Phlow.

:::tip
For practical examples with real test outputs, see the [Basic Testing Examples](./basic-examples.md) section.
:::

## Running Tests

To run tests in a Phlow file, use the `--test` or `-t` flag:

```bash
phlow --test main.phlow
```

When you run tests, Phlow will:
1. Load the Phlow file
2. Download any required modules
3. Execute each test case
4. Display the results with a summary

## Test Structure

Tests are defined in the `tests` section of your Phlow file. Each test case consists of:

- **`main`**: Input values for the main context
- **`payload`**: Initial payload value (optional)
- **`assert`**: Expression-based assertion using PHS
- **`assert_eq`**: Direct value comparison assertion

### Basic Test Example

```yaml
name: Basic Math Test
version: 1.0.0
description: Testing basic arithmetic operations

tests:
  - main:
      x: 10
      y: 20
    payload: 5
    assert: !phs payload == 35
  - main:
      x: 0
      y: 0
    payload: 100
    assert: !phs payload == 100
  - main:
      x: -5
      y: 5
    payload: 10
    assert: !phs payload > 0

steps:
  - payload: !phs main.x + main.y + payload
```

## Test Output

When you run the above test, you'll see output like this:

```
[2025-07-15T00:07:55Z INFO  phlow::loader] Downloading modules...
[2025-07-15T00:07:55Z INFO  phlow::loader] All modules downloaded and extracted successfully
🧪 Running 3 test(s)...

Test 1: ✅ PASSED - Assertion passed: {{ payload == 35 }}
Test 2: ✅ PASSED - Assertion passed: {{ payload == 100 }}
Test 3: ✅ PASSED - Assertion passed: {{ payload > 0 }}

📊 Test Results:
   Total: 3
   Passed: 3 ✅
   Failed: 0 ❌

🎉 All tests passed!
```

## Assertion Types

### Expression Assertions (`assert`)

Expression assertions use PHS (Phlow Scripting) to evaluate conditions:

```yaml
tests:
  - main:
      name: "John"
      age: 25
    payload: "active"
    assert: !phs payload == "active"
  - main:
      count: 10
    payload: 5
    assert: !phs payload < main.count
  - main:
      items: [1, 2, 3]
    payload: 3
    assert: !phs payload == main.items.length
```

### Direct Value Assertions (`assert_eq`)

Direct value assertions compare the final payload with an expected value:

```yaml
tests:
  - main:
      multiplier: 2
    payload: 10
    assert_eq: "Total is 20"
  - main:
      name: "Alice"
    payload: "Hello"
    assert_eq: "Hello Alice"

steps:
  - payload: !phs main.multiplier * payload
  - payload: !phs `Total is ${payload}`
```

## Testing with Modules

You can test workflows that use modules:

```yaml
name: HTTP Request Test
version: 1.0.0
description: Testing HTTP requests

modules:
  - module: http_request
    version: latest

tests:
  - main:
      url: "https://httpbin.org/json"
    payload: null
    assert: !phs payload.slideshow != null

steps:
  - http_request:
      url: !phs main.url
      method: GET
```

## Complex Test Scenarios

### Testing Conditional Logic

```yaml
name: Age Verification Test
version: 1.0.0
description: Testing age verification logic

tests:
  - main:
      age: 25
    payload: null
    assert: !phs payload == "Adult"
  - main:
      age: 16
    payload: null
    assert: !phs payload == "Minor"
  - main:
      age: 18
    payload: null
    assert: !phs payload == "Adult"

steps:
  - assert: !phs main.age >= 18
    then:
      payload: "Adult"
    else:
      payload: "Minor"
```

### Testing Data Transformation

```yaml
name: Data Processing Test
version: 1.0.0
description: Testing data transformation

tests:
  - main:
      users: [
        {"name": "John", "age": 30},
        {"name": "Jane", "age": 25}
      ]
    payload: null
    assert: !phs payload.length == 2
  - main:
      users: [
        {"name": "Bob", "age": 35}
      ]
    payload: null
    assert: !phs payload[0].status == "processed"

steps:
  - payload: !phs main.users.map(user => ({ ...user, status: "processed" }))
```

## Test Failures

When tests fail, you'll see detailed error messages:

```
🧪 Running 2 test(s)...

Test 1: ❌ FAILED - Expected Total is 20, got Total is 30
Test 2: ✅ PASSED - Assertion passed: {{ payload == "Total is 15" }}

📊 Test Results:
   Total: 2
   Passed: 1 ✅
   Failed: 1 ❌

❌ Some tests failed!
```

## Test Best Practices

### 1. Use Descriptive Test Names

```yaml
name: User Registration Validation
description: Tests for user registration validation rules

tests:
  - main:
      email: "user@example.com"
      password: "secure123"
    payload: null
    assert: !phs payload.valid == true
```

### 2. Test Edge Cases

```yaml
tests:
  - main:
      value: 0
    payload: null
    assert: !phs payload == "zero"
  - main:
      value: -1
    payload: null
    assert: !phs payload == "negative"
  - main:
      value: null
    payload: null
    assert: !phs payload == "null"
```

### 3. Test Error Conditions

```yaml
tests:
  - main:
      input: ""
    payload: null
    assert: !phs payload.error == "Input cannot be empty"
  - main:
      input: "invalid"
    payload: null
    assert: !phs payload.error != null
```

## Advanced Testing Features

### Testing Asynchronous Operations

```yaml
name: Async Operation Test
version: 1.0.0
description: Testing asynchronous operations

modules:
  - module: http_request
    version: latest
  - module: sleep
    version: latest

tests:
  - main:
      delay: 1
    payload: "start"
    assert: !phs payload == "completed"

steps:
  - sleep:
      seconds: !phs main.delay
  - payload: "completed"
```

### Testing Multiple Scenarios

```yaml
name: Calculator Test Suite
version: 1.0.0
description: Comprehensive calculator tests

tests:
  # Addition tests
  - main: { operation: "add", a: 5, b: 3 }
    payload: null
    assert: !phs payload == 8
  - main: { operation: "add", a: -5, b: 3 }
    payload: null
    assert: !phs payload == -2
  
  # Subtraction tests
  - main: { operation: "subtract", a: 10, b: 4 }
    payload: null
    assert: !phs payload == 6
  - main: { operation: "subtract", a: 0, b: 5 }
    payload: null
    assert: !phs payload == -5
  
  # Multiplication tests
  - main: { operation: "multiply", a: 6, b: 7 }
    payload: null
    assert: !phs payload == 42
  - main: { operation: "multiply", a: -3, b: 4 }
    payload: null
    assert: !phs payload == -12

steps:
  - assert: !phs main.operation == "add"
    then:
      payload: !phs main.a + main.b
  - assert: !phs main.operation == "subtract"
    then:
      payload: !phs main.a - main.b
  - assert: !phs main.operation == "multiply"
    then:
      payload: !phs main.a * main.b
```

## Debugging Failed Tests

When tests fail, you can debug them by:

1. **Running the flow normally** to see the actual output:
   ```bash
   phlow main.phlow
   ```

2. **Adding debug information** to your steps:
   ```yaml
   steps:
     - log: !phs `Debug: main = ${JSON.stringify(main)}`
     - log: !phs `Debug: payload = ${JSON.stringify(payload)}`
     - payload: !phs main.x + main.y + payload
   ```

3. **Using the `--show-steps` flag** to see step execution:
   ```bash
   phlow --show-steps --test main.phlow
   ```

## Testing CLI Applications

For CLI applications, you can test argument processing:

```yaml
name: CLI Application Test
version: 1.0.0
description: Testing CLI argument processing

main: cli
modules:
  - module: cli
    with:
      args:
        - name: name
          description: User name
          index: 1
          type: string
          required: true
        - name: age
          description: User age
          index: 2
          type: number
          required: true

tests:
  - main:
      name: "John"
      age: 25
    payload: null
    assert: !phs payload == "John (25 years old)"
  - main:
      name: "Jane"
      age: 30
    payload: null
    assert: !phs payload == "Jane (30 years old)"

steps:
  - payload: !phs `${main.name} (${main.age} years old)`
```

## Integration with CI/CD

You can integrate Phlow tests into your CI/CD pipeline:

```bash
#!/bin/bash
# test-runner.sh

# Run all tests in the project
for test_file in tests/*.phlow; do
    echo "Running tests in $test_file"
    if ! phlow --test "$test_file"; then
        echo "Tests failed in $test_file"
        exit 1
    fi
done

echo "All tests passed!"
```

## Common Testing Patterns

### Setup and Teardown

```yaml
name: Database Test
version: 1.0.0
description: Testing database operations

modules:
  - module: postgres
    version: latest

tests:
  - main:
      table: "users"
      data: {"name": "John", "email": "john@example.com"}
    payload: null
    assert: !phs payload.success == true

steps:
  # Setup
  - postgres:
      query: "CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name VARCHAR(100), email VARCHAR(100))"
  
  # Test operation
  - postgres:
      query: !phs `INSERT INTO users (name, email) VALUES ('${main.data.name}', '${main.data.email}')`
  
  # Verify result
  - postgres:
      query: !phs `SELECT * FROM users WHERE email = '${main.data.email}'`
  - payload: !phs { success: payload.length > 0 }
  
  # Teardown
  - postgres:
      query: "DROP TABLE IF EXISTS users"
```

This comprehensive testing framework makes it easy to ensure your Phlow applications work correctly and reliably across different scenarios and inputs.
