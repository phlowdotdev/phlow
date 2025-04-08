use phlow_sdk::tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use phlow_sdk::tracing_subscriber::util::SubscriberInitExt;
use phlow_sdk::tracing_subscriber::Layer;
use phlow_sdk::tracing_subscriber::{fmt, Registry};
use phlow_sdk::{tracing::Level, tracing_core::LevelFilter};

fn get_log_level() -> Level {
    match std::env::var("PHLOW_LOG") {
        Ok(level) => level.parse::<Level>().unwrap_or(Level::INFO),
        Err(_) => Level::INFO,
    }
}

pub fn init_tracing() {
    Registry::default()
        .with(fmt::layer().with_filter(LevelFilter::from_level(get_log_level())))
        .init()
}
