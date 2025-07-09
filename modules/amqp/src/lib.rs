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
        // Declare exchange if specified
        if !config.exchange.is_empty() {
            let exchange_kind = match config.exchange_type.as_str() {
                "fanout" => lapin::ExchangeKind::Fanout,
                "topic" => lapin::ExchangeKind::Topic,
                "headers" => lapin::ExchangeKind::Headers,
                _ => lapin::ExchangeKind::Direct,
            };

            let exchange_options = lapin::options::ExchangeDeclareOptions {
                durable: config.exchange_durable,
                ..Default::default()
            };

            channel
                .exchange_declare(
                    &config.exchange,
                    exchange_kind,
                    exchange_options,
                    lapin::types::FieldTable::default(),
                )
                .await?;
            debug!("Declared exchange: {} (durable: {})", config.exchange, config.exchange_durable);
        }

        // Declare queue if specified
        if !config.queue_name.is_empty() {
            let queue_options = lapin::options::QueueDeclareOptions {
                durable: config.queue_durable,
                exclusive: config.queue_exclusive,
                auto_delete: config.queue_auto_delete,
                ..Default::default()
            };

            channel
                .queue_declare(
                    &config.queue_name,
                    queue_options,
                    lapin::types::FieldTable::default(),
                )
                .await?;
            debug!("Declared queue: {} (durable: {}, exclusive: {}, auto_delete: {})", 
                   config.queue_name, config.queue_durable, config.queue_exclusive, config.queue_auto_delete);
        }

        // Bind queue to exchange if both are specified and auto_bind is true
        if config.auto_bind && !config.exchange.is_empty() && !config.queue_name.is_empty() && !config.routing_key.is_empty() {
            channel
                .queue_bind(
                    &config.queue_name,
                    &config.exchange,
                    &config.routing_key,
                    lapin::options::QueueBindOptions::default(),
                    lapin::types::FieldTable::default(),
                )
                .await?;
            debug!("Bound queue '{}' to exchange '{}' with routing_key '{}'", 
                   config.queue_name, config.exchange, config.routing_key);
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
