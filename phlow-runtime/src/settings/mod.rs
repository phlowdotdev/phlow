use cli::{Cli, Error};
use envs::Envs;

pub mod cli;
pub mod envs;

#[derive(Debug, Clone)]
pub struct Settings {
    pub script_main_absolute_path: String,
    pub only_download_modules: bool,
    pub package_path: Option<String>,
    pub no_run: bool,
    pub download: bool,
    pub print_yaml: bool,
    pub test: bool,
    pub test_filter: Option<String>,
    pub var_main: Option<String>,

    // envs
    pub package_consumer_count: i32,
    #[cfg(target_env = "gnu")]
    pub min_allocated_memory: usize,
    #[cfg(target_env = "gnu")]
    pub garbage_collection: bool,
    #[cfg(target_env = "gnu")]
    pub garbage_collection_interval: u64,
    pub default_package_repository_url: String,
}

impl Settings {
    pub fn try_load() -> Result<Self, Error> {
        let cli = Cli::load()?;
        let envs = Envs::load();

        let main_target = cli.main_target.unwrap_or(envs.main.clone());

        let current_path = std::env::current_dir()
            .unwrap_or_else(|_| "./".into())
            .to_string_lossy()
            .to_string();

        let script_main_absolute_path = if main_target.starts_with(".") {
            format!("{}/{}", current_path, main_target)
        } else {
            main_target.clone()
        };

        let settings = Self {
            script_main_absolute_path,
            only_download_modules: cli.only_download_modules,
            package_path: cli.package_path,
            no_run: cli.no_run,
            package_consumer_count: envs.package_consumer_count,
            #[cfg(target_env = "gnu")]
            min_allocated_memory: envs.min_allocated_memory,
            #[cfg(target_env = "gnu")]
            garbage_collection: envs.garbage_collection,
            #[cfg(target_env = "gnu")]
            garbage_collection_interval: envs.garbage_collection_interval,
            default_package_repository_url: envs.default_package_repository_url,
            download: cli.download,
            print_yaml: cli.print_yaml,
            test: cli.test,
            test_filter: cli.test_filter,
            var_main: cli.var_main,
        };

        Ok(settings)
    }
}
