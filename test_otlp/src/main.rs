use libloading::Library;
use sdk::otlp::init_tracing_subscriber;
use sdk::tracing::{dispatcher, span, Dispatch, Level};
use sdk::tracing_opentelemetry::OpenTelemetrySpanExt;
use sdk::valu3::prelude::*;
use sdk::Package;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

type PluginFn = unsafe extern "C" fn(*const Dispatch, *const Sender<Package>);

fn main() {
    let _guard = init_tracing_subscriber().expect("failed to initialize tracing");

    let dispatch = dispatcher::get_default(|d| Arc::new(d.clone()));
    let plugins = vec!["./target/debug/libtracer.so"];

    let (tx, rx) = std::sync::mpsc::channel::<Package>();

    for plugin_path in plugins.iter() {
        let dispatch_clone = dispatch.clone();
        let plugin_path = *plugin_path;
        let tx_clone = tx.clone();

        thread::spawn(move || unsafe {
            let lib = Library::new(plugin_path).expect("Falha ao carregar a biblioteca");
            let func: libloading::Symbol<PluginFn> =
                lib.get(b"plugin").expect("Falha ao obter símbolo");
            func(
                Arc::into_raw(dispatch_clone),
                &tx_clone as *const _ as *const Sender<Package>,
            );
        });
    }

    let handler = thread::spawn(move || {
        let package = rx.recv().unwrap();

        println!("Received package: {:?}", package);

        let dispatch = package.dispatch.unwrap();

        dispatcher::with_default(&dispatch, || {
            let parent = package.span.unwrap();

            let span = span!(Level::INFO, "receivers");
            span.set_parent(parent.context());
            let _enter = span.enter();

            if let Some(sender) = package.send {
                let result = sender.send("Hello from main".to_value());
                match result {
                    Ok(_) => println!("Sent value from main"),
                    Err(_) => println!("Failed to send value from main"),
                }
            }
        });
    });

    handler.join().unwrap();
}
