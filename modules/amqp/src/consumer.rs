use crate::setup::Config;
use lapin::message::DeliveryResult;
use lapin::{options::*, types::FieldTable};
use sdk::prelude::*;
use sdk::tracing::debug;

pub async fn consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting consumer");

    channel
        .queue_declare(
            &config.routing_key,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let consumer = channel
        .basic_consume(
            &config.routing_key,
            &config.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    consumer.set_delegate(move |delivery: DeliveryResult| {
        debug!("Received message");

        let sender = main_sender.clone();

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

            debug!("Received message: {:?}", data);

            let response_value = sender!(id, sender, Some(data)).await.unwrap_or(Value::Null);

            debug!("Response: {:?}", response_value);

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to ack send_webhook_event message");
        }
    });

    Ok(())
}
