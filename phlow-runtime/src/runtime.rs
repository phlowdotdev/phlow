use crate::loader::{Loader, load_module};
use crate::inline_module::{InlineModules, PhlowModuleRequest};
#[cfg(target_env = "gnu")]
use crate::memory::force_memory_release;
use crate::settings::Settings;
use crossbeam::channel;
use futures::future::join_all;
use log::{debug, error, info, warn};
use phlow_engine::phs::{Script, ScriptError, build_engine};
use phlow_engine::{Context, Phlow};
use phlow_sdk::structs::Package;
use phlow_sdk::tokio;
use phlow_sdk::{
    prelude::{Array, Value},
    structs::{ModulePackage, ModuleSetup, Modules},
    tracing::{self, Dispatch, dispatcher},
};
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::Arc;
use std::thread;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum RuntimeError {
    ModuleWithError(ScriptError),
    ModuleRegisterError,
    FlowExecutionError(String),
    InlineModuleError(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ModuleRegisterError => write!(f, "Module register error"),
            RuntimeError::FlowExecutionError(err) => write!(f, "Flow execution error: {}", err),
            RuntimeError::ModuleWithError(err) => write!(f, "Module with error: {}", err),
            RuntimeError::InlineModuleError(err) => write!(f, "Inline module error: {}", err),
        }
    }
}

fn parse_cli_value(flag: &str, value: &str) -> Result<Value, RuntimeError> {
    match Value::json_to_value(value) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            error!("Failed to parse --{} value '{}': {:?}", flag, value, err);
            Err(RuntimeError::FlowExecutionError(format!(
                "Failed to parse --{} value: {:?}",
                flag, err
            )))
        }
    }
}

pub struct Runtime {}

fn spawn_inline_module_worker(
    name: String,
    handler: crate::inline_module::PhlowModuleHandler,
    with: Value,
    app_data: phlow_sdk::structs::ApplicationData,
    dispatch: Dispatch,
    runtime_handle: tokio::runtime::Handle,
    receiver: channel::Receiver<ModulePackage>,
) {
    thread::spawn(move || {
        for package in receiver {
            let request = PhlowModuleRequest {
                input: package.input(),
                payload: package.payload(),
                with: with.clone(),
                app_data: app_data.clone(),
                dispatch: dispatch.clone(),
            };

            let response = dispatcher::with_default(&dispatch, || {
                runtime_handle.block_on((handler)(request))
            });

            if package.sender.send(response).is_err() {
                debug!("Inline module '{}' response channel closed", name);
            }
        }

        debug!("Inline module '{}' stopped", name);
    });
}

impl Runtime {
    async fn load_modules(
        loader: Loader,
        dispatch: Dispatch,
        settings: Settings,
        tx_main_package: channel::Sender<Package>,
        inline_modules: &InlineModules,
    ) -> Result<Modules, RuntimeError> {
        let mut modules = Modules::default();
        let engine = build_engine(None);
        let runtime_handle = tokio::runtime::Handle::current();
        // -------------------------
        // Load the modules
        // -------------------------
        let app_data = loader.app_data.clone();
        let loader_main_id = loader.main.clone();
        let mut unused_inline: HashSet<String> = inline_modules.keys().cloned().collect();

        for (id, module) in loader.modules.into_iter().enumerate() {
            let (setup_sender, setup_receive) =
                oneshot::channel::<Option<channel::Sender<ModulePackage>>>();

            let is_main = loader_main_id == id as i32;
            // Se --var-main foi especificado, não permitir que módulos principais sejam executados
            let main_sender = if is_main && settings.var_main.is_none() {
                Some(tx_main_package.clone())
            } else {
                None
            };

            let with = {
                let script = match Script::try_build(engine.clone(), &module.with) {
                    Ok(payload) => payload,
                    Err(err) => return Err(RuntimeError::ModuleWithError(err)),
                };

                let with: Value = script
                    .evaluate_without_context()
                    .map_err(|err| RuntimeError::ModuleWithError(err))?;

                log::debug!(
                    "Module '{}' with: {}",
                    module.name,
                    with.to_json(phlow_sdk::prelude::JsonMode::Indented)
                ); // Debug print
                with
            };

            let inline_module = inline_modules.get(&module.name).cloned();
            if inline_module.is_some() {
                unused_inline.remove(&module.name);
            }

            if inline_module.is_some() && is_main && settings.var_main.is_none() {
                return Err(RuntimeError::InlineModuleError(format!(
                    "Inline module '{}' is declared as main, but runtime is waiting for main output",
                    module.name
                )));
            }

            let mut module_data = module;
            let mut inline_worker = None;

            if let Some(inline_module) = inline_module {
                let handler = inline_module.handler().ok_or_else(|| {
                    RuntimeError::InlineModuleError(format!(
                        "Inline module '{}' is missing a handler",
                        module_data.name
                    ))
                })?;

                let schema = inline_module.schema();
                if !schema.input.is_null() {
                    module_data.input = schema.input.clone();
                }
                if !schema.output.is_null() {
                    module_data.output = schema.output.clone();
                }
                if !schema.input_order.is_empty() {
                    module_data.input_order = Value::Array(Array::from(schema.input_order.clone()));
                }
                module_data.with = with.clone();

                let (sender, receiver) = channel::unbounded::<ModulePackage>();
                if setup_sender.send(Some(sender)).is_err() {
                    return Err(RuntimeError::InlineModuleError(format!(
                        "Inline module '{}' failed to register",
                        module_data.name
                    )));
                }

                inline_worker = Some((
                    module_data.name.clone(),
                    handler,
                    with,
                    app_data.clone(),
                    dispatch.clone(),
                    runtime_handle.clone(),
                    receiver,
                ));
            } else {
                let setup = ModuleSetup {
                    id,
                    setup_sender,
                    main_sender,
                    with,
                    dispatch: dispatch.clone(),
                    app_data: app_data.clone(),
                    is_test_mode: false,
                };

                let module_target = module_data.module.clone();
                let module_version = module_data.version.clone();
                let local_path = module_data.local_path.clone();
                let settings = settings.clone();

                thread::spawn(move || {
                    let result: Result<(), crate::loader::error::Error> =
                        load_module(setup, &module_target, &module_version, local_path, settings);

                    if let Err(err) = result {
                        error!("Runtime Error Load Module: {:?}", err)
                    }
                });

                debug!(
                    "Module {} loaded with name \"{}\" and version \"{}\"",
                    module_data.module, module_data.name, module_data.version
                );
            }

            match setup_receive.await {
                Ok(Some(sender)) => {
                    debug!("Module {} registered", module_data.name);
                    modules.register(module_data, sender);
                }
                Ok(None) => {
                    debug!("Module {} did not register", module_data.name);
                }
                Err(_) => {
                    return Err(RuntimeError::ModuleRegisterError);
                }
            }

            if let Some((
                name,
                handler,
                with,
                app_data,
                dispatch,
                runtime_handle,
                receiver,
            )) = inline_worker
            {
                spawn_inline_module_worker(
                    name,
                    handler,
                    with,
                    app_data,
                    dispatch,
                    runtime_handle,
                    receiver,
                );
            }
        }

        if !unused_inline.is_empty() {
            warn!(
                "Inline modules not declared in pipeline: {}",
                unused_inline.into_iter().collect::<Vec<_>>().join(", ")
            );
        }

        Ok(modules)
    }

    async fn listener(
        rx_main_package: channel::Receiver<Package>,
        steps: Value,
        modules: Modules,
        settings: Settings,
        default_context: Option<Context>,
    ) -> Result<(), RuntimeError> {
        let phlow = Arc::new({
            match Phlow::try_from_value(&steps, Some(Arc::new(modules))) {
                Ok(phlow) => phlow,
                Err(err) => return Err(RuntimeError::FlowExecutionError(err.to_string())),
            }
        });
        if let Some(controller) = phlow_engine::debug::debug_controller() {
            controller.set_script(phlow.script()).await;
        }

        let start_step = if let Some(step_id) = settings.start_step.as_deref() {
            match phlow.find_step_reference(step_id) {
                Some(step_ref) => Some(step_ref),
                None => {
                    return Err(RuntimeError::FlowExecutionError(format!(
                        "Step id '{}' not found",
                        step_id
                    )));
                }
            }
        } else {
            None
        };

        drop(steps);

        let mut handles = Vec::new();
        let default_context = default_context.clone();

        for _i in 0..settings.package_consumer_count {
            let rx_main_pkg = rx_main_package.clone();
            let phlow = phlow.clone();
            let default_context = default_context.clone();
            let start_step = start_step.clone();

            let handle = tokio::task::spawn_blocking(move || {
                for mut main_package in rx_main_pkg {
                    let phlow = phlow.clone();
                    let parent = match main_package.span.clone() {
                        Some(span) => span,
                        None => {
                            error!("Span not found in main module");
                            continue;
                        }
                    };
                    let dispatch = match main_package.dispatch.clone() {
                        Some(dispatch) => dispatch,
                        None => {
                            error!("Dispatch not found in main module");
                            continue;
                        }
                    };

                    let mut context = {
                        let data = main_package.get_data().cloned().unwrap_or(Value::Null);
                        if let Some(mut context) = default_context.clone() {
                            context.set_main(data);
                            context
                        } else {
                            Context::from_main(data)
                        }
                    };
                    let start_step = start_step.clone();

                    tokio::task::block_in_place(move || {
                        dispatcher::with_default(&dispatch, || {
                            let _enter = parent.enter();
                            let rt = tokio::runtime::Handle::current();

                            rt.block_on(async {
                                let result = if let Some(step_ref) = start_step.clone() {
                                    phlow.execute_from(&mut context, step_ref).await
                                } else {
                                    phlow.execute(&mut context).await
                                };
                                match result {
                                    Ok(result) => {
                                        let result_value = result.unwrap_or(Value::Undefined);
                                        main_package.send(result_value);
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

    pub async fn run(
        loader: Loader,
        dispatch: Dispatch,
        settings: Settings,
    ) -> Result<(), RuntimeError> {
        // -------------------------
        // Create the channels
        // -------------------------
        let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();

        let var_payload_value = match &settings.var_payload {
            Some(var_payload_str) => Some(parse_cli_value("var-payload", var_payload_str)?),
            None => None,
        };
        let default_context = var_payload_value.as_ref().map(|payload| {
            let mut context = Context::new();
            context.add_step_payload(Some(payload.clone()));
            context
        });

        let no_main = loader.main == -1 || settings.var_main.is_some();
        let steps = loader.get_steps();
        let inline_modules = InlineModules::default();
        let modules = Self::load_modules(
            loader,
            dispatch.clone(),
            settings.clone(),
            tx_main_package.clone(),
            &inline_modules,
        )
        .await?;

        // Se não há main definido ou --var-main foi especificado, forçar o início dos steps
        if no_main {
            // Criar um span padrão para o início dos steps
            let span = tracing::span!(
                tracing::Level::INFO,
                "auto_start_steps",
                otel.name = "phlow auto start"
            );

            // Se --var-main foi especificado, processar o valor usando valu3
            let request_data = if let Some(var_main_str) = &settings.var_main {
                // Usar valu3 para processar o valor da mesma forma que outros valores
                Some(parse_cli_value("var-main", var_main_str)?)
            } else {
                None
            };

            // Enviar um pacote com os dados do --var-main para iniciar os steps
            let package = Package {
                response: None,
                request_data,
                origin: 0,
                span: Some(span),
                dispatch: Some(dispatch.clone()),
            };

            if let Err(err) = tx_main_package.send(package) {
                error!("Failed to send package: {:?}", err);
                return Err(RuntimeError::FlowExecutionError(
                    "Failed to send package".to_string(),
                ));
            }

            if settings.var_main.is_some() {
                info!("Using --var-main to simulate main module output");
            }
        }

        drop(tx_main_package);

        #[cfg(target_env = "gnu")]
        if settings.garbage_collection {
            thread::spawn(move || {
                loop {
                    thread::sleep(std::time::Duration::from_secs(
                        settings.garbage_collection_interval,
                    ));
                    force_memory_release(settings.min_allocated_memory);
                }
            });
        }

        info!("Phlow!");

        // -------------------------
        // Create the phlow
        // -------------------------
        Self::listener(rx_main_package, steps, modules, settings, default_context)
            .await
            .map_err(|err| {
                error!("Runtime Error: {:?}", err);
                err
            })?;

        Ok(())
    }

    pub async fn run_script(
        tx_main_package: channel::Sender<Package>,
        rx_main_package: channel::Receiver<Package>,
        loader: Loader,
        dispatch: Dispatch,
        settings: Settings,
        context: Context,
    ) -> Result<(), RuntimeError> {
        let inline_modules = InlineModules::default();
        Self::run_script_with_modules(
            tx_main_package,
            rx_main_package,
            loader,
            dispatch,
            settings,
            context,
            inline_modules,
        )
        .await
    }

    pub async fn run_script_with_modules(
        tx_main_package: channel::Sender<Package>,
        rx_main_package: channel::Receiver<Package>,
        loader: Loader,
        dispatch: Dispatch,
        settings: Settings,
        context: Context,
        inline_modules: InlineModules,
    ) -> Result<(), RuntimeError> {
        let steps = loader.get_steps();
        let context = if let Some(var_payload_str) = &settings.var_payload {
            let payload = parse_cli_value("var-payload", var_payload_str)?;
            context.clone_with_output(payload)
        } else {
            context
        };

        let modules = Self::load_modules(
            loader,
            dispatch.clone(),
            settings.clone(),
            tx_main_package.clone(),
            &inline_modules,
        )
        .await?;

        drop(tx_main_package);

        Self::listener(rx_main_package, steps, modules, settings, Some(context))
            .await
            .map_err(|err| {
                error!("Runtime Error: {:?}", err);
                err
            })?;

        Ok(())
    }
}
