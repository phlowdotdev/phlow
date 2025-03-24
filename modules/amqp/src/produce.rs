use lapin::options::BasicPublishOptions;
use lapin::publisher_confirm::Confirmation;
use lapin::BasicProperties;
use sdk::modules::ModulePackage;
use sdk::prelude::*;
use sdk::tracing::debug;
use std::sync::mpsc;

use crate::setup::Config;

pub async fn producer(
    setup_sender: ModuleSetupSender,
    config: Config,
    channel: lapin::Channel,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = mpsc::channel::<ModulePackage>();
    setup_sender.send(Some(tx)).unwrap();

    debug!("Producer started");

    for package in rx {
        debug!("Received package");
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

        debug!("Published message to {}", config.queue);

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
