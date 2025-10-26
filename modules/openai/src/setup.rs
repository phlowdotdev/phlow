use openai_api_rust::Auth;
use phlow_sdk::prelude::{ToValueBehavior, Value};

pub struct Setup {
    pub auth: Auth,
    pub model: Value,
}

impl TryFrom<Value> for Setup {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if value.is_null() {
            let auth = if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                Auth::new(&key)
            } else {
                return Err(
                    "OpenAI API key not provided in config or OPENAI_API_KEY env variable".into(),
                );
            };

            return Ok(Setup {
                auth,
                model: "gpt-5-mini".to_value(),
            });
        }

        let auth: Auth = {
            if let Some(key) = value.get("api_key") {
                Auth::new(key.to_string().as_str())
            } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                Auth::new(&key)
            } else {
                return Err(
                    "OpenAI API key not provided in config or OPENAI_API_KEY env variable".into(),
                );
            }
        };

        let model = value
            .get("model")
            .map(|v| v.clone())
            .unwrap_or("gpt-5-mini".to_value());

        Ok(Setup { auth, model })
    }
}
