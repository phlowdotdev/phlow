#[macro_export]
macro_rules! listen {
    ($rx:expr, $resolve:expr) => {{
        for package in $rx {
            $crate::tokio::spawn(async move {
                $resolve(package).await;
            });
        }
    }};
    ($rx:expr, $resolve:expr, $( $arg:ident ),+ $(,)? ) => {{
        for package in $rx {
            $( let $arg = $arg.clone(); )+

            $crate::tokio::spawn(async move {
                $resolve(package, $( $arg ),+ ).await;
            });
        }
    }};
}

#[macro_export]
macro_rules! span_enter {
    ($span:expr) => {
        let span_enter_clone = $span.clone();
        let _enter = span_enter_clone.enter();
    };
}

#[macro_export]
macro_rules! sender_safe {
    ($sender:expr, $data:expr) => {
        if let Err(err) = $sender.send($data) {
            $crate::tracing::debug!("Error sending data: {:?}", err);
        }
    };
}

#[macro_export]
macro_rules! sender {
    ($id:expr, $sender:expr, $data:expr) => {{
        let (tx, rx) = $crate::tokio::sync::oneshot::channel::<$crate::valu3::value::Value>();

        let package = $crate::structs::Package {
            send: Some(tx),
            request_data: $data,
            origin: $id,
            span: None,
            dispatch: None,
        };

        sender_safe!($sender, package);

        rx
    }};
    ($span:expr, $dispatch:expr, $id:expr, $sender:expr, $data:expr) => {{
        let (tx, rx) = $crate::tokio::sync::oneshot::channel::<$crate::valu3::value::Value>();

        let package = $crate::structs::Package {
            send: Some(tx),
            request_data: $data,
            origin: $id,
            span: Some($span),
            dispatch: Some($dispatch),
        };

        sender_safe!($sender, package);

        rx
    }};
}

#[macro_export]
macro_rules! create_step {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: $crate::structs::ModuleSetup) {
            if let Ok(rt) = $crate::tokio::runtime::Runtime::new() {
                if let Err(e) = rt.block_on($handler(setup)) {
                    $crate::tracing::error!("Error in plugin: {:?}", e);
                }
            } else {
                $crate::tracing::error!("Error creating runtime");
                return;
            };
        }
    };
}

#[macro_export]
macro_rules! create_main {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: $crate::structs::ModuleSetup) {
            let dispatch = setup.dispatch.clone();
            $crate::tracing::dispatcher::with_default(&dispatch, || {
                let _guard = $crate::otel::init_tracing_subscriber();

                if let Ok(rt) = $crate::tokio::runtime::Runtime::new() {
                    rt.block_on(start_server(setup)).unwrap_or_else(|e| {
                        $crate::tracing::error!("Error in plugin: {:?}", e);
                    });
                    println!("Plugin loaded");
                } else {
                    $crate::tracing::error!("Error creating runtime");
                    println!("Plugin loaded");

                    return;
                };

                println!("Plugin loaded");
            });
        }
    };
}
