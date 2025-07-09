use crate::service::{PhlowRpc, PhlowRpcServer};
use crate::setup::Config;
use phlow_sdk::prelude::*;
use futures::StreamExt;
use tarpc::{server, tokio_serde::formats::Json};
use tarpc::server::Channel;

pub async fn start_rpc_server(
    config: Config,
    dispatch: Dispatch,
    main_sender: MainRuntimeSender,
    id: ModuleId,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting RPC server on {}", config.server_address());

    let server_addr: std::net::SocketAddr = config.server_address().parse()?;
    
    let server = PhlowRpcServer {
        dispatch: dispatch.clone(),
        service_name: config.service_name.clone(),
        main_sender: main_sender.clone(),
        id: id.clone(),
    };

    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    info!("RPC server listening on {}", server_addr);

    listener
        .filter_map(|r| async move {
            match r {
                Ok(transport) => {
                    info!("New RPC connection established");
                    Some(transport)
                }
                Err(e) => {
                    warn!("Failed to establish RPC connection: {}", e);
                    None
                }
            }
        })
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = server.clone();
            channel.execute(server.serve())
        })
        .map(|responses| {
            responses.for_each(|response| async move {
                // Each response is a future that completes when the RPC call finishes
                response.await;
            })
        })
        .buffer_unordered(config.max_connections)
        .for_each(|_| async {})
        .await;

    Ok(())
}
