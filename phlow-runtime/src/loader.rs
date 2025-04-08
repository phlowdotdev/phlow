use std::{fmt::Display, fs::File, path::Path};

use crate::{cli::ModuleExtension, yaml::yaml_helpers_transform};
use libloading::{Library, Symbol};
use phlow_sdk::{prelude::*, tracing::info};
use reqwest::Client;
use std::io::Write;

pub enum Error {
    ModuleLoaderError,
    ModuleNotFound(String),
    StepsNotDefined,
    LibLoadingError(libloading::Error),
    LoaderErrorJson(serde_json::Error),
    LoaderErrorYaml(serde_yaml::Error),
    LoaderErrorToml(toml::de::Error),
    GetFileError(reqwest::Error),
    FileCreateError(std::io::Error),
    BufferError(reqwest::Error),
    CopyError(std::io::Error),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ModuleLoaderError => write!(f, "Module loader error"),
            Error::StepsNotDefined => write!(f, "Steps not defined"),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            Error::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
            Error::LoaderErrorJson(err) => write!(f, "Json error: {:?}", err),
            Error::LoaderErrorYaml(err) => write!(f, "Yaml error: {:?}", err),
            Error::LoaderErrorToml(err) => write!(f, "Toml error: {:?}", err),
            Error::GetFileError(err) => write!(f, "Get file error: {:?}", err),
            Error::FileCreateError(err) => write!(f, "File create error: {:?}", err),
            Error::BufferError(err) => write!(f, "Buffer error: {:?}", err),
            Error::CopyError(err) => write!(f, "Copy error: {:?}", err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ModuleLoaderError => write!(f, "Module loader error"),
            Error::StepsNotDefined => write!(f, "Steps not defined"),
            Error::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            Error::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
            Error::LoaderErrorJson(err) => write!(f, "Json error: {:?}", err),
            Error::LoaderErrorYaml(err) => write!(f, "Yaml error: {:?}", err),
            Error::LoaderErrorToml(err) => write!(f, "Toml error: {:?}", err),
            Error::GetFileError(err) => write!(f, "Get file error: {:?}", err),
            Error::FileCreateError(err) => write!(f, "File create error: {:?}", err),
            Error::BufferError(err) => write!(f, "Buffer error: {:?}", err),
            Error::CopyError(err) => write!(f, "Copy error: {:?}", err),
        }
    }
}

#[derive(ToValue, FromValue, Clone, Debug)]
pub struct Module {
    pub version: String,
    pub repository: Option<String>,
    pub repository_path: Option<String>,
    pub module: String,
    pub name: String,
    pub with: Value,
}

impl TryFrom<Value> for Module {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Error> {
        let module = match value.get("module") {
            Some(module) => module.to_string(),
            None => return Err(Error::ModuleLoaderError),
        };
        let repository = value.get("repository").map(|v| v.to_string());

        let repository_path = if repository.is_none() {
            let mut padded = module.to_string();
            while padded.len() < 4 {
                padded.push('_');
            }

            let first_two = &padded[0..2];
            let third = &padded[2..3];

            let middle = if third == "_" || module.len() < 4 {
                format!("_{}", third)
            } else {
                format!("{}_", third)
            };

            let repository = format!("{}/{}/{}", first_two, middle, module);
            Some(repository)
        } else {
            None
        };

        let version = match value.get("version") {
            Some(version) => version.to_string(),
            None => return Err(Error::ModuleLoaderError),
        };

        let name = match value.get("name") {
            Some(name) => name.to_string(),
            None => module.clone(),
        };

        let with = match value.get("with") {
            Some(with) => with.clone(),
            None => Value::Null,
        };
        Ok(Module {
            module,
            repository,
            version,
            name,
            with,
            repository_path,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Loader {
    pub main: i32,
    pub modules: Vec<Module>,
    pub steps: Value,
    pub app_data: ApplicationData,
}

impl Loader {
    pub fn load(main_path: &str, main_ext: &ModuleExtension) -> Result<Self, Error> {
        let value = Self::load_main(main_path, main_ext)?;

        let (main, modules) = match value.get("modules") {
            Some(modules) => {
                if !modules.is_array() {
                    return Err(Error::ModuleLoaderError);
                }

                let main_name = match value.get("main") {
                    Some(main) => Some(main.to_string()),
                    None => None,
                };

                let mut main = -1;

                let mut modules_vec = Vec::new();
                let modules_array = match modules.as_array() {
                    Some(modules) => modules,
                    None => return Err(Error::ModuleLoaderError),
                };

                for module in modules_array {
                    let module = match Module::try_from(module.clone()) {
                        Ok(module) => module,
                        Err(_) => return Err(Error::ModuleLoaderError),
                    };

                    if Some(module.name.clone()) == main_name {
                        main = modules_vec.len() as i32;
                    }

                    modules_vec.push(module);
                }

                (main, modules_vec)
            }
            None => (-1, Vec::new()),
        };

        let steps = match value.get("steps") {
            Some(steps) => steps.clone(),
            None => return Err(Error::StepsNotDefined),
        };

        let name = value.get("name").map(|v| v.to_string());
        let version = value.get("version").map(|v| v.to_string());
        let environment = value.get("environment").map(|v| v.to_string());
        let author = value.get("author").map(|v| v.to_string());
        let description = value.get("description").map(|v| v.to_string());
        let license = value.get("license").map(|v| v.to_string());
        let repository = value.get("repository").map(|v| v.to_string());
        let homepage = value.get("homepage").map(|v| v.to_string());

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

        Ok(Self {
            main,
            modules,
            steps,
            app_data,
        })
    }

    fn load_main(main_file_path: &str, main_ext: &ModuleExtension) -> Result<Value, Error> {
        let file = match std::fs::read_to_string(main_file_path) {
            Ok(file) => file,
            Err(_) => return Err(Error::ModuleNotFound(main_file_path.to_string())),
        };

        let value: Value = match main_ext {
            ModuleExtension::Json => serde_json::from_str(&file).map_err(Error::LoaderErrorJson)?,
            ModuleExtension::Yaml => {
                let yaml_path = Path::new(&main_file_path)
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                let yaml = yaml_helpers_transform(&file, yaml_path);
                serde_yaml::from_str(&yaml).map_err(Error::LoaderErrorYaml)?
            }
            ModuleExtension::Toml => toml::from_str(&file).map_err(Error::LoaderErrorToml)?,
        };

        Ok(value)
    }

    pub fn load_module(setup: ModuleSetup, module_name: &str) -> Result<(), Error> {
        unsafe {
            let lib =
                match Library::new(format!("phlow_modules/{}/module.so", module_name).as_str()) {
                    Ok(lib) => lib,
                    Err(err) => return Err(Error::LibLoadingError(err)),
                };
            let func: Symbol<unsafe extern "C" fn(ModuleSetup)> = match lib.get(b"plugin") {
                Ok(func) => func,
                Err(err) => {
                    return Err(Error::LibLoadingError(err));
                }
            };

            func(setup);

            Ok(())
        }
    }

    pub fn get_steps(&self) -> Value {
        let steps = self.steps.clone();
        json!({
            "steps": steps
        })
    }

    pub async fn download(&self, default_package_repository_url: &str) -> Result<(), Error> {
        if !Path::new("phlow_modules").exists() {
            std::fs::create_dir("phlow_modules").map_err(Error::FileCreateError)?;
        }

        info!("Downloading modules...");

        let client = Client::new();

        for module in &self.modules {
            let base_url = match &module.repository {
                Some(repo) => repo.clone(),
                None => format!(
                    "{}/refs/heads/main/packages/{}",
                    default_package_repository_url,
                    module
                        .repository_path
                        .clone()
                        .ok_or_else(|| Error::ModuleNotFound(module.name.clone()))?
                ),
            };

            info!("Base URL: {}", base_url);

            let version = if module.version == "latest" {
                let metadata_url = format!("{}/metadata.json", base_url);

                let res = client
                    .get(&metadata_url)
                    .send()
                    .await
                    .map_err(Error::GetFileError)?;
                let metadata = {
                    let content = res.text().await.map_err(Error::BufferError)?;
                    serde_json::from_str::<serde_json::Value>(&content)
                        .map_err(Error::LoaderErrorJson)?
                };
                match metadata.get("latest") {
                    Some(version) => version
                        .as_str()
                        .ok_or(Error::ModuleLoaderError)?
                        .to_string(),
                    None => return Err(Error::ModuleLoaderError),
                }
            } else {
                module.version.clone()
            };

            Self::download_and_extract_tarball(&base_url, &module.module, &version).await?;
        }

        info!("All modules downloaded and extracted successfully");
        Ok(())
    }

    async fn download_and_extract_tarball(
        base_url: &str,
        module: &str,
        version: &str,
    ) -> Result<(), Error> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let tarball_name = format!("{}-{}.tar.gz", module, version);
        let target_url = format!("{}/{}", base_url.trim_end_matches('/'), tarball_name);
        let target_path = format!("phlow_modules/{}/{}", module, tarball_name);

        if Path::new(&format!("phlow_modules/{}/module.so", module)).exists() {
            return Ok(()); // já extraído
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
            .unpack(format!("phlow_modules/{}", module))
            .map_err(Error::CopyError)?;

        // Remove o tar.gz após extração
        std::fs::remove_file(&target_path).map_err(Error::FileCreateError)?;

        info!("Module extracted to phlow_modules/{}", module);

        Ok(())
    }
}
