use phlow_sdk::prelude::{NumberBehavior, Value};

pub struct Config {
    pub timeout: u64,
    pub verify_ssl: bool,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        if value.is_null() {
            return Config {
                timeout: 29,
                verify_ssl: true,
            };
        }

        let timeout = value.get("timeout").and_then(Value::to_i64).unwrap_or(29) as u64;
        let verify_ssl = *value
            .get("verify_ssl")
            .and_then(Value::as_bool)
            .unwrap_or(&true);

        Config {
            timeout,
            verify_ssl,
        }
    }
}
