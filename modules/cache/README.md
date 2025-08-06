# Phlow Cache Module

A high-performance in-memory cache module for Phlow, powered by [QuickLeaf](https://github.com/phlowdotdev/quickleaf).

## 🚀 Features

- **High Performance**: O(1) access time for get/set operations
- **TTL Support**: Automatic expiration with configurable Time To Live
- **LRU Eviction**: Automatic removal of least recently used items when capacity is reached
- **Advanced Filtering**: Prefix, suffix, and pattern matching for listing operations
- **Ordered Results**: Ascending/descending ordering with pagination support
- **Real-time Statistics**: Hit rate, memory usage, and operation counters
- **Thread Safety**: Safe for concurrent access across multiple Phlow flows
- **Action-Based API**: Multiple operations through a single module interface

## 📦 Installation

The cache module is automatically available when you include it in your Phlow configuration. No manual installation required.

## 🔧 Configuration

Configure the cache module in your `with` section:

```yaml
modules:
  - module: cache
    with:
      capacity: 1000        # Maximum number of items (default: 1000)
      default_ttl: 3600     # Default TTL in seconds (optional)
      enable_events: false  # Enable cache operation events (default: false)
```

## 📖 Usage Examples

### Basic Cache Operations

#### Store Data

```yaml
steps:
  - use: cache
    input:
      action: set
      key: "user:123"
      value:
        name: "Alice"
        role: "admin"
        email: "alice@example.com"
      ttl: 3600  # Optional TTL in seconds
```

#### Retrieve Data

```yaml
steps:
  - use: cache
    input:
      action: get
      key: "user:123"
  
  - assert: !phs payload.found
    then:
      return: !phs payload.value
    else:
      return: { error: "User not found" }
```

#### Check if Key Exists

```yaml
steps:
  - use: cache
    input:
      action: exists
      key: "user:123"
  
  - return: !phs `User exists: ${payload.found}`
```

### Advanced Operations

#### Remove Items

```yaml
steps:
  - use: cache
    input:
      action: remove
      key: "user:123"
  
  - return: !phs `Removed: ${payload.removed}`
```

#### Clear All Cache

```yaml
steps:
  - use: cache
    input:
      action: clear
  
  - return: !phs `Cleared cache, removed ${payload.previous_size} items`
```

### Filtering and Listing

#### List All Items

```yaml
steps:
  - use: cache
    input:
      action: list
      order: "asc"
      limit: 10
      offset: 0
```

#### Filter by Prefix

```yaml
steps:
  - use: cache
    input:
      action: list
      filter_type: "prefix"
      filter_prefix: "user:"
      order: "desc"
      limit: 20
```

#### Filter by Suffix

```yaml
steps:
  - use: cache
    input:
      action: list
      filter_type: "suffix"
      filter_suffix: ":session"
      order: "asc"
```

#### Pattern Matching (Prefix + Suffix)

```yaml
steps:
  - use: cache
    input:
      action: list
      filter_type: "pattern"
      filter_prefix: "cache_"
      filter_suffix: "_data"
      limit: 50
```

### Maintenance Operations

#### Cleanup Expired Items

```yaml
steps:
  - use: cache
    input:
      action: cleanup
  
  - return: !phs `Cleaned up ${payload.cleaned_count} expired items`
```

#### Get Cache Statistics

```yaml
steps:
  - use: cache
    input:
      action: stats
  
  - return: !phs payload.stats
```

## 🎯 Complete Example: User Session Cache

```yaml
name: User Session Management
version: 1.0.0
description: Complete example of cache usage for user sessions

modules:
  - module: cache
    with:
      capacity: 5000
      default_ttl: 1800  # 30 minutes default session timeout
  - module: log

steps:
  # Store user session
  - use: cache
    input:
      action: set
      key: !phs `session:${main.user_id}`
      value:
        user_id: !phs main.user_id
        username: !phs main.username
        role: !phs main.role
        login_time: !phs timestamp()
      ttl: 7200  # 2 hours for this specific session

  - use: log
    input:
      level: info
      message: !phs `Session created for user ${main.username}`

  # Retrieve session
  - use: cache
    input:
      action: get
      key: !phs `session:${main.user_id}`

  - assert: !phs payload.found
    then:
      steps:
        - use: log
          input:
            level: info
            message: !phs `Session found for user ${payload.value.username}`
        
        # List all active sessions
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
            message: !phs `Total active sessions: ${payload.total_count}`
        
        # Get cache statistics
        - use: cache
          input:
            action: stats
        
        - return:
            session: !phs steps[1].value  # session data
            active_sessions: !phs steps[3].total_count
            cache_stats: !phs payload.stats
    else:
      steps:
        - use: log
          input:
            level: warn
            message: !phs `Session not found for user ID ${main.user_id}`
        
        - return:
            error: "Session not found or expired"
```

## 📊 Action Reference

### Set Action
- **Purpose**: Store a key-value pair in cache
- **Required**: `key`, `value`
- **Optional**: `ttl` (in seconds)

### Get Action
- **Purpose**: Retrieve a value by key
- **Required**: `key`
- **Returns**: `found` (boolean), `value` (if found)

### Remove Action
- **Purpose**: Remove a key-value pair from cache
- **Required**: `key`
- **Returns**: `removed` (boolean)

### Clear Action
- **Purpose**: Clear all items from cache
- **Returns**: `previous_size` (number of items removed)

### Exists Action
- **Purpose**: Check if a key exists in cache
- **Required**: `key`
- **Returns**: `found` (boolean)

### List Action
- **Purpose**: List cache entries with filtering and ordering
- **Optional**: `filter_type`, `filter_prefix`, `filter_suffix`, `order`, `limit`, `offset`
- **Returns**: `items` (array), `total_count`, `has_more`

### Cleanup Action
- **Purpose**: Manually clean up expired items
- **Returns**: `cleaned_count` (number of items removed)

### Stats Action
- **Purpose**: Get cache statistics and information
- **Returns**: `stats` object with size, capacity, hit_rate, memory_usage, etc.

## 🔍 Output Structure

All cache operations return a standardized response:

```json
{
  "success": true,
  "error": "Error message (if failed)",
  
  // Action-specific fields
  "found": true,           // For get/exists
  "value": {...},          // For get
  "removed": true,         // For remove
  "cached": true,          // For set
  "cleaned_count": 5,      // For cleanup
  "items": [...],          // For list
  "total_count": 100,      // For list
  "has_more": false,       // For list
  "stats": {...}           // For stats
}
```

## 📝 Examples

Comprehensive examples are available in the `examples/cache/` directory:

- **`examples/cache/basic-usage.phlow`** - Complete feature demonstration
- **`examples/cache/simple-test.phlow`** - Basic test suite (12 tests)
- **`examples/cache/comprehensive-test.phlow`** - Advanced tests (23 tests)
- **`examples/cache/user-sessions.phlow`** - Session management example
- **`examples/cache/api-data-cache.phlow`** - API caching with TTL strategies

### Running Examples

```bash
# Basic usage demonstration
phlow examples/cache/basic-usage.phlow

# Run test suites
phlow --test examples/cache/simple-test.phlow
phlow --test examples/cache/comprehensive-test.phlow

# Real-world use cases
phlow examples/cache/user-sessions.phlow
phlow examples/cache/api-data-cache.phlow

# Test specific operations
phlow --test --test-filter "string" examples/cache/simple-test.phlow
```

See the [Examples README](../../examples/cache/README.md) for detailed documentation.

## ⚡ Performance Notes

- **Get Operations**: O(1) average time complexity
- **Set Operations**: O(log n) due to ordered insertion
- **List Operations**: O(n) with filtering applied
- **Memory Usage**: Approximately 220 bytes per cached item
- **Thread Safety**: Uses Arc<Mutex> for safe concurrent access

## 🏷️ Tags

- cache
- memory
- storage
- performance
- ttl
- lru

---

**Version**: 0.1.0  
**Author**: Philippe Assis <codephilippe@gmail.com>  
**License**: MIT  
**Repository**: https://github.com/phlowdotdev/phlow
