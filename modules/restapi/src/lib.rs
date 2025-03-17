pub mod setup;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use sdk::prelude::*;
use sdk::tokio::net::TcpListener;
use setup::Config;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

plugin_async!(start_server);

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config: Config = Config::from(setup.with);

    let addr: SocketAddr = format!(
        "{}:{}",
        config.host.as_deref().unwrap_or("0.0.0.0"),
        config.port.unwrap_or(3000),
    )
    .parse()?;

    let listener = TcpListener::bind(addr).await?;

    setup.setup_sender.send(None).unwrap();

    println!("Listening on http://{}", addr);

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
                .timer(TokioTimer::new())
                .serve_connection(io, service)
                .await
            {
                println!("Error serving connection: {:?}", err);
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
        let response = Response::builder().status(self.status_code);

        let response = self
            .headers
            .iter()
            .fold(response, |response, (key, value)| {
                response.header(key, value)
            });

        response
            .body(Full::new(Bytes::from(self.body.clone())))
            .unwrap()
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
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let client_ip: String = req
        .extensions()
        .get::<SocketAddr>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let headers = req
        .headers()
        .iter()
        .map(|(key, value)| {
            (
                key.as_str().to_string(),
                value.to_str().unwrap().to_string(),
            )
        })
        .collect::<HashMap<String, String>>();

    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let query = req.uri().query().unwrap_or_default().to_string();

    let body = {
        let body = req.body_mut().collect().await.unwrap().to_bytes();
        let body = body
            .iter()
            .map(|byte| *byte as char)
            .collect::<String>()
            .trim()
            .to_string();

        if body.starts_with('{') || body.starts_with('[') {
            Value::json_to_value(&body).unwrap_or(body.to_value())
        } else {
            body.to_value()
        }
    };

    let data = json!({
        "client_ip": client_ip,
        "headers": headers,
        "method": method,
        "path": path,
        "query": query,
        "body": body
    });

    println!("Request: {:?}", data);

    let response_value = sender!(id, sender, Some(data)).await.unwrap_or(Value::Null);

    let response = ResponseHandler::from(response_value).build();

    Ok(response)
}
