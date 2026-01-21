//! Phlow runtime API for in-memory pipelines.
//!
//! # Example
//!
//! ```no_run
//! use phlow_engine::Context;
//! use phlow_runtime::PhlowBuilder;
//! use phlow_sdk::prelude::json;
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let pipeline = json!({
//!     "steps": [
//!         { "payload": "{{ main.name }}" }
//!     ]
//! });
//! let context = Context::from_main(json!({ "name": "Phlow" }));
//!
//! let mut builder = PhlowBuilder::new();
//! builder.settings_mut().download = false;
//! let mut runtime = builder
//!     .set_pipeline(pipeline)
//!     .set_context(context)
//!     .build()
//!     .await
//!     .unwrap();
//!
//! let result = runtime.run().await.unwrap();
//! let _ = result;
//! runtime.shutdown().await.unwrap();
//! # });
//! ```
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

/// Errors returned by the runtime API.
#[derive(Debug)]
pub enum PhlowRuntimeError {
    /// Pipeline was not provided.
    MissingPipeline,
    /// Failed to load the pipeline into a loader.
    LoaderError(crate::loader::error::Error),
    /// Failed to send a package to the runtime loop.
    PackageSendError,
    /// Response channel closed before a result arrived.
    ResponseChannelClosed,
    /// Error reported by runtime execution.
    RuntimeError(RuntimeError),
    /// Join error from the runtime task.
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

/// Prepared runtime that can execute an in-memory pipeline.
pub struct PhlowRuntime {
    pipeline: Option<Value>,
    context: Option<Context>,
    settings: Settings,
    base_path: Option<PathBuf>,
    dispatch: Option<tracing::Dispatch>,
    prepared: Option<PreparedRuntime>,
}

/// Builder for creating a prepared [`PhlowRuntime`].
///
/// Use this when you want a fluent API that returns a ready runtime.
pub struct PhlowBuilder {
    pipeline: Option<Value>,
    context: Option<Context>,
    settings: Settings,
    base_path: Option<PathBuf>,
    dispatch: Option<tracing::Dispatch>,
}

impl Default for PhlowRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PhlowRuntime {
    /// Create a new runtime with default settings.
    ///
    /// This sets `var_main` to a default value so non-main pipelines auto-start.
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

    /// Create a new runtime using explicit settings.
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

    /// Set the pipeline to be executed.
    ///
    /// This clears any prepared runtime state.
    pub fn set_pipeline(&mut self, pipeline: Value) -> &mut Self {
        self.pipeline = Some(pipeline);
        self.prepared = None;
        self
    }

    /// Set the execution context.
    ///
    /// This clears any prepared runtime state.
    pub fn set_context(&mut self, context: Context) -> &mut Self {
        self.context = Some(context);
        self.prepared = None;
        self
    }

    /// Replace the runtime settings.
    ///
    /// This clears any prepared runtime state.
    pub fn set_settings(&mut self, settings: Settings) -> &mut Self {
        self.settings = settings;
        self.prepared = None;
        self
    }

    /// Set the base path used for resolving local module paths.
    ///
    /// This clears any prepared runtime state.
    pub fn set_base_path<P: Into<PathBuf>>(&mut self, base_path: P) -> &mut Self {
        self.base_path = Some(base_path.into());
        self.prepared = None;
        self
    }

    /// Provide a custom tracing dispatch instead of initializing OpenTelemetry.
    ///
    /// This clears any prepared runtime state.
    pub fn set_dispatch(&mut self, dispatch: tracing::Dispatch) -> &mut Self {
        self.dispatch = Some(dispatch);
        self.prepared = None;
        self
    }

    /// Read-only access to the current settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Mutable access to settings.
    ///
    /// This clears any prepared runtime state.
    pub fn settings_mut(&mut self) -> &mut Settings {
        self.prepared = None;
        &mut self.settings
    }

    /// Build and prepare the runtime (load modules, tracing, and start loop).
    ///
    /// Calling this multiple times is safe; it is a no-op if already prepared.
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

        let context = self.context.clone().unwrap_or_else(Context::new);
        let request_data = context.get_main();
        let context_for_runtime = context.clone();
        let auto_start = self.settings.var_main.is_some()
            || loader.main == -1
            || context.get_main().is_some();

        let app_name = loader
            .app_data
            .name
            .clone()
            .unwrap_or_else(|| "phlow runtime".to_string());

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
            auto_start,
        });

        Ok(())
    }

    /// Execute the pipeline and return its result.
    ///
    /// This can be called multiple times after [`build`](Self::build). When the
    /// pipeline cannot auto-start (for example, a main module is present and
    /// `var_main` is not set), this returns `Value::Undefined` and shuts down
    /// the prepared runtime. For normal execution, call [`shutdown`](Self::shutdown)
    /// when you are done to release resources.
    pub async fn run(&mut self) -> Result<Value, PhlowRuntimeError> {
        self.build().await?;

        let auto_start = match self.prepared.as_ref() {
            Some(prepared) => prepared.auto_start,
            None => return Err(PhlowRuntimeError::MissingPipeline),
        };

        if !auto_start {
            self.shutdown().await?;
            return Ok(Value::Undefined);
        }

        let (tx_main_package, dispatch, app_name, request_data) = match self.prepared.as_ref() {
            Some(prepared) => (
                prepared.tx_main_package.clone(),
                prepared.dispatch.clone(),
                prepared.app_name.clone(),
                prepared.request_data.clone(),
            ),
            None => return Err(PhlowRuntimeError::MissingPipeline),
        };

        let (response_tx, response_rx) = tokio::sync::oneshot::channel::<Value>();
        let package = tracing::dispatcher::with_default(&dispatch, || {
            let span = tracing::span!(
                tracing::Level::INFO,
                "phlow_run",
                otel.name = app_name.as_str()
            );

            Package {
                response: Some(response_tx),
                request_data,
                origin: 0,
                span: Some(span),
                dispatch: Some(dispatch.clone()),
            }
        });

        if tx_main_package.send(package).is_err() {
            return Err(PhlowRuntimeError::PackageSendError);
        }

        let result = response_rx
            .await
            .map_err(|_| PhlowRuntimeError::ResponseChannelClosed)?;

        Ok(result)
    }

    /// Shut down the prepared runtime and release resources.
    ///
    /// Call this when you are done reusing the runtime to close channels,
    /// wait for the runtime task, and flush tracing providers.
    pub async fn shutdown(&mut self) -> Result<(), PhlowRuntimeError> {
        let prepared = match self.prepared.take() {
            Some(prepared) => prepared,
            None => return Ok(()),
        };

        drop(prepared.tx_main_package);

        let runtime_result = prepared
            .runtime_handle
            .await
            .map_err(PhlowRuntimeError::RuntimeJoinError)?;
        runtime_result?;

        drop(prepared.guard);

        Ok(())
    }
}

impl PhlowBuilder {
    /// Create a new builder with default settings.
    ///
    /// This sets `var_main` to a default value so non-main pipelines auto-start.
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
        }
    }

    /// Create a new builder using explicit settings.
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            pipeline: None,
            context: None,
            settings,
            base_path: None,
            dispatch: None,
        }
    }

    /// Set the pipeline to be executed.
    ///
    /// Returns the builder for chaining.
    pub fn set_pipeline(mut self, pipeline: Value) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Set the execution context.
    ///
    /// Returns the builder for chaining.
    pub fn set_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    /// Replace the runtime settings.
    ///
    /// Returns the builder for chaining.
    pub fn set_settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    /// Set the base path used for resolving local module paths.
    ///
    /// Returns the builder for chaining.
    pub fn set_base_path<P: Into<PathBuf>>(mut self, base_path: P) -> Self {
        self.base_path = Some(base_path.into());
        self
    }

    /// Provide a custom tracing dispatch instead of initializing OpenTelemetry.
    ///
    /// Returns the builder for chaining.
    pub fn set_dispatch(mut self, dispatch: tracing::Dispatch) -> Self {
        self.dispatch = Some(dispatch);
        self
    }

    /// Read-only access to the current settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Mutable access to settings.
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Build and return a prepared [`PhlowRuntime`].
    ///
    /// This consumes the builder and prepares the runtime for execution.
    pub async fn build(mut self) -> Result<PhlowRuntime, PhlowRuntimeError> {
        let mut runtime = PhlowRuntime::with_settings(self.settings);

        if let Some(pipeline) = self.pipeline.take() {
            runtime.set_pipeline(pipeline);
        }

        if let Some(context) = self.context.take() {
            runtime.set_context(context);
        }

        if let Some(base_path) = self.base_path.take() {
            runtime.set_base_path(base_path);
        }

        if let Some(dispatch) = self.dispatch.take() {
            runtime.set_dispatch(dispatch);
        }

        runtime.build().await?;
        Ok(runtime)
    }
}

impl Default for PhlowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct PreparedRuntime {
    tx_main_package: channel::Sender<Package>,
    dispatch: tracing::Dispatch,
    runtime_handle: tokio::task::JoinHandle<Result<(), RuntimeError>>,
    guard: Option<OtelGuard>,
    app_name: String,
    request_data: Option<Value>,
    auto_start: bool,
}
