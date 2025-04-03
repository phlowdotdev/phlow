use sdk::{crossbeam::channel, modules::ModulePackage, prelude::*};

plugin_async!(echo);

pub async fn echo(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    match setup.setup_sender.send(Some(tx)) {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("{:?}", e).into());
        }
    };

    for package in rx {
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

    Ok(())
}
