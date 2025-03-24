mod setup;
use lapin::message::DeliveryResult;
use lapin::ExchangeKind;
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties,
};
use sdk::modules::ModulePackage;
use sdk::prelude::*;
use sdk::tracing::{debug, info};
use setup::Config;
use std::sync::mpsc;

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

    if config.is_producer() {
        producer(setup, config, channel).await?;
    } else {
        consumer(setup, config, channel).await?;
    }

    Ok(())
}

async fn producer(
    setup: ModuleSetup,
    config: Config,
    channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = mpsc::channel::<ModulePackage>();
    setup.setup_sender.send(Some(tx)).unwrap();

    for package in rx {
        let payload = package
            .context
            .input
            .unwrap_or(Value::Null)
            .to_json(sdk::prelude::JsonMode::Inline);

        let confirm = channel
            .basic_publish(
                &config.exchange,
                &config.queue,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;

        info!("Published message to {}", config.queue);

        let data = match confirm {
            Confirmation::NotRequested => {
                debug!("Published message without ack");
                true
            }
            Confirmation::Ack(basic_return_message) => {
                debug!("Published message with ack: {:?}", basic_return_message);
                true
            }
            Confirmation::Nack(basic_return_message) => {
                debug!("Published message with nack: {:?}", basic_return_message);
                false
            }
        }
        .to_value();

        package.sender.send(data).unwrap();
    }

    Ok(())
}

async fn consumer(
    setup: ModuleSetup,
    config: Config,
    channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    channel
        .queue_declare(
            &config.queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            &config.queue,
            &config.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let id = setup.id;

    consumer.set_delegate(move |delivery: DeliveryResult| {
        let sender = setup.main_sender.clone().unwrap();

        async move {
            let delivery = match delivery {
                Ok(Some(delivery)) => delivery,
                Ok(None) => return,
                Err(error) => {
                    dbg!("Failed to consume queue message {}", error);
                    return;
                }
            };

            let data: Value = String::from_utf8_lossy(&delivery.data)
                .to_string()
                .to_value();

            let response_value = sender!(id, sender, Some(data)).await.unwrap_or(Value::Null);

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack send_webhook_event message");
        }
    });

    Ok(())
}
