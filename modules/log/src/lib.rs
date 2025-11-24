use phlow_sdk::prelude::*;

create_step!(phlow_log(rx));

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
                _ => match value.get("action") {
                    Some(action) => match action.to_string().as_str() {
                        "info" => LogLevel::Info,
                        "debug" => LogLevel::Debug,
                        "warn" => LogLevel::Warn,
                        "error" => LogLevel::Error,
                        _ => LogLevel::Info,
                    },
                    _ => LogLevel::Info,
                },
            },
            _ => match value.get("action") {
                Some(action) => match action.to_string().as_str() {
                    "info" => LogLevel::Info,
                    "debug" => LogLevel::Debug,
                    "warn" => LogLevel::Warn,
                    "error" => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                _ => LogLevel::Info,
            },
        };

        let message = value.get("message").unwrap_or(&Value::Null).to_string();

        Self { level, message }
    }
}

pub async fn phlow_log(rx: ModuleReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Log module started, waiting for messages");

    listen!(rx, move |package: ModulePackage| async {
        let value = package.input().unwrap_or(Value::Null);
        log::debug!("Log module received input: {:?}", value);

        let log_value = Log::from(&value);
        log::debug!("Parsed log: {:?}", log_value);

        match log_value.level {
            LogLevel::Info => log::info!("{}", log_value.message),
            LogLevel::Debug => log::debug!("{}", log_value.message),
            LogLevel::Warn => log::warn!("{}", log_value.message),
            LogLevel::Error => log::error!("{}", log_value.message),
        }

        let payload = package.payload().unwrap_or(Value::Null);
        sender_safe!(package.sender, payload.into());
    });

    Ok(())
}
