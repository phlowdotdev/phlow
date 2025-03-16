use sdk::prelude::*;

plugin_async!(echo);

pub async fn echo(
    _id: ModuleId,
    _sender: MainRuntimeSender,
    _setup: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello from echo module!");
    Ok(())
}
