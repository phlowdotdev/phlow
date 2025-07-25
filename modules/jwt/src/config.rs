use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// JWT module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
        }
    }
}

impl TryFrom<Value> for JwtConfig {
    type Error = String;

    fn try_from(with_value: Value) -> Result<Self, Self::Error> {
        if !with_value.is_object() {
            return Err("JWT 'with' configuration must be an object".to_string());
        }

        // Extract secret (required)
        let secret = match with_value.get("secret") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'secret' in JWT configuration".to_string()),
        };

        if secret.is_empty() {
            return Err("JWT secret cannot be empty".to_string());
        }

        Ok(JwtConfig { secret })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_config_from_valid_value() {
        let value = json!({
            "secret": "test-secret-key"
        });

        let config = JwtConfig::try_from(value).unwrap();
        assert_eq!(config.secret, "test-secret-key");
    }

    #[test]
    fn test_jwt_config_from_minimal_value() {
        let value = json!({
            "secret": "test-secret-key"
        });

        let config = JwtConfig::try_from(value).unwrap();
        assert_eq!(config.secret, "test-secret-key");
    }

    #[test]
    fn test_jwt_config_missing_secret() {
        let value = json!({
            "expires_in": 7200
        });

        let result = JwtConfig::try_from(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required 'secret'"));
    }

    #[test]
    fn test_jwt_config_empty_secret() {
        let value = json!({
            "secret": "",
            "expires_in": 7200
        });

        let result = JwtConfig::try_from(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JWT secret cannot be empty"));
    }

    #[test]
    fn test_jwt_config_zero_expires_in() {
        let value = json!({
            "secret": "test-secret",
            "expires_in": 0
        });

        let result = JwtConfig::try_from(value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("expires_in must be greater than 0"));
    }

    #[test]
    fn test_jwt_config_invalid_with_type() {
        let value = json!("not-an-object");

        let result = JwtConfig::try_from(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be an object"));
    }
}
