use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::publisher_confirm::Confirmation;
use lapin::types::{AMQPValue, ShortString};
use lapin::BasicProperties;
use phlow_sdk::prelude::*;

use crate::setup::Config;

struct Input {
    message: String,
    basic_props: AMQPProperties,
}

impl From<&Value> for Input {
    fn from(value: &Value) -> Self {
        let message = value.get("message").unwrap_or(&Value::Null).to_string();
        let headers = value.get("headers").cloned();

        let basic_props = if let Some(headers) = headers {
            let mut field_table = lapin::types::FieldTable::default();

            if let Value::Object(map) = headers {
                for (key, val) in map.iter() {
                    let amqp_value = AMQPValue::LongString(val.to_string().into());
                    let amqp_key = ShortString::from(key.to_string());
                    field_table.insert(amqp_key, amqp_value);
                }
            }

            BasicProperties::default().with_headers(field_table)
        } else {
            BasicProperties::default()
        };

        Self {
            message,
            basic_props,
        }
    }
}

#[derive(Debug, ToValue)]
pub struct ProducerResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

impl ProducerResponse {
    pub fn from_error(error_message: &str) -> Self {
        Self {
            success: false,
            error_message: Some(error_message.to_string()),
        }
    }
}

pub async fn producer(
    setup_sender: ModuleSetupSender,
    config: Config,
    mut channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();
    setup_sender
        .send(Some(tx))
        .map_err(|e| format!("{:?}", e))?;

    log::debug!("Producer started");

    // Create connection for potential channel recreation
    let uri = match config.uri.clone() {
        Some(uri) => uri,
        None => config.to_connection_string(),
    };
    let conn = lapin::Connection::connect(&uri, lapin::ConnectionProperties::default()).await?;

    for package in rx {
        log::debug!("Received package");

        let input = match package.input {
            Some(input) => Input::from(&input),
            None => {
                let response = ProducerResponse::from_error("No input provided");
                let _ = package.sender.send(response.to_value().into());
                continue;
            }
        };

        let routing_key = match config.exchange_type.as_str() {
            "fanout" | "headers" => "",
            _ => config.routing_key.as_str(),
        };

        // Check if channel is closed and recreate if needed
        if !channel.status().connected() {
            log::debug!("Channel is closed, recreating...");
            match conn.create_channel().await {
                Ok(new_channel) => {
                    channel = new_channel;
                    log::debug!("Channel recreated successfully");
                }
                Err(e) => {
                    let response =
                        ProducerResponse::from_error(&format!("Failed to recreate channel: {}", e));
                    let _ = package.sender.send(response.to_value().into());
                    continue;
                }
            }
        }

        let publish_result = channel
            .basic_publish(
                &config.exchange,
                routing_key,
                BasicPublishOptions::default(),
                input.message.as_bytes(),
                input.basic_props,
            )
            .await;

        let confirm = match publish_result {
            Ok(confirm_future) => match confirm_future.await {
                Ok(confirm) => confirm,
                Err(e) => {
                    let response =
                        ProducerResponse::from_error(&format!("Publish confirmation error: {}", e));
                    let _ = package.sender.send(response.to_value().into());
                    continue;
                }
            },
            Err(e) => {
                let response = ProducerResponse::from_error(&format!("Publish error: {}", e));
                let _ = package.sender.send(response.to_value().into());
                continue;
            }
        };

        log::debug!("Published message to {} ({})", config.exchange, routing_key);

        let (success, error_message) = match confirm {
            Confirmation::NotRequested => {
                log::debug!("Published message without ack");
                (true, None)
            }
            Confirmation::Ack(msg) => {
                log::debug!("Ack: {:?}", msg);
                (true, None)
            }
            Confirmation::Nack(msg) => {
                let err = format!("Nack: {:?}", msg);
                log::debug!("{}", err);
                (false, Some(err))
            }
        };

        let _ = package.sender.send(
            ProducerResponse {
                success,
                error_message,
            }
            .to_value()
            .into(),
        );
    }

    Ok(())
}
