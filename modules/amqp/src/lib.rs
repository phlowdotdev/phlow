mod consumer;
mod produce;
mod setup;
use lapin::ExchangeKind;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use phlow_sdk::prelude::*;
use produce::producer;
use setup::Config;

create_main!(start_server(setup));

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::try_from(&setup.with).map_err(|e| format!("{:?}", e))?;

    let conn = Connection::connect(
        &config.to_connection_string(),
        ConnectionProperties::default(),
    )
    .await?;

    debug!("Connected to RabbitMQ");

    let channel = conn.create_channel().await?;

    debug!("Created channel");

    if config.declare {
        if !config.exchange.is_empty() {
            let exchange_kind = match config.exchange_type.as_str() {
                "fanout" => lapin::ExchangeKind::Fanout,
                "topic" => lapin::ExchangeKind::Topic,
                "headers" => lapin::ExchangeKind::Headers,
                _ => lapin::ExchangeKind::Direct,
            };

            channel
                .exchange_declare(
                    &config.exchange,
                    exchange_kind,
                    lapin::options::ExchangeDeclareOptions::default(),
                    lapin::types::FieldTable::default(),
                )
                .await?;
            debug!("Producer declared exchange: {}", config.exchange);
        }

        if !config.routing_key.is_empty() {
            channel
                .queue_declare(
                    &config.routing_key,
                    lapin::options::QueueDeclareOptions::default(),
                    lapin::types::FieldTable::default(),
                )
                .await?;
            debug!("Producer declared queue: {}", config.routing_key);
        }
    }

    if setup.is_main() {
        info!("Main module started");
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
    debug!("Producer finished");
    Ok(())
}
