# Cache Module Examples

This directory contains practical examples demonstrating how to use the Phlow cache module in different scenarios. Each example showcases specific use cases and best practices for caching in Phlow applications.

## Available Examples

### 1. `basic-usage.phlow` - Basic Operations
**Purpose**: Comprehensive demonstration of all cache module features

**What it shows**:
- Basic set, get, remove, and clear operations
- TTL (Time-To-Live) functionality with different expiration times
- List operations with filtering (prefix, suffix, pattern)
- Pagination and ordering
- Cache statistics and memory usage
- Manual cleanup of expired items

**Best for**: Understanding all available cache operations and their parameters

**Run with**:
```bash
phlow examples/cache/basic-usage.phlow
```

### 2. `simple-test.phlow` - Testing Suite
**Purpose**: Complete test suite for cache functionality

**What it shows**:
- 12 automated tests covering all cache operations
- Different data types (strings, numbers, objects, arrays)
- Test patterns for validation and error handling
- Using the Phlow testing framework with `--test` flag

**Best for**: Learning how to test cache operations and understanding expected behaviors

**Run with**:
```bash
# Run all tests
phlow --test examples/cache/simple-test.phlow

# Run specific test categories
phlow --test --test-filter "string" examples/cache/simple-test.phlow
phlow --test --test-filter "object" examples/cache/simple-test.phlow
```

### 3. `comprehensive-test.phlow` - Advanced Testing
**Purpose**: Extended test suite with edge cases and advanced scenarios

**What it shows**:
- 23 comprehensive tests
- TTL edge cases and expiration handling
- Advanced filtering and pagination scenarios
- Error condition testing
- Performance and capacity testing

**Best for**: Advanced testing scenarios and edge case validation

**Run with**:
```bash
phlow --test examples/cache/comprehensive-test.phlow
```

### 4. `user-sessions.phlow` - Session Management
**Purpose**: Real-world example of using cache for user session management

**What it shows**:
- Session creation and validation
- User profile caching
- Session renewal and timeout handling
- Session cleanup and logout
- Administrative session listing
- Security best practices for session data

**Best for**: Web applications, authentication systems, user management

**Run with**:
```bash
phlow examples/cache/user-sessions.phlow
```

### 5. `api-data-cache.phlow` - API Response Caching
**Purpose**: Caching strategies for API responses and computed data

**What it shows**:
- Multiple TTL strategies for different data types:
  - **Short TTL (5 min)**: Frequently changing data (user lists)
  - **Medium TTL (30 min)**: Individual records (user profiles)  
  - **Long TTL (24 hours)**: Computed statistics
  - **Very Long TTL (7 days)**: Configuration data
- Cache invalidation patterns
- Performance optimization techniques
- Cache hit/miss monitoring

**Best for**: APIs, microservices, data-heavy applications

**Run with**:
```bash
phlow examples/cache/api-data-cache.phlow
```

## Common Use Cases Covered

### 1. **Web Session Management**
- User login/logout workflows
- Session validation and renewal
- Security considerations
- Session cleanup strategies

### 2. **API Response Caching**
- Different TTL strategies by data type
- Cache invalidation patterns
- Performance monitoring
- Memory usage optimization

### 3. **Configuration Caching**
- Application settings
- Feature flags
- System configuration
- Long-term storage patterns

### 4. **Computed Data Caching**
- Expensive calculations
- Database query results
- Report generation
- Analytics data

## Cache Configuration Patterns

### Development/Testing
```phlow
modules:
  - module: cache
    with:
      capacity: 10           # Small capacity for testing
      default_ttl: 300       # 5 minutes
      enable_events: false
```

### Production - High Performance
```phlow
modules:
  - module: cache
    with:
      capacity: 10000        # Large capacity
      default_ttl: 1800      # 30 minutes
      enable_events: true    # Enable for monitoring
```

### Production - Memory Constrained
```phlow
modules:
  - module: cache
    with:
      capacity: 1000         # Moderate capacity
      default_ttl: 600       # 10 minutes
      enable_events: false
```

## Best Practices Demonstrated

### 1. **Key Naming Conventions**
```phlow
# Good naming patterns from examples
"session:{user_id}"           # User sessions
"user:profile:{user_id}"      # User profiles
"api:{endpoint}:list"         # API list responses
"api:user:{id}"              # Individual records
"api:config:{type}"          # Configuration data
"api:stats:{period}"         # Statistics data
```

### 2. **TTL Strategies**
- **Short TTL (1-10 minutes)**: Frequently changing data
- **Medium TTL (30-60 minutes)**: User-specific data
- **Long TTL (hours)**: Computed/aggregated data
- **Very Long TTL (days)**: Configuration/static data

### 3. **Error Handling Patterns**
```phlow
# Check before operations
- use: cache
  input:
    action: exists
    key: "my:key"

- assert: !phs payload.found
  then:
    # Handle cache hit
  else:
    # Handle cache miss
```

### 4. **Cache Maintenance**
```phlow
# Regular cleanup
- use: cache
  input:
    action: cleanup

# Monitor performance  
- use: cache
  input:
    action: stats
```

## Testing Your Cache Implementation

### Quick Validation
```bash
# Test basic functionality
phlow --test examples/cache/simple-test.phlow

# Test specific operations
phlow --test --test-filter "set" examples/cache/simple-test.phlow
phlow --test --test-filter "get" examples/cache/simple-test.phlow
```

### Performance Testing
```bash
# Run comprehensive tests
phlow --test examples/cache/comprehensive-test.phlow

# Monitor during real usage
phlow examples/cache/api-data-cache.phlow
```

## Integration Examples

### With HTTP Server
```phlow
modules:
  - module: http_server
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800

# Use cache to store API responses
steps:
  - use: cache
    input:
      action: get
      key: !phs `api:users:${request.path}`
  
  - assert: !phs !payload.found
    then:
      # Cache miss - fetch from database
      - use: database
        # ... database query
      - use: cache
        input:
          action: set
          key: !phs `api:users:${request.path}`
          value: !phs database_result
          ttl: 600
```

### With Database
```phlow
modules:
  - module: postgres
  - module: cache

# Cache expensive queries
steps:
  - use: cache
    input:
      action: get  
      key: "expensive:query:results"
      
  - assert: !phs !payload.found
    then:
      - use: postgres
        input:
          query: "SELECT * FROM complex_view WHERE ..."
      - use: cache
        input:
          action: set
          key: "expensive:query:results"
          value: !phs payload
          ttl: 3600
```

## Monitoring and Debugging

### Cache Statistics
All examples include cache statistics monitoring:
```phlow
- use: cache
  input:
    action: stats

# Monitor: size, capacity, hit_rate, memory_usage
```

### Performance Metrics
- **Hit Rate**: Percentage of successful cache retrievals
- **Memory Usage**: Estimated cache memory consumption  
- **Operation Counts**: Total sets, gets, removes performed
- **Capacity Utilization**: Current size vs. maximum capacity

## Next Steps

1. **Start with `simple-test.phlow`** to understand basic operations
2. **Review `basic-usage.phlow`** for comprehensive feature overview
3. **Choose a specific use case** (`user-sessions.phlow` or `api-data-cache.phlow`)
4. **Adapt examples** to your specific requirements
5. **Run tests** to validate your implementation
6. **Monitor performance** using cache statistics

For more information, see the [Cache Module Documentation](../../modules/cache/README.md) and [Testing Guide](../../modules/cache/TEST_DOCUMENTATION.md).
