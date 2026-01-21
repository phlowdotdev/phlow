pub mod error;
pub mod loader;
use crate::MODULE_EXTENSION;
use crate::RUNTIME_ARCH;
use crate::scripts::run_script;
use crate::settings::Settings;
use error::{Error, ModuleError};
use libloading::{Library, Symbol};
use loader::{load_external_module_info, load_local_module_info, load_script};
use log::debug;
use log::info;
use phlow_sdk::prelude::Value;
use phlow_sdk::structs::{ApplicationData, ModuleData, ModuleSetup};
use phlow_sdk::valu3::json;
use reqwest::Client;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub async fn load_script_value(
    script_absolute_path: &str,
    print_yaml: bool,
    print_output: crate::settings::PrintOutput,
    analyzer: Option<&crate::analyzer::Analyzer>,
) -> Result<(Value, String), Error> {
    let script_loaded = load_script(script_absolute_path, print_yaml, print_output, analyzer).await?;
    Ok((script_loaded.script, script_loaded.script_file_path))
}

enum ModuleType {
    Binary,
    Script,
}

struct ModuleTarget {
    pub path: String,
    pub module_type: ModuleType,
}

static LOADED_LIBRARIES: OnceLock<Mutex<Vec<Library>>> = OnceLock::new();

fn retain_library(lib: Library) {
    let libraries = LOADED_LIBRARIES.get_or_init(|| Mutex::new(Vec::new()));
    let mut libraries = libraries.lock().unwrap_or_else(|err| err.into_inner());
    libraries.push(lib);
}

#[derive(Debug, Clone)]
pub struct Loader {
    pub main: i32,
    pub modules: Vec<ModuleData>,
    pub steps: Value,
    pub app_data: ApplicationData,
    pub tests: Option<Value>,
}

impl Loader {
    pub async fn load(
        script_absolute_path: &str,
        print_yaml: bool,
        print_output: crate::settings::PrintOutput,
        analyzer: Option<&crate::analyzer::Analyzer>,
    ) -> Result<Self, Error> {
        let script_loaded =
            load_script(script_absolute_path, print_yaml, print_output, analyzer).await?;

        let base_path = Path::new(&script_loaded.script_file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "./".to_string());

        Self::from_script_value(script_loaded.script, &base_path)
    }

    pub fn from_value(script: &Value, base_path: Option<&Path>) -> Result<Self, Error> {
        let base_path = base_path
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./")));
        let base_path = base_path.to_string_lossy().to_string();

        Self::from_script_value(script.clone(), &base_path)
    }

    fn from_script_value(script: Value, base_path: &str) -> Result<Self, Error> {
        let (main, modules) = match script.get("modules") {
            Some(modules) => {
                if !modules.is_array() {
                    return Err(Error::ModuleLoaderError("Modules not an array".to_string()));
                }

                let main_name = match script.get("main") {
                    Some(main) => Some(main.to_string()),
                    None => None,
                };

                let mut main = -1;

                let mut modules_vec = Vec::new();
                let modules_array = modules.as_array().unwrap();

                for module in modules_array {
                    let mut module = ModuleData::try_from(module.clone())
                        .map_err(|_| Error::ModuleLoaderError("Module not found".to_string()))?;

                    if Some(module.name.clone()) == main_name {
                        main = modules_vec.len() as i32;
                    }

                    if let Some(local_path) = module.local_path {
                        let local_path_fix = format!("{}/{}", base_path, &local_path);
                        module.local_path = Some(local_path_fix);
                    }

                    modules_vec.push(module);
                }

                (main, modules_vec)
            }
            None => (-1, Vec::new()),
        };

        let steps = match script.get("steps") {
            Some(steps) => steps.clone(),
            None => return Err(Error::StepsNotDefined),
        };

        let name = script.get("name").map(|v| v.to_string());
        let version = script.get("version").map(|v| v.to_string());
        let environment = script.get("environment").map(|v| v.to_string());
        let author = script.get("author").map(|v| v.to_string());
        let description = script.get("description").map(|v| v.to_string());
        let license = script.get("license").map(|v| v.to_string());
        let repository = script.get("repository").map(|v| v.to_string());
        let homepage = script.get("homepage").map(|v| v.to_string());

        let app_data = ApplicationData {
            name,
            version,
            environment,
            author,
            description,
            license,
            repository,
            homepage,
        };

        // Extract tests if they exist
        let tests = script.get("tests").cloned();

        Ok(Self {
            main,
            modules,
            steps,
            app_data,
            tests,
        })
    }

    fn find_module_path(module_relative_path: &str) -> Result<ModuleTarget, Error> {
        let path = format!("{}/module.{}", module_relative_path, MODULE_EXTENSION);

        debug!("Find {}...", path);

        if Path::new(&path).exists() {
            Ok(ModuleTarget {
                path,
                module_type: ModuleType::Binary,
            })
        } else {
            let path = format!("{}/module.{}", module_relative_path, "phlow");

            debug!("Find {}...", path);

            if Path::new(&path).exists() {
                Ok(ModuleTarget {
                    path,
                    module_type: ModuleType::Script,
                })
            } else {
                let path = format!("{}.{}", module_relative_path, "phlow");

                debug!("Find {}...", path);

                if Path::new(&path).exists() {
                    Ok(ModuleTarget {
                        path,
                        module_type: ModuleType::Script,
                    })
                } else {
                    debug!("Module not found: {}", module_relative_path);
                    Err(Error::ModuleNotFound(format!(
                        "Module not found at path: {}",
                        module_relative_path
                    )))
                }
            }
        }
    }

    pub fn get_steps(&self) -> Value {
        let steps = self.steps.clone();
        json!({
            "steps": steps
        })
    }

    pub async fn download(&self, default_package_repository_url: &str) -> Result<(), Error> {
        if !Path::new("phlow_packages").exists() {
            std::fs::create_dir("phlow_packages").map_err(Error::FileCreateError)?;
        }

        info!("Downloading modules...");

        let client = Client::new();

        let mut downloads = Vec::new();

        for module in &self.modules {
            // Skip local path modules - they don't need to be downloaded
            if module.local_path.is_some() {
                info!(
                    "Module {} is a local path module, skipping download",
                    module.name
                );
                continue;
            }

            let module_so_path = format!(
                "phlow_packages/{}/module.{}",
                module.module, MODULE_EXTENSION
            );
            if Path::new(&module_so_path).exists() {
                info!(
                    "Module {} ({}) already exists at {}, skipping download",
                    module.name, module.version, module_so_path
                );
                continue;
            }

            let base_url = match &module.repository {
                Some(repo) => repo.clone(),
                None => format!(
                    "{}/{}",
                    if regex::Regex::new(r"^(https?://|\.git|.*@.*)")
                        .unwrap()
                        .is_match(default_package_repository_url)
                    {
                        default_package_repository_url.to_string()
                    } else {
                        format!(
                            "https://raw.githubusercontent.com/{}",
                            default_package_repository_url
                        )
                    },
                    module
                        .repository_path
                        .clone()
                        .ok_or_else(|| Error::ModuleNotFound(module.name.clone()))?
                ),
            };

            info!("Base URL: {}", base_url);

            let version = if module.version == "latest" {
                let metadata_url = format!("{}/metadata.json", base_url);
                info!("Metadata URL: {}", metadata_url);

                let res = client
                    .get(&metadata_url)
                    .send()
                    .await
                    .map_err(Error::GetFileError)?;
                let metadata = {
                    let content = res.text().await.map_err(Error::BufferError)?;
                    Value::json_to_value(&content).map_err(Error::LoaderErrorJsonValu3)?
                };

                match metadata.get("latest") {
                    Some(version) => version.to_string(),
                    None => {
                        return Err(Error::VersionNotFound(ModuleError {
                            module: module.name.clone(),
                        }));
                    }
                }
            } else {
                module.version.clone()
            };

            let handler = Self::download_and_extract_tarball(
                base_url.clone(),
                module.module.clone(),
                version.clone(),
            );

            downloads.push(handler);
        }

        let results = futures::future::join_all(downloads).await;
        for result in results {
            if let Err(err) = result {
                return Err(err);
            }
        }

        info!("All modules downloaded and extracted successfully");
        Ok(())
    }

    async fn download_and_extract_tarball(
        base_url: String,
        module: String,
        version: String,
    ) -> Result<(), Error> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let tarball_name = format!("{}-{}-{}.tar.gz", module, version, RUNTIME_ARCH);
        let target_url = format!("{}/{}", base_url.trim_end_matches('/'), tarball_name);
        let target_path = format!("phlow_packages/{}/{}", module, tarball_name);

        if Path::new(&format!(
            "phlow_packages/{}/module.{}",
            module, MODULE_EXTENSION
        ))
        .exists()
        {
            return Ok(());
        }

        info!(
            "Downloading module tarball {} from {}",
            tarball_name, target_url
        );

        if let Some(parent) = Path::new(&target_path).parent() {
            std::fs::create_dir_all(parent).map_err(Error::FileCreateError)?;
        }

        let client = Client::new();
        let response = client
            .get(&target_url)
            .send()
            .await
            .map_err(Error::GetFileError)?;
        let content = response.bytes().await.map_err(Error::BufferError)?;

        // Salva o tarball temporariamente
        let mut file = File::create(&target_path).map_err(Error::FileCreateError)?;
        file.write_all(&content).map_err(Error::CopyError)?;

        // Extrai o conteúdo
        let tar_gz = File::open(&target_path).map_err(Error::FileCreateError)?;
        let decompressor = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(decompressor);
        archive
            .unpack(format!("phlow_packages/{}", module))
            .map_err(Error::CopyError)?;

        // Remove o tar.gz após extração
        std::fs::remove_file(&target_path).map_err(Error::FileCreateError)?;

        info!("Module extracted to phlow_packages/{}", module);

        Ok(())
    }

    pub fn update_info(&mut self) {
        debug!("update_info");

        for module in &mut self.modules {
            let value = if let Some(local_path) = &module.local_path {
                load_local_module_info(local_path)
            } else {
                load_external_module_info(&module.module)
            };

            debug!("module info loaded");
            module.set_info(value);
        }
    }
}

pub fn load_module(
    setup: ModuleSetup,
    module_name: &str,
    module_version: &str,
    local_path: Option<String>,
    settings: Settings,
) -> Result<(), Error> {
    let target = {
        let module_relative_path = match local_path {
            Some(local_path) => local_path,
            None => format!("phlow_packages/{}", module_name),
        };

        let target = Loader::find_module_path(&module_relative_path)?;

        info!(
            "🧪 Load Module: {} ({}), in {}",
            module_name, module_version, target.path
        );

        target
    };

    match target.module_type {
        ModuleType::Script => {
            run_script(&target.path, setup, &settings);
        }
        ModuleType::Binary => unsafe {
            info!("Loading binary module: {}", target.path);

            let lib: Library = match Library::new(&target.path) {
                Ok(lib) => lib,
                Err(err) => return Err(Error::LibLoadingError(err)),
            };

            {
                let func: Symbol<unsafe extern "C" fn(ModuleSetup)> = match lib.get(b"plugin") {
                    Ok(func) => func,
                    Err(err) => return Err(Error::LibLoadingError(err)),
                };

                func(setup);
            }

            retain_library(lib);
        },
    }

    Ok(())
}
