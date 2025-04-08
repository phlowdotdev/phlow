use phlow_sdk::tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use phlow_sdk::tracing_subscriber::util::SubscriberInitExt;
use phlow_sdk::tracing_subscriber::Layer;
use phlow_sdk::{
    otel::get_log_level,
    prelude::*,
    tracing_core::LevelFilter,
    tracing_subscriber::{fmt, Registry},
};

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
    Registry::default()
        .with(fmt::layer().with_filter(LevelFilter::from_level(get_log_level())))
        .init();

    debug!("PHLOW_OTEL is set to false, using default subscriber");

    listen!(rx, move |package: ModulePackage| async {
        let value = package.context.input.unwrap_or(Value::Null);
        let log = Log::from(&value);
        println!("Received log package: {:?}", log);

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
