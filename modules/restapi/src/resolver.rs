use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{HeaderMap, Request, Response};
use sdk::{
    prelude::*,
    tracing::{error, info, Span},
};
use std::{collections::HashMap, convert::Infallible};

use crate::{middleware::RequestContext, response::ResponseHandler};

pub async fn proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let context = req
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .expect("RequestContext not found");

    let query = req.uri().query().unwrap_or_default().to_string();

    let headers = resolve_headers(req.headers().clone(), &context.span);
    let body = resolve_body(req);

    let query_params = resolve_query_params(&query);

    let query_params = query_params.await;
    let body = body.await;
    let headers = headers.await;

    let data = HashMap::from([
        ("client_ip", context.client_ip.to_value()),
        ("headers", headers.to_value()),
        ("method", context.method.to_value()),
        ("path", context.path.to_value()),
        ("query_string", query.to_value()),
        ("query_params", query_params),
        ("body", body),
    ])
    .to_value();

    info!("Received request: {:?}", data);

    let response_value = sender!(
        context.span.clone(),
        context.dispatch.clone(),
        context.id,
        context.sender,
        Some(data)
    )
    .await
    .unwrap_or(Value::Null);

    let response = ResponseHandler::from(response_value);

    context
        .span
        .record("http.response.status_code", response.status_code);
    context
        .span
        .record("http.response.body.size", response.body.len());

    Ok(response.build())
}

pub async fn resolve_query_params(query: &str) -> Value {
    let mut map = HashMap::new();

    for pair in query.split('&') {
        let mut parts = pair.split('=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();

        map.insert(key.to_string(), value.to_string());
    }

    map.to_value()
}

pub async fn resolve_body(req: Request<hyper::body::Incoming>) -> Value {
    let body_bytes: Bytes = match req.into_body().collect().await {
        Ok(full_body) => full_body.to_bytes(),
        Err(e) => {
            error!("Error reading request body: {:?}", e);
            Bytes::new()
        }
    };

    let body = match std::str::from_utf8(&body_bytes) {
        Ok(s) => {
            let s = s.trim().to_string();
            if s.starts_with('{') || s.starts_with('[') {
                Value::json_to_value(&s).unwrap_or_else(|_| s.to_value())
            } else {
                s.to_value()
            }
        }
        Err(e) => {
            error!("Error parsing request body: {:?}", e);
            "".to_string().to_value()
        }
    };

    body
}

pub async fn resolve_headers(headers: HeaderMap, span: &Span) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(key, value)| match value.to_str() {
            Ok(val_str) => {
                if key == "x-request-id" {
                    span.record("http.request.header.x_request_id", val_str);
                } else if key == "origin" {
                    span.record("http.request.header.origin", val_str);
                } else if key == "referer" {
                    span.record("http.request.header.referer", val_str);
                } else if key == "user-agent" {
                    span.record("http.request.header.user_agent", val_str);
                } else if key == "host" {
                    span.record("http.request.header.host", val_str);
                } else if key == "x-transaction-id" {
                    span.record("http.request.header.x_transaction_id", val_str);
                } else if key == "accept" {
                    span.record("http.request.header.accept", val_str);
                } else if key == "content-type" {
                    span.record("http.request.header.content_type", val_str);
                } else if key == "x-forwarded-for" {
                    span.record("http.request.header.x_forwarded_for", val_str);
                } else if key == "x-real-ip" {
                    span.record("http.request.header.x_real_ip", val_str);
                } else if key == "cache-control" {
                    span.record("http.request.header.cache_control", val_str);
                } else if key == "accept-encoding" {
                    span.record("http.request.header.accept_encoding", val_str);
                }

                Some((key.as_str().to_string(), val_str.to_string()))
            }
            Err(e) => {
                error!("Header value is not a valid UTF-8 string: {:?}", e);
                None
            }
        })
        .collect::<HashMap<String, String>>()
}
