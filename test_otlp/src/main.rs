use libloading::Library;
use sdk::otlp::init_tracing_subscriber;
use sdk::tracing::{dispatcher, info, span, Dispatch, Level, Span};
use sdk::valu3::prelude::*;
use sdk::Package;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

type PluginFn = unsafe extern "C" fn(*mut Span, *const Dispatch, *const Sender<Package>);

fn main() {
    let _guard = init_tracing_subscriber().expect("failed to initialize tracing");

    let span = span!(Level::INFO, "main");
    let _enter = span.enter();

    let dispatch = dispatcher::get_default(|d| Arc::new(d.clone()));
    let plugins = vec!["./target/debug/libtracer.so"];

    let (tx, rx) = std::sync::mpsc::channel::<Package>();

    for plugin_path in plugins.iter() {
        let span_clone = span.clone();
        let dispatch_clone = dispatch.clone();
        let plugin_path = *plugin_path;
        let tx_clone = tx.clone();

        thread::spawn(move || unsafe {
            let lib = Library::new(plugin_path).expect("Falha ao carregar a biblioteca");
            let func: libloading::Symbol<PluginFn> =
                lib.get(b"plugin").expect("Falha ao obter símbolo");
            func(
                &span_clone as *const _ as *mut _,
                Arc::into_raw(dispatch_clone),
                &tx_clone as *const _ as *const Sender<Package>,
            );
        });
    }

    for package in rx.iter() {
        println!("Received package: {:?}", package);

        let dispatch = package.dispatch.unwrap();

        dispatcher::with_default(&dispatch, || {
            if let Some(span) = package.span {
                let parent = span.clone();
                let _parent_enter = parent.enter();
                info!("Log do main");
            }

            let span = span!(Level::INFO, "receivers");
            let _enter = span.enter();

            if let Some(sender) = package.send {
                let result = sender.send("Hello from main".to_value());
                match result {
                    Ok(_) => println!("Sent value from main"),
                    Err(_) => println!("Failed to send value from main"),
                }
            }
        });
    }

    std::thread::sleep(std::time::Duration::from_secs(3));
}
