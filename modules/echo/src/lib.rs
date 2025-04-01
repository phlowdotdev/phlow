use sdk::{crossbeam::channel, modules::ModulePackage, prelude::*};

// plugin_async!(echo);
#[no_mangle]
pub extern "C" fn plugin(setup: ModuleSetup) {
    sdk::otel::init_tracing_subscriber_plugin().expect("failed to initialize tracing");

    if let Ok(rt) = tokio::runtime::Runtime::new() {
        if let Err(e) = rt.block_on(echo(setup)) {
            sdk::tracing::error!("Error in plugin: {:?}", e);
        }
    } else {
        sdk::tracing::error!("Error creating runtime");
        return;
    };
}

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
