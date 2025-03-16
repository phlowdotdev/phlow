use clap::{Arg, Command};
use libloading::{Library, Symbol};
use sdk::prelude::*;
use std::fmt::Display;
use tracing::debug;
use valu3::json;

pub enum LoaderError {
    ModuleLoaderError,
    ModuleNotFound(String),
    MainNotDefined,
    StepsNotDefined,
    NoMainFile,
    ValueParseError(valu3::Error),
    LibLoadingError(libloading::Error),
}

impl std::fmt::Debug for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::MainNotDefined => write!(f, "Main not defined"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
            LoaderError::ValueParseError(err) => write!(f, "Value parse error: {:?}", err),
            LoaderError::NoMainFile => write!(f, "No main file"),
            LoaderError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            LoaderError::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
        }
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::MainNotDefined => write!(f, "Main not defined"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
            LoaderError::ValueParseError(err) => write!(f, "Value parse error: {:?}", err),
            LoaderError::NoMainFile => write!(f, "No main file"),
            LoaderError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            LoaderError::LibLoadingError(err) => write!(f, "Lib loading error: {:?}", err),
        }
    }
}

pub fn load_module(setup: ModuleSetup, module: &Module) -> Result<(), LoaderError> {
    unsafe {
        debug!("Loading module: {}", module.name);
        let lib = match Library::new(format!("phlow_modules/{}.so", module.name).as_str()) {
            Ok(lib) => lib,
            Err(err) => return Err(LoaderError::LibLoadingError(err)),
        };
        let func: Symbol<unsafe extern "C" fn(ModuleSetup, Value)> = match lib.get(b"plugin") {
            Ok(func) => func,
            Err(err) => {
                return Err(LoaderError::LibLoadingError(err));
            }
        };

        func(setup, module.with.clone());

        Ok(())
    }
}

fn load_config() -> Result<Value, LoaderError> {
    let matches = Command::new("Phlow Runtime")
        .version("0.1.0")
        .arg(
            Arg::new("main_file")
                .help("Main file to load")
                .required(true)
                .index(1),
        )
        .get_matches();

    match matches.get_one::<String>("main_file") {
        Some(file) => {
            let file = std::fs::read_to_string(file).unwrap();
            match Value::json_to_value(&file) {
                Ok(value) => Ok(value),
                Err(err) => {
                    return Err(LoaderError::ValueParseError(err));
                }
            }
        }
        None => {
            return Err(LoaderError::NoMainFile);
        }
    }
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
                    Some(main) => main.to_string(),
                    None => return Err(LoaderError::MainNotDefined),
                };

                let mut main = 0;

                let mut modules_vec = Vec::new();
                for module in modules.as_array().unwrap() {
                    let module = match Module::try_from(module.clone()) {
                        Ok(module) => module,
                        Err(_) => return Err(LoaderError::ModuleLoaderError),
                    };

                    if module.name == main_name {
                        main = modules_vec.len() as i32;
                    }

                    let module_path = format!("phlow_modules/{}.so", module.name);

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
