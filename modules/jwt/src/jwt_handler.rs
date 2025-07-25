use crate::config::JwtConfig;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_EXPIRES_IN: u64 = 3600; // Default expiration time in seconds

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Issued at (timestamp)
    iat: i64,
    /// Expiration time (timestamp)  
    exp: i64,
    /// Custom data as map
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

/// JWT Handler for creating and verifying tokens
#[derive(Clone)]
pub struct JwtHandler {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtHandler {
    /// Create a new JWT handler
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            encoding_key,
            decoding_key,
        }
    }

    /// Create a new JWT token
    pub async fn create_token(
        &self,
        data: Option<Value>,
        expires_in: Option<u64>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let exp = now + chrono::Duration::seconds(expires_in.unwrap_or(DEFAULT_EXPIRES_IN) as i64);

        // Convert data to a HashMap<String, serde_json::Value>
        let data_map = match data {
            Some(Value::Object(map)) => {
                // Convert Object to HashMap<String, serde_json::Value>
                let mut result = HashMap::new();
                for (key, value) in map.iter() {
                    if let Ok(json_value) = serde_json::to_value(value) {
                        result.insert(key.to_string(), json_value);
                    }
                }
                result
            }
            Some(other) => {
                // If it's not an object, wrap it in a "data" field
                let mut map = HashMap::new();
                if let Ok(json_value) = serde_json::to_value(&other) {
                    map.insert("data".to_string(), json_value);
                }
                map
            }
            None => HashMap::new(),
        };

        let claims = Claims {
            iat: now.timestamp(),
            exp: exp.timestamp(),
            data: data_map,
        };

        log::debug!("Creating JWT with claims: {:?}", claims);

        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| format!("Failed to encode JWT: {}", e))?;

        log::debug!("JWT token created successfully");

        let result = HashMap::from([
            ("token", token.to_value()),
            ("expires_at", exp.to_rfc3339().to_value()),
            ("issued_at", now.to_rfc3339().to_value()),
        ])
        .to_value();
        Ok(result)
    }

    /// Verify a JWT token
    pub async fn verify_token(
        &self,
        token: String,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("Verifying JWT token");

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = false;

        match decode::<Claims>(&token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                log::debug!("JWT token verified successfully");

                let claims = token_data.claims;
                let mut data_map = claims.data;

                // Add standard claims if they don't exist
                if !data_map.contains_key("iat") {
                    data_map.insert(
                        "iat".to_string(),
                        serde_json::Value::Number(claims.iat.into()),
                    );
                }
                if !data_map.contains_key("exp") {
                    data_map.insert(
                        "exp".to_string(),
                        serde_json::Value::Number(claims.exp.into()),
                    );
                }

                // Convert back to phlow Value
                let json_obj = serde_json::Value::Object(data_map.into_iter().collect());
                let data_value = if let Ok(json_str) = serde_json::to_string(&json_obj) {
                    Value::json_to_value(&json_str).unwrap_or(Value::Null)
                } else {
                    Value::Null
                };

                let result = HashMap::from([
                    ("valid", true.to_value()),
                    ("data", data_value),
                    ("expired", false.to_value()),
                ])
                .to_value();
                Ok(result)
            }
            Err(err) => {
                log::warn!("JWT token verification failed: {}", err);

                let (expired, error_msg) = match err.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        (true, "Token has expired".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        (false, "Invalid token signature".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        (false, "Invalid token format".to_string())
                    }
                    _ => (false, format!("Token validation failed: {}", err)),
                };

                let result = HashMap::from([
                    ("valid", false.to_value()),
                    ("data", Value::Null),
                    ("error", error_msg.to_value()),
                    ("expired", expired.to_value()),
                ])
                .to_value();
                Ok(result)
            }
        }
    }
}
