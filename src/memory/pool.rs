//! Memory pooling for reducing allocations

use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;

use crate::core::Context;

/// Configuration for memory pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of contexts to keep in pool
    pub max_pooled: usize,
    /// Initial capacity for context data hashmap
    pub initial_capacity: usize,
    /// Enable pool statistics tracking
    pub track_stats: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_pooled: 100,
            initial_capacity: 16,
            track_stats: true,
        }
    }
}

/// Statistics for pool usage
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    /// Total number of contexts acquired from pool
    pub total_acquired: u64,
    /// Number of times a pooled context was reused
    pub reused: u64,
    /// Number of times a new context was created
    pub created: u64,
    /// Current pool size
    pub current_pool_size: usize,
    /// Peak pool size
    pub peak_pool_size: usize,
}

/// Memory pool for Context objects to reduce allocations
pub struct ContextPool {
    pool: Arc<Mutex<Vec<Context>>>,
    config: PoolConfig,
    stats: Arc<Mutex<PoolStats>>,
}

impl ContextPool {
    /// Create a new context pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new context pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(config.max_pooled))),
            config,
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Acquire a context from the pool, or create a new one
    pub fn acquire(&self) -> Context {
        let mut pool = self.pool.lock();

        if let Some(mut ctx) = pool.pop() {
            // Reuse pooled context
            ctx.data.clear();

            if self.config.track_stats {
                let mut stats = self.stats.lock();
                stats.total_acquired += 1;
                stats.reused += 1;
                stats.current_pool_size = pool.len();
            }

            ctx
        } else {
            // Create new context
            if self.config.track_stats {
                let mut stats = self.stats.lock();
                stats.total_acquired += 1;
                stats.created += 1;
            }

            Context {
                data: HashMap::with_capacity(self.config.initial_capacity),
            }
        }
    }

    /// Return a context to the pool
    pub fn release(&self, ctx: Context) {
        let mut pool = self.pool.lock();

        if pool.len() < self.config.max_pooled {
            pool.push(ctx);

            if self.config.track_stats {
                let mut stats = self.stats.lock();
                stats.current_pool_size = pool.len();
                if pool.len() > stats.peak_pool_size {
                    stats.peak_pool_size = pool.len();
                }
            }
        }
        // If pool is full, context is dropped
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().clone()
    }

    /// Clear all pooled contexts
    pub fn clear(&self) {
        let mut pool = self.pool.lock();
        pool.clear();

        if self.config.track_stats {
            let mut stats = self.stats.lock();
            stats.current_pool_size = 0;
        }
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.lock().len()
    }

    /// Get reuse rate (percentage of acquired contexts that were reused)
    pub fn reuse_rate(&self) -> f64 {
        let stats = self.stats.lock();
        if stats.total_acquired == 0 {
            0.0
        } else {
            (stats.reused as f64 / stats.total_acquired as f64) * 100.0
        }
    }
}

impl Default for ContextPool {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for pooled contexts
pub struct PooledContext {
    context: Option<Context>,
    pool: Arc<Mutex<Vec<Context>>>,
    max_pooled: usize,
}

impl PooledContext {
    /// Create a new pooled context guard
    pub fn new(context: Context, pool: Arc<Mutex<Vec<Context>>>, max_pooled: usize) -> Self {
        Self {
            context: Some(context),
            pool,
            max_pooled,
        }
    }

    /// Get mutable reference to the context
    pub fn get_mut(&mut self) -> &mut Context {
        self.context.as_mut().unwrap()
    }

    /// Get immutable reference to the context
    pub fn get(&self) -> &Context {
        self.context.as_ref().unwrap()
    }
}

impl Drop for PooledContext {
    fn drop(&mut self) {
        if let Some(ctx) = self.context.take() {
            let mut pool = self.pool.lock();
            if pool.len() < self.max_pooled {
                pool.push(ctx);
            }
        }
    }
}

impl std::ops::Deref for PooledContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for PooledContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_and_release() {
        let pool = ContextPool::new();

        let ctx1 = pool.acquire();
        assert_eq!(pool.size(), 0);

        pool.release(ctx1);
        assert_eq!(pool.size(), 1);

        let ctx2 = pool.acquire();
        assert_eq!(pool.size(), 0);

        pool.release(ctx2);
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_pool_max_size() {
        let config = PoolConfig {
            max_pooled: 2,
            initial_capacity: 16,
            track_stats: true,
        };
        let pool = ContextPool::with_config(config);

        // Create 3 separate contexts and release them
        let ctx1 = Context::new();
        let ctx2 = Context::new();
        let ctx3 = Context::new();

        pool.release(ctx1);
        pool.release(ctx2);
        pool.release(ctx3);

        // Pool should only keep 2
        assert_eq!(pool.size(), 2);
    }

    #[test]
    fn test_pool_stats() {
        let pool = ContextPool::new();

        let _ctx1 = pool.acquire();
        let _ctx2 = pool.acquire();

        let stats = pool.stats();
        assert_eq!(stats.total_acquired, 2);
        assert_eq!(stats.created, 2);
        assert_eq!(stats.reused, 0);

        pool.release(_ctx1);
        pool.release(_ctx2);

        let _ctx3 = pool.acquire();
        let stats = pool.stats();
        assert_eq!(stats.total_acquired, 3);
        assert_eq!(stats.reused, 1);
    }

    #[test]
    fn test_reuse_rate() {
        let pool = ContextPool::new();

        // Create 2 contexts
        let ctx1 = pool.acquire();
        let ctx2 = pool.acquire();

        // Return them
        pool.release(ctx1);
        pool.release(ctx2);

        // Acquire 2 (reused)
        let _ctx3 = pool.acquire();
        let _ctx4 = pool.acquire();

        // Reuse rate should be 50% (2 reused out of 4 total)
        let rate = pool.reuse_rate();
        assert_eq!(rate, 50.0);
    }

    #[test]
    fn test_pooled_context_guard() {
        let pool_vec = Arc::new(Mutex::new(Vec::new()));
        let ctx = Context::new();

        {
            let _guard = PooledContext::new(ctx, pool_vec.clone(), 10);
            assert_eq!(pool_vec.lock().len(), 0);
        }

        // After drop, context should be in pool
        assert_eq!(pool_vec.lock().len(), 1);
    }
}
