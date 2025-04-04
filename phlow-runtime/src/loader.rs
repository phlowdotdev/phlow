use std::{fmt::Display, fs::File, path::Path};

use libloading::{Library, Symbol};
use phlow_sdk::prelude::*;

use crate::{cli::ModuleExtension, yaml::yaml_helpers_transform};

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
    pub version: Option<String>,
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
            let mut repository = String::new();

            for char in module.chars() {
                repository += &char.to_string();
                repository += "/";
            }

            repository.pop();
            repository += module.as_str();

            Some(repository)
        } else {
            None
        };

        let version = value.get("version").map(|v| v.to_string());

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

                    let module_path = format!("phlow_modules/{}/module.so", module.module);

                    if !std::path::Path::new(&module_path).exists() {
                        return Err(Error::ModuleNotFound(module.module));
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

        Ok(Self {
            main,
            modules,
            steps,
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
        // If phlow_modules directory does not exist, create it
        if !std::path::Path::new("phlow_modules").exists() {
            std::fs::create_dir("phlow_modules").map_err(|_| Error::ModuleLoaderError)?;
        }

        for module in &self.modules {
            let url = match &module.repository {
                Some(repository) => repository.clone(),
                None => {
                    format!(
                        "{}/refs/heads/main/packages/{}",
                        default_package_repository_url,
                        module
                            .repository_path
                            .clone()
                            .expect("Repository path not found")
                    )
                }
            };

            Self::download_file(&url, &module.module, "phlow.yaml").await?;
            Self::download_file(&url, &module.module, "module.so").await?;
        }

        Ok(())
    }

    async fn download_file(url: &str, module: &str, target: &str) -> Result<(), Error> {
        let response = reqwest::get(url).await.map_err(Error::GetFileError)?;

        let mut file = File::create(format!("phlow_modules/{}/{}", module, target))
            .map_err(Error::FileCreateError)?;

        let content = response.bytes().await.map_err(Error::BufferError)?;

        std::io::copy(&mut content.as_ref(), &mut file).map_err(Error::CopyError)?;
        Ok(())
    }
}
