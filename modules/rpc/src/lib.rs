mod client;
mod server;
mod setup;
use phlow_sdk::prelude::*;
use setup::Config;

create_main!(start_server(setup));

#[tarpc::service]
pub trait World {
    /// Returns a greeting for name.
    async fn hello(name: String) -> String;
}

pub async fn start_server(
    setup: ModuleSetup,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let is_main = setup.is_main();
    let config = Config::from(setup.with);

    if is_main {
        return server::main(config).await;
    }

    return Ok(());
}
