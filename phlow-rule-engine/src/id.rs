use serde::Serialize;
use std::fmt::Display;
use valu3::value::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Eq, Hash)]
pub struct ID(Option<String>);

impl ID {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl From<String> for ID {
    fn from(id: String) -> Self {
        Self(Some(id))
    }
}

impl From<&Value> for ID {
    fn from(value: &Value) -> Self {
        Self::from(value.to_string())
    }
}

impl From<Value> for ID {
    fn from(value: Value) -> Self {
        Self::from(value.to_string())
    }
}

impl From<&String> for ID {
    fn from(id: &String) -> Self {
        Self::from(id.to_string())
    }
}

impl From<&str> for ID {
    fn from(id: &str) -> Self {
        Self::from(id.to_string())
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if self.0.is_some() {
                self.0.as_ref().unwrap()
            } else {
                ""
            }
        )
    }
}

impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}
