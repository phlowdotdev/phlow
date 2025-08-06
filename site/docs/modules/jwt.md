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

## 📋 Configuração

### Configuração Básica

```phlow
modules:
  - name: "jwt_handler"
    module: "jwt"
    version: "0.0.1"
    with:
      secret: "minha-chave-secreta-super-segura"
      expires_in: 3600  # 1 hora em segundos (opcional)
```

### Configuração com Variáveis de Ambiente

```bash
# Secret via ambiente (mais seguro)
export JWT_SECRET="minha-chave-secreta-do-ambiente"
```

```phlow
modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "{{ env.JWT_SECRET }}"
      expires_in: 7200  # 2 horas
```

## 🔧 Parâmetros de Configuração

### Configuração do Módulo (with)
- `secret` (string, obrigatório): Chave secreta para assinatura do JWT
- `expires_in` (integer, opcional): Tempo de expiração em segundos (padrão: 3600)

### Entrada (input)
- `action` (string, obrigatório): Ação a executar ["create", "verify"]
- `data` (object, opcional): Dados para incluir no token (apenas para "create")
- `token` (string, opcional): Token para validação (apenas para "verify")

### Saída (output)

#### Para ação "create":
- `token` (string): Token JWT gerado
- `expires_at` (string): Timestamp de expiração (ISO 8601)
- `issued_at` (string): Timestamp de criação (ISO 8601)

#### Para ação "verify":
- `valid` (boolean): Se o token é válido
- `data` (object): Dados decodificados do token (se válido)
- `error` (string): Mensagem de erro (se inválido)
- `expired` (boolean): Se o token expirou

## 💻 Exemplos de Uso

### Criação de Token

```phlow
steps:
  - name: "create_user_token"
    use: "jwt_handler"
    input:
      action: "create"
      data:
        user_id: 123
        email: "usuario@example.com"
        roles: ["user", "admin"]
        permissions: ["read", "write"]
```

### Validação de Token

```phlow
steps:
  - name: "verify_token"
    use: "jwt_handler"
    input:
      action: "verify"
      token: "{{ $request.headers.authorization | replace('Bearer ', '') }}"
      
  - name: "check_validation"
    condition:
      left: "{{ $verify_token.valid }}"
      operator: "equals"
      right: true
    then:
      return: "{{ $verify_token.data }}"
    else:
      return:
        error: "Token inválido"
        message: "{{ $verify_token.error }}"
```

### Teste de Expiração de Token

```phlow
name: jwt-expiration-test
version: 1.0.0
description: Demonstração de expiração automática de token JWT

modules:
  - module: jwt
    with:
      secret: "minha-chave-secreta-super-segura"
  - module: sleep

steps:
  # Criar token com expiração de 1 segundo
  - use: jwt
    input:
      action: create
      data:
        user_id: 12345
        role: admin
      expires_in: 1

  # Aguardar 5 segundos (token expira durante este período)
  - use: sleep
    input:
      seconds: 5

  # Tentar verificar o token expirado
  - use: jwt
    input:
      action: verify
      token: !phs payload.token

  # Validar que o token está expirado
  - assert: !phs payload.valid
    then:
      return: "❌ Token ainda válido - Erro!"
    else:
      return: "✅ Token expirado corretamente - Sucesso!"
```

**Resultado esperado:**
```json
{
  "valid": false,
  "expired": true,
  "error": "Token has expired",
  "data": null
}
```

## 🌐 Exemplo Completo - Sistema de Autenticação

```phlow
name: "auth-system"
version: "1.0.0"
description: "Sistema de autenticação com JWT"

modules:
  - name: "jwt_handler"
    module: "jwt"
    with:
      secret: "{{ env.JWT_SECRET }}"
      expires_in: 3600  # 1 hora
      
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
      message: "Tentativa de autenticação para {{ $input.email }}"
      
  - name: "validate_user"
    use: "db"
    input:
      query: "SELECT id, email, password_hash, roles FROM users WHERE email = $1 AND active = true"
      params: ["{{ $input.email }}"]
      
  - name: "check_user_exists"
    condition:
      left: "{{ $validate_user.result.count }}"
      operator: "greater_than"
      right: 0
    then:
      # Usuário encontrado, verificar senha
      name: "verify_password"
      script: |
        // Simular verificação de senha
        let user = $validate_user.result.rows[0];
        let passwordValid = $input.password === "senha123"; // Em produção, usar bcrypt
        
        if (passwordValid) {
          {
            valid: true,
            user: user
          }
        } else {
          {
            valid: false,
            error: "Senha incorreta"
          }
        }
    else:
      return:
        success: false
        error: "Usuário não encontrado"
        
  - name: "check_password"
    condition:
      left: "{{ $verify_password.valid }}"
      operator: "equals"
      right: true
    then:
      # Senha válida, criar token
      use: "jwt_handler"
      input:
        action: "create"
        data:
          user_id: "{{ $verify_password.user.id }}"
          email: "{{ $verify_password.user.email }}"
          roles: "{{ $verify_password.user.roles }}"
          auth_time: "{{ timestamp() }}"
    else:
      # Senha inválida
      return:
        success: false
        error: "{{ $verify_password.error }}"
        
  - name: "log_success"
    use: "logger"
    input:
      level: "info"
      message: "Login bem-sucedido para {{ $input.email }}"
      
  - name: "return_token"
    return:
      success: true
      token: "{{ $check_password.token }}"
      expires_at: "{{ $check_password.expires_at }}"
      user:
        id: "{{ $verify_password.user.id }}"
        email: "{{ $verify_password.user.email }}"
```

## 🔐 Middleware de Autenticação

```phlow
name: "auth-middleware"
version: "1.0.0"
description: "Middleware para validação de JWT"

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
        { error: "Token não fornecido" }
      } else if (!authHeader.startsWith("Bearer ")) {
        { error: "Formato de token inválido" }
      } else {
        { token: authHeader.replace("Bearer ", "") }
      }
      
  - name: "check_token_extraction"
    condition:
      left: "{{ $extract_token.error }}"
      operator: "exists"
      right: true
    then:
      return:
        status_code: 401
        body:
          error: "Unauthorized"
          message: "{{ $extract_token.error }}"
    else:
      # Token extraído com sucesso
      use: "jwt_handler"
      input:
        action: "verify"
        token: "{{ $extract_token.token }}"
        
  - name: "validate_jwt"
    condition:
      left: "{{ $check_token_extraction.valid }}"
      operator: "equals"
      right: true
    then:
      # Token válido, continuar processamento
      script: |
        {
          user: $check_token_extraction.data,
          authenticated: true
        }
    else:
      # Token inválido
      condition:
        left: "{{ $check_token_extraction.expired }}"
        operator: "equals"
        right: true
      then:
        return:
          status_code: 401
          body:
            error: "Token Expired"
            message: "Token expirou, faça login novamente"
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
      message: "Usuário {{ $validate_jwt.user.email }} autenticado com sucesso"
      
  - name: "return_user_context"
    return:
      user: "{{ $validate_jwt.user }}"
      authenticated: true
```

## 🔍 Estrutura do Token JWT

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
  "user_id": 123,              // Dados customizados
  "email": "user@example.com", // do parâmetro 'data'
  "roles": ["user", "admin"]
}
```

## 🔧 Implementação Técnica

### Validação Dupla de Expiração

O módulo implementa uma estratégia de validação dupla para garantir que tokens expirados sejam sempre detectados:

1. **Validação da biblioteca jsonwebtoken**: Utiliza `validate_exp = true`
2. **Validação manual adicional**: Compara timestamp atual com `exp` claim

```rust
// Validação manual como backup
if current_timestamp > claims.exp {
    return Ok(jwt_error_response("Token has expired", true));
}
```

### Logging de Debug

O módulo fornece logging detalhado para debugging:

```
[DEBUG] Creating JWT token with data: {...}, expires_in: 1
[DEBUG] Token expiration time: 2025-01-01T10:00:01Z
[DEBUG] Verifying JWT token with value: eyJ0eXAi...
[DEBUG] Current timestamp: 1640998806
[DEBUG] Token claims - iat: 1640998800, exp: 1640998801, current: 1640998806
[WARN]  Token manually detected as expired: 1640998806 > 1640998801
```

### Gestão de Timestamps

- **Criação**: `iat` = timestamp atual, `exp` = iat + expires_in
- **Validação**: Compara timestamp atual com `exp` claim
- **Precisão**: Utiliza chrono::Utc para timestamps UTC precisos

## 📊 Observabilidade

O módulo automaticamente gera spans do OpenTelemetry com os seguintes atributos:

### Span Attributes
- `jwt.action`: "create" ou "verify"
- `jwt.algorithm`: "HS256"
- `jwt.valid`: true/false (para verify)
- `jwt.expired`: true/false (para verify)
- `jwt.user_id`: ID do usuário (se presente nos dados)
- `jwt.expires_in`: Tempo de expiração em segundos

## 🛡️ Segurança

### Boas Práticas
- **Secret forte**: Use chaves com pelo menos 256 bits
- **Variáveis de ambiente**: Nunca hardcode secrets
- **TTL apropriado**: Configure expiração adequada
- **HTTPS obrigatório**: Sempre use conexões seguras
- **Rotação de chaves**: Implemente rotação periódica

### Exemplo de Secret Seguro
```bash
# Gerar secret seguro
export JWT_SECRET=$(openssl rand -base64 32)
```

## 🔧 Tratamento de Erros

### Erros de Criação
```json
{
  "error": "Invalid data format",
  "message": "Os dados devem ser um objeto válido"
}
```

### Erros de Validação
```json
{
  "valid": false,
  "error": "Token expired",
  "expired": true,
  "message": "O token expirou em 2024-01-01T10:00:00Z"
}
```

## 💡 Casos de Uso

1. **Autenticação de APIs**: Validação de usuários em endpoints
2. **Single Sign-On (SSO)**: Tokens compartilhados entre serviços
3. **Autorização**: Controle de acesso baseado em roles
4. **Sessões**: Alternativa a cookies para SPAs
5. **Microserviços**: Propagação de identidade entre serviços

## 🏷️ Tags

- jwt
- auth
- authentication
- authorization
- token
- security

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
