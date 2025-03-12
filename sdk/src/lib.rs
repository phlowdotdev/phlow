use valu3::value::Value;

pub type CallbackFn = extern "C" fn(*const Value) -> *const Value;

pub mod prelude {
    pub use crate::CallbackFn;
    pub use valu3::prelude::*;
}
