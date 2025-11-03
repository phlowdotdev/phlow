use cli::{Cli, Error};
use envs::Envs;

pub mod cli;
pub mod envs;

#[derive(Debug, Clone)]
pub struct Settings {
    pub script_main_absolute_path: String,
    pub only_download_modules: bool,
    pub package_path: Option<String>,
    pub create_tar: bool,
    pub no_run: bool,
    pub download: bool,
    pub print_yaml: bool,
    pub test: bool,
    pub test_filter: Option<String>,
    pub var_main: Option<String>,
    // analyzer
    pub analyzer: bool,
    pub analyzer_files: bool,
    pub analyzer_modules: bool,
    pub analyzer_total_steps: bool,
    pub analyzer_total_pipelines: bool,
    pub analyzer_json: bool,
    pub analyzer_inner: bool,
    pub analyzer_all: bool,

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
            create_tar: cli.create_tar,
            no_run: cli.no_run,
            analyzer: cli.analyzer,
            analyzer_files: cli.analyzer_files,
            analyzer_modules: cli.analyzer_modules,
            analyzer_total_steps: cli.analyzer_total_steps,
            analyzer_total_pipelines: cli.analyzer_total_pipelines,
            analyzer_json: cli.analyzer_json,
            analyzer_inner: cli.analyzer_inner,
            analyzer_all: cli.analyzer_all,
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
