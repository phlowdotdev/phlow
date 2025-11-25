mod consumer;
mod produce;
mod setup;
use lapin::{Connection, ConnectionProperties};
use phlow_sdk::prelude::*;
use produce::producer;
use setup::Config;

create_main!(start_server(setup));

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use_log!();
    log::debug!("AMQP start_server called");
    let config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;
    log::debug!("Config created successfully: {:?}", config);

    let uri: String = match config.uri.clone() {
        Some(uri) => uri,
        None => config.to_connection_string(),
    };

    log::debug!("Connecting to RabbitMQ at {}", uri);

    let conn = Connection::connect(&uri, ConnectionProperties::default()).await?;

    log::debug!("Connected to RabbitMQ");

    let channel = conn.create_channel().await?;

    if setup.is_main() {
        log::info!("Main module started");
        let dispatch = setup.dispatch.clone();
        let channel = conn.create_channel().await?;
        let main_sender = match setup.main_sender.clone() {
            Some(sender) => sender,
            None => {
                return Err("Main sender is None".into());
            }
        };
        let id = setup.id.clone();
        let config = config.clone();
        tokio::task::spawn(async move {
            let _ = consumer::consumer(id, main_sender, config.clone(), channel, dispatch).await;
        });
    }

    producer(setup.setup_sender, config, channel).await?;
    log::debug!("Producer finished");
    Ok(())
}
