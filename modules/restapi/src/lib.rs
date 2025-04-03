mod middleware;
mod resolver;
mod response;
mod settings;
mod setup;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use middleware::TracingMiddleware;
use resolver::proxy;
use sdk::{
    prelude::*,
    tokio::net::TcpListener,
    tracing::{debug, info, warn},
};
use settings::Settings;
use setup::Config;
use std::{net::SocketAddr, sync::Arc};

main_plugin_async!(start_server);

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
    let settings = Arc::new(Settings::load());

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
        let settings = settings.clone();
        let dispatch = setup.dispatch.clone();
        let sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };

        let (tcp, peer_addr) = listener.accept().await?;
        let io: TokioIo<tokio::net::TcpStream> = TokioIo::new(tcp);

        tokio::task::spawn(async move {
            let service = service_fn(proxy);

            let middleware = TracingMiddleware {
                inner: service,
                dispatch: dispatch.clone(),
                sender: sender.clone(),
                id,
                peer_addr,
                settings,
            };

            if let Err(e) = http1::Builder::new()
                .keep_alive(true)
                .serve_connection(io, middleware)
                .await
            {
                debug!("Error serving connection: {}", e);
            }
        });
    }
}
