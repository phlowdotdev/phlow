use cli::{Cli, Error};
use envs::Envs;

pub mod cli;
pub mod envs;

#[derive(Debug)]
pub struct Settings {
    pub main_target: String,
    pub only_download_modules: bool,
    pub package_path: Option<String>,
    pub no_run: bool,
    pub download: bool,

    // envs
    pub package_consumer_count: i32,
    pub min_allocated_memory: usize,
    pub garbage_collection: bool,
    pub garbage_collection_interval: u64,
    pub default_package_repository_url: String,
}

impl Settings {
    pub fn try_load() -> Result<Self, Error> {
        let cli = Cli::load()?;
        let envs = Envs::load();

        let main_target = cli.main_target.unwrap_or(envs.main.clone());

        let settings = Self {
            main_target,
            only_download_modules: cli.only_download_modules,
            package_path: cli.package_path,
            no_run: cli.no_run,
            package_consumer_count: envs.package_consumer_count,
            min_allocated_memory: envs.min_allocated_memory,
            garbage_collection: envs.garbage_collection,
            garbage_collection_interval: envs.garbage_collection_interval,
            default_package_repository_url: envs.default_package_repository_url,
            download: cli.download,
        };

        Ok(settings)
    }
}
