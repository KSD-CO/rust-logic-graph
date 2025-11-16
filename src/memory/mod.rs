//! Memory optimization utilities
//!
//! This module provides memory pooling and optimization features to reduce
//! allocations and improve performance in high-throughput scenarios.

pub mod pool;
pub mod metrics;

pub use pool::{ContextPool, PoolConfig};
pub use metrics::{MemoryMetrics, AllocationTracker};

use std::sync::Arc;
use parking_lot::RwLock;

/// Global memory metrics instance
static MEMORY_METRICS: once_cell::sync::Lazy<Arc<RwLock<MemoryMetrics>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(MemoryMetrics::default())));

/// Get the global memory metrics instance
pub fn global_metrics() -> Arc<RwLock<MemoryMetrics>> {
    MEMORY_METRICS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_metrics() {
        let metrics = global_metrics();
        assert_eq!(metrics.read().total_allocations(), 0);
    }
}
