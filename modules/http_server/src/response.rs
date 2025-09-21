use crate::setup::CorsConfig;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use phlow_sdk::prelude::*;
use std::collections::HashMap;

#[derive(ToValue)]
pub struct ResponseHandler {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ResponseHandler {
    pub fn build(&self) -> Response<Full<Bytes>> {
        let response_builder = Response::builder().status(self.status_code);
        let response_builder = self
            .headers
            .iter()
            .fold(response_builder, |builder, (key, value)| {
                builder.header(key, value)
            });

        match response_builder.body(Full::new(Bytes::from(self.body.clone()))) {
            Ok(response) => response,
            Err(e) => {
                log::error!("Error creating response: {:?}", e);
                Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Internal Server Error"}"#,
                    )))
                    .expect("Failed to build response")
            }
        }
    }

    /// Apply CORS headers to the response based on the request origin
    pub fn apply_cors_headers(&mut self, cors_config: Option<&CorsConfig>, origin: Option<&str>) {
        // If no CORS config is provided, don't apply any CORS headers
        let cors_config = match cors_config {
            Some(config) => config,
            None => return,
        };
        // Handle Access-Control-Allow-Origin
        let allowed_origin = if cors_config.origins.contains(&"*".to_string()) {
            "*".to_string()
        } else if let Some(origin) = origin {
            if cors_config.origins.iter().any(|allowed| {
                allowed == origin
                    || (allowed.starts_with("http") && origin.starts_with(allowed))
                    || allowed == "*"
            }) {
                origin.to_string()
            } else {
                // Origin not allowed, don't set CORS headers
                return;
            }
        } else {
            cors_config
                .origins
                .get(0)
                .unwrap_or(&"*".to_string())
                .clone()
        };

        self.headers
            .insert("access-control-allow-origin".to_string(), allowed_origin);

        // Access-Control-Allow-Methods
        self.headers.insert(
            "access-control-allow-methods".to_string(),
            cors_config.methods.join(", "),
        );

        // Access-Control-Allow-Headers
        self.headers.insert(
            "access-control-allow-headers".to_string(),
            cors_config.headers.join(", "),
        );

        // Access-Control-Allow-Credentials
        if cors_config.credentials {
            self.headers.insert(
                "access-control-allow-credentials".to_string(),
                "true".to_string(),
            );
        }

        // Access-Control-Max-Age (for preflight requests)
        self.headers.insert(
            "access-control-max-age".to_string(),
            cors_config.max_age.to_string(),
        );
    }

    /// Create a preflight CORS response
    pub fn create_preflight_response(
        cors_config: Option<&CorsConfig>,
        origin: Option<&str>,
    ) -> Self {
        let mut response = Self {
            status_code: 200,
            headers: HashMap::new(),
            body: String::new(),
        };

        response.apply_cors_headers(cors_config, origin);
        response
    }
}

impl From<Value> for ResponseHandler {
    fn from(value: Value) -> Self {
        let status_code = match value.get("status_code") {
            Some(Value::Number(n)) => n.to_i64().unwrap_or(200) as u16,
            _ => 200,
        };

        let mut headers: HashMap<_, _> = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        if let Some(Value::Object(obj)) = value.get("headers") {
            obj.iter().for_each(|(key, value)| {
                let key = key.to_string().to_lowercase();
                let value = value.to_string();
                headers.insert(key, value);
            });
        };

        let body = match value.get("body") {
            Some(value) => value.to_json(JsonMode::Inline),
            _ => "".to_string(),
        };

        Self {
            status_code,
            headers,
            body,
        }
    }
}
