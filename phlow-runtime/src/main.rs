mod loader;
mod memory;
mod package;
mod runtime;
mod settings;
mod test_runner;
mod yaml;
use loader::Loader;
use log::debug;
use package::Package;
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::tracing::error;
use phlow_sdk::{tracing, use_log};
use runtime::Runtime;
use settings::Settings;
mod scripts;

#[cfg(all(feature = "mimalloc", target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(feature = "jemalloc", target_env = "musl"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "macos")]
pub const MODULE_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
pub const MODULE_EXTENSION: &str = "so";
#[cfg(target_os = "windows")]
pub const MODULE_EXTENSION: &str = "dll";

#[cfg(target_os = "macos")]
pub const RUNTIME_ARCH: &str = "darwin";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const RUNTIME_ARCH: &str = "linux-aarch64";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const RUNTIME_ARCH: &str = "linux-amd64";

#[tokio::main]
async fn main() {
    use_log!();
    log::debug!("Starting Phlow Runtime");

    let settings = match Settings::try_load() {
        Ok(settings) => settings,
        Err(err) => {
            error!("Error loading settings: {:?}", err);
            std::process::exit(1);
        }
    };

    if let Some(publish_path) = settings.package_path.clone() {
        match Package::try_from(publish_path) {
            Ok(publish) => {
                if let Err(err) = publish.run() {
                    error!("Error publishing module: {:?}", err);
                    return;
                }
            }
            Err(err) => {
                error!("Error creating publish instance: {:?}", err);
                return;
            }
        }
    }

    let mut loader =
        match Loader::load("./".to_string(), &settings.main_target, settings.print_yaml).await {
            Ok(main) => main,
            Err(err) => {
                eprintln!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

    if settings.no_run {
        return;
    }

    let guard = init_tracing_subscriber(loader.app_data.clone());

    if let Err(err) = tracing::dispatcher::set_global_default(guard.dispatch.clone()) {
        error!("Failed to set global subscriber: {:?}", err);
        std::process::exit(1);
    }

    let dispatch = guard.dispatch.clone();
    let fut = async {
        if settings.download {
            if let Err(err) = loader
                .download(&settings.default_package_repository_url)
                .await
            {
                error!("Download failed: {:?}", err);
                return;
            }
        }

        loader.update_info();

        if !settings.only_download_modules {
            if settings.test {
                debug!("Run test");
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
                        eprintln!("Test execution error: {}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                debug!("Run application");
                // Run normal workflow
                if let Err(rr) = Runtime::run(loader, dispatch.clone(), settings).await {
                    error!("Runtime Error: {:?}", rr);
                }
            }
        }
    };

    // passamos a future para o escopo correto de dispatcher
    tracing::dispatcher::with_default(&dispatch, || fut).await;
}
