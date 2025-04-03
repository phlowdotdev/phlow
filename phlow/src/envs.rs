use std::env;

pub struct Envs {
    pub package_consumer_count: i32,
}

impl Envs {
    pub fn load() -> Self {
        let package_consumer_count = env::var("PHLOW_PACKAGE_CONSUMERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(100);

        Self {
            package_consumer_count: package_consumer_count as i32,
        }
    }
}
