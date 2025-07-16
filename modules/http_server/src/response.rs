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
