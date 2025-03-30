use sdk::opentelemetry::Context;
use sdk::tracing_opentelemetry::OpenTelemetrySpanExt;
use sdk::{
    otlp::init_tracing_subscriber_plugin,
    tokio::sync::oneshot,
    tracing::{dispatcher, info, span, Dispatch, Level, Span},
    Package,
};
use std::sync::mpsc::Sender;

#[no_mangle]
pub extern "C" fn plugin(
    context_ptr: *const Context,
    dispatch_ptr: *const Dispatch,
    sender: *const Sender<Package>,
) {
    let sender: &Sender<Package> = unsafe { &*sender };

    unsafe {
        let _guard = init_tracing_subscriber_plugin().expect("failed to initialize tracing");

        let dispatch = &*dispatch_ptr;
        let parent_context = &*context_ptr;

        dispatcher::with_default(dispatch, || {
            let span = span!(Level::INFO, "plugin");
            span.set_parent(parent_context.clone());
            let _enter = span.enter();

            execution(sender, span.clone(), dispatch);
            info!("Log do plugin dentro do plugin");
        });

        drop(Box::from_raw(context_ptr as *mut Context));
    }
}

pub fn execution(sender: &Sender<Package>, span: Span, dispatch: &Dispatch) {
    let (tx, rx) = oneshot::channel();
    info!("Log antes de enviar");

    let package = Package {
        origin: 0,
        request_data: None,
        send: Some(tx),
        span: Some(span),
        dispatch: Some(dispatch.clone()),
    };

    sender.send(package).unwrap();

    let result = rx.blocking_recv();
    match result {
        Ok(value) => info!(
            "Log do plugin dentro do execution - resultado recebido: {}",
            value
        ),
        Err(_) => info!("Log do plugin dentro do execution - erro ao receber resultado"),
    }

    info!("Log do plugin dentro do execution - resultado recebido");

    println!("Finalizando plugin");
}
