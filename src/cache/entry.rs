//! Cache entry and key types

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime};

/// Key for cache entries, combining node ID and input hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub node_id: String,
    pub input_hash: u64,
}

impl CacheKey {
    /// Create a new cache key from node ID and inputs
    pub fn new(node_id: impl Into<String>, inputs: &serde_json::Value) -> Self {
        let node_id = node_id.into();
        let input_hash = Self::hash_inputs(inputs);
        Self {
            node_id,
            input_hash,
        }
    }

    /// Hash the inputs to create a stable key
    fn hash_inputs(inputs: &serde_json::Value) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Convert to canonical JSON string for consistent hashing
        if let Ok(json_str) = serde_json::to_string(inputs) {
            json_str.hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// A cached entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: CacheKey,
    pub value: serde_json::Value,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub ttl: Option<Duration>,
    pub size_bytes: usize,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(key: CacheKey, value: serde_json::Value, ttl: Option<Duration>) -> Self {
        let size_bytes = Self::estimate_size(&value);
        let now = SystemTime::now();

        Self {
            key,
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
            size_bytes,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            if let Ok(elapsed) = self.created_at.elapsed() {
                return elapsed > ttl;
            }
        }
        false
    }

    /// Update access metadata
    pub fn mark_accessed(&mut self) {
        self.last_accessed = SystemTime::now();
        self.access_count += 1;
    }

    /// Estimate the memory size of the cached value
    fn estimate_size(value: &serde_json::Value) -> usize {
        // Rough estimation based on JSON serialization
        match serde_json::to_string(value) {
            Ok(s) => s.len(),
            Err(_) => 0,
        }
    }

    /// Get age of the entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed().unwrap_or(Duration::ZERO)
    }

    /// Get time since last access
    pub fn idle_time(&self) -> Duration {
        self.last_accessed.elapsed().unwrap_or(Duration::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_key_creation() {
        let inputs = json!({"x": 10, "y": 20});
        let key1 = CacheKey::new("node1", &inputs);
        let key2 = CacheKey::new("node1", &inputs);

        assert_eq!(key1, key2);
        assert_eq!(key1.node_id, "node1");
    }

    #[test]
    fn test_cache_key_different_inputs() {
        let inputs1 = json!({"x": 10});
        let inputs2 = json!({"x": 20});

        let key1 = CacheKey::new("node1", &inputs1);
        let key2 = CacheKey::new("node1", &inputs2);

        assert_ne!(key1.input_hash, key2.input_hash);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let key = CacheKey::new("node1", &json!({}));
        let mut entry = CacheEntry::new(key, json!({"result": 42}), Some(Duration::from_millis(1)));

        assert!(!entry.is_expired());

        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_no_expiration() {
        let key = CacheKey::new("node1", &json!({}));
        let entry = CacheEntry::new(key, json!({"result": 42}), None);

        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_access_tracking() {
        let key = CacheKey::new("node1", &json!({}));
        let mut entry = CacheEntry::new(key, json!({"result": 42}), None);

        assert_eq!(entry.access_count, 0);

        entry.mark_accessed();
        assert_eq!(entry.access_count, 1);

        entry.mark_accessed();
        assert_eq!(entry.access_count, 2);
    }
}
