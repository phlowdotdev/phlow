use crate::settings::{self, AuthorizationSpanMode, Settings};
use crate::{middleware::RequestContext, response::ResponseHandler};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use hyper::{HeaderMap, Request, Response};
use sdk::{
    prelude::*,
    tracing::{error, Span},
};
use std::sync::Arc;
use std::{collections::HashMap, convert::Infallible};

macro_rules! to_span_format {
    ($target:expr, $key:expr) => {{
        let key = $key.as_str().to_lowercase();
        format!($target, key).as_str()
    }};
}

pub async fn ping(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from("{\"status\": \"ok\"}")))
        .unwrap();

    Ok(response)
}

pub async fn proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let context = req
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .expect("RequestContext not found");

    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let body_size = req.body().size_hint().lower();
    let request_size = req.size_hint().lower();
    let query = req.uri().query().unwrap_or_default().to_string();

    let headers = resolve_headers(req.headers().clone(), &context.span, &context.settings);
    let body = resolve_body(req);
    let query_params = resolve_query_params(&query);

    context
        .span
        .record("otel.name", format!("{} {}", method, path));
    context.span.record("http.request.body.size", body_size);
    context.span.record("http.request.size", request_size);
    context.span.record("http.request.method", &method);
    context.span.record("http.request.path", &path);

    let query_params = query_params.await;
    let body = body.await;
    let headers = headers.await;

    let data = HashMap::from([
        ("client_ip", context.client_ip.to_value()),
        ("headers", headers),
        ("method", method.to_value()),
        ("path", path.to_value()),
        ("query_string", query.to_value()),
        ("query_params", query_params),
        ("body", body),
    ])
    .to_value();

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

    response.headers.iter().for_each(|(key, value)| {
        context
            .span
            .record(to_span_format!("http.response.header.{}", key), value);
    });

    Ok(response.build())
}

async fn resolve_query_params(query: &str) -> Value {
    let mut map = HashMap::new();

    for pair in query.split('&') {
        let mut parts = pair.split('=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();

        map.insert(key.to_string(), value.to_string());
    }

    map.to_value()
}

async fn resolve_body(req: Request<hyper::body::Incoming>) -> Value {
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

fn resolve_authorization(authorization: &str, mode: &AuthorizationSpanMode) -> String {
    match mode {
        AuthorizationSpanMode::None => "".to_string(),
        AuthorizationSpanMode::Hidden => "x".repeat(authorization.len()),
        AuthorizationSpanMode::Prefix => format!("{}...", &authorization[0..12]),
        AuthorizationSpanMode::Suffix => {
            format!("...{}", &authorization[authorization.len() - 6..])
        }
        AuthorizationSpanMode::All => authorization.to_string(),
    }
}

async fn resolve_headers(headers: HeaderMap, span: &Span, settings: &Arc<Settings>) -> Value {
    headers
        .iter()
        .filter_map(|(key, value)| match value.to_str() {
            Ok(val_str) => {
                if key == "authorization" {
                    let authorization =
                        resolve_authorization(val_str, &settings.authorization_span_mode);
                    span.record("http.request.header.authorization", authorization);
                } else {
                    span.record(to_span_format!("http.request.header.{}", key), val_str);
                }

                Some((key.as_str().to_string(), val_str.to_string()))
            }
            Err(e) => {
                error!("Header value is not a valid UTF-8 string: {:?}", e);
                None
            }
        })
        .collect::<HashMap<String, String>>()
        .to_value()
}
