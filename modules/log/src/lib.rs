use phlow_sdk::prelude::*;

create_step!(log(rx));

enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

struct Log {
    level: LogLevel,
    message: String,
}

impl From<&Value> for Log {
    fn from(value: &Value) -> Self {
        let level = match value.get("level") {
            Some(level) => match level.to_string().as_str() {
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => LogLevel::Info,
            },
            _ => LogLevel::Info,
        };

        let message = value.get("message").unwrap_or(&Value::Null).to_string();

        Self { level, message }
    }
}

pub async fn log(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    listen!(rx, move |package: ModulePackage| async {
        let value = package.context.input.unwrap_or(Value::Null);
        let log = Log::from(&value);

        match log.level {
            LogLevel::Info => info!("{}", log.message),
            LogLevel::Debug => debug!("{}", log.message),
            LogLevel::Warn => warn!("{}", log.message),
            LogLevel::Error => error!("{}", log.message),
        }

        sender_safe!(package.sender, Value::Null);
    });

    Ok(())
}
