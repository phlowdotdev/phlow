mod client;
mod server;
mod service;
mod setup;

use client::TarRpcClient;
use phlow_sdk::prelude::*;
use server::start_server;
use setup::{Config, RpcInput, RpcOutput};
use std::time::Instant;

create_main!(start_tarrpc_module(setup));

pub async fn start_tarrpc_module(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;
    
    debug!("Starting tarrpc module with config: {:?}", config);

    if setup.is_main() {
        info!("Starting tarrpc server as main module");
        
        let dispatch = setup.dispatch.clone();
        let main_sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => return Err("Main sender is None".into()),
        };
        let id = setup.id.clone();
        let config_clone = config.clone();
        
        // Start server in background
        tokio::task::spawn(async move {
            let result = match config_clone.transport.as_str() {
                "tcp" => start_server(config_clone, dispatch, main_sender, id).await,
                "memory" => server::start_memory_server(config_clone, dispatch, main_sender, id).await,
                _ => Err("Unsupported transport type".into()),
            };
            
            if let Err(e) = result {
                error!("tarrpc server error: {}", e);
            }
        });
    }

    // Handle client requests
    handle_client_requests(setup.setup_sender, config).await?;

    debug!("tarrpc module finished");
    Ok(())
}

async fn handle_client_requests(
    setup_sender: ModuleSetupSender,
    config: Config,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();
    setup_sender
        .send(Some(tx))
        .map_err(|e| format!("{:?}", e))?;

    debug!("tarrpc client handler started");

    for package in rx {
        debug!("Received tarrpc client request");
        
        let config = config.clone();
        let client = TarRpcClient::new(config);
        
        tokio::task::spawn(async move {
            let start_time = Instant::now();
            
            let result = match package.input {
                Some(input) => {
                    let rpc_input = RpcInput::from(&input);
                    
                    // Handle special methods
                    match rpc_input.method.as_str() {
                        "health_check" => client.health_check().await,
                        "get_service_info" => client.get_service_info().await,
                        _ => {
                            // Try with retry for regular methods
                            client.execute_with_retry(rpc_input).await
                        }
                    }
                }
                None => {
                    let error_msg = "No input provided";
                    error!("tarrpc client error: {}", error_msg);
                    Err(error_msg.into())
                }
            };

            let execution_time = start_time.elapsed().as_millis() as f64;

            let output = match result {
                Ok(response) => {
                    debug!("tarrpc client response received");
                    RpcOutput::success(response, execution_time)
                }
                Err(e) => {
                    error!("tarrpc client error: {}", e);
                    RpcOutput::error(e.to_string())
                }
            };

            let output_value: Value = output.into();
            sender_safe!(package.sender, output_value.into());
        });
    }

    Ok(())
}
