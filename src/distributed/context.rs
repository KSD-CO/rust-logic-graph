//! Distributed Context with Serialization
//!
//! Provides context management with efficient serialization for remote execution.

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A distributed context that can be serialized and shared across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedContext {
    /// Unique session identifier
    pub session_id: String,

    /// Context data
    pub data: HashMap<String, Value>,

    /// Metadata for tracking
    pub metadata: ContextMetadata,
}

/// Metadata for distributed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Creation timestamp (Unix timestamp in milliseconds)
    pub created_at: u64,

    /// Last updated timestamp
    pub updated_at: u64,

    /// Version number for conflict resolution
    pub version: u64,

    /// Service that last modified this context
    pub modified_by: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

impl DistributedContext {
    /// Create a new distributed context
    ///
    /// # Example
    ///
    /// ```
    /// use rust_logic_graph::distributed::DistributedContext;
    ///
    /// let context = DistributedContext::new("session-123");
    /// assert_eq!(context.session_id, "session-123");
    /// ```
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            session_id: session_id.into(),
            data: HashMap::new(),
            metadata: ContextMetadata {
                created_at: now,
                updated_at: now,
                version: 1,
                modified_by: None,
                tags: Vec::new(),
            },
        }
    }

    /// Set a value in the context
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
        self.bump_version();
    }

    /// Get a value from the context
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Remove a value from the context
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        let result = self.data.remove(key);
        if result.is_some() {
            self.bump_version();
        }
        result
    }

    /// Serialize context to bytes for transmission
    ///
    /// Uses MessagePack for efficient binary serialization
    pub fn serialize(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).context("Failed to serialize distributed context")
    }

    /// Deserialize context from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(bytes).context("Failed to deserialize distributed context")
    }

    /// Serialize to JSON (for debugging/human-readable)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize context to JSON")
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialize context from JSON")
    }

    /// Create a snapshot of current context state
    pub fn snapshot(&self) -> ContextSnapshot {
        ContextSnapshot {
            session_id: self.session_id.clone(),
            data: self.data.clone(),
            version: self.metadata.version,
            timestamp: self.metadata.updated_at,
        }
    }

    /// Merge another context into this one
    ///
    /// Performs a simple merge where newer values win
    pub fn merge(&mut self, other: &DistributedContext) {
        for (key, value) in &other.data {
            self.data.insert(key.clone(), value.clone());
        }
        self.bump_version();
    }

    /// Increment version and update timestamp
    fn bump_version(&mut self) {
        self.metadata.version += 1;
        self.metadata.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    /// Add a tag to the context
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.metadata.tags.contains(&tag) {
            self.metadata.tags.push(tag);
        }
    }

    /// Set the service that modified this context
    pub fn set_modified_by(&mut self, service: impl Into<String>) {
        self.metadata.modified_by = Some(service.into());
        self.bump_version();
    }
}

/// A lightweight snapshot of context state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub session_id: String,
    pub data: HashMap<String, Value>,
    pub version: u64,
    pub timestamp: u64,
}

/// Thread-safe wrapper for distributed context
#[derive(Debug, Clone)]
pub struct SharedContext {
    inner: Arc<RwLock<DistributedContext>>,
}

impl SharedContext {
    /// Create a new shared context
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DistributedContext::new(session_id))),
        }
    }

    /// Get a value from the context
    pub async fn get(&self, key: &str) -> Option<Value> {
        let ctx = self.inner.read().await;
        ctx.get(key).cloned()
    }

    /// Set a value in the context
    pub async fn set(&self, key: impl Into<String>, value: Value) {
        let mut ctx = self.inner.write().await;
        ctx.set(key, value);
    }

    /// Serialize the context
    pub async fn serialize(&self) -> Result<Vec<u8>> {
        let ctx = self.inner.read().await;
        ctx.serialize()
    }

    /// Get current version
    pub async fn version(&self) -> u64 {
        let ctx = self.inner.read().await;
        ctx.metadata.version
    }

    /// Create a snapshot
    pub async fn snapshot(&self) -> ContextSnapshot {
        let ctx = self.inner.read().await;
        ctx.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_creation() {
        let ctx = DistributedContext::new("test-session");
        assert_eq!(ctx.session_id, "test-session");
        assert_eq!(ctx.metadata.version, 1);
    }

    #[test]
    fn test_set_and_get() {
        let mut ctx = DistributedContext::new("test");
        ctx.set("key1", json!("value1"));

        assert_eq!(ctx.get("key1"), Some(&json!("value1")));
        assert_eq!(ctx.metadata.version, 2);
    }

    #[test]
    fn test_serialization() {
        let mut ctx = DistributedContext::new("test");
        ctx.set("user_id", json!("user-123"));
        ctx.set("count", json!(42));

        let bytes = ctx.serialize().unwrap();
        let deserialized = DistributedContext::deserialize(&bytes).unwrap();

        assert_eq!(deserialized.session_id, "test");
        assert_eq!(deserialized.get("user_id"), Some(&json!("user-123")));
        assert_eq!(deserialized.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_json_serialization() {
        let mut ctx = DistributedContext::new("test");
        ctx.set("name", json!("Alice"));

        let json_str = ctx.to_json().unwrap();
        let deserialized = DistributedContext::from_json(&json_str).unwrap();

        assert_eq!(deserialized.session_id, "test");
        assert_eq!(deserialized.get("name"), Some(&json!("Alice")));
    }

    #[test]
    fn test_merge() {
        let mut ctx1 = DistributedContext::new("test");
        ctx1.set("key1", json!("value1"));

        let mut ctx2 = DistributedContext::new("test");
        ctx2.set("key2", json!("value2"));

        ctx1.merge(&ctx2);

        assert_eq!(ctx1.get("key1"), Some(&json!("value1")));
        assert_eq!(ctx1.get("key2"), Some(&json!("value2")));
    }

    #[tokio::test]
    async fn test_shared_context() {
        let ctx = SharedContext::new("test");

        ctx.set("key1", json!("value1")).await;
        let value = ctx.get("key1").await;

        assert_eq!(value, Some(json!("value1")));
        assert_eq!(ctx.version().await, 2);
    }
}
