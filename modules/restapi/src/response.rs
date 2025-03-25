use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use sdk::prelude::*;
use sdk::tracing::error;
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
                error!("Error creating response: {:?}", e);
                Response::builder()
                    .status(500)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"error": "Internal Server Error"}"#,
                    )))
                    .expect("Failed to build response")
            }
        }
    }
}

impl From<Value> for ResponseHandler {
    fn from(value: Value) -> Self {
        let status_code = match value.get("status_code") {
            Some(Value::Number(n)) => n.to_i64().unwrap_or(200) as u16,
            _ => 200,
        };

        let headers = match value.get("headers") {
            Some(Value::Object(obj)) => obj
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            _ => {
                let mut map = HashMap::new();
                map.insert("Content-Type".to_string(), "application/json".to_string());
                map
            }
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
