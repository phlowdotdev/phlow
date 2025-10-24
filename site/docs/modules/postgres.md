---
sidebar_position: 7
title: Postgres Module
hide_title: true
---

---
sidebar_position: 7
title: Postgres Module
hide_title: true
---

# PostgreSQL Module

The PostgreSQL module provides complete PostgreSQL database operations with connection pooling, query caching, SSL support, and batch operations.

## 🚀 Features

### Key Features

- ✅ **Connection pool**: Efficient connection management
- ✅ **Query cache**: Configurable cache for performance
- ✅ **SSL support**: Multiple SSL modes (disable, prefer, require)
- ✅ **Batch operations**: Bulk inserts for high performance
- ✅ **Safe parameters**: SQL injection prevention
- ✅ **Observability**: Complete tracing with OpenTelemetry

## 📋 Configuração

### Configuração Básica

```phlow
modules:
  - name: "db"
    module: "postgres"
    with:
      host: "localhost"
      port: 5432
      user: "myuser"
      password: "mypassword"
      database: "mydb"
      ssl_mode: "prefer"
      max_pool_size: 20
      cache_query: true
```

## 🔧 Parâmetros

### Configuração (with)
- `host` (string): Hostname do servidor PostgreSQL
- `port` (integer): Porta (padrão: 5432)
- `user` (string): Nome de usuário
- `password` (string): Senha
- `database` (string): Nome do banco
- `ssl_mode` (enum): Modo SSL [disable, prefer, require]
- `max_pool_size` (integer): Tamanho máximo do pool (padrão: 10)
- `cache_query` (boolean): Cache de queries (padrão: true)

### Entrada (input)
- `query` (string): Query SQL
- `params` (array): Parâmetros da query
- `batch` (boolean): Modo batch
- `cache_query` (boolean): Cache específico da query

### Saída (output)
- `result.rows` (array): Linhas retornadas
- `result.count` (integer): Número de linhas
- `message` (string): Mensagem da operação
- `status` (string): Status (success/failure)

## 💻 Exemplos de Uso

### SELECT Básico

```phlow
steps:
  - name: "get_users"
    use: "db"
    input:
      query: "SELECT id, name, email FROM users WHERE active = $1"
      params: [true]
```

### INSERT com Retorno

```phlow
steps:
  - name: "create_user"
    use: "db"
    input:
      query: "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id"
      params: ["João Silva", "joao@example.com"]
```

### Batch Insert

```phlow
steps:
  - name: "batch_insert"
    use: "db"
    input:
      query: "INSERT INTO logs (message, level, timestamp) VALUES ($1, $2, $3)"
      batch: true
      params: [
        ["Error 1", "ERROR", "2024-01-01T00:00:00Z"],
        ["Info 1", "INFO", "2024-01-01T00:01:00Z"],
        ["Debug 1", "DEBUG", "2024-01-01T00:02:00Z"]
      ]
```

## 🌐 Exemplo Completo

```phlow
name: "user-management"
version: "1.0.0"

modules:
  - name: "db"
    module: "postgres"
    with:
      host: "localhost"
      database: "users_db"
      user: "app_user"
      password: "secure_password"
      ssl_mode: "require"
      max_pool_size: 20

steps:
  - name: "create_user"
    use: "db"
    input:
      query: |
        INSERT INTO users (name, email, created_at) 
        VALUES ($1, $2, NOW()) 
        RETURNING id, name, email, created_at
      params: ["{{ $name }}", "{{ $email }}"]
      
  - name: "get_user_stats"
    use: "db"
    input:
      query: |
        SELECT 
          COUNT(*) as total_users,
          COUNT(CASE WHEN active = true THEN 1 END) as active_users,
          MAX(created_at) as last_user_created
        FROM users
        
  - name: "audit_log"
    use: "db"
    input:
      query: "INSERT INTO audit_log (action, user_id, timestamp) VALUES ($1, $2, NOW())"
      params: ["user_created", "{{ $create_user.result.rows[0].id }}"]
```

## 🏷️ Tags

- postgres
- database
- sql
- query

---

**Versão**: 0.0.1  
**Autor**: Philippe Assis `<codephilippe@gmail.com>`  
**Licença**: MIT  
**Repositório**: https://github.com/phlowdotdev/phlow
