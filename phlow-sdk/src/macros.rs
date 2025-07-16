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
macro_rules! sender_package {
    ($id:expr, $sender:expr, $data:expr) => {{
        let (tx, rx) = $crate::tokio::sync::oneshot::channel::<$crate::valu3::value::Value>();

        let package = $crate::structs::Package {
            response: Some(tx),
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
            response: Some(tx),
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
macro_rules! module_channel {
    ($setup:expr) => {{
        let (tx, rx) = $crate::crossbeam::channel::unbounded::<ModulePackage>();

        sender_safe!($setup.setup_sender, Some(tx));

        rx
    }};
}

#[macro_export]
macro_rules! use_log {
    () => {
        env_logger::Builder::from_env(
            env_logger::Env::new()
                .default_filter_or("info")
                .filter_or("PHLOW_LOG", "info"),
        )
        .try_init()
    };
}

#[macro_export]
macro_rules! create_step {
    ($handler:ident(setup)) => {
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

    ($handler:ident(rx)) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: $crate::structs::ModuleSetup) {
            if let Ok(rt) = $crate::tokio::runtime::Runtime::new() {
                let rx = module_channel!(setup);

                // During tests, don't run the handler as it would block forever
                if !setup.is_test_mode {
                    if let Err(e) = rt.block_on($handler(rx)) {
                        $crate::tracing::error!("Error in plugin: {:?}", e);
                    }
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
    ($handler:ident(setup)) => {
        #[no_mangle]
        pub extern "C" fn plugin(setup: $crate::structs::ModuleSetup) {
            let dispatch = setup.dispatch.clone();
            $crate::tracing::dispatcher::with_default(&dispatch, || {
                let _guard = $crate::otel::init_tracing_subscriber(setup.app_data.clone());
                let _ = env_logger::Builder::from_env(
                    env_logger::Env::new()
                        .default_filter_or("info")
                        .filter_or("PHLOW_LOG", "info"),
                )
                .try_init();

                if let Ok(rt) = $crate::tokio::runtime::Runtime::new() {
                    rt.block_on($handler(setup)).unwrap_or_else(|e| {
                        $crate::tracing::error!("Error in plugin: {:?}", e);
                    });
                } else {
                    $crate::tracing::error!("Error creating runtime");
                    return;
                }
            });
        }
    };
}
