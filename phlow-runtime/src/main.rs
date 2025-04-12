mod cli;
mod loader;
mod log;
mod memory;
mod package;
mod runtime;
mod settings;
mod yaml;
use cli::Cli;
use loader::Loader;
use log::init_tracing;
use package::Package;
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::prelude::*;
use phlow_sdk::tracing::error;
use runtime::Runtime;
use settings::Settings;

#[tokio::main]
async fn main() {
    let settings = Settings::load();
    let cli = Cli::load().expect("Error loading CLI");

    if let Some(publish_path) = cli.package_path {
        init_tracing();

        match Package::try_from(publish_path) {
            Ok(publish) => {
                if let Err(err) = publish.run() {
                    error!("Error publishing module: {:?}", err);
                    return;
                }
            }
            Err(err) => {
                error!("Error creating publish instance: {:?}", err);
                return;
            }
        }
    }

    if let Some(main) = &cli.main {
        let loader = match Loader::load(&main.path, &main.ext) {
            Ok(main) => main,
            Err(err) => {
                eprintln!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

        if cli.show_steps {
            let _ = loader.print_step_yaml();
        }

        if cli.no_run {
            return;
        }

        let guard = init_tracing_subscriber(loader.app_data.clone());

        loader
            .download(&settings.default_package_repository_url)
            .await
            .expect("Error downloading modules");

        if cli.only_download_modules {
            return;
        }

        Runtime::run(loader, guard.dispatch.clone(), settings).await;
    }
}
