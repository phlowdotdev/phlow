use crate::setup::Config;
use lapin::message::DeliveryResult;
use lapin::{options::*, types::FieldTable};
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::debug;

pub async fn consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: lapin::Channel,
    dispatch: Dispatch,
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

    let client_hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown_host".to_string());

    consumer.set_delegate(move |delivery: DeliveryResult| {
        phlow_sdk::tracing::dispatcher::with_default(&dispatch.clone(), || {
            let span = tracing::span!(
                Level::INFO,
                "message_receive",
                // Atributos gerais
                "messaging.system" = "amqp",
                "messaging.destination.name" = &config.routing_key,
                "messaging.destination.kind" = "queue",
                "messaging.operation" = "receive",
                "messaging.protocol" = "AMQP",
                "messaging.protocol_version" = "0.9.1",
                "messaging.amqp.consumer_tag" = &config.consumer_tag,
                "messaging.client.id" = &client_hostname,
                // Campos opcionais para debugging
                "messaging.message.payload_size_bytes" = field::Empty,
                "messaging.message.conversation_id" = field::Empty,
            );

            span_enter!(span);

            debug!("Received message");

            let sender = main_sender.clone();

            Box::pin(async move {
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

                let response_value = sender_package!(id, sender, Some(data))
                    .await
                    .unwrap_or(Value::Null);

                debug!("Response: {:?}", response_value);

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack send_webhook_event message");
            })
        })
    });

    Ok(())
}
