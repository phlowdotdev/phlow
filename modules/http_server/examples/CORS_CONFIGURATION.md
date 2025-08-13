# CORS Configuration in HTTP Server Module

The HTTP server module now supports **optional CORS configuration**. This means that CORS headers are only applied when explicitly configured.

## Default Behavior (No CORS)

When no `cors` configuration is provided in your flow configuration, the HTTP server **will not apply any CORS headers**. This is the new default behavior.

```yaml
modules:
  http_server:
    port: 3000
    host: "0.0.0.0"
    # No CORS configuration = No CORS headers applied
```

## Enabling CORS

To enable CORS, add a `cors` section to your HTTP server module configuration:

```yaml
modules:
  http_server:
    port: 3000
    host: "0.0.0.0"
    cors:
      origins:
        - "http://localhost:3000"
        - "http://localhost:5173"
        - "https://myapp.com"
      methods:
        - "GET"
        - "POST"
        - "PUT"
        - "PATCH" 
        - "DELETE"
        - "OPTIONS"
      headers:
        - "Content-Type"
        - "Authorization"
        - "X-Requested-With"
      credentials: true
      max_age: 86400
```

## CORS Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `origins` | Array of strings | `["*"]` | Allowed origins for cross-origin requests |
| `methods` | Array of strings | `["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]` | Allowed HTTP methods |
| `headers` | Array of strings | `["Content-Type", "Authorization", "X-Requested-With"]` | Allowed request headers |
| `credentials` | Boolean | `true` | Whether to allow credentials (cookies, authorization headers) |
| `max_age` | Number | `86400` | Cache duration for preflight requests (in seconds) |

## Security Considerations

- **Wildcard origins with credentials**: If you set `credentials: true`, you cannot use `"*"` as an origin. The system will automatically set `credentials: false` if wildcard origins are detected, and log a security warning.

- **Specific origins**: For production applications with credentials, always specify exact origins instead of using wildcards.

## Examples

### 1. API with CORS Enabled

See `examples/api-cors/api-cors.phlow` for a comprehensive example with CORS enabled for cross-origin requests.

### 2. API without CORS

See `examples/api-no-cors/api-no-cors.phlow` for an API that runs without any CORS headers (default behavior).

## Behavior Changes

### Before
- CORS was always enabled with default configuration
- All responses included CORS headers regardless of configuration

### After
- CORS is **optional** and disabled by default
- CORS headers are only applied when `cors` is explicitly configured
- No CORS headers are sent when `cors` configuration is absent

## Migration Guide

If you were relying on the previous default CORS behavior, you need to explicitly add CORS configuration to maintain the same functionality:

```yaml
# Add this to maintain previous behavior
modules:
  http_server:
    # ... your existing config
    cors:
      origins: ["*"]
      methods: ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
      headers: ["Content-Type", "Authorization", "X-Requested-With"]
      credentials: false  # Note: false when using wildcard origins
      max_age: 86400
```

## Testing CORS Configuration

You can test your CORS configuration using curl:

### Test without CORS (should have no CORS headers)
```bash
curl -H "Origin: http://example.com" http://localhost:3001/api/status -v
```

### Test with CORS (should include CORS headers)
```bash
curl -H "Origin: http://localhost:3000" http://localhost:3000/api/status -v
```

### Test preflight requests
```bash
curl -X OPTIONS \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  http://localhost:3000/api/users -v
```
