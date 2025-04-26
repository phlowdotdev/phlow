mod loader;
mod log;
mod memory;
mod package;
mod runtime;
mod settings;
mod yaml;
use ::log::debug;
use loader::Loader;
use log::init_tracing;
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

#[tokio::main]
async fn main() {
    env_logger::init();

    debug!("Starting Phlow Runtime");

    let settings = Settings::try_load().expect("Error loading settings");

    if let Some(publish_path) = settings.package_path.clone() {
        init_tracing();

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

    let mut loader = match Loader::load(&settings.main_target).await {
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
            Runtime::run(loader, dispatch.clone(), settings).await;
        }
    };

    // passamos a future para o escopo correto de dispatcher
    tracing::dispatcher::with_default(&dispatch, || fut).await;
}
