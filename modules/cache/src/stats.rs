/// Statistics tracker for cache operations
#[derive(Debug, Clone)]
pub struct CacheStats {
    total_gets: u64,
    total_hits: u64,
    total_sets: u64,
    total_removes: u64,
    total_clears: u64,
}

impl CacheStats {
    /// Create a new statistics tracker
    pub fn new() -> Self {
        Self {
            total_gets: 0,
            total_hits: 0,
            total_sets: 0,
            total_removes: 0,
            total_clears: 0,
        }
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.total_gets += 1;
        self.total_hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.total_gets += 1;
    }

    /// Record a set operation
    pub fn record_set(&mut self) {
        self.total_sets += 1;
    }

    /// Record a remove operation
    pub fn record_remove(&mut self) {
        self.total_removes += 1;
    }

    /// Record a clear operation
    pub fn record_clear(&mut self, _items_removed: usize) {
        self.total_clears += 1;
    }

    /// Get hit rate as a percentage
    pub fn get_hit_rate(&self) -> f64 {
        if self.total_gets == 0 {
            0.0
        } else {
            (self.total_hits as f64 / self.total_gets as f64) * 100.0
        }
    }

    /// Get total number of get operations (hits + misses)
    pub fn get_total_gets(&self) -> u64 {
        self.total_gets
    }

    /// Get total number of cache hits
    pub fn get_total_hits(&self) -> u64 {
        self.total_hits
    }

    /// Get total number of set operations
    pub fn get_total_sets(&self) -> u64 {
        self.total_sets
    }

    /// Get total number of remove operations
    pub fn get_total_removes(&self) -> u64 {
        self.total_removes
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stats() {
        let stats = CacheStats::new();
        assert_eq!(stats.get_total_gets(), 0);
        assert_eq!(stats.get_total_hits(), 0);
        assert_eq!(stats.get_hit_rate(), 0.0);
    }

    #[test]
    fn test_record_operations() {
        let mut stats = CacheStats::new();

        // Record some hits and misses
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();

        assert_eq!(stats.get_total_gets(), 3);
        assert_eq!(stats.get_total_hits(), 2);
        assert!((stats.get_hit_rate() - 66.66666666666667).abs() < 0.01);

        // Record some sets and removes
        stats.record_set();
        stats.record_set();
        stats.record_remove();

        assert_eq!(stats.get_total_sets(), 2);
        assert_eq!(stats.get_total_removes(), 1);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut stats = CacheStats::new();

        // 100% hit rate
        stats.record_hit();
        stats.record_hit();
        assert_eq!(stats.get_hit_rate(), 100.0);

        // 50% hit rate
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.get_hit_rate(), 50.0);

        // 0% hit rate
        stats.record_miss();
        stats.record_miss();
        assert_eq!(stats.get_hit_rate(), 33.33333333333333);
    }
}
