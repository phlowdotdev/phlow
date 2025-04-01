mod middleware;
mod resolver;
mod response;
mod setup;
use hyper::{server::conn::http1, service::service_fn, Request};
use hyper_util::rt::{TokioIo, TokioTimer};
use resolver::resolve;
use sdk::{
    prelude::*,
    tokio::net::TcpListener,
    tracing::{info, info_span, warn},
};
use setup::Config;
use std::net::SocketAddr;

// plugin_async!(start_server);

#[no_mangle]
pub extern "C" fn plugin(setup: ModuleSetup) {
    sdk::otel::init_tracing_subscriber_plugin().expect("failed to initialize tracing");

    if let Ok(rt) = tokio::runtime::Runtime::new() {
        if let Err(e) = rt.block_on(start_server(setup)) {
            sdk::tracing::error!("Error in plugin: {:?}", e);
        }
    } else {
        sdk::tracing::error!("Error creating runtime");
        return;
    };
}

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !setup.is_main() {
        warn!("This module is not the main module, exiting");
        match setup.setup_sender.send(None) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("{:?}", e).into());
            }
        };
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

    info!("Listening on {}", listener.local_addr()?);

    match setup.setup_sender.send(None) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e).into());
        }
    };

    let id = setup.id;

    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io: TokioIo<tokio::net::TcpStream> = TokioIo::new(tcp);
        let dispatch = setup.dispatch.clone();
        let sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };

        let handler = tokio::task::spawn(async move {
            let dispatch_clone = dispatch.clone();
            let base_service = service_fn(move |mut req: Request<hyper::body::Incoming>| {
                sdk::tracing::dispatcher::with_default(&dispatch_clone.clone(), || {
                    let path = req.uri().path().to_string();
                    let method = req.method().to_string();

                    let span = info_span!("http_request", %method, %path);
                    span.record("otel.name", &path);

                    let _enter = span.enter();

                    req.extensions_mut().insert(peer_addr);
                    resolve(
                        id,
                        sender.clone(),
                        req,
                        dispatch_clone.clone(),
                        span.clone(),
                        method,
                        path,
                    )
                })
            });

            http1::Builder::new()
                .keep_alive(true)
                .timer(TokioTimer::new())
                .serve_connection(io, base_service)
                .await
                .unwrap_or_else(|e| {
                    sdk::tracing::error!("Error serving connection: {:?}", e);
                });
        });

        handler.await?;
    }
}
