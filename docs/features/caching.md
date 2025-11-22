# 🗄️ Cache Layer Documentation

The Rust Logic Graph caching layer provides a high-performance, configurable caching system for node execution results. This document covers all caching features, configuration options, and best practices.

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [Cache Strategies](#cache-strategies)
6. [Usage Examples](#usage-examples)
7. [Best Practices](#best-practices)
8. [Performance Considerations](#performance-considerations)
9. [API Reference](#api-reference)

---

## Overview

The caching layer automatically stores node execution results and retrieves them on subsequent runs with the same inputs, significantly improving performance for:

- Expensive computations
- Database queries
- API calls
- AI/LLM requests
- Any deterministic operations

### Key Concepts

- **Cache Key**: Combination of node ID + input hash
- **Cache Entry**: Stored result with metadata (TTL, access count, size)
- **Eviction Policy**: Strategy for removing entries when limits are reached
- **TTL (Time To Live)**: Automatic expiration of cached entries

---

## Features

### ✅ Node Result Caching

- Automatic caching based on node ID and input data
- Stable key generation using content hashing
- Thread-safe concurrent access with `DashMap`

### ✅ Cache Invalidation Strategies

- **LRU (Least Recently Used)**: Evict entries not accessed recently
- **FIFO (First In First Out)**: Evict oldest entries first
- **LFU (Least Frequently Used)**: Evict entries with lowest access count
- **Manual**: Explicit invalidation only

### ✅ TTL Support

- Configurable time-to-live per entry or globally
- Automatic expiration checking on access
- Background cleanup task for expired entries
- Sub-second precision

### ✅ Memory Limits

- Configurable maximum entries
- Configurable maximum memory usage
- Automatic eviction when limits reached
- Memory size estimation for cached values

### ✅ Monitoring & Statistics

- Hit/miss tracking
- Eviction counting
- Memory usage tracking
- Hit rate calculation

---

## Quick Start

### 1. Add Cache Dependency

The cache module is included in the main crate:

```rust
use rust_logic_graph::{
    CacheManager, CacheConfig, EvictionPolicy, Executor
};
```

### 2. Create Cache Manager

```rust
use std::time::Duration;

let cache_config = CacheConfig {
    max_entries: 10000,
    max_memory_bytes: 100 * 1024 * 1024, // 100MB
    default_ttl: Some(Duration::from_secs(300)), // 5 minutes
    eviction_policy: EvictionPolicy::LRU,
    enable_background_cleanup: true,
};

let cache = CacheManager::new(cache_config).await?;
```

### 3. Enable Caching in Executor

```rust
// Create executor with cache
let executor = Executor::with_cache(cache);

// Or add cache to existing executor
let mut executor = Executor::new();
executor.set_cache(cache);
```

### 4. Execute Graph

The cache is now automatically used for all node executions:

```rust
executor.execute(&mut graph).await?;

// Check cache statistics
let stats = executor.cache().unwrap().stats();
println!("Hit rate: {:.1}%", stats.hit_rate());
```

---

## Configuration

### CacheConfig Structure

```rust
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    
    /// Maximum memory usage in bytes (approximate)
    pub max_memory_bytes: usize,
    
    /// Default TTL for cache entries (None = no expiration)
    pub default_ttl: Option<Duration>,
    
    /// Eviction policy when limits are reached
    pub eviction_policy: EvictionPolicy,
    
    /// Enable background cleanup task for expired entries
    pub enable_background_cleanup: bool,
}
```

### Default Configuration

```rust
let config = CacheConfig::default();
// Equivalent to:
// max_entries: 10000
// max_memory_bytes: 100MB
// default_ttl: 5 minutes
// eviction_policy: LRU
// enable_background_cleanup: true
```

### Configuration Examples

#### High-Performance, Short-Lived Cache

```rust
let config = CacheConfig {
    max_entries: 100000,
    max_memory_bytes: 1024 * 1024 * 1024, // 1GB
    default_ttl: Some(Duration::from_secs(60)), // 1 minute
    eviction_policy: EvictionPolicy::LRU,
    enable_background_cleanup: true,
};
```

#### Long-Term, Memory-Constrained Cache

```rust
let config = CacheConfig {
    max_entries: 1000,
    max_memory_bytes: 10 * 1024 * 1024, // 10MB
    default_ttl: Some(Duration::from_secs(3600)), // 1 hour
    eviction_policy: EvictionPolicy::LFU,
    enable_background_cleanup: true,
};
```

#### Persistent Cache (No Expiration)

```rust
let config = CacheConfig {
    max_entries: 50000,
    max_memory_bytes: 500 * 1024 * 1024, // 500MB
    default_ttl: None, // Never expire
    eviction_policy: EvictionPolicy::LRU,
    enable_background_cleanup: false,
};
```

---

## Cache Strategies

### Eviction Policies

#### LRU (Least Recently Used)

Best for: General-purpose caching, frequently accessed data

```rust
eviction_policy: EvictionPolicy::LRU
```

Evicts entries that haven't been accessed recently. Tracks `last_accessed` timestamp.

**Pros:**
- Works well for most workloads
- Adapts to changing access patterns
- Good balance of simplicity and effectiveness

**Cons:**
- May evict entries that will be needed soon
- Doesn't consider access frequency

#### FIFO (First In First Out)

Best for: Streaming data, time-series operations

```rust
eviction_policy: EvictionPolicy::FIFO
```

Evicts oldest entries first. Tracks `created_at` timestamp.

**Pros:**
- Simple and predictable
- Good for temporal data
- Low overhead

**Cons:**
- Doesn't consider usage patterns
- May evict frequently-used entries

#### LFU (Least Frequently Used)

Best for: Stable workloads, hot data optimization

```rust
eviction_policy: EvictionPolicy::LFU
```

Evicts entries with the lowest access count.

**Pros:**
- Keeps frequently-accessed data
- Excellent for stable workloads
- Maximizes hit rate for hot data

**Cons:**
- Can't adapt quickly to changing patterns
- Newly-added popular entries may be evicted

#### None (Manual Only)

Best for: Full control, predictable behavior

```rust
eviction_policy: EvictionPolicy::None
```

No automatic eviction. Only manual invalidation.

**Pros:**
- Complete control
- Predictable behavior
- No surprise evictions

**Cons:**
- Must manually manage cache
- Can exceed limits if not careful

---

## Usage Examples

### Basic Caching

```rust
use rust_logic_graph::{CacheManager, CacheConfig, Executor, Graph};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create cache
    let cache = CacheManager::new(CacheConfig::default()).await?;
    
    // Create executor with cache
    let mut executor = Executor::with_cache(cache.clone());
    
    // Register nodes
    executor.register_node(Box::new(MyExpensiveNode::new()));
    
    // First execution - cache miss
    executor.execute(&mut graph1).await?;
    
    // Second execution - cache hit (if same inputs)
    executor.execute(&mut graph2).await?;
    
    // Check stats
    let stats = cache.stats();
    println!("Hit rate: {:.1}%", stats.hit_rate());
    
    Ok(())
}
```

### Custom TTL Per Execution

```rust
use std::time::Duration;

// Create cache with no default TTL
let config = CacheConfig {
    default_ttl: None,
    ..Default::default()
};
let cache = CacheManager::new(config).await?;

// Manually set TTL for specific entries
let key = CacheKey::new("my_node", &json!({"input": "data"}));
cache.put(key, result, Some(Duration::from_secs(600)))?; // 10 min TTL
```

### Manual Cache Invalidation

```rust
// Invalidate specific entry
let key = CacheKey::new("node1", &json!({"x": 10}));
cache.invalidate(&key);

// Invalidate all entries for a node
cache.invalidate_node("node1");

// Clear entire cache
cache.clear();
```

### Cache Statistics Monitoring

```rust
use std::time::Duration;
use tokio::time::interval;

// Monitor cache stats periodically
let cache = cache.clone();
tokio::spawn(async move {
    let mut interval = interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        let stats = cache.stats();
        println!("Cache Stats:");
        println!("  Entries: {}", stats.current_entries);
        println!("  Memory: {} MB", stats.current_memory_bytes / 1024 / 1024);
        println!("  Hit rate: {:.1}%", stats.hit_rate());
        println!("  Evictions: {}", stats.evictions);
    }
});
```

### Conditional Caching

```rust
// Only enable cache for production
let cache = if cfg!(debug_assertions) {
    None
} else {
    Some(CacheManager::new(CacheConfig::default()).await?)
};

let mut executor = Executor::new();
if let Some(cache) = cache {
    executor.set_cache(cache);
}
```

---

## Best Practices

### 1. Choose Appropriate TTL

```rust
// Short TTL for rapidly changing data
default_ttl: Some(Duration::from_secs(30))

// Medium TTL for semi-stable data
default_ttl: Some(Duration::from_secs(300)) // 5 minutes

// Long TTL for stable data
default_ttl: Some(Duration::from_secs(3600)) // 1 hour

// No TTL for immutable data
default_ttl: None
```

### 2. Set Reasonable Memory Limits

```rust
// Consider your available RAM
let total_ram = 16 * 1024 * 1024 * 1024; // 16GB
let cache_limit = total_ram / 10; // Use 10% for cache

let config = CacheConfig {
    max_memory_bytes: cache_limit,
    ..Default::default()
};
```

### 3. Monitor Cache Performance

```rust
// Log cache stats periodically
if let Some(cache) = executor.cache() {
    let stats = cache.stats();
    
    // Alert if hit rate is low
    if stats.hit_rate() < 50.0 {
        warn!("Low cache hit rate: {:.1}%", stats.hit_rate());
    }
    
    // Alert if eviction rate is high
    let total_ops = stats.hits + stats.misses;
    let eviction_rate = stats.evictions as f64 / total_ops as f64;
    if eviction_rate > 0.1 {
        warn!("High eviction rate: {:.1}%", eviction_rate * 100.0);
    }
}
```

### 4. Invalidate on Data Changes

```rust
// Invalidate cache when underlying data changes
async fn update_user(user_id: &str) -> Result<()> {
    // Update database
    db.update_user(user_id).await?;
    
    // Invalidate related cache entries
    cache.invalidate_node(&format!("get_user_{}", user_id));
    
    Ok(())
}
```

### 5. Use Appropriate Eviction Policy

- **LRU**: Default choice for most applications
- **FIFO**: Time-series data, logs, streaming
- **LFU**: Stable workloads with clear hot/cold data
- **None**: When you need full control

### 6. Enable Background Cleanup

```rust
let config = CacheConfig {
    enable_background_cleanup: true, // Highly recommended
    default_ttl: Some(Duration::from_secs(300)),
    ..Default::default()
};
```

Benefits:
- Automatic cleanup of expired entries
- Frees memory without manual intervention
- Runs every 60 seconds in background

---

## Performance Considerations

### Memory Estimation

The cache estimates memory usage by serializing values to JSON. This is approximate:

```rust
// Actual memory usage may be higher due to:
// - HashMap overhead
// - Metadata structures
// - Memory fragmentation

// Consider setting limit 10-20% lower than actual limit
let actual_limit = 100 * 1024 * 1024; // 100MB
let config_limit = (actual_limit as f64 * 0.85) as usize; // 85MB
```

### Concurrent Access

The cache uses `DashMap` for lock-free concurrent access:

- Read operations are highly concurrent
- Write operations have minimal contention
- Safe to share across threads/tasks

```rust
let cache = Arc::new(cache); // Clone is cheap (Arc internally)
```

### Cache Key Generation

Keys are generated by hashing input data:

```rust
// Efficient for most data types
let key = CacheKey::new("node1", &json!({"x": 10, "y": 20}));

// Large inputs may impact performance
let large_input = json!(vec![1; 10000]);
let key = CacheKey::new("node1", &large_input); // Slower due to hashing
```

### Eviction Performance

- **LRU/FIFO**: O(n) - scans all entries
- **LFU**: O(n) - scans all entries
- **Optimization**: Consider using smaller `max_entries` if evictions are frequent

---

## API Reference

### CacheManager

#### Construction

```rust
pub async fn new(config: CacheConfig) -> Result<Self>
```

Create a new cache manager.

#### Cache Operations

```rust
pub fn get(&self, key: &CacheKey) -> Option<serde_json::Value>
pub fn put(&self, key: CacheKey, value: serde_json::Value, ttl: Option<Duration>) -> Result<()>
pub fn invalidate(&self, key: &CacheKey) -> bool
pub fn invalidate_node(&self, node_id: &str) -> usize
pub fn clear(&self)
```

#### Statistics

```rust
pub fn stats(&self) -> CacheStats
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn contains_key(&self, key: &CacheKey) -> bool
```

### CacheConfig

```rust
pub struct CacheConfig {
    pub max_entries: usize,
    pub max_memory_bytes: usize,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub enable_background_cleanup: bool,
}
```

### CacheKey

```rust
pub fn new(node_id: impl Into<String>, inputs: &serde_json::Value) -> Self
```

### CacheStats

```rust
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_entries: usize,
    pub current_memory_bytes: usize,
}

pub fn hit_rate(&self) -> f64
```

### EvictionPolicy

```rust
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    FIFO, // First In First Out
    LFU,  // Least Frequently Used
    None, // No automatic eviction
}
```

---

## Examples

See the complete example:

```bash
cargo run --example cache_flow
```

This demonstrates:
- Basic caching with hit/miss tracking
- TTL expiration
- All eviction policies
- Memory limits
- Manual invalidation
- Performance comparisons

---

## Troubleshooting

### Low Hit Rate

**Symptom**: `stats.hit_rate()` < 30%

**Possible Causes:**
- Inputs vary too much (high cardinality)
- TTL too short
- Cache size too small

**Solutions:**
```rust
// Increase cache size
max_entries: 100000

// Increase memory
max_memory_bytes: 500 * 1024 * 1024

// Longer TTL
default_ttl: Some(Duration::from_secs(600))
```

### High Memory Usage

**Symptom**: Cache using too much RAM

**Solutions:**
```rust
// Reduce limits
max_memory_bytes: 50 * 1024 * 1024

// Shorter TTL
default_ttl: Some(Duration::from_secs(60))

// More aggressive eviction
eviction_policy: EvictionPolicy::LFU
```

### Frequent Evictions

**Symptom**: `stats.evictions` increasing rapidly

**Solutions:**
```rust
// Increase capacity
max_entries: 50000

// More memory
max_memory_bytes: 200 * 1024 * 1024

// Change policy
eviction_policy: EvictionPolicy::LRU
```

---

## Conclusion

The caching layer provides a powerful, flexible system for optimizing graph execution performance. By choosing appropriate configuration and monitoring cache statistics, you can achieve significant performance improvements for computation-heavy workflows.

For more information, see:
- [Examples](../examples/cache_flow.rs)
- [Main README](../README.md)
- [Integration Guide](INTEGRATIONS.md)

<div align="center">

**Happy Caching! 🚀**

