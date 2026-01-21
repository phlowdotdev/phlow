---
sidebar_position: 2
title: Initial Payload (--var-payload)
---

# Initial Payload

The `--var-payload` parameter sets the initial `payload` value before the first step runs. This is useful when you want to start a flow with a known payload without adding a setup step.

## Usage

```bash
phlow --var-payload VALUE [file]
```

The `VALUE` must be valid JSON and will be parsed using the same mechanism as other values in Phlow.

## How it Works

When `--var-payload` is specified:

1. **The value becomes available as `payload` in the first step**
2. **Main modules still run normally** (unless `--var-main` is also used)
3. **Subsequent steps overwrite `payload` as usual**

## Examples

### Simple Values

```bash
# String value
phlow flow.phlow --var-payload '"hello world"'

# Number value
phlow flow.phlow --var-payload '42'

# Object value
phlow flow.phlow --var-payload '{"user_id": 123, "name": "John"}'
```

### Flow Example

```phlow
steps:
  - payload: !phs payload ?? 0
  - payload: !phs payload + 5
  - return: !phs payload
```

```bash
phlow flow.phlow --var-payload 10
# Output: 15
```

### Combine with --var-main

```bash
phlow flow.phlow --var-main '{"total": 2}' --var-payload 10
```

## Error Handling

If the JSON value is invalid, Phlow will show an error:

```bash
phlow flow.phlow --var-payload '{"invalid": json}'
# Error: Failed to parse --var-payload value: expected `,` or `}` at line 1 column 18
```
