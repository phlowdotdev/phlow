use phlow_sdk::prelude::*;

create_step!(log(rx));

#[derive(Debug)]
enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

#[derive(Debug)]
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
    log::debug!("PHLOW_OTEL is set to false, using default subscriber");

    listen!(rx, move |package: ModulePackage| async {
        let value = package.input.unwrap_or(Value::Null);
        let log = Log::from(&value);

        match log.level {
            LogLevel::Info => log::info!("{}", log.message),
            LogLevel::Debug => log::debug!("{}", log.message),
            LogLevel::Warn => log::warn!("{}", log.message),
            LogLevel::Error => log::error!("{}", log.message),
        }

        sender_safe!(package.sender, Value::Null.into());
    });

    Ok(())
}
