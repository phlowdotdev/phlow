pub mod modules;
use crate::sender_safe;
use crossbeam::channel::{self, Receiver};
pub use modules::*;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use tokio::sync::oneshot;
use valu3::{traits::ToValueBehavior, value::Value};
pub type ModuleId = usize;
pub type MainRuntimeSender = channel::Sender<Package>;
pub type ModuleSetupSender = oneshot::Sender<Option<channel::Sender<ModulePackage>>>;

pub type ModuleReceiver = Receiver<ModulePackage>;

#[derive(Debug, Clone)]
pub struct ApplicationData {
    pub name: Option<String>,
    pub version: Option<String>,
    pub environment: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug)]
pub struct ModuleSetup {
    pub id: ModuleId,
    pub setup_sender: ModuleSetupSender,
    pub main_sender: Option<MainRuntimeSender>,
    pub with: Value,
    pub dispatch: tracing::Dispatch,
    pub app_data: ApplicationData,
    pub is_test_mode: bool,
}

impl ModuleSetup {
    pub fn is_main(&self) -> bool {
        self.main_sender.is_some() || self.is_test_mode
    }
}

#[derive(Default)]
pub struct Package {
    pub response: Option<oneshot::Sender<Value>>,
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
        if let Some(send) = self.response.take() {
            sender_safe!(send, response_data);
        }
    }
}
