mod setup;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Buf;
use hyper::body::Bytes;
use hyper::body::Frame;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use sdk::prelude::*;
use setup::Setup;
use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use valu3::json;

plugin_async!(setup);

async fn setup(value: &Value) {
    println!("setup {:?}", value);
    if value.is_null() {
        println!("Value is null");
        return;
    }

    let setup = Arc::new(Setup::from(value.clone()));

    start_server(setup).await.unwrap();
}

async fn resolve(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let client_ip = req
        .extensions()
        .get::<SocketAddr>() // Recuperando o IP do cliente
        .map(|addr| addr.ip().to_string()) // Extraindo apenas o IP (sem a porta)
        .unwrap_or_else(|| "Unknown".to_string());

    let headers: HashMap<String, String> = req
        .headers()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect();

    let path = req.uri().path().to_value();
    let query = req.uri().query().unwrap_or_default().to_value();
    let method = req.method().to_string().to_value();

    let body = {
        let body = req.collect().await.unwrap().aggregate();
        let body = body.reader().bytes();
        let body = String::from_utf8(body.collect::<Result<Vec<u8>, _>>().unwrap())
            .unwrap_or_else(|_| "Error".to_string());

        match Value::json_to_value(&body) {
            Ok(value) => value,
            Err(_) => body.to_value(),
        }
    };

    let response_json = json!({
        "client_ip": client_ip,
        "headers": headers,
        "method": method,
        "path": path,
        "query": query,
        "body": body
    })
    .to_string();

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(response_json)))
        .unwrap())
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

async fn start_server(setup: Arc<Setup>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (tcp, peer_addr) = listener.accept().await?; // Obtendo o IP do cliente
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            let service = service_fn(move |mut req: Request<hyper::body::Incoming>| {
                req.extensions_mut().insert(peer_addr); // Inserindo o IP do cliente nas extensions
                resolve(req)
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
