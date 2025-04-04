use crate::loader::Loader;
use crate::memory::force_memory_release;
use crate::settings::Settings;
use crossbeam::channel;
use futures::future::join_all;
use phlow_engine::{
    modules::{ModulePackage, Modules},
    Context, Phlow,
};
use phlow_sdk::tracing::{debug, dispatcher, error, info, warn};
use phlow_sdk::{otel::init_tracing_subscriber, prelude::*};
use std::{sync::Arc, thread};
use tokio::sync::oneshot;

pub struct Runtime {}

impl Runtime {
    pub async fn run(loader: Loader, settings: Settings) {
        let guard = init_tracing_subscriber();

        let steps: Value = loader.get_steps();
        let mut modules = Modules::default();

        // -------------------------
        // Create the channels
        // -------------------------
        let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

        // -------------------------
        // Load the modules
        // -------------------------
        for (id, module) in loader.modules.into_iter().enumerate() {
            let (setup_sender, setup_receive) =
                oneshot::channel::<Option<channel::Sender<ModulePackage>>>();

            let main_sender = if loader.main == id as i32 {
                Some(tx_main_package.clone())
            } else {
                None
            };

            let setup = ModuleSetup {
                id,
                setup_sender,
                main_sender,
                with: module.with.clone(),
                dispatch: guard.dispatch.clone(),
            };

            let module_target = module.module.clone();

            std::thread::spawn(move || {
                if let Err(err) = Loader::load_module(setup, &module_target) {
                    error!("Runtime Error Load Module: {:?}", err)
                }
            });

            debug!(
                "Module {} loaded with name \"{}\"",
                module.module, module.name
            );

            match setup_receive.await {
                Ok(Some(sender)) => {
                    debug!("Module {} registered", module.name);
                    modules.register(&module.name, sender);
                }
                Ok(None) => {
                    debug!("Module {} did not register", module.name);
                }
                Err(err) => {
                    error!("Runtime Error Setup Receive: {:?}", err);
                    return;
                }
            }
        }

        if loader.main == -1 {
            error!("Runtime Error Main Module: No main module found");
            return;
        }

        drop(tx_main_package);

        #[cfg(target_os = "linux")]
        if settings.garbage_collection {
            thread::spawn(move || loop {
                thread::sleep(std::time::Duration::from_secs(
                    settings.garbage_collection_interval,
                ));
                force_memory_release(settings.min_allocated_memory);
            });
        }

        // -------------------------
        // Create the flow
        // -------------------------
        let flow = Arc::new({
            match Phlow::try_from_value(&steps, Some(Arc::new(modules)), None) {
                Ok(flow) => flow,
                Err(err) => {
                    error!("Runtime Error To Value: {:?}", err);
                    return;
                }
            }
        });

        let mut handles = Vec::new();

        info!("Phlow!");

        for _i in 0..settings.package_consumer_count {
            let rx_pkg = rx_main_package.clone();
            let flow = flow.clone();

            let handle = tokio::task::spawn_blocking(move || {
                for mut package in rx_pkg {
                    let flow = flow.clone();
                    let parent = package.span.clone().expect("Span not found in main module");
                    let dispatch = package
                        .dispatch
                        .clone()
                        .expect("Dispatch not found in main module");

                    tokio::task::block_in_place(move || {
                        dispatcher::with_default(&dispatch, || {
                            let _enter = parent.enter();
                            let rt = tokio::runtime::Handle::current();

                            rt.block_on(async {
                                if let Some(data) = package.get_data() {
                                    let mut context = Context::from_main(data.clone());
                                    match flow.execute(&mut context).await {
                                        Ok(result) => {
                                            package.send(result.unwrap_or(Value::Null));
                                        }
                                        Err(err) => {
                                            warn!("Runtime Error Execute Steps: {:?}", err);
                                        }
                                    }
                                }
                            });
                        });
                    });
                }
            });

            handles.push(handle);
        }

        join_all(handles).await;
    }
}
