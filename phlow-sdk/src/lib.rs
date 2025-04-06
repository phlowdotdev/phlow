pub mod context;
pub mod count;
pub mod id;
pub mod macros;
pub mod modules;
pub mod otel;
use context::Context;
pub use crossbeam;
use crossbeam::channel;
use modules::ModulePackage;
pub use opentelemetry;
pub use opentelemetry_sdk;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
pub use tokio;
use tokio::sync::oneshot;
pub use tracing;
pub use tracing_core;
pub use tracing_opentelemetry;
pub use tracing_subscriber;
pub use valu3;
use valu3::{traits::ToValueBehavior, value::Value};

pub type ModuleId = usize;
pub type MainRuntimeSender = channel::Sender<Package>;
pub type ModuleSetupSender = oneshot::Sender<Option<channel::Sender<ModulePackage>>>;

#[derive(Debug)]
pub struct ModuleSetup {
    pub id: ModuleId,
    pub setup_sender: ModuleSetupSender,
    pub main_sender: Option<MainRuntimeSender>,
    pub with: Value,
    pub dispatch: tracing::Dispatch,
}

impl ModuleSetup {
    pub fn is_main(&self) -> bool {
        self.main_sender.is_some()
    }
}

#[derive(Default)]
pub struct Package {
    pub send: Option<oneshot::Sender<Value>>,
    pub request_data: Option<Value>,
    pub origin: ModuleId,
    pub span: Option<tracing::Span>,
    pub dispatch: Option<tracing::Dispatch>,
}

// Only production mode
impl Debug for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let map: HashMap<_, _> = vec![
            ("request_data", self.request_data.to_value()),
            ("step_position", self.origin.to_value()),
        ]
        .into_iter()
        .collect();

        write!(
            f,
            "{}",
            map.to_value().to_json(valu3::prelude::JsonMode::Inline)
        )
    }
}

impl Package {
    pub fn get_data(&self) -> Option<&Value> {
        self.request_data.as_ref()
    }

    pub fn send(&mut self, response_data: Value) {
        if let Some(send) = self.send.take() {
            sender_safe!(send, response_data);
        }
    }
}

pub mod prelude {
    pub use crate::*;
    pub use valu3::json;
    pub use valu3::prelude::*;
}
