use valu3::value::Value;

pub type CallbackFn = extern "C" fn(*const Value) -> *const Value;

pub struct PluginData {
    pub setup: *const Value,
    pub callback: CallbackFn,
}

pub mod prelude {
    pub use crate::CallbackFn;
    pub use valu3::prelude::*;
}
