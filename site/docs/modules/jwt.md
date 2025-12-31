---
sidebar_position: 10
title: JWT Module
hide_title: true
---

# JWT Module

The JWT module provides comprehensive functionality for creating and validating JSON Web Tokens (JWT), enabling secure authentication and authorization in Phlow applications.

## 🚀 Features

### Key Features

- ✅ **JWT token creation**: Generation of tokens with custom data
- ✅ **Token validation**: Automatic verification of signature and expiration
- ✅ **Configurable secret**: Secret key defined per module
- ✅ **Custom claims**: Support for arbitrary data in payload
- ✅ **Automatic expiration**: TTL configuration for tokens with strict validation
- ✅ **HS256 algorithm**: Industry standard for signing
- ✅ **Double validation**: jsonwebtoken library + manual expiration validation
- ✅ **Error handling**: Structured responses for failures

## 📋 Configuration

### Basic Configuration

```phlow
modules:
  - name: "jwt_handler"
    module: "jwt"
    version: "0.0.1"
    with:
      secret: "my-super-secure-secret-key"
      expires_in: 3600  # 1 hour in seconds (optional)
```

### Configuration with Environment Variables

```bash
# Secret via environment (more secure)
export JWT_SECRET="my-secret-key-from-environment"
```

```phlow
modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "{{ env.JWT_SECRET }}"
      expires_in: 7200  # 2 hours
```

## 🔧 Configuration Parameters

### Module Configuration (with)
- `secret` (string, required): Secret key for JWT signing
- `expires_in` (integer, optional): Expiration time in seconds (default: 3600)

### Input (input)
- `action` (string, required): Action to execute ["create", "verify"]
- `data` (object, optional): Data to include in token (only for "create")
- `token` (string, optional): Token for validation (only for "verify")

### Output (output)

#### For "create" action:
- `token` (string): Generated JWT token
- `expires_at` (string): Expiration timestamp (ISO 8601)
- `issued_at` (string): Creation timestamp (ISO 8601)

#### For "verify" action:
- `valid` (boolean): Whether the token is valid
- `data` (object): Decoded token data (if valid)
- `error` (string): Error message (if invalid)
- `expired` (boolean): Whether the token has expired

## 💻 Usage Examples

### Token Creation

```phlow
steps:
  - name: "create_user_token"
    use: "jwt_handler"
    input:
      action: "create"
      data:
        user_id: 123
        email: "user@example.com"
        roles: ["user", "admin"]
        permissions: ["read", "write"]
```

### Token Validation

```phlow
steps:
  - name: "verify_token"
    use: "jwt_handler"
    input:
      action: "verify"
      token: "{{ $request.headers.authorization | replace('Bearer ', '') }}"
      
  - name: "check_validation"
    assert: "{{ $verify_token.valid == true }}"
    then:
      return: "{{ $verify_token.data }}"
    else:
      return:
        error: "Invalid token"
        message: "{{ $verify_token.error }}"
```

### Token Expiration Test

```phlow
name: jwt-expiration-test
version: 1.0.0
description: JWT token automatic expiration demonstration

modules:
  - module: jwt
    with:
      secret: "my-super-secure-secret-key"
  - module: sleep

steps:
  # Create token with 1 second expiration
  - use: jwt
    input:
      action: create
      data:
        user_id: 12345
        role: admin
      expires_in: 1

  # Wait 5 seconds (token expires during this period)
  - use: sleep
    input:
      seconds: 5

  # Try to verify the expired token
  - use: jwt
    input:
      action: verify
      token: !phs payload.token

  # Validate that the token is expired
  - assert: !phs payload.valid
    then:
      return: "❌ Token is still valid - Error!"
    else:
      return: "✅ Token expired correctly - Success!"
```

**Expected result:**
```json
{
  "valid": false,
  "expired": true,
  "error": "Token has expired",
  "data": null
}
```

## 🌐 Complete Example - Authentication System

```phlow
name: "auth-system"
version: "1.0.0"
description: "JWT authentication system"

modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "{{ env.JWT_SECRET }}"
      expires_in: 3600  # 1 hour
      
  - name: "logger"
    module: "log"
    
  - name: "db"
    module: "postgres"
    with:
      host: "localhost"
      database: "auth_db"
      user: "app_user"
      password: "{{ env.DB_PASSWORD }}"

steps:
  - name: "log_auth_attempt"
    use: "logger"
    input:
      level: "info"
      message: "Authentication attempt for {{ $input.email }}"
      
  - name: "validate_user"
    use: "db"
    input:
      query: "SELECT id, email, password_hash, roles FROM users WHERE email = $1 AND active = true"
      params: ["{{ $input.email }}"]
      
  - name: "check_user_exists"
    assert: "{{ $validate_user.result.count > 0 }}"
    then:
      # User found, verify password
      name: "verify_password"
      script: |
        // Simulate password verification
        let user = $validate_user.result.rows[0];
        let passwordValid = $input.password === "password123"; // In production, use bcrypt
        
        if (passwordValid) {
          {
            valid: true,
            user: user
          }
        } else {
          {
            valid: false,
            error: "Incorrect password"
          }
        }
    else:
      return:
        success: false
        error: "User not found"
        
  - name: "check_password"
    assert: "{{ $verify_password.valid == true }}"
    then:
      # Valid password, create token
      use: "jwt_handler"
      input:
        action: "create"
        data:
          user_id: "{{ $verify_password.user.id }}"
          email: "{{ $verify_password.user.email }}"
          roles: "{{ $verify_password.user.roles }}"
          auth_time: "{{ timestamp() }}"
    else:
      # Invalid password
      return:
        success: false
        error: "{{ $verify_password.error }}"
        
  - name: "log_success"
    use: "logger"
    input:
      level: "info"
      message: "Successful login for {{ $input.email }}"
      
  - name: "return_token"
    return:
      success: true
      token: "{{ $check_password.token }}"
      expires_at: "{{ $check_password.expires_at }}"
      user:
        id: "{{ $verify_password.user.id }}"
        email: "{{ $verify_password.user.email }}"
```

## 🔐 Authentication Middleware

```phlow
name: "auth-middleware"
version: "1.0.0"
description: "JWT validation middleware"

modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "{{ env.JWT_SECRET }}"
      
  - name: "logger"
    module: "log"

steps:
  - name: "extract_token"
    script: |
      let authHeader = $input.headers.authorization;
      if (!authHeader) {
        { error: "Token not provided" }
      } else if (!authHeader.startsWith("Bearer ")) {
        { error: "Invalid token format" }
      } else {
        { token: authHeader.replace("Bearer ", "") }
      }
      
  - name: "check_token_extraction"
    assert: "{{ is_not_null($extract_token.error) }}"
    then:
      return:
        status_code: 401
        body:
          error: "Unauthorized"
          message: "{{ $extract_token.error }}"
    else:
      # Token extracted successfully
      use: "jwt_handler"
      input:
        action: "verify"
        token: "{{ $extract_token.token }}"
        
  - name: "validate_jwt"
    assert: "{{ $check_token_extraction.valid == true }}"
    then:
      # Valid token, continue processing
      script: |
        {
          user: $check_token_extraction.data,
          authenticated: true
        }
    else:
      # Invalid token
      assert: "{{ $check_token_extraction.expired == true }}"
      then:
        return:
          status_code: 401
          body:
            error: "Token Expired"
            message: "Token has expired, please login again"
      else:
        return:
          status_code: 401
          body:
            error: "Invalid Token"
            message: "{{ $check_token_extraction.error }}"
            
  - name: "log_auth_success"
    use: "logger"
    input:
      level: "debug"
      message: "User {{ $validate_jwt.user.email }} authenticated successfully"
      
  - name: "return_user_context"
    return:
      user: "{{ $validate_jwt.user }}"
      authenticated: true
```

## 🔍 JWT Token Structure

### Header
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

### Payload (Claims)
```json
{
  "iat": 1640995200,           // Issued At (timestamp)
  "exp": 1640998800,           // Expiration (timestamp)
  "user_id": 123,              // Custom data
  "email": "user@example.com", // from 'data' parameter
  "roles": ["user", "admin"]
}
```

## 🔧 Technical Implementation

### Double Expiration Validation

The module implements a double validation strategy to ensure expired tokens are always detected:

1. **jsonwebtoken library validation**: Uses `validate_exp = true`
2. **Additional manual validation**: Compares current timestamp with `exp` claim

```rust
// Manual validation as backup
if current_timestamp > claims.exp {
    return Ok(jwt_error_response("Token has expired", true));
}
```

### Debug Logging

The module provides detailed logging for debugging:

```
[DEBUG] Creating JWT token with data: {...}, expires_in: 1
[DEBUG] Token expiration time: 2025-01-01T10:00:01Z
[DEBUG] Verifying JWT token with value: eyJ0eXAi...
[DEBUG] Current timestamp: 1640998806
[DEBUG] Token claims - iat: 1640998800, exp: 1640998801, current: 1640998806
[WARN]  Token manually detected as expired: 1640998806 > 1640998801
```

### Timestamp Management

- **Creation**: `iat` = current timestamp, `exp` = iat + expires_in
- **Validation**: Compares current timestamp with `exp` claim
- **Precision**: Uses chrono::Utc for precise UTC timestamps

## 📊 Observability

The module automatically generates OpenTelemetry spans with the following attributes:

### Span Attributes
- `jwt.action`: "create" or "verify"
- `jwt.algorithm`: "HS256"
- `jwt.valid`: true/false (for verify)
- `jwt.expired`: true/false (for verify)
- `jwt.user_id`: User ID (if present in data)
- `jwt.expires_in`: Expiration time in seconds

## 🛡️ Security

### Best Practices
- **Strong secret**: Use keys with at least 256 bits
- **Environment variables**: Never hardcode secrets
- **Appropriate TTL**: Configure adequate expiration
- **HTTPS mandatory**: Always use secure connections
- **Key rotation**: Implement periodic rotation

### Secure Secret Example
```bash
# Generate secure secret
export JWT_SECRET=$(openssl rand -base64 32)
```

## 🔧 Error Handling

### Creation Errors
```json
{
  "error": "Invalid data format",
  "message": "Data must be a valid object"
}
```

### Validation Errors
```json
{
  "valid": false,
  "error": "Token expired",
  "expired": true,
  "message": "Token expired at 2024-01-01T10:00:00Z"
}
```

## 💡 Use Cases

1. **API Authentication**: User validation in endpoints
2. **Single Sign-On (SSO)**: Tokens shared between services
3. **Authorization**: Role-based access control
4. **Sessions**: Alternative to cookies for SPAs
5. **Microservices**: Identity propagation between services

## 🏷️ Tags

- jwt
- auth
- authentication
- authorization
- token
- security

---

**Version**: 0.0.1  
**Author**: Philippe Assis `<codephilippe@gmail.com>`  
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
