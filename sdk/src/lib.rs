use std::sync::mpsc::Sender;
use tokio::sync::oneshot;
use valu3::value::Value;

pub type Broker = Sender<Package>;

#[macro_export]
macro_rules! plugin {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(sender: Broker, value: Value) {
            $handler(sender, value)
        }
    };
}
#[macro_export]
macro_rules! plugin_async {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(sender: Broker, value: Value) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on($handler(sender, value));
        }
    };
}

#[derive(Default, Debug)]
pub struct Package {
    pub send: Option<oneshot::Sender<Value>>,
    pub request_data: Option<Value>,
    pub origin: i32,
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
}
