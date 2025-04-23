mod loader;
mod log;
mod memory;
mod package;
mod runtime;
mod settings;
mod yaml;
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
    let settings = Settings::try_load().expect("Error loading settings");

    if let Some(publish_path) = settings.package_path.clone() {
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

    if let Some(main) = &settings.main_target {
        let mut loader = match Loader::load(&main).await {
            Ok(main) => main,
            Err(err) => {
                eprintln!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

        if settings.no_run {
            return;
        }

        let guard = init_tracing_subscriber(loader.app_data.clone());

        loader
            .download(&settings.default_package_repository_url)
            .await
            .expect("Error downloading modules");

        loader.update_info();

        if settings.only_download_modules {
            return;
        }

        Runtime::run(loader, guard.dispatch.clone(), settings).await;
    }
}
