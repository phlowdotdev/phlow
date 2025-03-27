use libloading::Library;
use sdk::otlp::init_tracing_subscriber;
use sdk::tracing::{dispatcher, span, Dispatch, Level, Span};
use std::sync::Arc;
use std::thread;

type PluginFn = unsafe extern "C" fn(*mut Span, *const Dispatch);

fn main() {
    let _guard = init_tracing_subscriber().expect("failed to initialize tracing");

    let span = span!(Level::INFO, "main", component = "main_binary");
    let _enter = span.enter();

    let dispatch = dispatcher::get_default(|d| Arc::new(d.clone()));
    let plugins = vec!["./target/debug/libtracer.so"];

    for plugin_path in plugins.iter() {
        let span_clone = span.clone();
        let dispatch_clone = dispatch.clone();
        let plugin_path = *plugin_path;
        thread::spawn(move || unsafe {
            let lib = Library::new(plugin_path).expect("Falha ao carregar a biblioteca");
            let func: libloading::Symbol<PluginFn> =
                lib.get(b"plugin").expect("Falha ao obter símbolo");
            func(
                &span_clone as *const _ as *mut _,
                Arc::into_raw(dispatch_clone),
            );
        })
        .join()
        .expect("Falha na thread");
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
}
