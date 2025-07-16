use phlow_sdk::prelude::*;
use reqwest::header::{self, HeaderMap};
use reqwest::Method;

pub struct Input {
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<String>,
}
impl Input {
    pub fn new(value: Value, default_user_agent: &Option<String>) -> Self {
        let method = match value.get("method") {
            Some(Value::String(method)) => match method.as_str() {
                "GET" => Method::GET,
                "POST" => Method::POST,
                "PUT" => Method::PUT,
                "PATCH" => Method::PATCH,
                "OPTIONS" => Method::OPTIONS,
                "HEAD" => Method::HEAD,
                "TRACE" => Method::TRACE,
                "CONNECT" => Method::CONNECT,
                "DELETE" => Method::DELETE,
                _ => Method::GET,
            },
            _ => Method::GET,
        };

        let url = value.get("url").unwrap_or(&Value::Null).to_string();
        let mut headers: HeaderMap = HeaderMap::default();

        if let Some(Value::Object(header_map)) = value.get("headers") {
            for (key, value) in header_map.iter() {
                let key = key.to_string();
                let value = value.to_string();
                if let Ok(header_name) = header::HeaderName::from_bytes(key.as_bytes()) {
                    if let Ok(header_value) = header::HeaderValue::from_str(&value) {
                        headers.insert(header_name, header_value);
                    } else {
                        tracing::log::error!("Invalid header value: {}", value);
                    }
                } else {
                    tracing::log::error!("Invalid header name: {}", key);
                }
            }
        }

        if default_user_agent.is_some()
            && headers.get("User-Agent").is_none()
            && headers.get("user-agent").is_none()
        {
            let default_user_agent = default_user_agent.as_ref().unwrap();
            headers.insert(
                header::USER_AGENT,
                header::HeaderValue::from_str(default_user_agent).unwrap(),
            );
        }

        let body = value.get("body").map(|v| v.to_string());

        if let Some(body) = &body {
            if body.starts_with('{')
                && body.ends_with('}')
                && headers.get("Content-Type").is_none()
                && headers.get("content-type").is_none()
            {
                headers.insert(
                    "content-type",
                    header::HeaderValue::from_static("application/json"),
                );
            }
        }

        Input {
            method,
            url,
            headers,
            body,
        }
    }
}
