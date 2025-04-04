mod cli;
mod loader;
mod memory;
mod publish;
mod runtime;
mod settings;
mod yaml;
use cli::Cli;
use crossbeam::channel;
use futures::future::join_all;
use loader::Loader;
use memory::force_memory_release;
use phlow_engine::{
    modules::{ModulePackage, Modules},
    Context, Phlow,
};
use phlow_sdk::tracing::{debug, dispatcher, error, info, warn};
use phlow_sdk::{otel::init_tracing_subscriber, prelude::*};
use runtime::Runtime;
use settings::Settings;
use std::{sync::Arc, thread};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let settings = Settings::load();
    let cli = Cli::load().expect("Error loading CLI");

    // if cli.publish_path.is_some() {
    //     // let publish_path = cli.publish_path.unwrap();
    //     // let publish_path = publish_path.trim_start_matches("phlow://");
    //     // let publish_path = publish_path.trim_end_matches(".phlow");
    //     // settings.set_publish_path(publish_path);
    // }

    let loader = match Loader::load(&cli.main_path, &cli.main_ext) {
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
