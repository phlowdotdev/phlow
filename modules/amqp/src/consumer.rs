use crate::setup::Config;
use lapin::message::DeliveryResult;
use lapin::{options::*, types::FieldTable, BasicProperties};
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::{field, Dispatch, Level};

use std::sync::Arc;

// Função para verificar se DLQ está configurada via API do RabbitMQ
async fn check_dlq_configured(config: &Config) -> bool {
    let management_port = 15672; // Porta padrão da Management API
    let client = reqwest::Client::new();
    
    // Verificar se a fila tem dead letter exchange configurado
    let url = format!(
        "http://{}:{}/api/queues/{}/{}",
        config.host,
        management_port,
        urlencoding::encode(&config.vhost),
        urlencoding::encode(&config.queue_name)
    );
    
    match client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(queue_info) => {
                        // Verificar se há x-dead-letter-exchange configurado
                        if let Some(arguments) = queue_info.get("arguments") {
                            if let Some(dle) = arguments.get("x-dead-letter-exchange") {
                                if !dle.is_null() && dle.as_str().unwrap_or("").trim() != "" {
                                    log::debug!("DLQ configured: x-dead-letter-exchange = {}", dle);
                                    return true;
                                }
                            }
                        }
                        log::debug!("DLQ not configured: no x-dead-letter-exchange found");
                        false
                    }
                    Err(e) => {
                        log::warn!("Failed to parse queue info: {}", e);
                        false
                    }
                }
            } else {
                log::warn!("Failed to get queue info: HTTP {}", response.status());
                false
            }
        }
        Err(e) => {
            log::warn!("Failed to connect to RabbitMQ Management API: {}", e);
            false
        }
    }
}

pub async fn consumer(
    id: ModuleId,
    main_sender: MainRuntimeSender,
    config: Config,
    channel: lapin::Channel,
    dispatch: Dispatch,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Starting consumer with max_retry={} and dlq_enable={}", config.max_retry, config.dlq_enable);

    let config = Arc::new(config);
    let main_sender = Arc::new(main_sender);
    let id = Arc::new(id);
    let channel = Arc::new(channel);

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
                        // Processamento falhou
                        if retry_count < config.max_retry {
                            let mut headers = delivery.properties.headers().as_ref().cloned().unwrap_or_default();
                            headers.insert("x-retry-count".into(), (retry_count + 1).into());
                            
                            let properties = BasicProperties::default().with_headers(headers);
                            
                            match channel.basic_publish(
                                &config.exchange,
                                &config.routing_key,
                                BasicPublishOptions::default(),
                                &delivery.data,
                                properties,
                            ).await {
                                Ok(_) => {
                                    log::warn!("Message requeued for retry {}/{}", retry_count + 1, config.max_retry);
                                    // ACK a mensagem original para removê-la da fila
                                    let _ = delivery.ack(BasicAckOptions::default()).await;
                                }
                                Err(e) => {
                                    log::error!("Failed to requeue message: {}", e);
                                    // Em caso de erro no requeue, manter mensagem na fila
                                    let _ = delivery.nack(BasicNackOptions {
                                        multiple: false,
                                        requeue: true,
                                    }).await;
                                }
                            }
                        } else if config.dlq_enable {
                            // Excedeu limite de tentativas - verificar se DLQ está configurada
                            log::warn!("Max retries ({}) exceeded for message, checking DLQ configuration...", config.max_retry);
                            
                            // Verificar se DLQ está configurada via API do RabbitMQ
                            let dlq_configured = check_dlq_configured(&config).await;
                            
                            if dlq_configured {
                                // DLQ está configurada - enviar para DLQ
                                log::warn!("DLQ is configured, sending message to DLQ");
                                match delivery.nack(BasicNackOptions {
                                    multiple: false,
                                    requeue: false, // false = vai para DLQ
                                }).await {
                                    Ok(_) => {
                                        log::warn!("Message rejected (NACK) after {} retries - sent to DLQ", retry_count);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to nack message to DLQ: {}", e);
                                    }
                                }
                            } else {
                                // DLQ NÃO está configurada - requeue com warning
                                log::warn!("DLQ is enabled but NOT configured (no x-dead-letter-exchange found)");
                                log::warn!("Requeuing message to continue processing (DLQ enabled but not configured)");
                                
                                // Requeue para continuar processando
                                match delivery.nack(BasicNackOptions {
                                    multiple: false,
                                    requeue: true,
                                }).await {
                                    Ok(_) => {
                                        log::warn!("Message requeued - will continue processing until DLQ is properly configured");
                                    }
                                    Err(e) => {
                                        log::error!("Failed to requeue message: {}", e);
                                    }
                                }
                            }
                        } else {
                            // DLQ desabilitada - descartar mensagem
                            match delivery.ack(BasicAckOptions::default()).await {
                                Ok(_) => {
                                    log::warn!("DLQ disabled - discarding message after {} retries", retry_count);
                                }
                                Err(e) => {
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
