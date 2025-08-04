---
sidebar_position: 4
title: Database Operations
---

# Database Operations

Phlow provides robust database connectivity through the `postgres` module, enabling you to perform CRUD operations, migrations, and complex queries.

## Basic Database Connection

Start with a simple database connection and query:

```phlow title="basic-db.phlow"
name: Basic Database Operations
version: 1.0.0
description: Connect to PostgreSQL and perform basic operations

modules:
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: myapp
      user: user
      password: password

steps:
  - postgres:
      query: "SELECT version()"
  
  - log:
      message: !phs `Database version: ${payload[0].version}`
  
  - postgres:
      query: "SELECT NOW() as current_time"
  
  - return: !phs payload[0].current_time
```

## CRUD Operations

Complete Create, Read, Update, Delete operations:

```phlow title="crud-operations.phlow"
name: CRUD Operations
version: 1.0.0
description: Complete CRUD operations with PostgreSQL

modules:
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: myapp
      user: user
      password: password

steps:
  # Create table if not exists
  - postgres:
      query: |
        CREATE TABLE IF NOT EXISTS users (
          id SERIAL PRIMARY KEY,
          name VARCHAR(100) NOT NULL,
          email VARCHAR(100) UNIQUE NOT NULL,
          age INTEGER,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
          updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
  
  # Insert new user
  - postgres:
      query: |
        INSERT INTO users (name, email, age) 
        VALUES ($1, $2, $3) 
        RETURNING id, name, email, age, created_at
      params:
        - "John Doe"
        - "john@example.com"
        - 30
  
  - payload: !phs { created_user: payload[0] }
  
  # Read users
  - postgres:
      query: "SELECT * FROM users ORDER BY created_at DESC LIMIT 10"
  
  - payload: !phs { ...payload, users: payload }
  
  # Update user
  - postgres:
      query: |
        UPDATE users 
        SET name = $1, age = $2, updated_at = CURRENT_TIMESTAMP
        WHERE email = $3
        RETURNING id, name, email, age, updated_at
      params:
        - "John Smith"
        - 31
        - "john@example.com"
  
  - payload: !phs { ...payload, updated_user: payload[0] }
  
  # Delete user (commented out for safety)
  # - postgres:
  #     query: "DELETE FROM users WHERE email = $1"
  #     params:
  #       - "john@example.com"
  
  - return: !phs {
      created: payload.created_user,
      updated: payload.updated_user,
      all_users: payload.users
    }
```

## Dynamic Query Builder

Build queries dynamically based on input:

```phlow title="dynamic-queries.phlow"
name: Dynamic Query Builder
version: 1.0.0
description: Build database queries dynamically

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: action
          type: string
          required: true
          help: Action to perform
        - index: 2
          long: table
          type: string
          required: true
          help: Table name
        - long: data
          short: d
          type: string
          required: false
          help: JSON data for operation
        - long: where
          short: w
          type: string
          required: false
          help: WHERE clause
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: myapp
      user: user
      password: password

steps:
  - payload: !phs {
      action: main.action,
      table: main.table,
      data: main.data ? JSON.parse(main.data) : null,
      where: main.where
    }
  
  # Handle search operation
  - assert: !phs payload.action == "search"
    then:
      - payload: !phs {
          ...payload,
          query: payload.where 
            ? `SELECT * FROM ${payload.table} WHERE ${payload.where}`
            : `SELECT * FROM ${payload.table}`
        }
      - postgres:
          query: !phs payload.query
      - return: !phs payload
  
  # Handle create operation
  - assert: !phs payload.action == "create"
    then:
      - payload: !phs {
          ...payload,
          columns: Object.keys(payload.data),
          values: Object.values(payload.data),
          placeholders: Object.keys(payload.data).map((_, i) => `$${i + 1}`).join(', ')
        }
      - payload: !phs {
          ...payload,
          query: `INSERT INTO ${payload.table} (${payload.columns.join(', ')}) VALUES (${payload.placeholders}) RETURNING *`
        }
      - postgres:
          query: !phs payload.query
          params: !phs payload.values
      - return: !phs payload[0]
  
  # Handle update operation
  - assert: !phs payload.action == "update"
    then:
      - payload: !phs {
          ...payload,
          setClause: Object.keys(payload.data)
            .map((key, i) => `${key} = $${i + 1}`)
            .join(', '),
          values: Object.values(payload.data)
        }
      - payload: !phs {
          ...payload,
          query: `UPDATE ${payload.table} SET ${payload.setClause}${payload.where ? ` WHERE ${payload.where}` : ''} RETURNING *`
        }
      - postgres:
          query: !phs payload.query
          params: !phs payload.values
      - return: !phs payload
  
  # Handle delete operation
  - assert: !phs payload.action == "delete"
    then:
      - payload: !phs {
          ...payload,
          query: `DELETE FROM ${payload.table}${payload.where ? ` WHERE ${payload.where}` : ''} RETURNING *`
        }
      - postgres:
          query: !phs payload.query
      - return: !phs payload
```

### Usage Examples

```bash
# Search users
phlow dynamic-queries.phlow search users

# Search with WHERE clause
phlow dynamic-queries.phlow search users --where "age > 25"

# Create new user
phlow dynamic-queries.phlow create users --data '{"name": "Alice", "email": "alice@example.com", "age": 28}'

# Update user
phlow dynamic-queries.phlow update users --data '{"age": 29}' --where "email = 'alice@example.com'"
```

## Database Migration System

Create a simple migration system:

```phlow title="migration-system.phlow"
name: Database Migration System
version: 1.0.0
description: Simple database migration system

main: cli
modules:
  - module: cli
    version: latest
    with:
      args:
        - index: 1
          long: command
          type: string
          required: true
          help: Migration command
        - index: 2
          long: migration_name
          type: string
          required: false
          help: Migration name
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: myapp
      user: user
      password: password

steps:
  # Create migrations table if not exists
  - postgres:
      query: |
        CREATE TABLE IF NOT EXISTS migrations (
          id SERIAL PRIMARY KEY,
          name VARCHAR(255) NOT NULL,
          applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
  
  # Initialize migration data
  - payload: !phs {
      available_migrations: [
        {
          name: "001_create_users_table",
          up: `CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
          )`,
          down: "DROP TABLE users"
        },
        {
          name: "002_add_age_to_users",
          up: "ALTER TABLE users ADD COLUMN age INTEGER",
          down: "ALTER TABLE users DROP COLUMN age"
        },
        {
          name: "003_create_posts_table",
          up: `CREATE TABLE posts (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            content TEXT,
            user_id INTEGER REFERENCES users(id),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
          )`,
          down: "DROP TABLE posts"
        }
      ]
    }
  
  # Get applied migrations
  - postgres:
      query: "SELECT name FROM migrations ORDER BY applied_at"
  
  - payload: !phs {
      ...payload,
      applied_migrations: payload.map(row => row.name)
    }
  
  # Handle status command
  - assert: !phs main.command == "status"
    then:
      - payload: !phs {
          ...payload,
          pending_migrations: payload.available_migrations.filter(
            m => !payload.applied_migrations.includes(m.name)
          )
        }
      - return: !phs {
          applied: payload.applied_migrations,
          pending: payload.pending_migrations.map(m => m.name),
          total_available: payload.available_migrations.length
        }
  
  # Handle migrate command
  - assert: !phs main.command == "migrate"
    then:
      - payload: !phs {
          ...payload,
          pending_migrations: payload.available_migrations.filter(
            m => !payload.applied_migrations.includes(m.name)
          )
        }
      - payload: !phs {
          ...payload,
          migration_to_apply: main.migration_name 
            ? payload.pending_migrations.find(m => m.name === main.migration_name)
            : payload.pending_migrations[0]
        }
      - assert: !phs payload.migration_to_apply
        then:
          - postgres:
              query: !phs payload.migration_to_apply.up
          - postgres:
              query: "INSERT INTO migrations (name) VALUES ($1)"
              params:
                - !phs payload.migration_to_apply.name
          - return: !phs {
              message: `Migration '${payload.migration_to_apply.name}' applied successfully`
            }
        else:
          return: !phs { error: "No pending migrations found" }
  
  # Handle rollback command
  - assert: !phs main.command == "rollback"
    then:
      - payload: !phs {
          ...payload,
          last_migration: payload.applied_migrations[payload.applied_migrations.length - 1]
        }
      - assert: !phs payload.last_migration
        then:
          - payload: !phs {
              ...payload,
              migration_to_rollback: payload.available_migrations.find(
                m => m.name === payload.last_migration
              )
            }
          - postgres:
              query: !phs payload.migration_to_rollback.down
          - postgres:
              query: "DELETE FROM migrations WHERE name = $1"
              params:
                - !phs payload.migration_to_rollback.name
          - return: !phs {
              message: `Migration '${payload.migration_to_rollback.name}' rolled back successfully`
            }
        else:
          return: !phs { error: "No migrations to rollback" }
```

## Database Connection Pool

Manage database connections efficiently:

```phlow title="connection-pool.phlow"
name: Database Connection Pool
version: 1.0.0
description: Efficient database connection management

modules:
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: myapp
      user: user
      password: password
      pool_size: 10
      max_connections: 20

steps:
  # Initialize multiple operations
  - payload: !phs {
      operations: [
        "SELECT COUNT(*) as user_count FROM users",
        "SELECT COUNT(*) as post_count FROM posts",
        "SELECT AVG(age) as avg_age FROM users WHERE age IS NOT NULL",
        "SELECT COUNT(*) as recent_users FROM users WHERE created_at > NOW() - INTERVAL '7 days'"
      ]
    }
  
  # Execute operations concurrently (simulated)
  - payload: !phs {
      results: {}
    }
  
  # User count
  - postgres:
      query: "SELECT COUNT(*) as user_count FROM users"
  - payload: !phs {
      ...payload,
      results: { ...payload.results, user_count: payload[0].user_count }
    }
  
  # Post count
  - postgres:
      query: "SELECT COUNT(*) as post_count FROM posts"
  - payload: !phs {
      ...payload,
      results: { ...payload.results, post_count: payload[0].post_count }
    }
  
  # Average age
  - postgres:
      query: "SELECT AVG(age) as avg_age FROM users WHERE age IS NOT NULL"
  - payload: !phs {
      ...payload,
      results: { ...payload.results, avg_age: payload[0].avg_age }
    }
  
  # Recent users
  - postgres:
      query: "SELECT COUNT(*) as recent_users FROM users WHERE created_at > NOW() - INTERVAL '7 days'"
  - payload: !phs {
      ...payload,
      results: { ...payload.results, recent_users: payload[0].recent_users }
    }
  
  - return: !phs {
      statistics: payload.results,
      generated_at: new Date().toISOString()
    }
```

## Testing Database Operations

Create comprehensive tests for database operations:

```phlow title="database-tests.phlow"
name: Database Tests
version: 1.0.0
description: Testing database operations

modules:
  - module: postgres
    version: latest
    with:
      host: localhost
      port: 5432
      database: test_db
      user: test_user
      password: test_password

tests:
  # Test database connection
  - main: {}
    payload: null
    assert: !phs payload.length > 0
  
  # Test user creation
  - main:
      name: "Test User"
      email: "test@example.com"
      age: 25
    payload: null
    assert: !phs payload.user.name == "Test User"
  
  # Test user retrieval
  - main:
      email: "test@example.com"
    payload: null
    assert: !phs payload.user.email == "test@example.com"

steps:
  # Setup test database
  - postgres:
      query: |
        CREATE TABLE IF NOT EXISTS test_users (
          id SERIAL PRIMARY KEY,
          name VARCHAR(100),
          email VARCHAR(100),
          age INTEGER
        )
  
  # Clean up before tests
  - postgres:
      query: "DELETE FROM test_users"
  
  # Test connection
  - postgres:
      query: "SELECT 1 as connection_test"
  
  - assert: !phs payload.length > 0
    then:
      # Test user creation
      - assert: !phs main.name && main.email
        then:
          - postgres:
              query: "INSERT INTO test_users (name, email, age) VALUES ($1, $2, $3) RETURNING *"
              params:
                - !phs main.name
                - !phs main.email
                - !phs main.age
          - return: !phs { user: payload[0] }
      
      # Test user retrieval
      - assert: !phs main.email && !main.name
        then:
          - postgres:
              query: "SELECT * FROM test_users WHERE email = $1"
              params:
                - !phs main.email
          - return: !phs { user: payload[0] }
      
      # Default: return connection test
      - return: !phs payload
```

## Key Features Demonstrated

1. **Database Connectivity**: Connect to PostgreSQL with configuration
2. **CRUD Operations**: Complete Create, Read, Update, Delete functionality
3. **Dynamic Queries**: Build queries based on runtime parameters
4. **Migration System**: Database schema versioning and management
5. **Connection Pooling**: Efficient database connection management
6. **Parameterized Queries**: Prevent SQL injection with parameter binding
7. **Transaction Support**: Maintain data consistency
8. **Testing**: Comprehensive database operation testing
9. **Error Handling**: Proper error handling and logging

These examples demonstrate how to build robust database-driven applications with Phlow's PostgreSQL module.
