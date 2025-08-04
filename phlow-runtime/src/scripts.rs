use crate::runtime::Runtime;
use crate::settings::Settings;
use log::{debug, error};
use phlow_engine::Context;
use phlow_sdk::{
    module_channel,
    structs::{ModuleResponse, ModuleSetup},
};
use phlow_sdk::{otel, prelude::*};

use crate::loader::Loader;

pub fn run_script(path: &str, setup: ModuleSetup, settings: &Settings) {
    debug!("Running script at path: {}", path);
    let dispatch = setup.dispatch.clone();

    tracing::dispatcher::with_default(&dispatch, || {
        let _guard = otel::init_tracing_subscriber(setup.app_data.clone());
        use_log!();

        if let Ok(rt) = tokio::runtime::Runtime::new() {
            rt.block_on(async move {
                let loader = Loader::load(&path, settings.print_yaml).await.unwrap();
                let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();
                let app_data = loader.app_data.clone();
                let dispatch = setup.dispatch.clone();
                let dispatch_for_runtime = dispatch.clone();
                let settings_cloned = settings.clone();

                // Criar uma task para o runtime que não irá dropar o tx_main_package
                let tx_for_runtime = tx_main_package.clone();
                let context = Context::from_setup(setup.with.clone());

                let runtime_handle = tokio::task::spawn(async move {
                    Runtime::run_script(
                        tx_for_runtime,
                        rx_main_package,
                        loader,
                        dispatch_for_runtime,
                        settings_cloned,
                        context,
                    )
                    .await
                });

                let rx = module_channel!(setup);

                debug!("Script module loaded, starting main loop");

                for package in rx {
                    debug!("Received package: {:?}", package);

                    let span = tracing::span!(
                        tracing::Level::INFO,
                        "auto_start_steps",
                        otel.name = app_data.name.clone().unwrap_or("unknown".to_string()),
                    );

                    // Criar um canal para receber a resposta do runtime
                    let (response_tx, response_rx) = tokio::sync::oneshot::channel::<Value>();

                    let runtime_package = Package {
                        response: Some(response_tx),
                        request_data: package.input(),
                        origin: 0,
                        span: Some(span),
                        dispatch: Some(dispatch.clone()),
                    };

                    debug!("Sending package to main loop: {:?}", runtime_package);

                    if let Err(err) = tx_main_package.send(runtime_package) {
                        error!("Failed to send package: {:?}", err);
                        continue;
                    }

                    debug!("Package sent to main loop, waiting for response");

                    // Aguardar a resposta do runtime sem timeout
                    match response_rx.await {
                        Ok(result) => {
                            println!("Received response: {:?}", result);
                            let response = ModuleResponse::from_success(result);
                            if let Err(err) = package.sender.send(response) {
                                error!("Failed to send response back to module: {:?}", err);
                            }
                        }
                        Err(err) => {
                            let response =
                                ModuleResponse::from_error(format!("Runtime error: {}", err));
                            if let Err(err) = package.sender.send(response) {
                                error!("Failed to send error response back to module: {:?}", err);
                            }
                        }
                    }

                    debug!("Response sent back to module");
                }

                debug!("Script module no listeners, waiting for runtime to finish");

                runtime_handle
                    .await
                    .unwrap_or_else(|err| {
                        error!("Runtime task error: {:?}", err);
                        std::process::exit(1);
                    })
                    .unwrap_or_else(|err| {
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
