use futures::future::BoxFuture;
use futures::FutureExt;
use phlow_sdk::prelude::Value;
use phlow_sdk::structs::{ApplicationData, ModuleResponse};
use phlow_sdk::tracing::Dispatch;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

/// Inline module schema describing input/output shapes and input ordering.
#[derive(Debug, Clone, Default)]
pub struct PhlowModuleSchema {
    /// Module input schema.
    pub input: Value,
    /// Module output schema.
    pub output: Value,
    /// Preferred input ordering.
    pub input_order: Vec<String>,
}

impl PhlowModuleSchema {
    /// Create an empty schema with null input/output and no ordering.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the input schema.
    pub fn with_input(mut self, input: Value) -> Self {
        self.input = input;
        self
    }

    /// Set the output schema.
    pub fn with_output(mut self, output: Value) -> Self {
        self.output = output;
        self
    }

    /// Set the input ordering used by UIs or helpers.
    pub fn with_input_order<I, S>(mut self, input_order: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.input_order = input_order.into_iter().map(Into::into).collect();
        self
    }
}

/// Data passed to inline module handlers.
#[derive(Clone)]
pub struct PhlowModuleRequest {
    /// Step input for the module invocation.
    pub input: Option<Value>,
    /// Previous payload when the step did not specify input.
    pub payload: Option<Value>,
    /// Evaluated module `with` configuration.
    pub with: Value,
    /// Application metadata from the pipeline.
    pub app_data: ApplicationData,
    /// Tracing dispatch for the runtime.
    pub dispatch: Dispatch,
}

/// Async handler signature for inline modules.
pub type PhlowModuleHandler =
    Arc<dyn Fn(PhlowModuleRequest) -> BoxFuture<'static, ModuleResponse> + Send + Sync>;

/// Inline module definition used by the runtime API.
#[derive(Clone, Default)]
pub struct PhlowModule {
    schema: PhlowModuleSchema,
    handler: Option<PhlowModuleHandler>,
}

impl PhlowModule {
    /// Create a new inline module without schema or handler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the schema for this module.
    pub fn set_schema(&mut self, schema: PhlowModuleSchema) -> &mut Self {
        self.schema = schema;
        self
    }

    /// Set the async handler for this module.
    pub fn set_handler<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(PhlowModuleRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ModuleResponse> + Send + 'static,
    {
        self.handler = Some(Arc::new(move |request| handler(request).boxed()));
        self
    }

    /// Access the schema.
    pub fn schema(&self) -> &PhlowModuleSchema {
        &self.schema
    }

    pub(crate) fn handler(&self) -> Option<PhlowModuleHandler> {
        self.handler.clone()
    }
}

/// Inline module registry keyed by module name.
pub type InlineModules = HashMap<String, PhlowModule>;
