mod analyzer;
mod loader;
mod memory;
mod package;
mod preprocessor;
mod runtime;
mod settings;
mod test_runner;
use loader::Loader;
use package::Package;
use phlow_sdk::otel::init_tracing_subscriber;
use phlow_sdk::{tracing, use_log};
use runtime::Runtime;
use serde_json;
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
            log::error!("Error loading settings: {:?}", err);
            std::process::exit(1);
        }
    };

    if let Some(publish_path) = settings.package_path.clone() {
        match Package::try_from(publish_path) {
            Ok(publish) => {
                if let Err(err) = publish.run() {
                    log::error!("Error publishing module: {:?}", err);
                    return;
                }
            }
            Err(err) => {
                log::error!("Error creating publish instance: {:?}", err);
                return;
            }
        }
    }

    // Analyzer mode: if enabled, run analyzer and exit without executing runtime
    if settings.analyzer {
        let mut af = settings.analyzer_files;
        let mut am = settings.analyzer_modules;
        let mut ats = settings.analyzer_total_steps;
        let mut atp = settings.analyzer_total_pipelines;
        let show_json = settings.analyzer_json;

        // If no specific analyzer flags were provided, show all
        if !af && !am && !ats && !atp {
            af = true;
            am = true;
            ats = true;
            atp = true;
        }

        match analyzer::analyze(&settings.script_main_absolute_path, af, am, ats, atp).await {
            Ok(result) => {
                if show_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result).unwrap_or_default()
                    );
                } else {
                    // text output
                    if af {
                        if let Some(files) = result.get("files") {
                            println!("Files:");
                            if let Some(arr) = files.as_array() {
                                for f in arr {
                                    println!("  - {}", f.as_str().unwrap_or(""));
                                }
                            }
                        }
                    }

                    if am {
                        if let Some(mods) = result.get("modules") {
                            println!("Modules:");
                            if let Some(arr) = mods.as_array() {
                                for m in arr {
                                    let declared =
                                        m.get("declared").and_then(|v| v.as_str()).unwrap_or("");
                                    let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let downloaded = m
                                        .get("downloaded")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    println!(
                                        "  - {} ({}): downloaded={}",
                                        declared, name, downloaded
                                    );
                                }
                            }
                        }
                    }

                    if ats {
                        if let Some(ts) = result.get("total_steps") {
                            println!("Total steps: {}", ts.as_i64().unwrap_or(0));
                        }
                    }

                    if atp {
                        if let Some(tp) = result.get("total_pipelines") {
                            println!("Total pipelines: {}", tp.as_i64().unwrap_or(0));
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Analyzer error: {:?}", err);
                std::process::exit(1);
            }
        }

        return;
    }

    // Load the script into Loader (parsing / preprocessing) for normal runtime path
    let mut loader =
        match Loader::load(&settings.script_main_absolute_path, settings.print_yaml).await {
            Ok(main) => main,
            Err(err) => {
                log::error!("Runtime Error Main File: {:?}", err);
                return;
            }
        };

    if settings.no_run {
        return;
    }

    let guard = init_tracing_subscriber(loader.app_data.clone());

    if let Err(err) = tracing::dispatcher::set_global_default(guard.dispatch.clone()) {
        log::error!("Failed to set global subscriber: {:?}", err);
        std::process::exit(1);
    }

    let dispatch = guard.dispatch.clone();
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

    // passamos a future para o escopo correto de dispatcher
    tracing::dispatcher::with_default(&dispatch, || fut).await;
}
