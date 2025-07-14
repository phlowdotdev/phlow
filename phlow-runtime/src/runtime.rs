use crate::loader::Loader;
#[cfg(target_env = "gnu")]
use crate::memory::force_memory_release;
use crate::settings::Settings;
use crossbeam::channel;
use futures::future::join_all;
use log::{debug, error, info};
use phlow_engine::phs::{build_engine, Script, ScriptError};
use phlow_engine::{Context, Phlow};
use phlow_sdk::structs::Package;
use phlow_sdk::tokio;
use phlow_sdk::{
    prelude::Value,
    structs::{ModulePackage, ModuleSetup, Modules},
    tracing::{self, dispatcher, Dispatch},
};
use std::fmt::Display;
use std::sync::Arc;
#[cfg(target_env = "gnu")]
use std::thread;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum RuntimeError {
    ModuleWithError(ScriptError),
    ModuleRegisterError,
    FlowExecutionError(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                    .evaluate_without_context()
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

        // Se não há main definido, forçar o início dos steps
        if loader.main == -1 {
            // Criar um span padrão para o início dos steps
            let span = tracing::span!(
                tracing::Level::INFO,
                "auto_start_steps",
                otel.name = "phlow auto start"
            );

            // Enviar um pacote vazio para iniciar os steps
            let empty_package = Package {
                response: None,
                request_data: None,
                origin: 0,
                span: Some(span),
                dispatch: Some(dispatch.clone()),
            };
            if let Err(err) = tx_main_package.send(empty_package) {
                error!("Failed to send empty package: {:?}", err);
                return Err(RuntimeError::FlowExecutionError("Failed to send empty package".to_string()));
            }
        }

        drop(tx_main_package);

        #[cfg(target_env = "gnu")]
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
                    let parent = match package.span.clone() {
                        Some(span) => span,
                        None => {
                            error!("Span not found in main module");
                            continue;
                        }
                    };
                    let dispatch = match package.dispatch.clone() {
                        Some(dispatch) => dispatch,
                        None => {
                            error!("Dispatch not found in main module");
                            continue;
                        }
                    };

                    tokio::task::block_in_place(move || {
                        dispatcher::with_default(&dispatch, || {
                            let _enter = parent.enter();
                            let rt = tokio::runtime::Handle::current();

                            rt.block_on(async {
                                // Se há dados, use-os; senão, use um contexto vazio
                                let data = package.get_data().cloned().unwrap_or(Value::Null);
                                let mut context = Context::from_main(data);
                                match flow.execute(&mut context).await {
                                    Ok(result) => {
                                        let result_value = result.unwrap_or(Value::Null);
                                        // Se não há response (módulo principal), imprimir o resultado
                                        if package.response.is_none() {
                                            println!("{}", result_value);
                                        }
                                        package.send(result_value);
                                    }
                                    Err(err) => {
                                        error!("Runtime Error Execute Steps: {:?}", err);
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
