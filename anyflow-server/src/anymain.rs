use std::fmt::Display;

use sdk::prelude::*;

pub enum MainError {
    ModuleNotFound,
    MainNotDefined,
}

impl std::fmt::Debug for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainError::ModuleNotFound => write!(f, "Module not found"),
            MainError::MainNotDefined => write!(f, "Main not defined"),
        }
    }
}

impl Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainError::ModuleNotFound => write!(f, "Module not found"),
            MainError::MainNotDefined => write!(f, "Main not defined"),
        }
    }
}

#[derive(ToValue, FromValue)]
pub struct Main {
    pub module: String,
    pub with: Value,
}

impl TryFrom<Value> for Main {
    type Error = MainError;

    fn try_from(value: Value) -> Result<Self, MainError> {
        let main = match value.get("main") {
            Some(main) => main,
            None => return Err(MainError::MainNotDefined),
        };

        let module = match main.get("module") {
            Some(module) => module.to_string(),
            None => return Err(MainError::ModuleNotFound),
        };

        let with = match main.get("with") {
            Some(with) => with.clone(),
            None => Value::Null,
        };

        Ok(Main { module, with })
    }
}
