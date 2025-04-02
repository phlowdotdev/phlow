mod middleware;
mod resolver;
mod response;
mod setup;
use hyper::{
    server::conn::http1,
    service::{self, service_fn},
    Request,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use middleware::TracingMiddleware;
use resolver::proxy;
use sdk::{
    opentelemetry::{
        global,
        trace::{FutureExt, Span, SpanKind, Tracer},
    },
    otel::get_tracer,
    prelude::*,
    tokio::net::TcpListener,
    tracing::{dispatcher, info, info_span, warn, Instrument},
    tracing_opentelemetry::OpenTelemetrySpanExt,
};
use setup::Config;
use std::net::SocketAddr;

// plugin_async!(start_server);

#[no_mangle]
pub extern "C" fn plugin(setup: ModuleSetup) {
    let dispatch = setup.dispatch.clone();
    dispatcher::with_default(&dispatch, || {
        sdk::otel::init_tracing_subscriber().expect("failed to initialize tracing");

        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(start_server(setup)).unwrap_or_else(|e| {
                sdk::tracing::error!("Error in plugin: {:?}", e);
            });
        } else {
            sdk::tracing::error!("Error creating runtime");
            return;
        };
    });
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
            let base_service = service_fn(proxy);

            let middleware = TracingMiddleware {
                inner: base_service,
                dispatch: dispatch.clone(),
                sender: sender.clone(),
                id,
                peer_addr,
            };

            http1::Builder::new()
                .keep_alive(true)
                .timer(TokioTimer::new())
                .serve_connection(io, middleware)
                .await
                .unwrap_or_else(|e| {
                    sdk::tracing::error!("Error serving connection: {:?}", e);
                });
        });

        handler.await?;
    }
}
