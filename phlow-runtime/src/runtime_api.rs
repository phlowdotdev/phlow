use crate::debug_server;
use crate::loader::Loader;
use crate::runtime::Runtime;
use crate::runtime::RuntimeError;
use crate::settings::Settings;
use crossbeam::channel;
use phlow_engine::Context;
use phlow_sdk::otel::{OtelGuard, init_tracing_subscriber};
use phlow_sdk::prelude::Value;
use phlow_sdk::structs::Package;
use phlow_sdk::{tracing, use_log};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub enum PhlowRuntimeError {
    MissingPipeline,
    LoaderError(crate::loader::error::Error),
    PackageSendError,
    ResponseChannelClosed,
    RuntimeError(RuntimeError),
    RuntimeJoinError(tokio::task::JoinError),
}

impl Display for PhlowRuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PhlowRuntimeError::MissingPipeline => write!(f, "Pipeline not set"),
            PhlowRuntimeError::LoaderError(err) => write!(f, "Loader error: {}", err),
            PhlowRuntimeError::PackageSendError => write!(f, "Failed to send package"),
            PhlowRuntimeError::ResponseChannelClosed => write!(f, "Response channel closed"),
            PhlowRuntimeError::RuntimeError(err) => write!(f, "Runtime error: {}", err),
            PhlowRuntimeError::RuntimeJoinError(err) => write!(f, "Runtime task error: {}", err),
        }
    }
}

impl std::error::Error for PhlowRuntimeError {}

impl From<crate::loader::error::Error> for PhlowRuntimeError {
    fn from(err: crate::loader::error::Error) -> Self {
        PhlowRuntimeError::LoaderError(err)
    }
}

impl From<RuntimeError> for PhlowRuntimeError {
    fn from(err: RuntimeError) -> Self {
        PhlowRuntimeError::RuntimeError(err)
    }
}

pub struct PhlowRuntime {
    pipeline: Option<Value>,
    context: Option<Context>,
    settings: Settings,
    base_path: Option<PathBuf>,
    dispatch: Option<tracing::Dispatch>,
    prepared: Option<PreparedRuntime>,
}

impl Default for PhlowRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PhlowRuntime {
    pub fn new() -> Self {
        let mut settings = Settings::for_runtime();
        if settings.var_main.is_none() {
            settings.var_main = Some("__phlow_runtime__".to_string());
        }

        Self {
            pipeline: None,
            context: None,
            settings,
            base_path: None,
            dispatch: None,
            prepared: None,
        }
    }

    pub fn with_settings(settings: Settings) -> Self {
        Self {
            pipeline: None,
            context: None,
            settings,
            base_path: None,
            dispatch: None,
            prepared: None,
        }
    }

    pub fn set_pipeline(&mut self, pipeline: Value) -> &mut Self {
        self.pipeline = Some(pipeline);
        self.prepared = None;
        self
    }

    pub fn set_context(&mut self, context: Context) -> &mut Self {
        self.context = Some(context);
        self.prepared = None;
        self
    }

    pub fn set_settings(&mut self, settings: Settings) -> &mut Self {
        self.settings = settings;
        self.prepared = None;
        self
    }

    pub fn set_base_path<P: Into<PathBuf>>(&mut self, base_path: P) -> &mut Self {
        self.base_path = Some(base_path.into());
        self.prepared = None;
        self
    }

    pub fn set_dispatch(&mut self, dispatch: tracing::Dispatch) -> &mut Self {
        self.dispatch = Some(dispatch);
        self.prepared = None;
        self
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        self.prepared = None;
        &mut self.settings
    }

    pub async fn build(&mut self) -> Result<(), PhlowRuntimeError> {
        if self.prepared.is_some() {
            return Ok(());
        }

        use_log!();

        let pipeline = self
            .pipeline
            .as_ref()
            .ok_or(PhlowRuntimeError::MissingPipeline)?;

        let base_path = self.base_path.clone().unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./"))
        });

        let mut loader = Loader::from_value(pipeline, Some(base_path.as_path()))?;

        if self.settings.download {
            loader
                .download(&self.settings.default_package_repository_url)
                .await?;
        }

        loader.update_info();

        let mut guard: Option<OtelGuard> = None;
        let dispatch = if let Some(dispatch) = self.dispatch.clone() {
            dispatch
        } else {
            let next_guard = init_tracing_subscriber(loader.app_data.clone());
            let dispatch = next_guard.dispatch.clone();
            guard = Some(next_guard);
            dispatch
        };

        let debug_enabled = std::env::var("PHLOW_DEBUG")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if debug_enabled {
            let controller = Arc::new(phlow_engine::debug::DebugController::new());
            match debug_server::spawn(controller.clone()).await {
                Ok(()) => {
                    if phlow_engine::debug::set_debug_controller(controller).is_err() {
                        log::warn!("Debug controller already set");
                    }
                    log::info!("Phlow debug enabled");
                }
                Err(err) => {
                    log::error!("Failed to start debug server: {}", err);
                }
            }
        }

        let app_name = loader
            .app_data
            .name
            .clone()
            .unwrap_or_else(|| "phlow runtime".to_string());

        let context = self.context.clone().unwrap_or_else(Context::new);
        let request_data = context.get_main();
        let context_for_runtime = context.clone();

        let settings = self.settings.clone();
        let (tx_main_package, rx_main_package) = channel::unbounded::<Package>();
        let tx_for_runtime = tx_main_package.clone();
        let dispatch_for_runtime = dispatch.clone();

        let runtime_handle = tokio::spawn(async move {
            tracing::dispatcher::with_default(&dispatch_for_runtime, || {
                Runtime::run_script(
                    tx_for_runtime,
                    rx_main_package,
                    loader,
                    dispatch_for_runtime.clone(),
                    settings,
                    context_for_runtime,
                )
            })
            .await
        });

        self.prepared = Some(PreparedRuntime {
            tx_main_package,
            dispatch,
            runtime_handle,
            guard,
            app_name,
            request_data,
        });

        Ok(())
    }

    pub async fn run(&mut self) -> Result<Value, PhlowRuntimeError> {
        self.build().await?;

        let prepared = self
            .prepared
            .take()
            .ok_or(PhlowRuntimeError::MissingPipeline)?;

        let (response_tx, response_rx) = tokio::sync::oneshot::channel::<Value>();
        let package = tracing::dispatcher::with_default(&prepared.dispatch, || {
            let span = tracing::span!(
                tracing::Level::INFO,
                "phlow_run",
                otel.name = prepared.app_name.as_str()
            );

            Package {
                response: Some(response_tx),
                request_data: prepared.request_data.clone(),
                origin: 0,
                span: Some(span),
                dispatch: Some(prepared.dispatch.clone()),
            }
        });

        if prepared.tx_main_package.send(package).is_err() {
            return Err(PhlowRuntimeError::PackageSendError);
        }

        drop(prepared.tx_main_package);

        let result = response_rx
            .await
            .map_err(|_| PhlowRuntimeError::ResponseChannelClosed)?;

        let runtime_result = prepared
            .runtime_handle
            .await
            .map_err(PhlowRuntimeError::RuntimeJoinError)?;
        runtime_result?;

        drop(prepared.guard);

        Ok(result)
    }
}

struct PreparedRuntime {
    tx_main_package: channel::Sender<Package>,
    dispatch: tracing::Dispatch,
    runtime_handle: tokio::task::JoinHandle<Result<(), RuntimeError>>,
    guard: Option<OtelGuard>,
    app_name: String,
    request_data: Option<Value>,
}
