use phlow_runtime::{analyzer, debug_server, test_runner, Loader, Package, Runtime, Settings};
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::{tracing, use_log};
use std::sync::Arc;

#[cfg(all(feature = "mimalloc", target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(feature = "jemalloc", target_env = "musl"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    use_log!();
    log::debug!("Starting Phlow Runtime");

    let settings = match Settings::try_load() {
        Ok(settings) => settings,
        Err(err) => {
            log::error!("Error loading settings: {:?}", err);
            std::process::exit(1);
        }
    };

    if let Some(publish_path) = settings.package_path.clone() {
        match Package::new(
            publish_path.into(),
            settings.package_target.clone(),
            settings.create_tar,
        ) {
            Ok(publish) => {
                if let Err(err) = publish.run() {
                    log::error!("Error creating module: {:?}", err);
                }

                return;
            }
            Err(err) => {
                log::error!("Error creating instance: {:?}", err);
                return;
            }
        }
    }

    // Build Analyzer and pass it down to loader; loader/load_script will execute the analyzer
    let analyzer = analyzer::Analyzer::from_settings(&settings);

    // Load the script into Loader (parsing / preprocessing) for normal runtime path
    let mut loader = match Loader::load(
        &settings.script_main_absolute_path,
        settings.print_yaml,
        settings.print_output,
        Some(&analyzer),
    )
    .await
    {
        Ok(main) => main,
        Err(err) => {
            log::error!("Runtime Error Main File: {:?}", err);
            return;
        }
    };

    if settings.no_run || settings.analyzer {
        return;
    }

    let guard = init_tracing_subscriber(loader.app_data.clone());

    if let Err(err) = tracing::dispatcher::set_global_default(guard.dispatch.clone()) {
        log::error!("Failed to set global subscriber: {:?}", err);
        std::process::exit(1);
    }

    let dispatch = guard.dispatch.clone();

    let debug_enabled = std::env::var("PHLOW_DEBUG")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if debug_enabled {
        let controller = Arc::new(phlow_engine::debug::DebugController::new());
        match debug_server::spawn(controller.clone()).await {
            Ok(()) => {
                if phlow_engine::debug::set_debug_controller(controller).is_err() {
                    log::warn!("Debug controller already set");
                }
                log::info!("Phlow debug enabled");
            }
            Err(err) => {
                log::error!("Failed to start debug server: {}", err);
            }
        }
    }

    let fut = async {
        if settings.download {
            if let Err(err) = loader
                .download(&settings.default_package_repository_url)
                .await
            {
                log::error!("Download failed: {:?}", err);
                return;
            }
        }

        loader.update_info();

        if !settings.only_download_modules {
            if settings.test {
                log::debug!("Run test");
                // Run tests
                match test_runner::run_tests(
                    loader,
                    settings.test_filter.as_deref(),
                    settings.clone(),
                )
                .await
                {
                    Ok(summary) => {
                        // Exit with error code if tests failed
                        if summary.failed > 0 {
                            std::process::exit(1);
                        }
                    }
                    Err(err) => {
                        log::error!("Test execution error: {}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                log::debug!("Run application");
                // Run normal workflow
                if let Err(rr) = Runtime::run(loader, dispatch.clone(), settings).await {
                    log::error!("Runtime Error: {:?}", rr);
                }
            }
        }
    };

    // Aguarda execução normal ou sinal de encerramento (Ctrl+C/SIGTERM)
    #[cfg(not(unix))]
    let shutdown = async {
        // Ctrl+C (todas as plataformas)
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let shutdown = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigint = signal(SignalKind::interrupt()).ok();
        let mut sigterm = signal(SignalKind::terminate()).ok();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = async { if let Some(ref mut s)=sigint { s.recv().await; } } => {},
            _ = async { if let Some(ref mut s)=sigterm { s.recv().await; } } => {},
        }
    };

    tokio::select! {
        _ = tracing::dispatcher::with_default(&dispatch, || fut) => {
            // Execução terminou normalmente
        },
        _ = shutdown => {
            log::info!("Received shutdown signal. Bye bye!");
            // Liberar / descarregar instrumentação antes de sair
            drop(guard);
            // Sair com código 130 (Ctrl+C) para indicar interrupção pelo usuário
            std::process::exit(130);
        }
    }
}
