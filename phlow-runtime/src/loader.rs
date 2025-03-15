use sdk::prelude::*;
use std::fmt::Display;
use valu3::json;

pub enum LoaderError {
    ModuleLoaderError,
    MainNotDefined,
    StepsNotDefined,
}

impl std::fmt::Debug for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::MainNotDefined => write!(f, "Main not defined"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
        }
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::ModuleLoaderError => write!(f, "Module loader error"),
            LoaderError::MainNotDefined => write!(f, "Main not defined"),
            LoaderError::StepsNotDefined => write!(f, "Steps not defined"),
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
