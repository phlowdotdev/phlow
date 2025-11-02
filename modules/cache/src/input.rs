use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Cache input actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum CacheInput {
    #[serde(rename = "set")]
    Set {
        key: String,
        value: Value,
        ttl: Option<u64>,
    },
    #[serde(rename = "get")]
    Get { key: String },
    #[serde(rename = "remove")]
    Remove { key: String },
    #[serde(rename = "clear")]
    Clear,
    #[serde(rename = "exists")]
    Exists { key: String },
    #[serde(rename = "list")]
    List {
        filter_type: String,
        filter_value: Option<String>,
        filter_prefix: Option<String>,
        filter_suffix: Option<String>,
        order: String,
        limit: Option<u64>,
        offset: u64,
    },
    #[serde(rename = "cleanup")]
    Cleanup,
    #[serde(rename = "stats")]
    Stats,
}

impl TryFrom<Option<Value>> for CacheInput {
    type Error = String;

    fn try_from(input_value: Option<Value>) -> Result<Self, Self::Error> {
        let input_value = input_value.ok_or("Missing input for cache module")?;

        if !input_value.is_object() {
            return Err("Cache input must be an object".to_string());
        }

        // Extract action (required)
        let action = match input_value.get("action") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'action' field in cache input".to_string()),
        };

        match action.as_str() {
            "set" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for set action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for set action".to_string());
                }

                let value = input_value
                    .get("value")
                    .ok_or("Missing 'value' field for set action")?
                    .clone();

                let ttl = input_value.get("ttl").and_then(|v| v.to_u64());

                Ok(CacheInput::Set { key, value, ttl })
            }
            "get" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for get action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for get action".to_string());
                }

                Ok(CacheInput::Get { key })
            }
            "remove" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for remove action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for remove action".to_string());
                }

                Ok(CacheInput::Remove { key })
            }
            "clear" => Ok(CacheInput::Clear),
            "exists" => {
                let key = input_value
                    .get("key")
                    .ok_or("Missing 'key' field for exists action")?
                    .to_string();

                if key.is_empty() {
                    return Err("Key cannot be empty for exists action".to_string());
                }

                Ok(CacheInput::Exists { key })
            }
            "list" => {
                let filter_type = input_value
                    .get("filter_type")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "none".to_string());

                let filter_value = input_value.get("filter_value").map(|v| v.to_string());
                let filter_prefix = input_value.get("filter_prefix").map(|v| v.to_string());
                let filter_suffix = input_value.get("filter_suffix").map(|v| v.to_string());

                let order = input_value
                    .get("order")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "asc".to_string());

                let limit = input_value.get("limit").and_then(|v| v.to_u64());
                let offset = input_value
                    .get("offset")
                    .and_then(|v| v.to_u64())
                    .unwrap_or(0);

                Ok(CacheInput::List {
                    filter_type,
                    filter_value,
                    filter_prefix,
                    filter_suffix,
                    order,
                    limit,
                    offset,
                })
            }
            "cleanup" => Ok(CacheInput::Cleanup),
            "stats" => Ok(CacheInput::Stats),
            _ => Err(format!(
                "Invalid action '{}'. Must be one of: set, get, remove, clear, exists, list, cleanup, stats",
                action
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_input_set() {
        let value = json!({
            "action": "set",
            "key": "test_key",
            "value": "test_value",
            "ttl": 3600
        });

        let input = CacheInput::try_from(Some(value)).unwrap();
        match input {
            CacheInput::Set { key, value, ttl } => {
                assert_eq!(key, "test_key");
                assert_eq!(value.to_string(), "test_value");
                assert_eq!(ttl, Some(3600));
            }
            _ => panic!("Expected Set variant"),
        }
    }

    #[test]
    fn test_cache_input_get() {
        let value = json!({
            "action": "get",
            "key": "test_key"
        });

        let input = CacheInput::try_from(Some(value)).unwrap();
        match input {
            CacheInput::Get { key } => {
                assert_eq!(key, "test_key");
            }
            _ => panic!("Expected Get variant"),
        }
    }

    #[test]
    fn test_cache_input_list_with_filter() {
        let value = json!({
            "action": "list",
            "filter_type": "prefix",
            "filter_prefix": "user:",
            "order": "desc",
            "limit": 10,
            "offset": 5
        });

        let input = CacheInput::try_from(Some(value)).unwrap();
        match input {
            CacheInput::List {
                filter_type,
                filter_prefix,
                order,
                limit,
                offset,
                ..
            } => {
                assert_eq!(filter_type, "prefix");
                assert_eq!(filter_prefix, Some("user:".to_string()));
                assert_eq!(order, "desc");
                assert_eq!(limit, Some(10));
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_cache_input_invalid_action() {
        let value = json!({
            "action": "invalid",
            "key": "test_key"
        });

        let result = CacheInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid action 'invalid'"));
    }

    #[test]
    fn test_cache_input_missing_action() {
        let value = json!({
            "key": "test_key"
        });

        let result = CacheInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required 'action'"));
    }
}
