use std::collections::HashMap;

use phlow_sdk::prelude::*;

/// Configuration for the cache module
#[derive(Debug, Clone)]
pub struct OpenaiConfig {
    pub model: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub n: Option<usize>,
    pub stream: Option<bool>,
    pub stop: Option<Vec<String>>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub logit_bias: Option<HashMap<String, String>>,
    pub user: Option<String>,
}

impl Default for OpenaiConfig {
    fn default() -> Self {
        Self {
            model: "gtp-5-mini".to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

impl TryFrom<&Value> for OpenaiConfig {
    type Error = String;

    // Value is value3::Value and not serde_json::Value
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let mut config = OpenaiConfig::default();

        if let Some(model) = value.get("model") {
            config.model = model.to_string();
        }

        if let Some(max_tokens) = value.get("max_tokens") {
            config.max_tokens = Some(max_tokens.to_u64().unwrap() as usize);
        }

        if let Some(temperature) = value.get("temperature") {
            config.temperature = Some(temperature.to_f64().unwrap() as f32);
        }

        if let Some(top_p) = value.get("top_p") {
            config.top_p = Some(top_p.to_f64().unwrap() as f32);
        }

        if let Some(n) = value.get("n") {
            config.n = Some(n.to_u64().unwrap() as usize);
        }

        if let Some(stream) = value.get("stream") {
            config.stream = stream.as_bool().cloned();
        }

        if let Some(stop) = value.get("stop") {
            let stop_vec: Vec<String> = stop
                .as_array()
                .unwrap()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            config.stop = Some(stop_vec);
        }

        if let Some(presence_penalty) = value.get("presence_penalty") {
            config.presence_penalty = Some(presence_penalty.to_f64().unwrap() as f32);
        }

        if let Some(frequency_penalty) = value.get("frequency_penalty") {
            config.frequency_penalty = Some(frequency_penalty.to_f64().unwrap() as f32);
        }

        if let Some(logit_bias) = value.get("logit_bias") {
            let mut bias_map = HashMap::new();

            match logit_bias.as_object() {
                Some(obj) => {
                    for (k, v) in obj.iter() {
                        bias_map.insert(k.to_string(), v.to_string());
                    }
                }
                None => return Err("logit_bias must be a JSON object".to_string()),
            }

            config.logit_bias = Some(bias_map);
        }

        if let Some(user) = value.get("user") {
            config.user = Some(user.to_string());
        }

        Ok(config)
    }
}
