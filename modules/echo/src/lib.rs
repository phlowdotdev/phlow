use phlow_sdk::prelude::*;

create_step!(echo(rx));

pub async fn echo(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let input = package.input().unwrap_or(Value::Null);
        sender_safe!(package.sender, input.into());
    });

    Ok(())
}
