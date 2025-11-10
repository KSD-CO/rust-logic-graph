//! Cache eviction policies

use serde::{Deserialize, Serialize};

/// Strategy for evicting cache entries when limits are reached
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used - evict entries that haven't been accessed recently
    LRU,
    /// First In First Out - evict oldest entries first
    FIFO,
    /// Least Frequently Used - evict entries with lowest access count
    LFU,
    /// No automatic eviction - only manual invalidation
    None,
}

impl Default for EvictionPolicy {
    fn default() -> Self {
        Self::LRU
    }
}

impl EvictionPolicy {
    /// Get a description of the policy
    pub fn description(&self) -> &str {
        match self {
            Self::LRU => "Least Recently Used - evicts entries not accessed recently",
            Self::FIFO => "First In First Out - evicts oldest entries first",
            Self::LFU => "Least Frequently Used - evicts entries with lowest access count",
            Self::None => "No automatic eviction",
        }
    }
}
