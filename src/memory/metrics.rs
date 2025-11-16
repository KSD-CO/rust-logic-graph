//! Memory metrics and tracking

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Memory allocation metrics
#[derive(Debug, Default)]
pub struct MemoryMetrics {
    /// Total number of allocations
    total_allocs: AtomicU64,
    /// Total number of deallocations
    total_deallocs: AtomicU64,
    /// Total bytes allocated
    total_bytes: AtomicUsize,
    /// Peak memory usage
    peak_bytes: AtomicUsize,
    /// Current memory usage
    current_bytes: AtomicUsize,
    /// Number of context allocations
    context_allocs: AtomicU64,
    /// Number of graph allocations
    graph_allocs: AtomicU64,
}

impl MemoryMetrics {
    /// Create new memory metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    pub fn record_alloc(&self, bytes: usize) {
        self.total_allocs.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);
        let current = self.current_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes;

        // Update peak if necessary
        let mut peak = self.peak_bytes.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_bytes.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => peak = x,
            }
        }
    }

    /// Record a deallocation
    pub fn record_dealloc(&self, bytes: usize) {
        self.total_deallocs.fetch_add(1, Ordering::Relaxed);
        self.current_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Record a context allocation
    pub fn record_context_alloc(&self) {
        self.context_allocs.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a graph allocation
    pub fn record_graph_alloc(&self) {
        self.graph_allocs.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total allocations
    pub fn total_allocations(&self) -> u64 {
        self.total_allocs.load(Ordering::Relaxed)
    }

    /// Get total deallocations
    pub fn total_deallocations(&self) -> u64 {
        self.total_deallocs.load(Ordering::Relaxed)
    }

    /// Get total bytes allocated
    pub fn total_bytes(&self) -> usize {
        self.total_bytes.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_bytes(&self) -> usize {
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// Get current memory usage
    pub fn current_bytes(&self) -> usize {
        self.current_bytes.load(Ordering::Relaxed)
    }

    /// Get context allocations
    pub fn context_allocations(&self) -> u64 {
        self.context_allocs.load(Ordering::Relaxed)
    }

    /// Get graph allocations
    pub fn graph_allocations(&self) -> u64 {
        self.graph_allocs.load(Ordering::Relaxed)
    }

    /// Get active allocations
    pub fn active_allocations(&self) -> u64 {
        let allocs = self.total_allocs.load(Ordering::Relaxed);
        let deallocs = self.total_deallocs.load(Ordering::Relaxed);
        allocs.saturating_sub(deallocs)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.total_allocs.store(0, Ordering::Relaxed);
        self.total_deallocs.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.peak_bytes.store(0, Ordering::Relaxed);
        self.current_bytes.store(0, Ordering::Relaxed);
        self.context_allocs.store(0, Ordering::Relaxed);
        self.graph_allocs.store(0, Ordering::Relaxed);
    }

    /// Print memory metrics summary
    pub fn summary(&self) -> String {
        format!(
            "Memory Metrics:\n\
             Total Allocations: {}\n\
             Total Deallocations: {}\n\
             Active Allocations: {}\n\
             Total Bytes: {} ({:.2} MB)\n\
             Current Bytes: {} ({:.2} MB)\n\
             Peak Bytes: {} ({:.2} MB)\n\
             Context Allocations: {}\n\
             Graph Allocations: {}",
            self.total_allocations(),
            self.total_deallocations(),
            self.active_allocations(),
            self.total_bytes(),
            self.total_bytes() as f64 / 1024.0 / 1024.0,
            self.current_bytes(),
            self.current_bytes() as f64 / 1024.0 / 1024.0,
            self.peak_bytes(),
            self.peak_bytes() as f64 / 1024.0 / 1024.0,
            self.context_allocations(),
            self.graph_allocations(),
        )
    }
}

/// Allocation tracker for scoped measurements
pub struct AllocationTracker {
    start_allocs: u64,
    start_bytes: usize,
    start_time: Instant,
    name: String,
}

impl AllocationTracker {
    /// Create a new allocation tracker
    pub fn new(name: impl Into<String>) -> Self {
        let metrics = crate::memory::global_metrics();
        let m = metrics.read();

        Self {
            start_allocs: m.total_allocations(),
            start_bytes: m.total_bytes(),
            start_time: Instant::now(),
            name: name.into(),
        }
    }

    /// Stop tracking and print results
    pub fn stop(&self) {
        let metrics = crate::memory::global_metrics();
        let m = metrics.read();

        let allocs = m.total_allocations() - self.start_allocs;
        let bytes = m.total_bytes() - self.start_bytes;
        let duration = self.start_time.elapsed();

        println!(
            "[{}] Allocations: {}, Bytes: {} ({:.2} MB), Duration: {:?}",
            self.name,
            allocs,
            bytes,
            bytes as f64 / 1024.0 / 1024.0,
            duration
        );
    }
}

impl Drop for AllocationTracker {
    fn drop(&mut self) {
        // Auto-print on drop if not explicitly stopped
    }
}

/// Memory profiling utilities
pub struct MemoryProfiler;

impl MemoryProfiler {
    /// Profile a function's memory usage
    pub fn profile<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let tracker = AllocationTracker::new(name);
        let result = f();
        tracker.stop();
        result
    }

    /// Profile an async function's memory usage
    pub async fn profile_async<F, Fut, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let tracker = AllocationTracker::new(name);
        let result = f().await;
        tracker.stop();
        result
    }

    /// Get current memory snapshot
    pub fn snapshot() -> MemorySnapshot {
        let metrics = crate::memory::global_metrics();
        let m = metrics.read();

        MemorySnapshot {
            total_allocs: m.total_allocations(),
            total_deallocs: m.total_deallocations(),
            current_bytes: m.current_bytes(),
            peak_bytes: m.peak_bytes(),
            timestamp: Instant::now(),
        }
    }
}

/// Memory snapshot at a point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub total_allocs: u64,
    pub total_deallocs: u64,
    pub current_bytes: usize,
    pub peak_bytes: usize,
    pub timestamp: Instant,
}

impl MemorySnapshot {
    /// Calculate difference from another snapshot
    pub fn diff(&self, other: &MemorySnapshot) -> MemoryDiff {
        MemoryDiff {
            allocs_delta: self.total_allocs as i64 - other.total_allocs as i64,
            bytes_delta: self.current_bytes as i64 - other.current_bytes as i64,
            duration: self.timestamp.duration_since(other.timestamp),
        }
    }
}

/// Difference between two memory snapshots
#[derive(Debug, Clone)]
pub struct MemoryDiff {
    pub allocs_delta: i64,
    pub bytes_delta: i64,
    pub duration: Duration,
}

impl MemoryDiff {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Δ Allocations: {:+}, Δ Bytes: {:+} ({:+.2} MB), Duration: {:?}",
            self.allocs_delta,
            self.bytes_delta,
            self.bytes_delta as f64 / 1024.0 / 1024.0,
            self.duration
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_metrics() {
        let metrics = MemoryMetrics::new();

        metrics.record_alloc(1024);
        assert_eq!(metrics.total_allocations(), 1);
        assert_eq!(metrics.current_bytes(), 1024);
        assert_eq!(metrics.peak_bytes(), 1024);

        metrics.record_alloc(2048);
        assert_eq!(metrics.total_allocations(), 2);
        assert_eq!(metrics.current_bytes(), 3072);
        assert_eq!(metrics.peak_bytes(), 3072);

        metrics.record_dealloc(1024);
        assert_eq!(metrics.current_bytes(), 2048);
        assert_eq!(metrics.peak_bytes(), 3072); // Peak doesn't decrease
    }

    #[test]
    fn test_active_allocations() {
        let metrics = MemoryMetrics::new();

        metrics.record_alloc(100);
        metrics.record_alloc(200);
        assert_eq!(metrics.active_allocations(), 2);

        metrics.record_dealloc(100);
        assert_eq!(metrics.active_allocations(), 1);
    }

    #[test]
    fn test_memory_snapshot_diff() {
        let snap1 = MemorySnapshot {
            total_allocs: 10,
            total_deallocs: 5,
            current_bytes: 1024,
            peak_bytes: 2048,
            timestamp: Instant::now(),
        };

        std::thread::sleep(Duration::from_millis(10));

        let snap2 = MemorySnapshot {
            total_allocs: 15,
            total_deallocs: 7,
            current_bytes: 2048,
            peak_bytes: 4096,
            timestamp: Instant::now(),
        };

        let diff = snap2.diff(&snap1);
        assert_eq!(diff.allocs_delta, 5);
        assert_eq!(diff.bytes_delta, 1024);
        assert!(diff.duration.as_millis() >= 10);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = MemoryMetrics::new();

        metrics.record_alloc(1024);
        metrics.record_context_alloc();
        assert_eq!(metrics.total_allocations(), 1);
        assert_eq!(metrics.context_allocations(), 1);

        metrics.reset();
        assert_eq!(metrics.total_allocations(), 0);
        assert_eq!(metrics.context_allocations(), 0);
    }
}
