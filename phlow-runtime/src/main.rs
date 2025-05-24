mod loader;
mod memory;
mod package;
mod runtime;
mod settings;
mod yaml;
use loader::Loader;
use log::debug;
use package::Package;
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::tracing::error;
use phlow_sdk::{tokio, tracing};
use runtime::Runtime;
use settings::Settings;

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

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .default_filter_or("info")
            .filter_or("PHLOW_LOG", "info"),
    )
    .init();

    debug!("Starting Phlow Runtime");

    let settings = Settings::try_load().expect("Error loading settings");

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

    let mut loader = match Loader::load(&settings.main_target, settings.print_yaml).await {
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

    tracing::dispatcher::set_global_default(guard.dispatch.clone())
        .expect("failed to set global subscriber"); // ✅ depois aplica global

    let dispatch = guard.dispatch.clone();
    let fut = async {
        if settings.download {
            loader
                .download(&settings.default_package_repository_url)
                .await
                .expect("Download failed");
        }

        loader.update_info();

        if !settings.only_download_modules {
            if let Err(rr) = Runtime::run(loader, dispatch.clone(), settings).await {
                error!("Runtime Error: {:?}", rr);
            }
        }
    };

    // passamos a future para o escopo correto de dispatcher
    tracing::dispatcher::with_default(&dispatch, || fut).await;
}
