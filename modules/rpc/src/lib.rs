mod client;
mod server;
mod service;
mod setup;

use client::RpcClient;
use phlow_sdk::prelude::*;
use server::start_rpc_server;
use setup::Config;

create_main!(start_rpc_module(setup));

pub async fn start_rpc_module(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    

    let config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;

    log::debug!("Starting RPC module with config: {:?}", config);

    if setup.is_main() {
        log::info!("Starting RPC server as main module");
        let dispatch = setup.dispatch.clone();
        let config_clone = config.clone();
        let main_sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };
        let id = setup.id.clone();

        // Start RPC server in background
        tokio::task::spawn(async move {
            if let Err(e) = start_rpc_server(config_clone, dispatch, main_sender, id).await {
                log::error!("RPC server error: {}", e);
            }
        });
    }

    // Handle RPC client requests
    handle_rpc_client(setup.setup_sender, config).await?;

    log::debug!("RPC module finished");
    Ok(())
}

async fn handle_rpc_client(
    setup_sender: ModuleSetupSender,
    config: Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();
    setup_sender
        .send(Some(tx))
        .map_err(|e| format!("{:?}", e))?;

    log::debug!("RPC client handler started");

    for package in rx {
        log::debug!("Received RPC client request: {:?}", package);

        let config = config.clone();
        let client = RpcClient::new(config);

        tokio::task::spawn(async move {
            let result = match package.input {
                Some(input) => match input.get("action") {
                    Some(Value::String(action)) => match action.as_str() {
                        "health" => client.health_check().await,
                        "info" => client.get_service_info().await,
                        _ => client.execute_call(input).await,
                    },
                    _ => client.execute_call(input).await,
                },
                None => {
                    let error_msg = "No input provided";
                    log::error!("RPC client error: {}", error_msg);
                    Ok(format!("{{\"error\": \"{}\", \"success\": false}}", error_msg).to_value())
                }
            };

            match result {
                Ok(response) => {
                    log::debug!("RPC client response: {:?}", response);
                    sender_safe!(package.sender, response.into());
                }
                Err(e) => {
                    log::error!("RPC client error: {}", e);
                    let error_response =
                        format!("{{\"error\": \"{}\", \"success\": false}}", e.to_string())
                            .to_value();
                    sender_safe!(package.sender, error_response.into());
                }
            }
        });
    }

    Ok(())
}
