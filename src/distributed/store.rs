//! Context Store Implementations
//!
//! Provides storage backends for distributed contexts including Redis and Memcached.

use crate::distributed::context::DistributedContext;
use anyhow::{Result, Context as AnyhowContext};
use async_trait::async_trait;
use std::time::Duration;

/// Trait for context storage backends
#[async_trait]
pub trait ContextStore: Send + Sync {
    /// Save a context to the store
    async fn save(&self, context: &DistributedContext, ttl: Option<Duration>) -> Result<()>;
    
    /// Load a context from the store
    async fn load(&self, session_id: &str) -> Result<Option<DistributedContext>>;
    
    /// Delete a context from the store
    async fn delete(&self, session_id: &str) -> Result<()>;
    
    /// Check if a context exists
    async fn exists(&self, session_id: &str) -> Result<bool>;
    
    /// List all session IDs (for debugging)
    async fn list_sessions(&self) -> Result<Vec<String>>;
}

/// Redis-based context store
#[cfg(feature = "redis")]
pub struct RedisStore {
    client: redis::Client,
    prefix: String,
}

#[cfg(feature = "redis")]
impl RedisStore {
    /// Create a new Redis store
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rust_logic_graph::distributed::RedisStore;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let store = RedisStore::new("redis://localhost:6379", "ctx").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(url: &str, prefix: impl Into<String>) -> Result<Self> {
        let client = redis::Client::open(url)
            .context("Failed to create Redis client")?;
        
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await
            .context("Failed to connect to Redis")?;
        
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .context("Redis connection test failed")?;
        
        Ok(Self {
            client,
            prefix: prefix.into(),
        })
    }
    
    fn make_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.prefix, session_id)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl ContextStore for RedisStore {
    async fn save(&self, context: &DistributedContext, ttl: Option<Duration>) -> Result<()> {
        use redis::AsyncCommands;
        
        let key = self.make_key(&context.session_id);
        let data = context.serialize()?;
        
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        if let Some(ttl) = ttl {
            conn.set_ex(&key, data, ttl.as_secs() as usize).await
                .context("Failed to save context to Redis with TTL")?;
        } else {
            conn.set(&key, data).await
                .context("Failed to save context to Redis")?;
        }
        
        Ok(())
    }
    
    async fn load(&self, session_id: &str) -> Result<Option<DistributedContext>> {
        use redis::AsyncCommands;
        
        let key = self.make_key(session_id);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let data: Option<Vec<u8>> = conn.get(&key).await
            .context("Failed to load context from Redis")?;
        
        match data {
            Some(bytes) => {
                let context = DistributedContext::deserialize(&bytes)?;
                Ok(Some(context))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&self, session_id: &str) -> Result<()> {
        use redis::AsyncCommands;
        
        let key = self.make_key(session_id);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        conn.del(&key).await
            .context("Failed to delete context from Redis")?;
        
        Ok(())
    }
    
    async fn exists(&self, session_id: &str) -> Result<bool> {
        use redis::AsyncCommands;
        
        let key = self.make_key(session_id);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let exists: bool = conn.exists(&key).await
            .context("Failed to check existence in Redis")?;
        
        Ok(exists)
    }
    
    async fn list_sessions(&self) -> Result<Vec<String>> {
        use redis::AsyncCommands;
        
        let pattern = format!("{}:*", self.prefix);
        let mut conn = self.client.get_multiplexed_async_connection().await
            .context("Failed to get Redis connection")?;
        
        let keys: Vec<String> = conn.keys(&pattern).await
            .context("Failed to list keys from Redis")?;
        
        // Remove prefix from keys
        let sessions = keys.into_iter()
            .filter_map(|k| k.strip_prefix(&format!("{}:", self.prefix)).map(|s| s.to_string()))
            .collect();
        
        Ok(sessions)
    }
}

/// Memcached-based context store
pub struct MemcachedStore {
    // Placeholder for memcached client
    servers: Vec<String>,
    prefix: String,
}

impl MemcachedStore {
    /// Create a new Memcached store
    pub fn new(servers: Vec<String>, prefix: impl Into<String>) -> Self {
        Self {
            servers,
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl ContextStore for MemcachedStore {
    async fn save(&self, _context: &DistributedContext, _ttl: Option<Duration>) -> Result<()> {
        // TODO: Implement memcached support
        anyhow::bail!("Memcached store not yet implemented")
    }
    
    async fn load(&self, _session_id: &str) -> Result<Option<DistributedContext>> {
        anyhow::bail!("Memcached store not yet implemented")
    }
    
    async fn delete(&self, _session_id: &str) -> Result<()> {
        anyhow::bail!("Memcached store not yet implemented")
    }
    
    async fn exists(&self, _session_id: &str) -> Result<bool> {
        anyhow::bail!("Memcached store not yet implemented")
    }
    
    async fn list_sessions(&self) -> Result<Vec<String>> {
        anyhow::bail!("Memcached store not yet implemented")
    }
}

/// In-memory store for testing
pub struct InMemoryStore {
    data: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>>,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            data: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextStore for InMemoryStore {
    async fn save(&self, context: &DistributedContext, _ttl: Option<Duration>) -> Result<()> {
        let data = context.serialize()?;
        let mut store = self.data.write().await;
        store.insert(context.session_id.clone(), data);
        Ok(())
    }
    
    async fn load(&self, session_id: &str) -> Result<Option<DistributedContext>> {
        let store = self.data.read().await;
        match store.get(session_id) {
            Some(bytes) => {
                let context = DistributedContext::deserialize(bytes)?;
                Ok(Some(context))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&self, session_id: &str) -> Result<()> {
        let mut store = self.data.write().await;
        store.remove(session_id);
        Ok(())
    }
    
    async fn exists(&self, session_id: &str) -> Result<bool> {
        let store = self.data.read().await;
        Ok(store.contains_key(session_id))
    }
    
    async fn list_sessions(&self) -> Result<Vec<String>> {
        let store = self.data.read().await;
        Ok(store.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryStore::new();
        let mut ctx = DistributedContext::new("test-session");
        ctx.set("key1", json!("value1"));
        
        // Save
        store.save(&ctx, None).await.unwrap();
        
        // Load
        let loaded = store.load("test-session").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.get("key1"), Some(&json!("value1")));
        
        // Exists
        assert!(store.exists("test-session").await.unwrap());
        
        // Delete
        store.delete("test-session").await.unwrap();
        assert!(!store.exists("test-session").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_list_sessions() {
        let store = InMemoryStore::new();
        
        let ctx1 = DistributedContext::new("session-1");
        let ctx2 = DistributedContext::new("session-2");
        
        store.save(&ctx1, None).await.unwrap();
        store.save(&ctx2, None).await.unwrap();
        
        let sessions = store.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-1".to_string()));
        assert!(sessions.contains(&"session-2".to_string()));
    }
}
