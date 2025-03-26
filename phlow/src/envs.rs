use std::env;

pub struct Envs {
    pub step_consumer_count: i32,
    pub package_consumer_count: i32,
}

impl Envs {
    pub fn load() -> Self {
        let step_consumer_count = env::var("STEP_CONSUMERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1); // Se não houver valor, assume 1

        let package_consumer_count = env::var("PACKAGE_CONSUMERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1); // Se não houver valor, assume 1

        Self {
            step_consumer_count: step_consumer_count as i32,
            package_consumer_count: package_consumer_count as i32,
        }
    }
}
