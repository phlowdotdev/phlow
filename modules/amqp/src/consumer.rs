use crate::setup::Config;
use lapin::message::DeliveryResult;
use lapin::{options::*, types::FieldTable, BasicProperties};
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::{field, Dispatch, Level};

use std::sync::Arc;

// Nota: decisão sobre DLQ foi simplificada para tentar NACK com requeue=false
// diretamente; se falhar, o código faz fallback para requeue=true.

pub async fn consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: lapin::Channel,
    dispatch: Dispatch,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use_log!();
    log::debug!("Starting consumer with max_retry={} and dlq_enable={}", config.max_retry, config.dlq_enable);

    let config = Arc::new(config);
    let main_sender = Arc::new(main_sender);
    let id = Arc::new(id);
    let channel = Arc::new(channel);

    // Se definido, limita o número de mensagens não confirmadas (concorrência)
    if config.max_concurrency > 0 {
        log::debug!(
            "Setting basic_qos prefetch_count={} (max_concurrency)",
            config.max_concurrency
        );
        channel
            .basic_qos(
                config.max_concurrency,
                lapin::options::BasicQosOptions { global: false },
            )
            .await?;
    } else {
        log::debug!("max_concurrency=0 (sem limites), não aplicando basic_qos");
    }

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
    let channel_cloned = Arc::clone(&channel);
    let hostname = match hostname::get() {
        Ok(name) => name.to_string_lossy().into_owned(),
        Err(_) => "unknown".to_string(),
    };

    consumer.set_delegate({
        let config = config_cloned;
        let dispatch = dispatch.clone();
        let main_sender = main_sender_cloned;
        let id = id_cloned;
        let channel = channel_cloned;

        move |delivery: DeliveryResult| {
            let config = Arc::clone(&config);
            let main_sender = Arc::clone(&main_sender);
            let id = Arc::clone(&id);
            let dispatch = dispatch.clone();
            let channel = Arc::clone(&channel);

            phlow_sdk::tracing::dispatcher::with_default(&dispatch.clone(), || {
                use_log!();
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

                    // Obter contagem de tentativas do header (se existir)
                    let retry_count = delivery.properties.headers()
                        .as_ref()
                        .and_then(|h| h.inner().get("x-retry-count"))
                        .and_then(|v| v.as_long_long_int())
                        .unwrap_or(0);

                    log::debug!("Received message (retry {}/{}): {:?}", retry_count, config.max_retry, data);

                    let response_value =
                        sender_package!(span.clone(), dispatch.clone(), id, sender, Some(data))
                            .await
                            .unwrap_or(Value::Null);

                    log::debug!("Response: {:?}", response_value);

                    let should_ack = match response_value {
                        Value::Null => true,
                        Value::Boolean(false) => false,
                        Value::Boolean(true) => true,
                        _ => true,
                    };

                    if should_ack {
                        match delivery.ack(BasicAckOptions::default()).await {
                            Ok(_) => {
                                log::debug!("Message acknowledged successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to ack message: {}", e);
                            }
                        }
                    } else {
                        log::debug!("Message processing indicated NACK (not acknowledged)");
                        // Processamento falhou
                        if retry_count < config.max_retry {
                            log::debug!("Retrying message, current retry count: {}, max_retry: {}", retry_count, config.max_retry);
                            let mut headers = delivery.properties.headers().as_ref().cloned().unwrap_or_default();
                            headers.insert("x-retry-count".into(), (retry_count + 1).into());
                            
                            let properties = BasicProperties::default().with_headers(headers);
                            
                            match channel.basic_publish(
                                "",
                                &config.queue_name,
                                BasicPublishOptions::default(),
                                &delivery.data,
                                properties,
                            ).await {
                                Ok(_) => {
                                    log::warn!("Message requeued for retry {}/{}", retry_count + 1, config.max_retry);
                                    log::debug!("Message requeued for retry {}/{}", retry_count + 1, config.max_retry);
                                    // ACK a mensagem original para removê-la da fila
                                    let _ = delivery.ack(BasicAckOptions::default()).await;
                                }
                                Err(e) => {
                                    log::error!("Failed to requeue message: {}", e);
                                    log::debug!("Failed to requeue message: {}", e);
                                    // Em caso de erro no requeue, manter mensagem na fila
                                    let _ = delivery.nack(BasicNackOptions {
                                        multiple: false,
                                        requeue: true,
                                    }).await;
                                }
                            }
                        } else if config.dlq_enable {
                            log::debug!("Max retries exceeded and DLQ enabled");
                            // Excedeu limite de tentativas - verificar se DLQ está configurada
                            log::warn!("Max retries ({}) exceeded for message, checking DLQ configuration...", config.max_retry);
                            
                            // Tentar enviar para DLQ (NACK requeue=false). Se falhar, fazer fallback para requeue=true
                            log::warn!(
                                "Max retries ({}) exceeded for message, attempting NACK to DLQ (requeue=false) with fallback to requeue=true",
                                config.max_retry
                            );

                            match delivery.nack(BasicNackOptions { multiple: false, requeue: false }).await {
                                Ok(_) => {
                                    log::debug!("Message rejected (NACK) after {} retries - sent to DLQ", retry_count);
                                    log::warn!("Message rejected (NACK) after {} retries - sent to DLQ", retry_count);
                                }
                                Err(e) => {
                                    log::debug!("Failed to nack message to DLQ: {}. Falling back to requeue", e);
                                    log::error!("Failed to nack message to DLQ: {}. Falling back to requeue", e);

                                    // Fallback: requeue para continuar processamento
                                    match delivery.nack(BasicNackOptions { multiple: false, requeue: true }).await {
                                        Ok(_) => {
                                            log::debug!("Message requeued - will continue processing until DLQ is properly configured");
                                            log::warn!("Message requeued - will continue processing until DLQ is properly configured");
                                        }
                                        Err(e) => {
                                            log::debug!("Failed to requeue message: {}", e);
                                            log::error!("Failed to requeue message: {}", e);
                                        }
                                    }
                                }
                            }
                        } else {
                            log::debug!("Max retries exceeded and DLQ disabled");
                            // DLQ desabilitada - descartar mensagem
                            match delivery.ack(BasicAckOptions::default()).await {
                                Ok(_) => {
                                    log::debug!("DLQ disabled - message discarded after {} retries", retry_count);
                                    log::warn!("DLQ disabled - discarding message after {} retries", retry_count);
                                }
                                Err(e) => {
                                    log::debug!("Failed to ack message for discard: {}", e);
                                    log::error!("Failed to ack message for discard: {}", e);
                                }
                            }
                        }
                    }
                })
            })
        }
    });

    Ok(())
}
