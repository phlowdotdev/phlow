mod consumer;
mod produce;
mod setup;
use lapin::ExchangeKind;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use produce::producer;
use sdk::prelude::*;
use sdk::tracing::{debug, info};
use setup::Config;

plugin_async!(start_server);

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config: Config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;

    let conn = Connection::connect(&config.uri, ConnectionProperties::default()).await?;

    debug!("Connected to RabbitMQ");

    let channel = conn.create_channel().await?;

    debug!("Created channel");

    if config.declare {
        if !config.exchange.is_empty() {
            channel
                .exchange_declare(
                    &config.exchange,
                    ExchangeKind::Direct,
                    ExchangeDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await?;

            debug!("Declared exchange");
        }

        channel
            .queue_declare(
                &config.queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        debug!("Declared queue");
    }

    if setup.is_main() {
        info!("Main module started");
        let channel = conn.create_channel().await?;
        let main_sender = setup.main_sender.clone().unwrap();
        let id = setup.id.clone();
        let config = config.clone();
        tokio::task::spawn(async move {
            let _ = consumer::consumer(id, main_sender, config.clone(), channel).await;
        });
    }

    producer(setup.setup_sender, config, channel).await?;
    debug!("Producer finished");
    Ok(())
}
