mod client;
mod server;
mod setup;
use phlow_sdk::prelude::*;
use setup::Config;

create_main!(start_server(setup));

#[tarpc::service]
pub trait RPC {
    /// Returns a greeting for name.
    async fn call(input: Value) -> Value;
}

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let is_main = setup.is_main();
    let config = Config::from(setup.with);

    let rx: channel::Receiver<ModulePackage> = module_channel!(setup);

    if is_main {
        let config_clone = config.clone();
        tokio::spawn(async move {
            server::main(config_clone).await.unwrap();
        });
    };

    client::main(rx, config).await;

    Ok(())
}
