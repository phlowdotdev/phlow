mod middleware;
mod openapi;
mod resolver;
mod response;
mod router;
mod settings;
mod setup;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use middleware::TracingMiddleware;
use phlow_sdk::{prelude::*, tokio::net::TcpListener};
use resolver::proxy;
use settings::Settings;
use setup::Config;
use std::{net::SocketAddr, sync::Arc};
#[cfg(test)]
mod openapi_tests;
create_main!(start_server(setup));

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting HTTP server module: {}", setup.id);

    if !setup.is_main() {
        log::debug!("This module is not the main module, exiting");
        match setup.setup_sender.send(None) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("{:?}", e).into());
            }
        };
        return Ok(());
    }

    // If we're in test mode, don't start the actual server
    if setup.is_test_mode {
        log::debug!("Test mode detected, not starting HTTP server");
        match setup.setup_sender.send(None) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("{:?}", e).into());
            }
        };
        return Ok(());
    }

    log::debug!("Loading server configuration from setup.with");
    let config: Config = Config::from(setup.with);
    let settings = Arc::new(Settings::load());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    log::debug!("Binding to {}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            log::error!(
                "Address already in use: {}. Outro processo pode estar escutando nessa porta. Sugestão: execute 'ss -ltnp | grep :{}' para identificar o processo.",
                addr,
                config.port
            );
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    };

    log::debug!("Listening on {}", listener.local_addr()?);

    sender_safe!(setup.setup_sender, None);

    loop {
        let dispatch = setup.dispatch.clone();
        let authorization_span_mode = settings.authorization_span_mode.clone();
        let router = config.router.clone();
        let cors_config = config.cors.clone();
        let sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };

        log::debug!("Waiting for incoming TCP connection...");
        let (tcp, peer_addr) = listener.accept().await?;
        log::debug!("Accepted connection from {}", peer_addr);
        let io: TokioIo<tokio::net::TcpStream> = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            log::debug!(
                "Spawning connection handler task for peer {} with tracing middleware",
                peer_addr
            );
            let service = service_fn(proxy);

            let middleware = TracingMiddleware {
                inner: service,
                dispatch: dispatch.clone(),
                sender: sender.clone(),
                id: setup.id,
                peer_addr,
                authorization_span_mode,
                router: router.clone(),
                openapi_validator: router.openapi_validator.clone(),
                cors: cors_config,
            };

            if let Err(e) = http1::Builder::new()
                .keep_alive(true)
                .serve_connection(io, middleware)
                .await
            {
                log::debug!("Error serving connection: {}", e);
            }
            log::debug!("Connection handler for {} finished", peer_addr);
        });
    }
}
