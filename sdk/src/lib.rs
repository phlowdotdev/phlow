use tokio::sync::oneshot::{channel, Receiver, Sender};
use valu3::value::Value;

pub struct Broker {
    send: Option<Sender<Value>>,
    data: Option<Value>,
    receiver: Option<Receiver<Value>>,
}

impl Broker {
    pub fn new(data: Option<Value>) -> Self {
        let (tx, rx) = channel();

        Self {
            data,
            send: Some(tx),
            receiver: Some(rx),
        }
    }

    pub fn get_package(self) -> Package {
        Package {
            send: self.send,
            data: self.data,
        }
    }

    pub fn blocking_receiver(&mut self) -> Option<Value> {
        if let Some(receiver) = self.receiver.take() {
            return Some(receiver.blocking_recv().unwrap_or(Value::Null));
        }

        None
    }
}

#[macro_export]
macro_rules! plugin {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: *const Value) {
            let value = unsafe { &*setup };
            $handler(value)
        }
    };
}
#[macro_export]
macro_rules! plugin_async {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: *const Value) {
            let value = unsafe { &*setup };
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on($handler(value));
        }
    };
}

#[derive(Default)]
pub struct Package {
    pub send: Option<Sender<Value>>,
    pub data: Option<Value>,
}

impl Package {
    pub fn get_data(&self) -> Option<&Value> {
        self.data.as_ref()
    }

    pub fn send(&mut self) {
        if let Some(send) = self.send.take() {
            if let Some(data) = self.data.take() {
                let _ = send.send(data);
            }
        }
    }
}

pub mod prelude {
    pub use crate::*;
    pub use valu3::prelude::*;
    // export macro
    pub use crate::plugin;
}
