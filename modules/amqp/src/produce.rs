use lapin::options::BasicPublishOptions;
use lapin::publisher_confirm::Confirmation;
use lapin::BasicProperties;
use phlow_sdk::crossbeam::channel;
use phlow_sdk::modules::ModulePackage;
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::debug;

use crate::setup::Config;

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
    channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();
    match setup_sender.send(Some(tx)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e).into());
        }
    };

    debug!("Producer started");

    for package in rx {
        debug!("Received package");

        let payload = {
            let input = match package.context.input {
                Some(input) => input,
                None => {
                    let response = ProducerResponse::from_error("No input provided");
                    match package.sender.send(response.to_value()) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(format!("{:?}", e).into());
                        }
                    }
                    continue;
                }
            };

            input.get("message").unwrap_or(&Value::Null).to_string()
        };

        let confirm = channel
            .basic_publish(
                &config.exchange,
                &config.routing_key,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;

        debug!("Published message to {}", config.routing_key);

        let (success, error_message) = match confirm {
            Confirmation::NotRequested => {
                debug!("Published message without ack");
                (true, None)
            }
            Confirmation::Ack(basic_return_message) => {
                debug!("Published message with ack: {:?}", basic_return_message);
                (true, None)
            }
            Confirmation::Nack(basic_return_message) => {
                let error_message =
                    format!("Published message with nack: {:?}", basic_return_message);

                debug!(error_message);

                (false, Some(error_message))
            }
        };

        match package.sender.send(
            ProducerResponse {
                success,
                error_message,
            }
            .to_value(),
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("{:?}", e).into());
            }
        }
    }

    Ok(())
}
