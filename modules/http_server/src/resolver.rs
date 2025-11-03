use crate::settings::AuthorizationSpanMode;
use crate::{middleware::RequestContext, response::ResponseHandler, router::Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use hyper::{HeaderMap, Request, Response};
use phlow_sdk::span_enter;
use phlow_sdk::{prelude::*, tracing::Span};
use std::{collections::HashMap, convert::Infallible};

macro_rules! to_span_record {
    ($span:expr, $target:expr, $key:expr, $value:expr) => {{
        let formatted_key = format!($target, $key.as_str().to_lowercase());
        $span.record(formatted_key.as_str(), $value);
    }};
}

pub async fn proxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // Handle fixed routes first
    if req.method() == hyper::Method::GET && req.uri().path() == "/health" {
        log::debug!("/health probe received");
        let response = Response::builder()
            .status(200)
            .body(Full::new(Bytes::from(r#"ok"#)))
            .unwrap();

        return Ok(response);
    }

    // Handle CORS preflight requests (OPTIONS)
    if req.method() == hyper::Method::OPTIONS {
        log::debug!(
            "CORS preflight request: path={} origin={:?}",
            req.uri().path(),
            req.headers().get("origin").and_then(|h| h.to_str().ok())
        );
        let context = req
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("RequestContext not found");

        let origin = req.headers().get("origin").and_then(|h| h.to_str().ok());

        let cors_response =
            ResponseHandler::create_preflight_response(context.cors.as_ref(), origin);

        // Record CORS preflight in tracing
        context
            .span
            .record("http.response.status_code", cors_response.status_code);
        context
            .span
            .record("http.response.body.size", cors_response.body.len());

        cors_response.headers.iter().for_each(|(key, value)| {
            to_span_record!(context.span, "http.response.header.{}", key, value);
        });

        return Ok(cors_response.build());
    }

    // Handle OpenAPI spec route
    if req.method() == hyper::Method::GET && req.uri().path() == "/openapi.json" {
        log::debug!("OpenAPI spec requested at /openapi.json");
        let context = req
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .expect("RequestContext not found");

        if let Some(ref openapi_validator) = context.openapi_validator {
            let spec = openapi_validator.get_spec();
            let response = Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .header("access-control-allow-origin", "*")
                .body(Full::new(Bytes::from(spec)))
                .unwrap();

            return Ok(response);
        } else {
            log::debug!("OpenAPI spec not configured; returning 404");
            let error_response = r#"{"error":"OPENAPI_NOT_CONFIGURED","message":"OpenAPI specification is not configured for this server"}"#;

            let response = Response::builder()
                .status(404)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(error_response)))
                .unwrap();

            return Ok(response);
        }
    }

    let context = req
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .expect("RequestContext not found");

    span_enter!(context.span);

    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let body_size = req.body().size_hint().lower();
    let request_size = req.size_hint().lower();
    let query = req.uri().query().unwrap_or_default().to_string();
    let uri = req.uri().to_string();
    let headers_clone = req.headers().clone();
    log::debug!(
        "Request received: method={} path={} query='{}' body_hint={} size_hint={}",
        method,
        path,
        query,
        body_size,
        request_size
    );

    // Check Content-Type for POST, PUT, PATCH requests
    let method_requires_content_type = matches!(method.as_str(), "POST" | "PUT" | "PATCH");
    if method_requires_content_type {
        let content_type = headers_clone
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");
        log::debug!("Content-Type detected: '{}'", content_type);

        // Define accepted Content-Types
        let accepted_content_types = [
            "application/json",
            "application/octet-stream",
            "application/xml",
            "text/plain",
            "text/html",
            "application/x-www-form-urlencoded",
            "multipart/form-data",
        ];

        // For POST, PUT, PATCH methods, if Content-Type is specified, validate it
        let has_explicit_content_type = !content_type.is_empty();

        if has_explicit_content_type {
            let is_accepted_content_type = accepted_content_types
                .iter()
                .any(|&accepted| content_type.starts_with(accepted));

            if !is_accepted_content_type {
                let accepted_types_str = accepted_content_types.join(", ");
                let error_message = format!("Content-Type must be one of: {}", accepted_types_str);

                let error_response_json = format!(
                    r#"{{"error":"Validation failed","details":[{{"type":"InvalidRequestBody","message":"{}","field":"content-type"}}]}}"#,
                    error_message
                );
                let error_body_size = error_response_json.len();

                let response = Response::builder()
                    .status(400)
                    .header("content-type", "application/json")
                    .body(Full::new(Bytes::from(error_response_json)))
                    .unwrap();

                context.span.record("http.response.status_code", 400);
                context
                    .span
                    .record("http.response.body.size", error_body_size);
                context
                    .span
                    .record("http.response.header.content-type", "application/json");

                log::debug!("Rejected request due to unsupported Content-Type");
                return Ok(response);
            }
        }
    }

    let headers = resolve_headers(
        headers_clone,
        &context.span,
        &context.authorization_span_mode,
    );
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
    log::debug!(
        "Resolved request parts: headers={} query_params={} body_kind={}",
        headers.len(),
        match &query_params {
            Value::Object(o) => o.len(),
            _ => 0,
        },
        match &body {
            Value::Null => "null",
            Value::Undefined => "undefined",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Boolean(_) => "bool",
            _ => "other",
        }
    );

    // Convert query_params HashMap for validation
    let query_map: std::collections::HashMap<String, String> =
        if let Value::Object(obj) = &query_params {
            obj.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    // Validate request and extract path parameters
    let (path_params, original_path, validation_error) =
        validate_request_and_extract_params(&method, &path, &query_map, &body, &context.router);
    log::debug!(
        "Validation result: matched_route={:?} path_params_count={} has_error={}",
        original_path,
        match &path_params {
            Value::Object(o) => o.len(),
            _ => 0,
        },
        validation_error.is_some()
    );

    // If validation failed, return error response immediately
    if let Some(error_response) = validation_error {
        let error_handler = ResponseHandler::from(error_response);

        context
            .span
            .record("http.response.status_code", error_handler.status_code);
        context
            .span
            .record("http.response.body.size", error_handler.body.len());

        error_handler.headers.iter().for_each(|(key, value)| {
            to_span_record!(context.span, "http.response.header.{}", key, value);
        });

        log::debug!(
            "Returning validation error response: status={}",
            error_handler.status_code
        );
        return Ok(error_handler.build());
    }

    let mut data_map = HashMap::from([
        ("client_ip", context.client_ip.to_value()),
        ("headers", headers),
        ("method", method.to_value()),
        ("resolved_path", path.to_value()),
        ("query_string", query.to_value()),
        ("query_params", query_params),
        ("uri", uri.to_value()),
        ("body", body),
        ("body_size", body_size.to_value()),
        ("path_params", path_params),
    ]);

    // Add path (original OpenAPI pattern) if available
    if let Some(openapi_path) = original_path {
        data_map.insert("path", openapi_path);
    } else {
        // Fallback to resolved_path if no OpenAPI pattern matched
        data_map.insert("path", path.to_value());
    }

    let data = data_map.to_value();
    log::debug!("Dispatching request to runtime package with collected data");

    let response_value = sender_package!(
        context.span.clone(),
        context.dispatch.clone(),
        context.id,
        context.sender,
        Some(data)
    )
    .await
    .unwrap_or(Value::Null);

    let mut response = ResponseHandler::from(response_value);
    log::debug!(
        "Runtime returned response object: status={} headers={}",
        response.status_code,
        response.headers.len()
    );

    // Apply CORS headers to the response
    let origin = data_map
        .get("headers")
        .and_then(|h| h.get("origin"))
        .and_then(|o| o.as_string_b())
        .map(|s| s.as_str());

    response.apply_cors_headers(context.cors.as_ref(), origin);

    context
        .span
        .record("http.response.status_code", response.status_code);
    context
        .span
        .record("http.response.body.size", response.body.len());

    response.headers.iter().for_each(|(key, value)| {
        to_span_record!(context.span, "http.response.header.{}", key, value);
    });

    log::debug!(
        "Sending final HTTP response: status={} headers={}",
        response.status_code,
        response.headers.len()
    );
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
            log::debug!("Error reading request body: {:?}", e);
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
            log::debug!("Error parsing request body: {:?}", e);
            Value::Undefined
        }
    };

    log::debug!(
        "Resolved request body: bytes={} kind={}",
        body_bytes.len(),
        match &body {
            Value::Null => "null",
            Value::Undefined => "undefined",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Boolean(_) => "bool",
            _ => "other",
        }
    );
    body
}

fn resolve_authorization(authorization: &str, mode: &AuthorizationSpanMode) -> String {
    match mode {
        AuthorizationSpanMode::None => "".to_string(),
        AuthorizationSpanMode::Hidden => "x".repeat(authorization.len()),
        AuthorizationSpanMode::Prefix => {
            let prefix_len = 12.min(authorization.len());
            format!("{}...", &authorization[..prefix_len])
        }
        AuthorizationSpanMode::Suffix => {
            let suffix_len = 6.min(authorization.len());
            format!("...{}", &authorization[authorization.len() - suffix_len..])
        }
        AuthorizationSpanMode::All => authorization.to_string(),
    }
}

async fn resolve_headers(
    headers: HeaderMap,
    span: &Span,
    authorization_span_mode: &AuthorizationSpanMode,
) -> Value {
    headers
        .iter()
        .filter_map(|(key, value)| match value.to_str() {
            Ok(val_str) => {
                if key == "authorization" {
                    let authorization = resolve_authorization(val_str, authorization_span_mode);
                    span.record("http.request.header.authorization", authorization);
                } else {
                    to_span_record!(span, "http.request.header.{}", key, val_str);
                }

                Some((key.as_str().to_string(), val_str.to_string()))
            }
            Err(e) => {
                log::debug!("Header value is not a valid UTF-8 string: {:?}", e);
                None
            }
        })
        .collect::<HashMap<String, String>>()
        .to_value()
}

/// Validates request and extracts path parameters using OpenAPI if available
fn validate_request_and_extract_params(
    method: &str,
    path: &str,
    query_params: &std::collections::HashMap<String, String>,
    body: &Value,
    router: &Router,
) -> (Value, Option<Value>, Option<Value>) {
    let validation_result = router.validate_and_extract(method, path, query_params, body);

    let path_params = validation_result.path_params.to_value();
    let original_path = validation_result
        .matched_route
        .as_ref()
        .map(|route| route.to_value());

    // If validation failed, return error response
    if let Some(validation) = &validation_result.validation_result {
        if !validation.is_valid {
            let error_details: Vec<Value> = validation
                .errors
                .iter()
                .map(|e| {
                    let mut error_obj = HashMap::new();
                    error_obj.insert("type".to_string(), format!("{:?}", e.error_type).to_value());
                    error_obj.insert("message".to_string(), e.message.to_value());
                    error_obj.insert(
                        "field".to_string(),
                        e.field
                            .as_ref()
                            .unwrap_or(&"unknown".to_string())
                            .to_value(),
                    );
                    error_obj.to_value()
                })
                .collect();

            let mut body_obj = HashMap::new();
            body_obj.insert("error".to_string(), "Validation failed".to_value());
            body_obj.insert("details".to_string(), error_details.to_value());

            let mut headers_obj = HashMap::new();
            headers_obj.insert("Content-Type".to_string(), "application/json".to_value());

            let mut error_response_obj = HashMap::new();
            error_response_obj.insert("status_code".to_string(), validation.status_code.to_value());
            error_response_obj.insert("body".to_string(), body_obj.to_value());
            error_response_obj.insert("headers".to_string(), headers_obj.to_value());

            let error_response = error_response_obj.to_value();

            return (path_params, original_path, Some(error_response));
        }
    }

    (path_params, original_path, None)
}
