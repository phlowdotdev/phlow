use phlow_sdk::tracing::debug;

#[cfg(target_env = "gnu")]
extern crate libc;

#[cfg(target_env = "gnu")]
pub fn force_memory_release(min_allocated_memory: usize) {
    unsafe {
        let result = libc::malloc_trim(min_allocated_memory * 1024 * 1024);
        if result == 0 {
            debug!("Memory release failed");
        } else {
            debug!("Memory released successfully: {}", result);
        }
    }
}

#[cfg(not(target_env = "gnu"))]
pub fn force_memory_release(_: usize) {
    debug!("force_memory_release skipped: not supported in musl");
}
