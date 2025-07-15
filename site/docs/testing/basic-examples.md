---
sidebar_position: 1
title: Basic Testing Examples
---

# Basic Testing Examples

This section provides practical examples of testing in Phlow, showing different scenarios and their expected outputs.

## String Concatenation Test

Here's a simple example testing string concatenation:

```yaml title="string-test.phlow"
name: String Test
version: 1.0.0
description: Testing string operations

tests:
  - main:
      name: "John"
    payload: "Hello"
    assert_eq: "Hello John"
  - main:
      name: "Alice"
    payload: "Hi"
    assert_eq: "Hi Alice"
  - main:
      name: "Bob"
    payload: "Hey"
    assert_eq: "Hey Bob"

steps:
  - payload: !phs `${payload} ${main.name}`
```

### Running the Test

```bash
phlow --test string-test.phlow
```

### Expected Output

```
[2025-07-15T00:11:48Z INFO  phlow::loader] Downloading modules...
[2025-07-15T00:11:48Z INFO  phlow::loader] All modules downloaded and extracted successfully
🧪 Running 3 test(s)...

Test 1: ✅ PASSED - Expected and got: Hello John
Test 2: ✅ PASSED - Expected and got: Hi Alice
Test 3: ✅ PASSED - Expected and got: Hey Bob

📊 Test Results:
   Total: 3
   Passed: 3 ✅
   Failed: 0 ❌

🎉 All tests passed!
```

## Basic Math Test

Here's an example testing basic arithmetic operations:

```yaml title="math-test.phlow"
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

### Running the Test

```bash
phlow --test math-test.phlow
```

### Expected Output

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

## Test with Failures

Here's an example showing what happens when tests fail:

```yaml title="fail-test.phlow"
name: Failing Test Example
version: 1.0.0
description: Example showing test failures

tests:
  - main:
      name: "John"
    payload: "Hello"
    assert_eq: "Hello John"
  - main:
      name: "Alice"
    payload: "Hi"
    assert_eq: "Wrong expectation"
  - main:
      name: "Bob"
    payload: "Hey"
    assert: !phs payload == "Something else"

steps:
  - payload: !phs `${payload} ${main.name}`
```

### Running the Test

```bash
phlow --test fail-test.phlow
```

### Expected Output

```
[2025-07-15T00:11:59Z INFO  phlow::loader] Downloading modules...
[2025-07-15T00:11:59Z INFO  phlow::loader] All modules downloaded and extracted successfully
🧪 Running 3 test(s)...

Test 1: ✅ PASSED - Expected and got: Hello John
Test 2: ❌ FAILED - Expected Wrong expectation, got Hi Alice
Test 3: ❌ FAILED - Assertion failed: {{ payload == "Something else" }}

📊 Test Results:
   Total: 3
   Passed: 1 ✅
   Failed: 2 ❌

❌ Some tests failed!
```

## Different Assertion Types

### Using `assert` (Expression-based)

```yaml
tests:
  - main:
      value: 42
    payload: 100
    assert: !phs payload > main.value
  - main:
      name: "test"
    payload: "test string"
    assert: !phs payload.includes(main.name)
  - main:
      items: [1, 2, 3]
    payload: 3
    assert: !phs payload == main.items.length
```

### Using `assert_eq` (Direct comparison)

```yaml
tests:
  - main:
      multiplier: 2
    payload: 10
    assert_eq: 20
  - main:
      prefix: "Hello"
    payload: "World"
    assert_eq: "Hello World"

steps:
  - payload: !phs main.multiplier * payload
  - payload: !phs `${main.prefix} ${payload}`
```

## Testing Conditional Logic

```yaml title="conditional-test.phlow"
name: Conditional Logic Test
version: 1.0.0
description: Testing conditional logic

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

This example tests different age values and ensures the conditional logic works correctly for determining adult vs minor status.

## Testing with Initial Payload

```yaml title="payload-test.phlow"
name: Payload Test
version: 1.0.0
description: Testing with initial payload values

tests:
  - main:
      factor: 2
    payload: 10
    assert: !phs payload == 20
  - main:
      factor: 0
    payload: 100
    assert: !phs payload == 0
  - main:
      factor: -1
    payload: 5
    assert: !phs payload == -5

steps:
  - payload: !phs main.factor * payload
```

This example shows how to test workflows that start with an initial payload value and transform it through the steps.

## Key Points

1. **`assert_eq`** is better for exact value comparisons
2. **`assert`** is better for complex conditions and expressions
3. **Test names** should be descriptive of what they're testing
4. **Error messages** clearly show what was expected vs what was received
5. **Exit codes** indicate success (0) or failure (non-zero) for CI/CD integration
