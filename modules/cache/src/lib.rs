mod config;
mod input;
mod stats;

use config::CacheConfig;
use input::CacheInput;
use stats::CacheStats;
use phlow_sdk::prelude::*;
use quickleaf::{Quickleaf, Filter, ListProps, Order, Duration};
use std::sync::{Arc, Mutex};

create_step!(cache_handler(setup));

/// Global cache instance wrapped in Arc<Mutex> for thread safety
type CacheInstance = Arc<Mutex<Quickleaf>>;

/// Cache handler that manages a QuickLeaf cache instance
pub async fn cache_handler(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rx = module_channel!(setup);

    // Parse cache configuration from 'with' parameters
    let config = CacheConfig::try_from(&setup.with)?;
    log::debug!("Cache module started with config: {:?}", config);

    // Initialize cache instance
    let cache = if let Some(default_ttl) = config.default_ttl {
        Arc::new(Mutex::new(Quickleaf::with_default_ttl(
            config.capacity,
            Duration::from_secs(default_ttl),
        )))
    } else {
        Arc::new(Mutex::new(Quickleaf::new(config.capacity)))
    };

    // Initialize statistics
    let stats = Arc::new(Mutex::new(CacheStats::new()));

    for package in rx {
        let cache = cache.clone();
        let stats = stats.clone();

        // Parse input based on action
        let input = match CacheInput::try_from(package.input.clone()) {
            Ok(input) => input,
            Err(e) => {
                log::error!("Invalid cache input: {}", e);
                let response = std::collections::HashMap::from([
                    ("success", false.to_value()),
                    ("error", format!("Invalid input: {}", e).to_value()),
                ])
                .to_value();
                sender_safe!(package.sender, response.into());
                continue;
            }
        };

        log::debug!("Cache module received input: {:?}", input);

        // Process based on action
        let result = match input {
            CacheInput::Set { key, value, ttl } => handle_set(cache, stats, key, value, ttl).await,
            CacheInput::Get { key } => handle_get(cache, stats, key).await,
            CacheInput::Remove { key } => handle_remove(cache, stats, key).await,
            CacheInput::Clear => handle_clear(cache, stats).await,
            CacheInput::Exists { key } => handle_exists(cache, stats, key).await,
            CacheInput::List {
                filter_type,
                filter_value,
                filter_prefix,
                filter_suffix,
                order,
                limit,
                offset,
            } => {
                handle_list(
                    cache,
                    filter_type,
                    filter_value,
                    filter_prefix,
                    filter_suffix,
                    order,
                    limit,
                    offset,
                )
                .await
            }
            CacheInput::Cleanup => handle_cleanup(cache).await,
            CacheInput::Stats => handle_stats(cache, stats).await,
        };

        match result {
            Ok(response_value) => {
                log::debug!("Cache operation successful");
                sender_safe!(package.sender, response_value.into());
            }
            Err(e) => {
                log::error!("Cache operation failed: {}", e);
                let response = std::collections::HashMap::from([
                    ("success", false.to_value()),
                    ("error", e.to_string().to_value()),
                ])
                .to_value();
                sender_safe!(package.sender, response.into());
            }
        }
    }

    Ok(())
}

/// Handle set action
async fn handle_set(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
    value: Value,
    ttl: Option<u64>,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    // Convert phlow-sdk Value to quickleaf Value
    let quickleaf_value = quickleaf::valu3::value::Value::from(value.clone());
    
    if let Some(ttl_secs) = ttl {
        cache_guard.insert_with_ttl(&key, quickleaf_value, Duration::from_secs(ttl_secs));
    } else {
        cache_guard.insert(&key, quickleaf_value);
    }

    // Update statistics
    if let Ok(mut stats_guard) = stats.lock() {
        stats_guard.record_set();
    }

    log::debug!("Set key '{}' with value: {:?}", key, value);

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("key", key.to_value()),
        ("cached", true.to_value()),
    ])
    .to_value())
}

/// Handle get action
async fn handle_get(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    match cache_guard.get(&key) {
        Some(value) => {
            // Cache hit
            if let Ok(mut stats_guard) = stats.lock() {
                stats_guard.record_hit();
            }

            log::debug!("Cache hit for key '{}'", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("found", true.to_value()),
                ("key", key.to_value()),
                ("value", value.clone()),
            ])
            .to_value())
        }
        None => {
            // Cache miss
            if let Ok(mut stats_guard) = stats.lock() {
                stats_guard.record_miss();
            }

            log::debug!("Cache miss for key '{}'", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("found", false.to_value()),
                ("key", key.to_value()),
                ("value", Value::Null),
            ])
            .to_value())
        }
    }
}

/// Handle remove action
async fn handle_remove(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    match cache_guard.remove(&key) {
        Ok(()) => {
            // Update statistics
            if let Ok(mut stats_guard) = stats.lock() {
                stats_guard.record_remove();
            }

            log::debug!("Removed key '{}'", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("key", key.to_value()),
                ("removed", true.to_value()),
            ])
            .to_value())
        }
        Err(_) => {
            log::debug!("Key '{}' not found for removal", key);

            Ok(std::collections::HashMap::from([
                ("success", true.to_value()),
                ("key", key.to_value()),
                ("removed", false.to_value()),
                ("error", "Key not found".to_value()),
            ])
            .to_value())
        }
    }
}

/// Handle clear action
async fn handle_clear(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    let previous_size = cache_guard.len();
    cache_guard.clear();

    // Update statistics
    if let Ok(mut stats_guard) = stats.lock() {
        stats_guard.record_clear(previous_size);
    }

    log::debug!("Cleared cache, removed {} items", previous_size);

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("cleared", true.to_value()),
        ("previous_size", previous_size.to_value()),
    ])
    .to_value())
}

/// Handle exists action
async fn handle_exists(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
    key: String,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    let exists = cache_guard.contains_key(&key);

    // Update statistics (consider this a type of get operation)
    if let Ok(mut stats_guard) = stats.lock() {
        if exists {
            stats_guard.record_hit();
        } else {
            stats_guard.record_miss();
        }
    }

    log::debug!("Key '{}' exists: {}", key, exists);

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("key", key.to_value()),
        ("found", exists.to_value()),
    ])
    .to_value())
}

/// Handle list action
async fn handle_list(
    cache: CacheInstance,
    filter_type: String,
    filter_value: Option<String>,
    filter_prefix: Option<String>,
    filter_suffix: Option<String>,
    order: String,
    limit: Option<u64>,
    offset: u64,
) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    // Determine filter
    let filter = match filter_type.as_str() {
        "prefix" => {
            if let Some(prefix) = filter_prefix.or(filter_value) {
                Filter::StartWith(prefix)
            } else {
                Filter::None
            }
        }
        "suffix" => {
            if let Some(suffix) = filter_suffix.or(filter_value) {
                Filter::EndWith(suffix)
            } else {
                Filter::None
            }
        }
        "pattern" => {
            if let (Some(prefix), Some(suffix)) = (filter_prefix.as_ref(), filter_suffix.as_ref()) {
                Filter::StartAndEndWith(prefix.clone(), suffix.clone())
            } else {
                Filter::None
            }
        }
        _ => Filter::None,
    };

    // Determine order
    let list_order = match order.as_str() {
        "desc" => Order::Desc,
        _ => Order::Asc,
    };

    // Build list properties
    let list_props = ListProps::default().filter(filter).order(list_order);

    // Get items from cache
    let items = cache_guard
        .list(list_props)
        .map_err(|e| format!("List operation failed: {:?}", e))?;

    // Apply pagination
    let total_count = items.len();
    let start_idx = offset as usize;
    let end_idx = if let Some(limit) = limit {
        std::cmp::min(start_idx + (limit as usize), total_count)
    } else {
        total_count
    };

    let paginated_items: Vec<_> = items
        .iter()
        .skip(start_idx)
        .take(end_idx.saturating_sub(start_idx))
        .map(|(key, value)| {
            std::collections::HashMap::from([
                ("key", key.to_value()),
                ("value", (*value).clone()),
            ])
            .to_value()
        })
        .collect();

    let has_more = end_idx < total_count;

    log::debug!(
        "Listed {} items (total: {}, offset: {}, limit: {:?})",
        paginated_items.len(),
        total_count,
        offset,
        limit
    );

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("items", paginated_items.to_value()),
        ("total_count", total_count.to_value()),
        ("offset", offset.to_value()),
        ("limit", limit.to_value()),
        ("has_more", has_more.to_value()),
    ])
    .to_value())
}

/// Handle cleanup action
async fn handle_cleanup(cache: CacheInstance) -> Result<Value, String> {
    let mut cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;

    let cleaned_count = cache_guard.cleanup_expired();

    log::debug!("Cleaned up {} expired items", cleaned_count);

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("cleaned_count", cleaned_count.to_value()),
    ])
    .to_value())
}

/// Handle stats action
async fn handle_stats(
    cache: CacheInstance,
    stats: Arc<Mutex<CacheStats>>,
) -> Result<Value, String> {
    let cache_guard = cache
        .lock()
        .map_err(|e| format!("Cache lock error: {}", e))?;
    let stats_guard = stats
        .lock()
        .map_err(|e| format!("Stats lock error: {}", e))?;

    let current_size = cache_guard.len();
    let capacity = cache_guard.capacity();
    let hit_rate = stats_guard.get_hit_rate();
    let estimated_memory = estimate_memory_usage(current_size, capacity);

    log::debug!(
        "Cache stats - Size: {}, Capacity: {}, Hit rate: {:.2}%",
        current_size,
        capacity,
        hit_rate
    );

    let stats_map = std::collections::HashMap::from([
        ("size", current_size.to_value()),
        ("capacity", capacity.to_value()),
        ("hit_rate", hit_rate.to_value()),
        ("memory_usage", estimated_memory.to_value()),
        ("total_gets", stats_guard.get_total_gets().to_value()),
        ("total_hits", stats_guard.get_total_hits().to_value()),
        ("total_sets", stats_guard.get_total_sets().to_value()),
        ("total_removes", stats_guard.get_total_removes().to_value()),
    ])
    .to_value();

    Ok(std::collections::HashMap::from([
        ("success", true.to_value()),
        ("stats", stats_map),
    ])
    .to_value())
}

/// Estimate memory usage in bytes (rough calculation)
fn estimate_memory_usage(current_size: usize, capacity: usize) -> u64 {
    // Rough estimation:
    // - Base overhead: ~200 bytes per cache instance
    // - Per item: ~(average_key_size + average_value_size + 100) bytes
    // - Assume average key size: 20 bytes
    // - Assume average value size: 100 bytes
    let base_overhead = 200u64;
    let per_item_overhead = 220u64; // 20 + 100 + 100 (metadata)
    let capacity_overhead = (capacity as u64) * 8; // Vec capacity

    base_overhead + (current_size as u64 * per_item_overhead) + capacity_overhead
}
