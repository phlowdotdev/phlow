use phlow_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// JWT input actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum JwtInput {
    #[serde(rename = "create")]
    Create {
        data: Option<Value>,
        expires_in: Option<u64>,
    },
    #[serde(rename = "verify")]
    Verify { token: String },
}

impl TryFrom<Option<Value>> for JwtInput {
    type Error = String;

    fn try_from(input_value: Option<Value>) -> Result<Self, Self::Error> {
        let input_value = input_value.ok_or("Missing input for JWT module")?;

        if !input_value.is_object() {
            return Err("JWT input must be an object".to_string());
        }

        // Extract action (required)
        let action = match input_value.get("action") {
            Some(Value::String(s)) => s.as_string(),
            Some(v) => v.to_string(),
            None => return Err("Missing required 'action' field in JWT input".to_string()),
        };

        match action.as_str() {
            "create" => {
                let data: Option<Value> = input_value.get("data").cloned();
                let expires_in = input_value.get("expires_in").and_then(|v| v.to_u64());
                Ok(JwtInput::Create { data, expires_in })
            }
            "verify" => {
                let token = match input_value.get("token") {
                    Some(Value::String(s)) => s.as_string(),
                    Some(v) => v.to_string(),
                    None => {
                        return Err("Missing required 'token' field for verify action".to_string())
                    }
                };

                if token.is_empty() {
                    return Err("Token cannot be empty for verify action".to_string());
                }

                Ok(JwtInput::Verify { token })
            }
            _ => Err(format!(
                "Invalid action '{}'. Must be 'create' or 'verify'",
                action
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_input_create_with_data() {
        let value = json!({
            "action": "create",
            "data": {
                "user_id": 123,
                "email": "user@example.com"
            }
        });

        let input = JwtInput::try_from(Some(value)).unwrap();
        match input {
            JwtInput::Create { data, expires_in } => {
                assert!(data.is_some());
                let data = data.unwrap();
                let user_id = data.get("user_id").unwrap().to_i64().unwrap();
                let email = data.get("email").unwrap().as_str();
                assert_eq!(expires_in, None);
                assert_eq!(user_id, 123i64);
                assert_eq!(email, "user@example.com");
            }
            _ => panic!("Expected Create variant"),
        }
    }

    #[test]
    fn test_jwt_input_create_without_data() {
        let value = json!({
            "action": "create",
            "expires_in": 3600
        });

        let input = JwtInput::try_from(Some(value)).unwrap();
        match input {
            JwtInput::Create { data, expires_in } => {
                assert!(data.is_none());
                assert_eq!(expires_in, Some(3600));
            }
            _ => panic!("Expected Create variant"),
        }
    }

    #[test]
    fn test_jwt_input_verify_with_token() {
        let value = json!({
            "action": "verify",
            "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test"
        });

        let input = JwtInput::try_from(Some(value)).unwrap();
        match input {
            JwtInput::Verify { token } => {
                assert_eq!(token, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test");
            }
            _ => panic!("Expected Verify variant"),
        }
    }

    #[test]
    fn test_jwt_input_verify_missing_token() {
        let value = json!({
            "action": "verify"
        });

        let result = JwtInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required 'token'"));
    }

    #[test]
    fn test_jwt_input_verify_empty_token() {
        let value = json!({
            "action": "verify",
            "token": ""
        });

        let result = JwtInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token cannot be empty"));
    }

    #[test]
    fn test_jwt_input_missing_action() {
        let value = json!({
            "data": {"user_id": 123}
        });

        let result = JwtInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required 'action'"));
    }

    #[test]
    fn test_jwt_input_invalid_action() {
        let value = json!({
            "action": "invalid",
            "data": {"user_id": 123}
        });

        let result = JwtInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid action 'invalid'"));
    }

    #[test]
    fn test_jwt_input_missing_input() {
        let result = JwtInput::try_from(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing input"));
    }

    #[test]
    fn test_jwt_input_invalid_input_type() {
        let value = json!("not-an-object");

        let result = JwtInput::try_from(Some(value));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be an object"));
    }
}
