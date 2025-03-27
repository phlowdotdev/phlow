use std::sync::mpsc::Sender;

use sdk::{
    otlp::init_tracing_subscriber_plugin,
    tokio::sync::oneshot,
    tracing::{self, dispatcher, info, span, Dispatch, Level, Span},
    Package,
};

#[no_mangle]
pub extern "C" fn plugin(
    span_ptr: *mut Span,
    dispatch_ptr: *const Dispatch,
    sender: *const Sender<Package>,
) {
    let sender: &Sender<Package> = unsafe { &*sender };

    unsafe {
        let _guard = init_tracing_subscriber_plugin().expect("failed to initialize tracing");

        let dispatch = &*dispatch_ptr;

        dispatcher::with_default(dispatch, || {
            let parent = span_ptr.as_ref().unwrap().clone();
            let _parent_enter = parent.enter();

            let span = span!(Level::INFO, "plugin");
            let _enter = span.enter();
            execution(sender, span.clone(), dispatch);
            info!("Log do plugin dentro do plugin");
        });
    }
}

pub fn execution(sender: &Sender<Package>, span: Span, dispatch: &Dispatch) {
    let (tx, rx) = oneshot::channel();

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
