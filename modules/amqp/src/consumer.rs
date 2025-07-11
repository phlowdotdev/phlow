use crate::setup::Config;
use lapin::message::DeliveryResult;
use lapin::{options::*, types::FieldTable};
use phlow_sdk::prelude::*;

use std::sync::Arc;

pub async fn consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: lapin::Channel,
    dispatch: Dispatch,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Starting consumer");

    let config = Arc::new(config);
    let main_sender = Arc::new(main_sender);
    let id = Arc::new(id);

    // Declare queue if not already declared
    let consumer = channel
        .basic_consume(
            &config.queue_name,
            &config.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let config_cloned = Arc::clone(&config);
    let main_sender_cloned = Arc::clone(&main_sender);
    let id_cloned = Arc::clone(&id);
    let hostname = match hostname::get() {
        Ok(name) => name.to_string_lossy().into_owned(),
        Err(_) => "unknown".to_string(),
    };

    consumer.set_delegate({
        let config = config_cloned;
        let dispatch = dispatch.clone();
        let main_sender = main_sender_cloned;
        let id = id_cloned;

        move |delivery: DeliveryResult| {
            let config = Arc::clone(&config);
            let main_sender = Arc::clone(&main_sender);
            let id = Arc::clone(&id);
            let dispatch = dispatch.clone();

            phlow_sdk::tracing::dispatcher::with_default(&dispatch.clone(), || {
                let span = tracing::span!(
                    Level::INFO,
                    "message_receive",
                    // Atributos gerais
                    "messaging.system" = "rabbitmq",
                    "messaging.destination.name" = &config.queue_name,
                    "messaging.destination.kind" = "queue",
                    "messaging.operation" = "receive",
                    "messaging.protocol" = "AMQP",
                    "messaging.protocol_version" = "0.9.1",
                    "messaging.rabbitmq.consumer_tag" = &config.consumer_tag,
                    "messaging.client.id" = hostname,
                    // Campos opcionais para debugging
                    "messaging.message.payload_size_bytes" = field::Empty,
                    "messaging.message.conversation_id" = field::Empty,
                );

                span_enter!(span);

                Box::pin(async move {
                    let sender = (*main_sender).clone();
                    let id = (*id).clone();

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

                    span.record("messaging.message.payload_size_bytes", delivery.data.len());
                    span.record("messaging.message.conversation_id", &id.to_string());

                    debug!("Received message: {:?}", data);

                    let response_value =
                        sender_package!(span.clone(), dispatch.clone(), id, sender, Some(data))
                            .await
                            .unwrap_or(Value::Null);

                    debug!("Response: {:?}", response_value);

                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack send_webhook_event message");
                })
            })
        }
    });

    Ok(())
}
