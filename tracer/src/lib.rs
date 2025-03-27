use sdk::{
    otlp::init_tracing_subscriber_plugin,
    tracing::{self, dispatcher, info, span, Level, Span},
};

#[no_mangle]
pub extern "C" fn plugin(span_ptr: *mut Span, dispatch_ptr: *const dispatcher::Dispatch) {
    unsafe {
        let _guard = init_tracing_subscriber_plugin().expect("failed to initialize tracing");

        let dispatch = &*dispatch_ptr;

        dispatcher::with_default(dispatch, || {
            let parent = span_ptr.as_ref().unwrap().clone();
            let _parent_enter = parent.enter();

            let span = span!(Level::INFO, "plugin", from = "dylib");
            let _enter = span.enter();
            info!("Log do plugin dentro do plugin");
            execution();
        });
    }
}

#[tracing::instrument]
pub fn execution() {
    info!("Log do plugin dentro do execution");
}
