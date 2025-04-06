mod middleware;
mod resolver;
mod response;
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

create_main!(start_server(setup));

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !setup.is_main() {
        debug!("This module is not the main module, exiting");
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

    debug!("Listening on {}", listener.local_addr()?);

    sender_safe!(setup.setup_sender, None);

    loop {
        let dispatch = setup.dispatch.clone();
        let authorization_span_mode = settings.authorization_span_mode.clone();
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
                id: setup.id,
                peer_addr,
                authorization_span_mode,
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
