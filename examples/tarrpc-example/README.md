# TarRPC Example

This example demonstrates how to use the tarrpc module for RPC communication using the tarpc framework.

## Files

- `server.yaml`: Defines a tarpc server that exposes RPC methods
- `client.yaml`: Defines a tarpc client that calls the server methods
- `config.yaml`: Configuration for both server and client

## Usage

### Starting the Server

```bash
# Run the server
phlow run server.yaml
```

The server will start on `localhost:8080` and expose the following RPC methods:
- `process_data`: Process incoming data
- `calculate`: Perform calculations
- `transform`: Transform data format
- `health_check`: Check server health (built-in)
- `get_service_info`: Get service information (built-in)

### Running the Client

```bash
# Run the client (make sure server is running first)
phlow run client.yaml
```

The client will:
1. Check server health
2. Get service information
3. Call various RPC methods with different parameters
4. Log the results

## Features Demonstrated

- **RPC Server**: Creating a tarpc server that handles multiple methods
- **RPC Client**: Making RPC calls to the server with retry logic
- **Method Handlers**: Routing RPC calls to specific step handlers
- **Error Handling**: Proper error handling and retry mechanisms
- **Observability**: Built-in tracing and logging
- **Configuration**: Flexible configuration for both server and client

## Configuration Options

### Server Configuration

```yaml
modules:
  - module: tarrpc
    with:
      host: localhost          # Server host
      port: 8080              # Server port
      service_name: my_service # Service name
      transport: tcp          # Transport type (tcp, memory)
      timeout: 30             # Default timeout in seconds
      max_connections: 100    # Maximum concurrent connections
      retry_attempts: 3       # Number of retry attempts
      methods:                # Available RPC methods
        - name: method_name
          handler: step_name
```

### Client Configuration

```yaml
modules:
  - module: tarrpc
    with:
      host: localhost          # Server host
      port: 8080              # Server port
      service_name: my_service # Service name
      transport: tcp          # Transport type
      timeout: 30             # Default timeout
      retry_attempts: 3       # Retry attempts

steps:
  - use: tarrpc
    input:
      method: method_name     # RPC method to call
      args:                   # Method arguments
        param1: value1
        param2: value2
      timeout: 15             # Override timeout
      context:                # Additional context
        user_id: user123
```

## Error Handling

The module provides comprehensive error handling:

- **Connection Errors**: Automatic retry with exponential backoff
- **Timeout Errors**: Configurable timeouts for each request
- **Method Errors**: Proper error responses for invalid methods
- **Serialization Errors**: Graceful handling of JSON parsing errors

## Observability

The module includes built-in observability:

- **Tracing**: OpenTelemetry traces for all RPC calls
- **Logging**: Structured logging with configurable levels
- **Metrics**: Execution time and success/failure rates
- **Health Checks**: Built-in health check endpoint
