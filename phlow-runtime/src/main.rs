use phlow_engine::Context;
use phlow_runtime::{
    PhlowBuilder,
    Settings,
    analyzer,
    loader::load_script_value,
    test_runner,
    Loader,
    Package,
};
use phlow_sdk::prelude::Value;
use phlow_sdk::use_log;
use std::path::{Path, PathBuf};

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

    let (script, script_file_path) = match load_script_value(
        &settings.script_main_absolute_path,
        settings.print_yaml,
        settings.print_output,
        Some(&analyzer),
    )
    .await
    {
        Ok(script) => script,
        Err(err) => {
            log::error!("Runtime Error Main File: {:?}", err);
            return;
        }
    };

    if settings.no_run || settings.analyzer {
        return;
    }

    let base_path = Path::new(&script_file_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("./"));

    if settings.only_download_modules || settings.test {
        let mut loader = match Loader::from_value(&script, Some(base_path.as_path())) {
            Ok(loader) => loader,
            Err(err) => {
                log::error!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

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

        if settings.only_download_modules {
            return;
        }

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
            return;
        }
    }

    let context = if let Some(var_main) = &settings.var_main {
        Context::from_main(parse_cli_value("var-main", var_main))
    } else {
        Context::new()
    };

    let mut runtime = match PhlowBuilder::with_settings(settings.clone())
        .set_base_path(base_path)
        .set_pipeline(script)
        .set_context(context)
        .build()
        .await
    {
        Ok(runtime) => runtime,
        Err(err) => {
            log::error!("Runtime Error: {:?}", err);
            return;
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

    let mut run_handle = tokio::spawn(async move { runtime.run().await });

    tokio::select! {
        result = &mut run_handle => {
            match result {
                Ok(Ok(_)) => {}
                Ok(Err(err)) => {
                    log::error!("Runtime Error: {:?}", err);
                }
                Err(err) => {
                    log::error!("Runtime task error: {:?}", err);
                }
            }
        },
        _ = shutdown => {
            log::info!("Received shutdown signal. Bye bye!");
            run_handle.abort();
            let _ = run_handle.await;
            // Sair com código 130 (Ctrl+C) para indicar interrupção pelo usuário
            std::process::exit(130);
        }
    }
}

fn parse_cli_value(flag: &str, value: &str) -> Value {
    match Value::json_to_value(value) {
        Ok(parsed) => parsed,
        Err(err) => {
            log::error!("Failed to parse --{} value '{}': {:?}", flag, value, err);
            std::process::exit(1);
        }
    }
}
