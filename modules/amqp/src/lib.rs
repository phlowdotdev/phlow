pub mod setup;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use sdk::prelude::*;
use setup::Setup;

plugin_async!(send_message);

pub async fn send_message(
    id: ModuleId,
    sender: RuntimeSender,
    setup: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let setup: Setup = Setup::from(setup);

    let addr = format!(
        "amqp://{}:{}@{}:{}",
        setup.username.unwrap_or("guest".to_string()),
        setup.password.unwrap_or("guest".to_string()),
        setup.host.unwrap_or("localhost".to_string()),
        setup.port.unwrap_or(5672),
    );

    println!("Connecting to {}", addr);

    Ok(())
}
