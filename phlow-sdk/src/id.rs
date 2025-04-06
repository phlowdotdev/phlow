use serde::Serialize;
use std::fmt::Display;
use valu3::{traits::ToValueBehavior, value::Value};

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

impl ToValueBehavior for ID {
    fn to_value(&self) -> Value {
        self.0.to_value()
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
                match self.0.as_ref() {
                    Some(id) => id,
                    None => "unknown",
                }
            } else {
                "unknown"
            }
        )
    }
}

impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}
