use sdk::{crossbeam::channel, modules::ModulePackage, prelude::*};

plugin_async!(echo);

pub async fn echo(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    setup.setup_sender.send(Some(tx)).unwrap();

    for package in rx {
        package
            .sender
            .send(match package.context.input {
                Some(value) => value,
                _ => Value::Null,
            })
            .unwrap();
    }

    Ok(())
}
