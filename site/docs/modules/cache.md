---
sidebar_position: 11
title: Cache Module
hide_title: true
---

# Cache Module

The Cache module provides comprehensive in-memory caching functionality for Phlow applications, allowing temporary data storage with high performance, TTL (Time To Live) control, and advanced filtering and sorting operations.

## 🚀 Features

### Key Features

- ✅ **High Performance**: O(1) access for get/set operations with QuickLeaf technology
- ✅ **Automatic TTL**: Automatic expiration of items with configurable Time To Live
- ✅ **LRU Eviction**: Automatic removal of least recently used items when capacity is reached
- ✅ **Advanced Filtering**: Filters by prefix, suffix, and custom patterns
- ✅ **Sorting and Pagination**: Ordered listing with limit/offset support
- ✅ **Real-time Statistics**: Hit rate, memory usage, and operation counters
- ✅ **Thread Safety**: Safe for concurrent access across multiple Phlow flows
- ✅ **Action-based API**: Multiple operations through a unified interface
- ✅ **Manual Cleanup**: Manual cleanup of expired items when needed

## 📋 Configuration

### Basic Configuration

```phlow
modules:
  - module: cache
    with:
      capacity: 1000      # Maximum of 1000 items
      default_ttl: 3600   # Default TTL of 1 hour
```

### Production Configuration

```phlow
modules:
  - module: cache
    with:
      capacity: 10000     # High capacity for production
      default_ttl: 1800   # 30 minutes default
```

### Development/Test Configuration

```phlow
modules:
  - module: cache
    with:
      capacity: 100       # Small capacity for testing
      default_ttl: 300    # 5 minutes for development
```

## 🔧 Configuration Parameters

### Module Configuration (with)
- `capacity` (integer, optional): Maximum number of items in cache (default: 1000)
- `default_ttl` (integer, optional): Default TTL in seconds for new items

### Input
- `action` (string, required): Action to execute ["set", "get", "remove", "clear", "exists", "list", "cleanup", "stats"]
- `key` (string): Item key (required for set, get, remove, exists)
- `value` (any): Value to store (required for set)
- `ttl` (integer, optional): TTL in seconds for the specific item
- `filter_type` (string, optional): Filter type for list ["prefix", "suffix", "pattern"]
- `filter_prefix` (string, optional): Prefix to filter (used with list)
- `filter_suffix` (string, optional): Suffix to filter (used with list)  
- `order` (string, optional): Sort order for list ["asc", "desc"] (default: "asc")
- `limit` (integer, optional): Maximum number of items for list
- `offset` (integer, optional): Number of items to skip in list (default: 0)

### Output
- `success` (boolean): Whether the operation was successful
- `error` (string): Error message (if failed)
- `found` (boolean): Whether the item was found (get, exists)
- `value` (any): Retrieved value (get)
- `cached` (boolean): Whether the item was stored (set)
- `removed` (boolean): Whether the item was removed (remove)
- `previous_size` (integer): Previous cache size (clear)
- `items` (array): List of items (list)
- `total_count` (integer): Total items found (list)
- `has_more` (boolean): Whether there are more items available (list)
- `cleaned_count` (integer): Number of items cleaned (cleanup)
- `stats` (object): Detailed cache statistics (stats)

## 💻 Usage Examples

### Basic Cache Operations

#### Store Data (Set)

```phlow
steps:
  - use: cache
    input:
      action: set
      key: "user:123"
      value:
        id: 123
        name: "John Smith"
        email: "john@example.com"
        role: "admin"
      ttl: 3600  # Expires in 1 hour
```

#### Retrieve Data (Get)

```phlow
steps:
  - use: cache
    input:
      action: get
      key: "user:123"
  
  - assert: !phs payload.found
    then:
      - return: !phs payload.value
    else:
      - return: 
          error: "User not found in cache"
```

#### Check Existence (Exists)

```phlow
steps:
  - use: cache
    input:
      action: exists
      key: "user:123"
  
  - return: !phs `User exists in cache: ${payload.found}`
```

#### Remove Item

```phlow
steps:
  - use: cache
    input:
      action: remove
      key: "user:123"
  
  - assert: !phs payload.removed
    then:
      - return: "User removed successfully"
    else:
      - return: "User was not in cache"
```

#### Clear Entire Cache

```phlow
steps:
  - use: cache
    input:
      action: clear
  
  - return: !phs `Cache cleared, ${payload.previous_size} items removed`
```

### Advanced Operations

#### Listing with Filters

##### Filter by Prefix
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "user:"
      order: "asc"
      limit: 10
```

##### Filter by Suffix
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "suffix"
      filter_suffix: ":session"
      order: "desc"
      limit: 20
```

##### Filter by Pattern (Prefix + Suffix)
```phlow
steps:
  - use: cache
    input:
      action: list
      filter_type: "pattern"
      filter_prefix: "cache_"
      filter_suffix: "_data"
      limit: 50
```

#### Pagination

```phlow
steps:
  # First page
  - use: cache
    input:
      action: list
      order: "asc"
      limit: 10
      offset: 0
  
  # Second page
  - use: cache
    input:
      action: list
      order: "asc"
      limit: 10
      offset: 10
```

#### Manual Cleanup

```phlow
steps:
  - use: cache
    input:
      action: cleanup
  
  - return: !phs `${payload.cleaned_count} expired items removed`
```

#### Cache Statistics (Stats)

```phlow
steps:
  - use: cache
    input:
      action: stats
  
  - return: !phs payload.stats
```

## 📊 Supported Data Types

### Strings
```phlow
- use: cache
  input:
    action: set
    key: "message"
    value: "Hello, world!"
    ttl: 300
```

### Numbers
```phlow
- use: cache
  input:
    action: set
    key: "counter"
    value: 42
    ttl: 600
```

### Complex Objects
```phlow
- use: cache
  input:
    action: set
    key: "user:profile"
      value:
        id: 123
        name: "Ana Costa"
        preferences:
          theme: "dark"
          language: "en-US"
      settings:
        notifications: true
        privacy: "public"
    ttl: 1800
```

### Arrays
```phlow
- use: cache
  input:
    action: set
    key: "user:permissions"
    value: ["read", "write", "admin", "delete"]
    ttl: 3600
```

## 🌐 Complete Examples

### User Session System

```phlow
name: "user-session-cache"
version: "1.0.0"
description: "Complete cache system for user sessions"

modules:
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800  # 30 minutes default
  - module: log

steps:
  # Create user session
  - use: cache
    input:
      action: set
      key: "session:12345"
      value:
        user_id: 12345
        username: "joao.silva"
        email: "joao@example.com" 
        login_time: "2025-08-06T23:10:00Z"
        last_activity: "2025-08-06T23:10:00Z"
        permissions: ["read", "write", "profile"]
        is_active: true
      ttl: 3600  # 1 hour for this specific session

  - use: log
    input:
      level: info
      message: "✅ Session created for user joao.silva"

  # Validate session exists
  - use: cache
    input:
      action: exists
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Session validation successful"
    else:
      - use: log
        input:
          level: error
          message: "❌ Session not found"

  # Retrieve session data
  - use: cache
    input:
      action: get
      key: "session:12345"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: !phs `👤 Session retrieved for ${payload.value.username}`
      
      # Renew session (update last_activity)
      - use: cache
        input:
          action: set
          key: "session:12345"
          value:
            user_id: !phs payload.value.user_id
            username: !phs payload.value.username
            email: !phs payload.value.email
            login_time: !phs payload.value.login_time
            last_activity: "2025-08-06T23:15:00Z"
            permissions: !phs payload.value.permissions
            is_active: true
          ttl: 3600  # Renew for another hour
      
      - use: log
        input:
          level: info
          message: "🔄 Session renewed successfully"

  # List all active sessions (admin)
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "session:"
      order: "desc"
      limit: 100

  - use: log
    input:
      level: info
      message: !phs `📊 Total of ${payload.total_count} active sessions`

  # Logout (remove session)
  - use: cache
    input:
      action: remove
      key: "session:12345"

  - assert: !phs payload.removed
    then:
      - use: log
        input:
          level: info
          message: "🚪 Logout performed successfully"

  # Check if session was removed
  - use: cache
    input:
      action: exists
      key: "session:12345"

  - assert: !phs !payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Confirmed: session was removed"

  # Final statistics
  - use: cache
    input:
      action: stats

  - return:
      message: "Session system processed successfully"
      cache_stats: !phs payload.stats
```

### API Response Cache

```phlow
name: "api-response-cache"
version: "1.0.0"
description: "Cache system for API responses with different TTL strategies"

modules:
  - module: cache
    with:
      capacity: 2000
      default_ttl: 600  # 10 minutes default
  - module: log

steps:
  # Cache data that changes frequently (short TTL)
  - use: cache
    input:
      action: set
      key: "api:users:list"
      value:
        data:
          - {id: 1, name: "Alice", status: "active"}
          - {id: 2, name: "Bob", status: "inactive"}
          - {id: 3, name: "Charlie", status: "active"}
        metadata:
          total_count: 3
          page: 1
          cached_at: "2025-08-06T23:10:00Z"
        query_time_ms: 245
      ttl: 300  # 5 minutes - rapidly changing data

  - use: log
    input:
      level: info
      message: "📋 User list cached (TTL: 5 min)"

  # Cache individual profile (medium TTL)
  - use: cache
    input:
      action: set
      key: "api:user:42"
      value:
        id: 42
        name: "Alice Johnson"
        email: "alice@example.com"
        profile:
          bio: "Software Developer"
          location: "New York, NY"
          joined: "2023-01-15"
        preferences:
          theme: "dark"
          notifications: true
      ttl: 1800  # 30 minutes - profile data

  - use: log
    input:
      level: info
      message: "👤 User profile cached (TTL: 30 min)"

  # Cache computed statistics (long TTL)
  - use: cache
    input:
      action: set
      key: "api:stats:daily"
      value:
        date: "2025-08-06"
        statistics:
          total_users: 15247
          active_users: 8934
          new_registrations: 127
          page_views: 45892
          api_calls: 12456
        computed_at: "2025-08-06T23:10:00Z"
        computation_time_ms: 1850
      ttl: 86400  # 24 hours - daily statistics

  - use: log
    input:
      level: info
      message: "📊 Daily statistics cached (TTL: 24h)"

  # Cache configuration (very long TTL)
  - use: cache
    input:
      action: set
      key: "api:config:app"
      value:
        version: "2.1.0"
        features:
          dark_mode: true
          notifications: true
          analytics: true
          beta_features: false
        limits:
          max_file_size_mb: 10
          max_requests_per_hour: 1000
        endpoints:
          - {path: "/api/users", methods: ["GET", "POST"]}
          - {path: "/api/users/:id", methods: ["GET", "PUT", "DELETE"]}
      ttl: 604800  # 7 days - application configuration

  - use: log
    input:
      level: info
      message: "⚙️ App configuration cached (TTL: 7 days)"

  # Simulate cache hit for user list
  - use: cache
    input:
      action: get
      key: "api:users:list"

  - assert: !phs payload.found
    then:
      - use: log
        input:
          level: info
          message: !phs `✅ Cache HIT: List with ${payload.value.metadata.total_count} users`
      - use: log
        input:
          level: info  
          message: !phs `⏱️ Original query took ${payload.value.query_time_ms}ms`

  # List all API caches
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "api:"
      order: "asc"

  - use: log
    input:
      level: info
      message: !phs `📂 Total of ${payload.total_count} API responses in cache`

  # Invalidate specific user cache (after update)
  - use: cache
    input:
      action: remove
      key: "api:user:42"

  - assert: !phs payload.removed
    then:
      - use: log
        input:
          level: info
          message: "🗑️ User 42 cache invalidated (e.g., after update)"

  # Check cache miss after invalidation
  - use: cache
    input:
      action: get
      key: "api:user:42"

  - assert: !phs !payload.found
    then:
      - use: log
        input:
          level: info
          message: "✅ Confirmed: Cache invalidated correctly"
      - use: log
        input:
          level: info
          message: "💡 Next API call will query the database"

  # Performance statistics
  - use: cache
    input:
      action: stats

  - use: log
    input:
      level: info
      message: !phs `📨 Hit rate: ${payload.stats.hit_rate.toFixed(1)}%, Memory: ${(payload.stats.memory_usage/1024).toFixed(1)}KB`

  - return:
      message: "API cache system processed"
      hit_rate: !phs payload.stats.hit_rate
      memory_usage_kb: !phs (payload.stats.memory_usage/1024).toFixed(1)
      categories_cached:
        - "User list (TTL: 5 min)"
        - "Individual profiles (TTL: 30 min)"
        - "Daily statistics (TTL: 24h)"
        - "Configuration (TTL: 7 days)"
```

## 🔍 TTL Strategies

### Short TTL (1-10 minutes)
**Ideal for**: Data that changes frequently
```phlow
ttl: 300  # 5 minutes
# Examples: user lists, real-time status, quotes
```

### Medium TTL (30-60 minutes)  
**Ideal for**: User-specific data
```phlow
ttl: 1800  # 30 minutes
# Examples: user profiles, preferences, sessions
```

### Long TTL (hours)
**Ideal for**: Computed/aggregated data
```phlow
ttl: 86400  # 24 hours
# Examples: reports, statistics, dashboards
```

### Very Long TTL (days)
**Ideal for**: Configuration and static data
```phlow
ttl: 604800  # 7 days
# Examples: app configuration, feature flags, metadata
```

## 📨 Monitoring and Statistics

### Available Metrics

```phlow
- use: cache
  input:
    action: stats

# Returns:
# {
#   "stats": {
#     "size": 150,              // Current items in cache
#     "capacity": 1000,         // Maximum capacity
#     "hit_rate": 85.4,         // Success rate (%)
#     "memory_usage": 33024,    // Estimated memory usage (bytes)
#     "total_gets": 500,        // Total get operations
#     "total_hits": 427,        // Total cache hits
#     "total_sets": 150,        // Total set operations
#     "total_removes": 23       // Total remove operations
#   }
# }
```

### Interpreting Metrics

- **Hit Rate**: Cache success rate (higher is better)
  - `> 80%`: Excellent performance
  - `60-80%`: Good performance  
  - `< 60%`: Consider adjusting TTL or capacity

- **Memory Usage**: Estimated memory usage
  - ~220 bytes per stored item
  - Monitor to avoid excessive consumption

- **Size vs Capacity**: Cache utilization
  - If close to capacity, old items will be removed (LRU)

## ⚡ Performance and Best Practices

### Operation Complexity

- **Get Operations**: O(1) - Constant time
- **Set Operations**: O(log n) - Sorted insertion
- **List Operations**: O(n) - With filters applied
- **Exists Operations**: O(1) - Constant time
- **Remove Operations**: O(1) - Constant time

### Key Naming Patterns

```phlow
# ✅ Good patterns
"user:123"              # User data
"session:abc123"        # User session
"api:users:list"        # API list
"api:user:123"          # Specific API user
"config:feature_flags"  # Configuration
"stats:daily:2025-08-06" # Statistics by date

# ❌ Avoid
"userdata"              # Too generic
"temp123"               # Not descriptive
"a:b:c:d:e:f"          # Too deeply nested
```

### Recommended Configurations

#### Development
```phlow
modules:
  - module: cache
    with:
      capacity: 100
      default_ttl: 300
```

#### Staging
```phlow
modules:
  - module: cache
    with:
      capacity: 1000
      default_ttl: 600
```

#### Production
```phlow
modules:
  - module: cache
    with:
      capacity: 10000
      default_ttl: 1800
```

## 🧪 Tests

### Available Test Types

#### 1. Unit Tests (Rust)
```bash
# Run module unit tests
cd modules/cache
cargo test

# Expected result: 8 tests passed
# - Input parsing tests (CacheInput)
# - Statistics tests (CacheStats)
# - Parameter and action validation
```

#### 2. Basic Functional Tests
```bash
# Simple linear test with fundamental operations
phlow modules/cache/test-basic.phlow

# Coverage:
# - Set/Get operations with different data types
# - Exists, Remove, Clear operations
# - List and Stats operations
# - Basic TTL
```

#### 3. Complete Functional Tests
```bash
# Comprehensive test with advanced cases
phlow modules/cache/test-complete.phlow

# Coverage:
# - Filters (prefix, suffix, pattern)
# - Pagination (limit/offset)
# - Sorting (asc/desc)
# - Complex objects and arrays
# - TTL with different strategies
# - Edge cases (nonexistent keys)
```

#### 4. Real Use Examples
```bash
# User session system
phlow examples/cache/user-sessions.phlow

# API cache system (in development)
phlow examples/cache/api-data-cache.phlow
```

### Run All Tests

```bash
# Run unit tests
cd modules/cache && cargo test

# Run functional tests
phlow modules/cache/test-basic.phlow
phlow modules/cache/test-complete.phlow

# Run practical examples
phlow examples/cache/user-sessions.phlow
```

### Test Results

**✅ Current Status**: All tests passed
- **Unit tests**: 8/8 ✅
- **Functional tests**: 2/2 ✅
- **Practical examples**: 1/1 ✅
- **Coverage**: ~95% of functionality

## 🚨 Error Handling

### Empty Key Error
```phlow
# Invalid input
input:
  action: set
  key: ""           # ❌ Empty key
  value: "test"

# Response
{
  "success": false,
  "error": "Key cannot be empty for set action"
}
```

### Invalid Action Error
```phlow
# Invalid input
input:
  action: "invalid"   # ❌ Unsupported action

# Response
{
  "success": false,
  "error": "Invalid action 'invalid'. Must be one of: set, get, remove, clear, exists, list, cleanup, stats"
}
```

### Cache Miss (Not an error)
```phlow
# Valid input
input:
  action: get
  key: "nonexistent"

# Response (success, but item not found)
{
  "success": true,
  "found": false,
  "key": "nonexistent",
  "value": null
}
```

## 🔗 Integration with Other Modules

### With HTTP Server
```phlow
modules:
  - module: http_server
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800

steps:
  # Check cache before processing request
  - use: cache
    input:
      action: get
      key: !phs `api:${request.path}`
  
  - assert: !phs payload.found
    then:
      # Cache hit - return cached data
      - return: !phs payload.value
    else:
      # Cache miss - process and store
      # ... processing logic ...
      - use: cache
        input:
          action: set
          key: !phs `api:${request.path}`
          value: !phs processed_data
          ttl: 600
      - return: !phs processed_data
```

### With Database (PostgreSQL)
```phlow
modules:
  - module: postgres
  - module: cache

steps:
  # Try to fetch from cache first
  - use: cache
    input:
      action: get
      key: "expensive_query_results"
  
  - assert: !phs !payload.found
    then:
      # Cache miss - execute database query
      - use: postgres
        input:
          query: "SELECT * FROM complex_view WHERE conditions..."
      
      # Store result in cache
      - use: cache
        input:
          action: set
          key: "expensive_query_results"
          value: !phs payload
          ttl: 3600
      
      - return: !phs payload
    else:
      # Cache hit - return cached data
      - return: !phs payload.value
```

## 🏷️ Tags

- cache
- memory
- storage
- performance
- ttl
- lru
- quickleaf
- high-performance

---

**Version**: 0.1.0  
**Author**: Philippe Assis \<codephilippe@gmail.com\>
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
