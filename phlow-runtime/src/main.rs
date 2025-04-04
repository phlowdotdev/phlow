mod cli;
mod loader;
mod memory;
mod publish;
mod runtime;
mod settings;
mod yaml;
use cli::Cli;
use loader::Loader;
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::error;
use publish::Publish;
use runtime::Runtime;
use settings::Settings;

#[tokio::main]
async fn main() {
    let settings = Settings::load();
    let cli = Cli::load().expect("Error loading CLI");

    if let Some(publish_path) = cli.publish_path {
        Publish::try_from(publish_path)
            .expect("Error publishing module")
            .run()
            .expect("Error publishing module");
    }

    if let Some(main) = &cli.main {
        let loader = match Loader::load(&main.path, &main.ext) {
            Ok(main) => main,
            Err(err) => {
                error!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

        loader
            .download(&settings.default_package_repository_url)
            .await
            .expect("Error downloading modules");

        if cli.only_download_modules {
            return;
        }

        Runtime::run(loader, settings).await;
    }
}
