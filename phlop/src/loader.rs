use clap::{Arg, Command};
use phlow_sdk::prelude::*;
use std::fmt::Display;

pub enum LoaderError {
    ModuleLoaderError,
    ModuleNotFound(String),
    StepsNotDefined,
    LoaderErrorJson(serde_json::Error),
    LoaderErrorYaml(serde_yaml::Error),
    LoaderErrorToml(toml::de::Error),
}

impl std::fmt::Debug for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
            LoaderError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            LoaderError::LoaderErrorJson(err) => write!(f, "Json error: {:?}", err),
            LoaderError::LoaderErrorYaml(err) => write!(f, "Yaml error: {:?}", err),
            LoaderError::LoaderErrorToml(err) => write!(f, "Toml error: {:?}", err),
        }
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
            LoaderError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            LoaderError::LoaderErrorJson(err) => write!(f, "Json error: {:?}", err),
            LoaderError::LoaderErrorYaml(err) => write!(f, "Yaml error: {:?}", err),
            LoaderError::LoaderErrorToml(err) => write!(f, "Toml error: {:?}", err),
        }
    }
}

pub enum ModuleExtension {
    Json,
    Yaml,
    Toml,
}

impl From<&str> for ModuleExtension {
    fn from(extension: &str) -> Self {
        match extension {
            "json" => ModuleExtension::Json,
            "yaml" => ModuleExtension::Yaml,
            "yml" => ModuleExtension::Yaml,
            "toml" => ModuleExtension::Toml,
            _ => ModuleExtension::Json,
        }
    }
}

fn get_main_file(main_path: &str) -> Result<(String, ModuleExtension), LoaderError> {
    let path = std::path::Path::new(&main_path);
    if path.is_dir() {
        let file = find_default_file(&main_path);
        match file {
            Some(data) => return Ok(data),
            None => return Err(LoaderError::ModuleNotFound("main".to_string())),
        }
    }

    if path.exists() {
        let extension = match main_path.split('.').last() {
            Some(extension) => extension,
            None => return Err(LoaderError::ModuleNotFound(main_path.to_string())),
        };
        return Ok((main_path.to_string(), ModuleExtension::from(extension)));
    }

    Err(LoaderError::ModuleNotFound(main_path.to_string()))
}

// find main.json, main.yaml, main.yml, main.toml
fn find_default_file(base: &str) -> Option<(String, ModuleExtension)> {
    let files = vec!["main.yaml", "main.yml", "main.json", "main.toml"];

    for file in files {
        let path = if base.is_empty() || base == "." {
            file.to_string()
        } else {
            format!("{}/{}", base, file)
        };
        if std::path::Path::new(&path).exists() {
            let extension = match file.split('.').last() {
                Some(extension) => extension,
                None => return None,
            };
            return Some((path.to_string(), ModuleExtension::from(extension)));
        }
    }

    None
}

fn load_config() -> Result<Value, LoaderError> {
    let matches = Command::new("Phlow Runtime")
        .version("0.1.0")
        .arg(
            Arg::new("main_path")
                .help("Main path/file to load")
                .required(false)
                .index(1),
        )
        .get_matches();

    let (main_file_path, main_ext) = match matches.get_one::<String>("main_path") {
        Some(file) => get_main_file(file)?,
        None => match find_default_file("") {
            Some((file, ext)) => (file, ext),
            None => return Err(LoaderError::ModuleNotFound("main".to_string())),
        },
    };

    let file = match std::fs::read_to_string(&main_file_path) {
        Ok(file) => file,
        Err(_) => return Err(LoaderError::ModuleNotFound(main_file_path)),
    };

    let value: Value = match main_ext {
        ModuleExtension::Json => {
            serde_json::from_str(&file).map_err(LoaderError::LoaderErrorJson)?
        }
        ModuleExtension::Yaml => {
            serde_yaml::from_str(&file).map_err(LoaderError::LoaderErrorYaml)?
        }
        ModuleExtension::Toml => toml::from_str(&file).map_err(LoaderError::LoaderErrorToml)?,
    };

    Ok(value)
}

#[derive(ToValue, FromValue, Clone)]
pub struct Module {
    pub version: Option<String>,
    pub repository: Option<String>,
    pub module: String,
}

impl TryFrom<Value> for Module {
    type Error = LoaderError;

    fn try_from(value: Value) -> Result<Self, LoaderError> {
        let module = match value.get("module") {
            Some(module) => module.to_string(),
            None => return Err(LoaderError::ModuleLoaderError),
        };
        let repository = value.get("repository").map(|v| v.to_string());
        let version = value.get("version").map(|v| v.to_string());

        Ok(Module {
            module,
            repository,
            version,
        })
    }
}

#[derive(ToValue, FromValue)]
pub struct Loader {
    pub modules: Vec<Module>,
}

impl Loader {
    pub fn load() -> Result<Self, LoaderError> {
        let config = load_config()?;
        Loader::try_from(config)
    }
}

impl TryFrom<Value> for Loader {
    type Error = LoaderError;

    fn try_from(value: Value) -> Result<Self, LoaderError> {
        let modules = match value.get("modules") {
            Some(modules) => {
                if !modules.is_array() {
                    return Err(LoaderError::ModuleLoaderError);
                }

                let mut modules_vec = Vec::new();
                let modules_array = match modules.as_array() {
                    Some(modules) => modules,
                    None => return Err(LoaderError::ModuleLoaderError),
                };

                for module in modules_array {
                    let module = match Module::try_from(module.clone()) {
                        Ok(module) => module,
                        Err(_) => return Err(LoaderError::ModuleLoaderError),
                    };

                    let module_path = format!("phlow_modules/{}/module.so", module.module);

                    if !std::path::Path::new(&module_path).exists() {
                        return Err(LoaderError::ModuleNotFound(module.module));
                    }

                    modules_vec.push(module);
                }

                modules_vec
            }
            None => Vec::new(),
        };

        Ok(Loader { modules })
    }
}
