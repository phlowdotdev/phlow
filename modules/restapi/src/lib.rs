mod resolver;
mod response;
mod setup;
use hyper::{server::conn::http1, service::service_fn, Request};
use hyper_util::rt::{TokioIo, TokioTimer};
use resolver::resolve;
use sdk::{
    prelude::*,
    tokio::net::TcpListener,
    tracing::{debug, info, warn},
};
use setup::Config;
use std::net::SocketAddr;

plugin_async!(start_server);

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

    match setup.setup_sender.send(None) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e).into());
        }
    };

    info!("Listening on http://{}", addr);
    let id = setup.id;

    loop {
        let (tcp, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(tcp);
        let sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };

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
                debug!("Connection timed out: {}", err);
            }
        });
    }
}
