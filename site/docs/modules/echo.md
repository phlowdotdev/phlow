---
sidebar_position: 3
title: Echo Module
hide_title: true
---

# Echo Module

The Echo module is a simple and fundamental module that returns exactly what it receives as input. It's useful for debugging, testing, data passing, and as a basic example of Phlow module implementation.

## 🚀 Features

### Key Features

- ✅ **Simplicity**: Returns exactly what it receives
- ✅ **Any type**: Accepts any type of input
- ✅ **Data preservation**: Maintains original structure and type
- ✅ **Performance**: Direct passthrough operation, no processing
- ✅ **Debug**: Useful for checking data in pipelines
- ✅ **Observability**: Fully integrated with OpenTelemetry

## 📝 Configuration

### Basic Configuration

```phlow
steps:
  - name: "echo_step"
    use: "echo_module"
    input: "Hello, World!"
```

### Configuration with Structured Data

```phlow
steps:
  - name: "echo_object"
    use: "echo_module"
    input:
      message: "Hello"
      timestamp: "2024-01-01T00:00:00Z"
      data:
        items: [1, 2, 3]
        active: true
```

## 🔧 Parameters

### Input
- **Type**: `any` (any type)
- **Required**: `true`
- **Description**: The message or data to be echoed
- **Default**: `null`

### Output
- **Type**: `any` (same type as input)
- **Required**: `true`
- **Description**: The echoed data (identical to input)
- **Default**: `null`

## 💻 Usage Examples

### Simple String Echo

```phlow
steps:
  - name: "simple_echo"
    use: "echo_module"
    input: "Esta mensagem será ecoada"
    
  # Output: "This message will be echoed"
```

### Number Echo

```phlow
steps:
  - name: "number_echo"
    use: "echo_module"
    input: 42
    
  # Output: 42
```

### Boolean Echo

```phlow
steps:
  - name: "boolean_echo"
    use: "echo_module"
    input: true
    
  # Output: true
```

### Array Echo

```phlow
steps:
  - name: "array_echo"
    use: "echo_module"
    input: [1, 2, 3, "test", true]
    
  # Output: [1, 2, 3, "test", true]
```

### Complex Object Echo

```phlow
steps:
  - name: "object_echo"
    use: "echo_module"
    input:
      user:
        id: 123
        name: "João Silva"
        email: "joao@example.com"
        active: true
        preferences:
          theme: "dark"
          notifications: true
        tags: ["admin", "premium"]
      metadata:
        created_at: "2024-01-01T00:00:00Z"
        updated_at: "2024-01-15T14:30:00Z"
        version: "1.2.3"
    
  # Output: (identical object to input)
```

### Echo with Dynamic Data

```phlow
steps:
  - name: "process_user"
    # Some processing that returns user data
    
  - name: "echo_user_data"
    use: "echo_module"
    input: "{{ $process_user }}"
    
  # Output: (user data from previous step)
```

## 🔍 Use Cases

### 1. Pipeline Debug

```phlow
steps:
  - name: "fetch_data"
    use: "http_request"
    input:
      url: "https://api.example.com/users"
      
  - name: "debug_response"
    use: "echo_module"
    input: "{{ $fetch_data }}"
    # Useful to see exactly what the API returned
    
  - name: "process_data"
    # Continue processing...
```

### 2. Data Passing

```phlow
steps:
  - name: "calculate_result"
    script: |
      let result = input.a + input.b;
      result * 2;
    
  - name: "pass_result"
    use: "echo_module"
    input: "{{ $calculate_result }}"
    
  - name: "format_output"
    input: "Result: {{ $pass_result }}"
```

### 3. Structure Validation

```phlow
steps:
  - name: "create_user_object"
    script: |
      {
        id: 123,
        name: "Test User",
        email: "test@example.com",
        created_at: new Date().toISOString()
      }
    
  - name: "validate_structure"
    use: "echo_module"
    input: "{{ $create_user_object }}"
    # Checks if object was created correctly
    
  - name: "save_user"
    use: "database_save"
    input: "{{ $validate_structure }}"
```

### 4. Testing and Development

```phlow
steps:
  - name: "mock_api_response"
    use: "echo_module"
    input:
      status: "success"
      data:
        users: [
          { id: 1, name: "Alice" },
          { id: 2, name: "Bob" }
        ]
      timestamp: "2024-01-01T00:00:00Z"
    
  - name: "process_users"
    # Process as if it came from a real API
    input: "{{ $mock_api_response.data.users }}"
```

## 🌐 Complete Example

```phlow
name: "echo-demo"
version: "1.0.0"
description: "Echo module demonstration"

modules:
  - name: "echo_module"
    module: "echo"
    version: "0.0.1"

steps:
  - name: "echo_string"
    use: "echo_module"
    input: "Hello from Echo!"
    
  - name: "echo_number"
    use: "echo_module"
    input: 3.14159
    
  - name: "echo_complex_object"
    use: "echo_module"
    input:
      application:
        name: "MyApp"
        version: "2.1.0"
        config:
          debug: true
          max_connections: 100
          features: ["auth", "cache", "logging"]
      environment:
        stage: "production"
        region: "us-east-1"
        
  - name: "echo_with_interpolation"
    use: "echo_module"
    input: "App: {{ $echo_complex_object.application.name }} v{{ $echo_complex_object.application.version }}"
    
  - name: "final_output"
    script: |
      {
        string_echo: $echo_string,
        number_echo: $echo_number,
        object_echo: $echo_complex_object,
        interpolated_echo: $echo_with_interpolation
      }
```

## 📊 Observability

The Echo module inherits the standard Phlow SDK observability:

- **Tracing**: Each execution generates OpenTelemetry spans
- **Logging**: Structured logs for debugging
- **Metrics**: Performance and usage metrics
- **Context**: Context propagation between steps

## 🔒 Security

- **Data preservation**: Does not modify or expose sensitive data
- **No side effects**: Purely functional operation
- **Memory**: Passes references when possible for efficiency

## 📈 Performance

- **Minimal latency**: Direct passthrough operation
- **Efficient memory**: No unnecessary copies
- **Threading**: Full support for asynchronous execution
- **Scalability**: No throughput limitations

## 🛠️ Implementation

The Echo module is implemented in a minimalist way:

```rust
pub async fn echo(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        sender_safe!(package.sender, input.into());
    });
    
    Ok(())
}
```

## 🏷️ Tags

- echo
- debug
- passthrough
- testing
- utility

---

**Version**: 0.0.1  
**Author**: Philippe Assis `<codephilippe@gmail.com>`
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
