pub mod setup;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::HeaderMap;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use sdk::prelude::*;
use sdk::tokio::net::TcpListener;
use sdk::tracing::debug;
use sdk::tracing::error;
use sdk::tracing::info;
use sdk::tracing::warn;
use setup::Config;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

plugin_async!(start_server);

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !setup.is_main() {
        warn!("This module is not the main module, exiting");
        setup.setup_sender.send(None).unwrap();
        return Ok(());
    }

    let config: Config = Config::from(setup.with);

    let addr: SocketAddr = format!(
        "{}:{}",
        config.host.as_deref().unwrap_or("0.0.0.0"),
        config.port.unwrap_or(4000),
    )
    .parse()?;

    let listener = TcpListener::bind(addr).await?;

    setup.setup_sender.send(None).unwrap();

    info!("Listening on http://{}", addr);

    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        let sender = setup.main_sender.clone().unwrap();
        let id = setup.id;

        tokio::task::spawn(async move {
            let service = service_fn(move |mut req: Request<hyper::body::Incoming>| {
                req.extensions_mut().insert(peer_addr);
                resolve(id, sender.clone(), req)
            });

            if let Err(err) = http1::Builder::new()
                .keep_alive(true)
                .timer(TokioTimer::new())
                .serve_connection(io, service)
                .await
            {
                if err.is_timeout() {
                    debug!("Connection timed out");
                    return;
                }
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[derive(ToValue)]
struct ResponseHandler {
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

async fn resolve(
    id: ModuleId,
    sender: MainRuntimeSender,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let client_ip: String = req
        .extensions()
        .get::<SocketAddr>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let query = req.uri().query().unwrap_or_default().to_string();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    let headers = resolve_headers(req.headers().clone());
    let body = resolve_body(req);

    let query_params = resolve_query_params(&query);

    let query_params = query_params.await;
    let body = body.await;
    let headers = headers.await;

    let data = json!({
        "client_ip": client_ip,
        "headers": headers,
        "method": method,
        "path": path,
        "query_string": query,
        "query_params": query_params,
        "body": body
    });

    debug!("Request: {:?}", data);

    let response_value = sender!(id, sender, Some(data)).await.unwrap_or(Value::Null);

    let response = ResponseHandler::from(response_value).build();

    Ok(response)
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

async fn resolve_headers(headers: HeaderMap) -> Value {
    headers
        .iter()
        .filter_map(|(key, value)| match value.to_str() {
            Ok(val_str) => Some((key.as_str().to_string(), val_str.to_string())),
            Err(e) => {
                error!("Header value is not a valid UTF-8 string: {:?}", e);
                None
            }
        })
        .collect::<HashMap<String, String>>()
        .to_value()
}
