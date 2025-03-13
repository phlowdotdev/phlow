mod setup;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Buf;
use hyper::body::Bytes;
use hyper::server::conn::http1;
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
use tokio::sync::oneshot::{channel, Receiver, Sender};
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

async fn start_server(setup: Arc<Setup>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!(
        "{}:{}",
        setup.host.clone().unwrap_or("0.0.0.0".to_string()),
        setup.port.clone().unwrap_or(3000),
    )
    .parse()?;

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            let service = service_fn(move |mut req: Request<hyper::body::Incoming>| {
                req.extensions_mut().insert(peer_addr);
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

async fn resolve(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let client_ip: String = req
        .extensions()
        .get::<SocketAddr>() // Recuperando o IP do cliente
        .map(|addr| addr.ip().to_string()) // Extraindo apenas o IP (sem a porta)
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
        let body = req.collect().await.unwrap().aggregate();
        let body = body.reader().bytes();
        let body = String::from_utf8(body.collect::<Result<Vec<u8>, _>>().unwrap())
            .unwrap_or_else(|_| "Error".to_string());

        body
    };

    let data = json!({
        "client_ip": client_ip,
        "headers": headers,
        "method": method,
        "path": path,
        "query": query,
        "body": body
    });

    let (tx, rx) = channel();

    let broker_request = Package {
        send: Some(tx),
        data: Some(data),
    };

    let broker_response = rx.blocking_recv().unwrap_or(Value::Null);

    return Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(broker_response.to_string())))
        .unwrap());

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .status(500)
        .body(Full::new(Bytes::from("Error".to_string())))
        .unwrap())
}
