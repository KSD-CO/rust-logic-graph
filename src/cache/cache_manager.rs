//! Cache manager implementation

use super::{CacheEntry, CacheKey, EvictionPolicy};
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Configuration for the cache manager
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            eviction_policy: EvictionPolicy::LRU,
            enable_background_cleanup: true,
        }
    }
}

/// Statistics about cache usage
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_entries: usize,
    pub current_memory_bytes: usize,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Thread-safe cache manager for node execution results
pub struct CacheManager {
    config: CacheConfig,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    evictions: Arc<AtomicU64>,
    current_memory: Arc<AtomicUsize>,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration
    pub async fn new(config: CacheConfig) -> Result<Self> {
        info!("Initializing cache manager with config: {:?}", config);

        let manager = Self {
            config: config.clone(),
            cache: Arc::new(DashMap::new()),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            current_memory: Arc::new(AtomicUsize::new(0)),
        };

        // Start background cleanup task if enabled
        if config.enable_background_cleanup {
            manager.start_cleanup_task();
        }

        Ok(manager)
    }

    /// Get a value from the cache
    pub fn get(&self, key: &CacheKey) -> Option<serde_json::Value> {
        match self.cache.get_mut(key) {
            Some(mut entry) => {
                // Check if expired
                if entry.is_expired() {
                    drop(entry); // Release the lock before removing
                    self.invalidate(key);
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    debug!("Cache miss (expired): {:?}", key);
                    return None;
                }

                // Update access metadata
                entry.mark_accessed();
                let value = entry.value.clone();
                
                self.hits.fetch_add(1, Ordering::Relaxed);
                debug!("Cache hit: {:?}", key);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                debug!("Cache miss (not found): {:?}", key);
                None
            }
        }
    }

    /// Put a value into the cache
    pub fn put(
        &self,
        key: CacheKey,
        value: serde_json::Value,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let ttl = ttl.or(self.config.default_ttl);
        let entry = CacheEntry::new(key.clone(), value, ttl);
        let entry_size = entry.size_bytes;

        // Check if we need to evict entries
        self.ensure_capacity(entry_size)?;

        // Insert the new entry
        self.current_memory.fetch_add(entry_size, Ordering::Relaxed);
        self.cache.insert(key.clone(), entry);

        debug!("Cached entry: {:?} ({} bytes)", key, entry_size);
        Ok(())
    }

    /// Invalidate (remove) a specific cache entry
    pub fn invalidate(&self, key: &CacheKey) -> bool {
        if let Some((_, entry)) = self.cache.remove(key) {
            self.current_memory.fetch_sub(entry.size_bytes, Ordering::Relaxed);
            debug!("Invalidated cache entry: {:?}", key);
            true
        } else {
            false
        }
    }

    /// Invalidate all entries for a specific node
    pub fn invalidate_node(&self, node_id: &str) -> usize {
        let keys_to_remove: Vec<CacheKey> = self
            .cache
            .iter()
            .filter(|entry| entry.key().node_id == node_id)
            .map(|entry| entry.key().clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.invalidate(&key);
        }

        info!("Invalidated {} entries for node '{}'", count, node_id);
        count
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        let count = self.cache.len();
        self.cache.clear();
        self.current_memory.store(0, Ordering::Relaxed);
        info!("Cleared cache ({} entries)", count);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            current_entries: self.cache.len(),
            current_memory_bytes: self.current_memory.load(Ordering::Relaxed),
        }
    }

    /// Ensure there's capacity for a new entry
    fn ensure_capacity(&self, new_entry_size: usize) -> Result<()> {
        // Check entry count limit
        while self.cache.len() >= self.config.max_entries {
            self.evict_one()?;
        }

        // Check memory limit
        while self.current_memory.load(Ordering::Relaxed) + new_entry_size
            > self.config.max_memory_bytes
        {
            self.evict_one()?;
        }

        Ok(())
    }

    /// Evict one entry based on the configured policy
    fn evict_one(&self) -> Result<()> {
        let key_to_evict = match self.config.eviction_policy {
            EvictionPolicy::LRU => self.find_lru_key(),
            EvictionPolicy::FIFO => self.find_fifo_key(),
            EvictionPolicy::LFU => self.find_lfu_key(),
            EvictionPolicy::None => {
                warn!("Eviction needed but policy is None");
                return Ok(());
            }
        };

        if let Some(key) = key_to_evict {
            self.invalidate(&key);
            self.evictions.fetch_add(1, Ordering::Relaxed);
            debug!("Evicted entry: {:?}", key);
        }

        Ok(())
    }

    /// Find the least recently used entry
    fn find_lru_key(&self) -> Option<CacheKey> {
        self.cache
            .iter()
            .min_by_key(|entry| entry.last_accessed)
            .map(|entry| entry.key().clone())
    }

    /// Find the oldest entry (FIFO)
    fn find_fifo_key(&self) -> Option<CacheKey> {
        self.cache
            .iter()
            .min_by_key(|entry| entry.created_at)
            .map(|entry| entry.key().clone())
    }

    /// Find the least frequently used entry
    fn find_lfu_key(&self) -> Option<CacheKey> {
        self.cache
            .iter()
            .min_by_key(|entry| entry.access_count)
            .map(|entry| entry.key().clone())
    }

    /// Start background task to clean up expired entries
    fn start_cleanup_task(&self) {
        let cache = Arc::clone(&self.cache);
        let current_memory = Arc::clone(&self.current_memory);

        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(60));

            loop {
                cleanup_interval.tick().await;

                let expired_keys: Vec<CacheKey> = cache
                    .iter()
                    .filter(|entry| entry.is_expired())
                    .map(|entry| entry.key().clone())
                    .collect();

                if !expired_keys.is_empty() {
                    for key in &expired_keys {
                        if let Some((_, entry)) = cache.remove(key) {
                            current_memory.fetch_sub(entry.size_bytes, Ordering::Relaxed);
                        }
                    }
                    info!("Cleaned up {} expired cache entries", expired_keys.len());
                }
            }
        });

        info!("Started background cache cleanup task");
    }

    /// Get all cache entries (useful for testing/debugging)
    pub fn entries(&self) -> Vec<CacheEntry> {
        self.cache
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Check if cache contains a key
    pub fn contains_key(&self, key: &CacheKey) -> bool {
        self.cache.contains_key(key)
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: Arc::clone(&self.cache),
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
            evictions: Arc::clone(&self.evictions),
            current_memory: Arc::clone(&self.current_memory),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            max_entries: 100,
            max_memory_bytes: 1024 * 1024,
            default_ttl: None,
            eviction_policy: EvictionPolicy::LRU,
            enable_background_cleanup: false,
        };

        let cache = CacheManager::new(config).await.unwrap();

        let key = CacheKey::new("node1", &json!({"x": 10}));
        let value = json!({"result": 42});

        // Should be a miss initially
        assert!(cache.get(&key).is_none());

        // Put value
        cache.put(key.clone(), value.clone(), None).unwrap();

        // Should be a hit now
        assert_eq!(cache.get(&key), Some(value));

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            max_entries: 100,
            max_memory_bytes: 1024 * 1024,
            default_ttl: Some(Duration::from_millis(50)),
            eviction_policy: EvictionPolicy::LRU,
            enable_background_cleanup: false,
        };

        let cache = CacheManager::new(config).await.unwrap();

        let key = CacheKey::new("node1", &json!({"x": 10}));
        let value = json!({"result": 42});

        cache.put(key.clone(), value.clone(), None).unwrap();

        // Should exist initially
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be expired now
        assert!(cache.get(&key).is_none());
    }

    #[tokio::test]
    async fn test_cache_max_entries() {
        let config = CacheConfig {
            max_entries: 3,
            max_memory_bytes: 1024 * 1024,
            default_ttl: None,
            eviction_policy: EvictionPolicy::FIFO,
            enable_background_cleanup: false,
        };

        let cache = CacheManager::new(config).await.unwrap();

        // Add 4 entries (should evict the first one)
        for i in 0..4 {
            let key = CacheKey::new(format!("node{}", i), &json!({"x": i}));
            let value = json!({"result": i});
            cache.put(key, value, None).unwrap();
        }

        // Should have exactly 3 entries
        assert_eq!(cache.len(), 3);

        // First entry should be evicted
        let first_key = CacheKey::new("node0", &json!({"x": 0}));
        assert!(!cache.contains_key(&first_key));
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let config = CacheConfig::default();
        let cache = CacheManager::new(config).await.unwrap();

        let key1 = CacheKey::new("node1", &json!({"x": 10}));
        let key2 = CacheKey::new("node1", &json!({"x": 20}));

        cache.put(key1.clone(), json!({"result": 1}), None).unwrap();
        cache.put(key2.clone(), json!({"result": 2}), None).unwrap();

        assert_eq!(cache.len(), 2);

        // Invalidate by node
        let count = cache.invalidate_node("node1");
        assert_eq!(count, 2);
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = CacheManager::new(config).await.unwrap();

        let key = CacheKey::new("node1", &json!({"x": 10}));

        // Miss
        cache.get(&key);

        // Put and hit
        cache.put(key.clone(), json!({"result": 42}), None).unwrap();
        cache.get(&key);
        cache.get(&key);

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.current_entries, 1);
        assert!(stats.hit_rate() > 0.0);
    }
}
