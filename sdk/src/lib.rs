use std::fmt::{Debug, Formatter};
use std::{collections::HashMap, sync::mpsc::Sender};
use tokio::sync::oneshot;
pub use valu3;
use valu3::{traits::ToValueBehavior, value::Value};

pub type ModuleId = usize;
pub type RuntimeSender = Sender<Package>;

#[macro_export]
macro_rules! plugin {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(id: ModuleId, sender: RuntimeSender, value: Value) {
            $handler(id, sender, value)
        }
    };
}
#[macro_export]
macro_rules! plugin_async {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(id: ModuleId, sender: RuntimeSender, value: Value) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on($handler(id, sender, value)).unwrap();
        }
    };
}

#[macro_export]
macro_rules! sender {
    ($id:expr, $sender:expr, $data:expr) => {{
        let (tx, rx) = tokio::sync::oneshot::channel::<valu3::value::Value>();

        let package = Package {
            send: Some(tx),
            request_data: $data,
            origin: $id,
        };

        $sender.send(package).unwrap();
        rx
    }};
}

#[derive(Default)]
pub struct Package {
    pub send: Option<oneshot::Sender<Value>>,
    pub request_data: Option<Value>,
    pub origin: ModuleId,
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
            let _ = send.send(response_data).unwrap();
        }
    }
}

pub mod prelude {
    pub use crate::*;
    pub use valu3::prelude::*;
    // export macro
    pub use crate::plugin;
    pub use valu3::json;
}
