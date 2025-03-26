use sdk::{
    crossbeam::channel,
    modules::ModulePackage,
    prelude::*,
    tracing::{debug, error, info, warn},
};

plugin!(log);

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
        println!("Log from value: {:?}", value);
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

pub fn log(setup: ModuleSetup) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Log module started");
    let (tx, rx) = channel::unbounded::<ModulePackage>();

    setup.setup_sender.send(Some(tx)).unwrap();

    println!("Log module setup sender sent");

    for package in rx {
        let log = match package.context.input {
            Some(value) => Log::from(&value),
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

    println!("Log module finished");

    Ok(())
}
