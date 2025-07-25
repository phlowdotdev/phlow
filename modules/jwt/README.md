# JWT Module

JSON Web Token (JWT) module for Phlow applications, providing secure token creation and validation capabilities.

## Features

- ✅ **Create JWT tokens** with custom data payload
- ✅ **Verify JWT tokens** with signature and expiration validation  
- ✅ **Configurable secret** for token signing
- ✅ **Automatic expiration** with configurable TTL
- ✅ **Standard JWT claims** (iat, exp) handling
- ✅ **Comprehensive error handling** for invalid tokens
- ✅ **Full test coverage** with unit tests

## Configuration

```yaml
modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "your-secret-key"  # Required
      expires_in: 3600          # Optional, default: 3600 seconds (1 hour)
```

## Usage

### Create Token

```yaml
steps:
  - name: "create_token"
    use: "jwt_handler"
    input:
      action: "create"
      data:
        user_id: 123
        email: "user@example.com"
        roles: ["admin", "user"]
```

**Output:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2024-01-01T13:00:00Z",
  "issued_at": "2024-01-01T12:00:00Z"
}
```

### Verify Token

```yaml
steps:
  - name: "verify_token"
    use: "jwt_handler"
    input:
      action: "verify"
      token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

**Output (Valid):**
```json
{
  "valid": true,
  "data": {
    "user_id": 123,
    "email": "user@example.com",
    "roles": ["admin", "user"],
    "iat": 1640995200,
    "exp": 1640998800
  },
  "expired": false
}
```

**Output (Invalid):**
```json
{
  "valid": false,
  "data": null,
  "error": "Token has expired",
  "expired": true
}
```

## Security

- Uses **HS256** algorithm for signing
- Validates token **signature** and **expiration**
- Supports **configurable secret keys**
- Provides detailed **error messages** for debugging

## Testing

Run the module tests:

```bash
cargo test
```

The module includes comprehensive tests for:
- Token creation with and without data
- Token verification with valid/invalid tokens
- Error handling for various failure scenarios
- Security validation with different secrets

## Dependencies

- `jsonwebtoken` - JWT encoding/decoding
- `chrono` - Date/time handling
- `serde` - Serialization/deserialization
- `phlow-sdk` - Phlow module framework
- `valu3` - Universal data manipulation library (via phlow-sdk)

## License

MIT
