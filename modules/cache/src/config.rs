use phlow_sdk::prelude::*;

/// Configuration for the cache module
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub capacity: usize,
    pub default_ttl: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            default_ttl: None,
        }
    }
}

impl TryFrom<&Value> for CacheConfig {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let mut config = CacheConfig::default();

        if let Some(capacity) = value.get("capacity") {
            match capacity.to_i64() {
                Some(cap) if cap > 0 => {
                    config.capacity = cap as usize;
                }
                Some(_) => {
                    return Err("Capacity must be a positive number".to_string());
                }
                None => {
                    return Err("Invalid capacity value".to_string());
                }
            }
        }

        if let Some(ttl) = value.get("default_ttl") {
            match ttl.to_i64() {
                Some(ttl_value) if ttl_value > 0 => {
                    config.default_ttl = Some(ttl_value as u64);
                }
                Some(_) => {
                    return Err("Default TTL must be a positive number".to_string());
                }
                None => {
                    return Err("Invalid default_ttl value".to_string());
                }
            }
        }

        Ok(config)
    }
}
