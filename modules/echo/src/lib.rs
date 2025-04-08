use phlow_sdk::prelude::*;

create_step!(echo(rx));

pub async fn echo(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        package.context.input.unwrap_or(Value::Null)
    });

    Ok(())
}
