use std::sync::mpsc::channel;

use sdk::{
    modules::ModulePackage,
    prelude::*,
    tracing::{debug, error, info, warn},
};

plugin_async!(log);

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

impl TryFrom<&Value> for Log {
    type Error = Error;

    fn try_from(value: &Value) -> Result<Self, Error> {
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

        let message = value.get("message").unwrap().to_string();

        Ok(Self { level, message })
    }
}

pub async fn log(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = channel::<ModulePackage>();

    setup.setup_sender.send(Some(tx)).unwrap();

    for package in rx {
        let log = match package.context.input {
            Some(value) => match Log::try_from(&value) {
                Ok(log) => log,
                Err(_) => Log {
                    level: LogLevel::Info,
                    message: value.to_string(),
                },
            },
            _ => Log {
                level: LogLevel::Info,
                message: "".to_string(),
            },
        };

        match log.level {
            LogLevel::Info => info!("{}", log.message),
            LogLevel::Debug => debug!("{}", log.message),
            LogLevel::Warn => warn!("{}", log.message),
            LogLevel::Error => error!("{}", log.message),
        }
    }

    Ok(())
}
