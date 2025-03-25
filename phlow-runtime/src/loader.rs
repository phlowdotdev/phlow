use clap::{Arg, Command};
use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::fmt::Display;
use tracing::debug;
use valu3::json;

use crate::yaml::yaml_helpers_transform;

pub enum LoaderError {
    ModuleLoaderError,
    ModuleNotFound(String),
    StepsNotDefined,
    LibLoadingError(libloading::Error),
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
            LoaderError::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
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
            LoaderError::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
            LoaderError::LoaderErrorJson(err) => write!(f, "Json error: {:?}", err),
            LoaderError::LoaderErrorYaml(err) => write!(f, "Yaml error: {:?}", err),
            LoaderError::LoaderErrorToml(err) => write!(f, "Toml error: {:?}", err),
        }
    }
}

pub fn load_module(setup: ModuleSetup, module_name: &str) -> Result<(), LoaderError> {
    unsafe {
        debug!("Loading module: {}", module_name);
        let lib = match Library::new(format!("phlow_modules/{}/module.so", module_name).as_str()) {
            Ok(lib) => lib,
            Err(err) => return Err(LoaderError::LibLoadingError(err)),
        };
        let func: Symbol<unsafe extern "C" fn(ModuleSetup)> = match lib.get(b"plugin") {
            Ok(func) => func,
            Err(err) => {
                return Err(LoaderError::LibLoadingError(err));
            }
        };

        func(setup);

        Ok(())
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

// find main.json, main.yaml, main.yml, main.toml
fn find_default_file() -> Option<(String, ModuleExtension)> {
    let files = vec!["main.json", "main.yaml", "main.yml", "main.toml"];

    for file in files {
        if std::path::Path::new(file).exists() {
            let extension = file.split('.').last().unwrap();
            return Some((file.to_string(), ModuleExtension::from(extension)));
        }
    }

    None
}

fn get_file_extension(file: &str) -> ModuleExtension {
    let extension = file.split('.').last().unwrap();
    ModuleExtension::from(extension)
}

fn load_config() -> Result<Value, LoaderError> {
    let matches = Command::new("Phlow Runtime")
        .version("0.1.0")
        .arg(
            Arg::new("main_file")
                .help("Main file to load")
                .required(false)
                .index(1),
        )
        .get_matches();

    let (main_file, main_ext) = match matches.get_one::<String>("main_file") {
        Some(file) => {
            let extension = get_file_extension(file);
            (file.clone(), extension)
        }
        None => match find_default_file() {
            Some(file) => file,
            None => return Err(LoaderError::ModuleNotFound("main".to_string())),
        },
    };

    let file = match std::fs::read_to_string(&main_file) {
        Ok(file) => file,
        Err(_) => return Err(LoaderError::ModuleNotFound(main_file)),
    };

    let value: Value = match main_ext {
        ModuleExtension::Json => {
            serde_json::from_str(&file).map_err(LoaderError::LoaderErrorJson)?
        }
        ModuleExtension::Yaml => {
            let yaml = yaml_helpers_transform(&file);
            println!("{}", yaml);
            serde_yaml::from_str(&yaml).map_err(LoaderError::LoaderErrorYaml)?
        }
        ModuleExtension::Toml => toml::from_str(&file).map_err(LoaderError::LoaderErrorToml)?,
    };

    Ok(value)
}

#[derive(ToValue, FromValue, Clone)]
pub struct Module {
    pub name: String,
    pub with: Value,
}

impl TryFrom<Value> for Module {
    type Error = LoaderError;

    fn try_from(value: Value) -> Result<Self, LoaderError> {
        let name = match value.get("name") {
            Some(name) => name.to_string(),
            None => return Err(LoaderError::ModuleLoaderError),
        };

        let with = match value.get("with") {
            Some(with) => with.clone(),
            None => Value::Null,
        };

        Ok(Module { name, with })
    }
}

#[derive(ToValue, FromValue)]
pub struct Loader {
    pub main: i32,
    pub modules: Vec<Module>,
    pub steps: Value,
}

impl Loader {
    pub fn load() -> Result<Self, LoaderError> {
        let config = load_config()?;
        Loader::try_from(config)
    }

    pub fn get_steps(&self) -> Value {
        let steps = self.steps.clone();
        json!({
            "steps": steps
        })
    }
}

impl TryFrom<Value> for Loader {
    type Error = LoaderError;

    fn try_from(value: Value) -> Result<Self, LoaderError> {
        let (main, modules) = match value.get("modules") {
            Some(modules) => {
                if !modules.is_array() {
                    return Err(LoaderError::ModuleLoaderError);
                }

                let main_name = match value.get("main") {
                    Some(main) => Some(main.to_string()),
                    None => None,
                };

                let mut main = -1;

                let mut modules_vec = Vec::new();
                for module in modules.as_array().unwrap() {
                    let module = match Module::try_from(module.clone()) {
                        Ok(module) => module,
                        Err(_) => return Err(LoaderError::ModuleLoaderError),
                    };

                    if Some(module.name.clone()) == main_name {
                        main = modules_vec.len() as i32;
                    }

                    let module_path = format!("phlow_modules/{}/module.so", module.name);

                    if !std::path::Path::new(&module_path).exists() {
                        return Err(LoaderError::ModuleNotFound(module.name));
                    }

                    modules_vec.push(module);
                }

                (main, modules_vec)
            }
            None => (0, Vec::new()),
        };

        let steps = match value.get("steps") {
            Some(steps) => steps.clone(),
            None => return Err(LoaderError::StepsNotDefined),
        };

        Ok(Loader {
            main,
            modules,
            steps,
        })
    }
}
