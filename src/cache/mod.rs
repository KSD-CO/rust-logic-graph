//! Caching layer for node execution results
//!
//! This module provides a flexible caching system for storing and retrieving
//! node execution results to avoid redundant computations.
//!
//! # Features
//!
//! - **Node Result Caching**: Store results keyed by node ID and input hash
//! - **Cache Invalidation**: LRU, FIFO, and manual invalidation strategies
//! - **TTL Support**: Automatic expiration of cached entries
//! - **Memory Limits**: Configurable memory bounds with automatic eviction
//!
//! # Example
//!
//! ```no_run
//! use rust_logic_graph::cache::{CacheManager, CacheConfig, EvictionPolicy};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = CacheConfig {
//!     max_entries: 1000,
//!     max_memory_bytes: 100 * 1024 * 1024, // 100MB
//!     default_ttl: Some(Duration::from_secs(300)),
//!     eviction_policy: EvictionPolicy::LRU,
//!     enable_background_cleanup: true,
//! };
//!
//! let cache = CacheManager::new(config).await?;
//!
//! // Cache is automatically integrated with the executor
//! # Ok(())
//! # }
//! ```

mod cache_manager;
mod entry;
mod policy;

pub use cache_manager::{CacheConfig, CacheManager};
pub use entry::{CacheEntry, CacheKey};
pub use policy::EvictionPolicy;
