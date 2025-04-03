use sdk::tracing::debug;

extern crate libc;

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
