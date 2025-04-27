use crate::loader::Loader;
use crate::memory::force_memory_release;
use crate::settings::Settings;
use crossbeam::channel;
use futures::future::join_all;
use log::{debug, error, info};
use phlow_engine::phs::{build_engine, Script, ScriptError};
use phlow_engine::{Context, Phlow};
use phlow_sdk::prelude::ToValueBehavior;
use phlow_sdk::structs::Package;
use phlow_sdk::tokio;
use phlow_sdk::{
    prelude::Value,
    structs::{ModulePackage, ModuleSetup, Modules},
    tracing::{dispatcher, Dispatch},
};
use std::collections::HashMap;
use std::fmt::Display;
use std::{sync::Arc, thread};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum RuntimeError {
    MainModuleNotFound,
    ModuleWithError(ScriptError),
    ModuleRegisterError,
    FlowExecutionError(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::MainModuleNotFound => write!(f, "Main module not found"),
            RuntimeError::ModuleRegisterError => write!(f, "Module register error"),
            RuntimeError::FlowExecutionError(err) => write!(f, "Flow execution error: {}", err),
            RuntimeError::ModuleWithError(err) => write!(f, "Module with error: {}", err),
        }
    }
}

pub struct Runtime {}

impl Runtime {
    pub async fn run(
        loader: Loader,
        dispatch: Dispatch,
        settings: Settings,
    ) -> Result<(), RuntimeError> {
        let mut modules = Modules::default();
        let steps: Value = loader.get_steps();

        // -------------------------
        // Create the channels
        // -------------------------
        let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

        let engine = build_engine(None);
        let envs = HashMap::from([("envs".to_string(), Context::get_all_envs().to_value())]);
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

            let with = {
                let script = match Script::try_build(engine.clone(), &module.with) {
                    Ok(payload) => payload,
                    Err(err) => return Err(RuntimeError::ModuleWithError(err)),
                };

                let with = script
                    .evaluate(&envs)
                    .map_err(|err| RuntimeError::ModuleWithError(err))?;

                with
            };

            let setup = ModuleSetup {
                id,
                setup_sender,
                main_sender,
                with,
                dispatch: dispatch.clone(),
                app_data: loader.app_data.clone(),
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

                    modules.register(module, sender);
                }
                Ok(None) => {
                    debug!("Module {} did not register", module.name);
                }
                Err(_) => {
                    return Err(RuntimeError::ModuleRegisterError);
                }
            }
        }

        if loader.main == -1 {
            return Err(RuntimeError::MainModuleNotFound);
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
            match Phlow::try_from_value(&steps, Some(Arc::new(modules))) {
                Ok(flow) => flow,
                Err(err) => return Err(RuntimeError::FlowExecutionError(err.to_string())),
            }
        });

        drop(steps);

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
                                            error!("Runtime Error Execute Steps: {:?}", err);
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

        Ok(())
    }
}
