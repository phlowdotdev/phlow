use phlow_sdk::{crossbeam::channel, modules::ModulePackage, prelude::*};

plugin_async!(echo);

pub async fn echo(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    sender_safe!(setup.setup_sender, Some(tx));

    listen!(rx, resolve);

    Ok(())
}

async fn resolve(package: ModulePackage) {
    match package.sender.send(match package.context.input {
        Some(value) => value,
        _ => Value::Null,
    }) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending package: {:?}", e);
        }
    }
}
