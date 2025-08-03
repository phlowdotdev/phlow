use crate::runtime::Runtime;
use crate::settings::Settings;
use log::error;
use phlow_sdk::{module_channel, structs::ModuleSetup};
use phlow_sdk::{otel, prelude::*};

use crate::loader::Loader;

pub async fn run_script(path: &str, setup: ModuleSetup, settings: &Settings) {
    let loader = Loader::load(&path, settings.print_yaml).await.unwrap();

    let rx = module_channel!(setup);

    let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

    let app_data = loader.app_data.clone();
    let dispatch = setup.dispatch.clone();

    tracing::dispatcher::with_default(&dispatch, || {
        let _guard = otel::init_tracing_subscriber(setup.app_data.clone());
        use_log!();

        let dispatch = dispatch.clone();

        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async move {
                let runtime = Runtime::run_script(
                    tx_main_package.clone(),
                    rx_main_package,
                    loader,
                    dispatch.clone(),
                    settings.clone(),
                );

                for package in rx {
                    let span = tracing::span!(
                        tracing::Level::INFO,
                        "auto_start_steps",
                        otel.name = app_data.name.clone().unwrap_or("unknown".to_string()),
                    );

                    let package = Package {
                        response: None,
                        request_data: package.input(),
                        origin: 0,
                        span: Some(span),
                        dispatch: Some(dispatch.clone()),
                    };

                    if let Err(err) = tx_main_package.send(package) {
                        error!("Failed to send package: {:?}", err);
                    }
                }

                runtime.await.unwrap_or_else(|err| {
                    error!("Runtime error: {:?}", err);
                    std::process::exit(1);
                });
            });
        } else {
            tracing::error!("Error creating runtime");
            return;
        }
    });
}

fn handler(setup: ModuleSetup) {}
