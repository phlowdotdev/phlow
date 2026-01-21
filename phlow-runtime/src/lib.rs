pub mod analyzer;
pub mod debug_server;
pub mod loader;
pub mod memory;
pub mod package;
pub mod preprocessor;
pub mod runtime;
pub mod scripts;
pub mod settings;
pub mod test_runner;

mod runtime_api;

pub use loader::Loader;
pub use package::Package;
pub use runtime::{Runtime, RuntimeError};
pub use runtime_api::{PhlowBuilder, PhlowRuntime, PhlowRuntimeError};
pub use settings::{PrintOutput, Settings};

#[cfg(target_os = "macos")]
pub const MODULE_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
pub const MODULE_EXTENSION: &str = "so";
#[cfg(target_os = "windows")]
pub const MODULE_EXTENSION: &str = "dll";

#[cfg(target_os = "macos")]
pub const RUNTIME_ARCH: &str = "darwin";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const RUNTIME_ARCH: &str = "linux-aarch64";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const RUNTIME_ARCH: &str = "linux-amd64";
