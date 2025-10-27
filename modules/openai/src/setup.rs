use openai_api_rust::Auth;
use phlow_sdk::prelude::{ToValueBehavior, Value};

pub struct Setup {
    pub auth: Auth,
    pub model: Value,
    pub proxy: Option<String>,
    pub api_url: String,
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
                proxy: None,
                api_url: "https://api.openai.com/v1/".to_string(),
            });
        }

        let auth: Auth = {
            if let Some(api_key) = value.get("api_key") {
                Auth::new(api_key.to_string().as_str())
            } else if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                Auth::new(&api_key)
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

        let proxy = value.get("proxy").map(|v| v.to_string());

        let api_url = value
            .get("api_url")
            .map(|v| v.to_string())
            .unwrap_or("https://api.openai.com/v1/".to_string());

        Ok(Setup {
            auth,
            model,
            proxy,
            api_url,
        })
    }
}
