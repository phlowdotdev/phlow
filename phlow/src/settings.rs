use std::env;

use sdk::tracing::debug;

pub struct Settings {
    /**
     * Number of package consumers
     *
     * This is the number of threads that will be used to process packages.
     * Environment variable: PHLOW_PACKAGE_CONSUMERS_COUNT
     * Default: 10
     */
    pub package_consumer_count: i32,
    /**
     * Minimum allocated memory in MB
     *
     * This is the minimum amount of memory that will be allocated to the process.
     * Environment variable: PHLOW_MIN_ALLOCATED_MEMORY_MB
     * Default: 10
     */
    pub min_allocated_memory: usize,
    /**
     * Enable garbage collection
     *
     * This will enable or disable garbage collection.
     * Environment variable: PHLOW_GARBAGE_COLLECTION_ENABLED
     * Default: true
     */
    pub garbage_collection: bool,
    /**
     * Garbage collection interval in seconds
     *
     * This is the interval at which garbage collection will be performed.
     * Environment variable: PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS
     * Default: 60
     */
    pub garbage_collection_interval: u64,
}

impl Settings {
    pub fn load() -> Self {
        let package_consumer_count = env::var("PHLOW_PACKAGE_CONSUMERS_COUNT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10) as i32;

        let min_allocated_memory = env::var("PHLOW_MIN_ALLOCATED_MEMORY_MB")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10);

        let garbage_collection = env::var("PHLOW_GARBAGE_COLLECTION_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let garbage_collection_interval = env::var("PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60);

        debug!("PHLOW_PACKAGE_CONSUMERS_COUNT = {}", package_consumer_count);
        debug!("PHLOW_MIN_ALLOCATED_MEMORY_MB = {}", min_allocated_memory);
        debug!("PHLOW_GARBAGE_COLLECTION_ENABLED = {}", garbage_collection);
        debug!(
            "PHLOW_GARBAGE_COLLECTION_INTERVAL_SECONDS = {}",
            garbage_collection_interval
        );

        Self {
            package_consumer_count,
            min_allocated_memory,
            garbage_collection,
            garbage_collection_interval,
        }
    }
}
