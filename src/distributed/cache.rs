//! Distributed Caching for Contexts
//!
//! Provides caching strategies for distributed context sharing.

use crate::distributed::context::DistributedContext;
use crate::distributed::store::ContextStore;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Caching strategy for distributed contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStrategy {
    /// Write-through: write to cache and store simultaneously
    WriteThrough,
    
    /// Write-behind: write to cache immediately, async write to store
    WriteBehind,
    
    /// Read-through: read from cache, fallback to store
    ReadThrough,
    
    /// Cache-aside: application manages cache and store
    CacheAside,
}

/// Distributed cache for contexts
pub struct DistributedCache {
    /// Primary store (e.g., Redis)
    store: Arc<dyn ContextStore>,
    
    /// Caching strategy
    strategy: CacheStrategy,
    
    /// Default TTL for cached contexts
    default_ttl: Option<Duration>,
}

impl DistributedCache {
    /// Create a new distributed cache
    pub fn new(store: Arc<dyn ContextStore>) -> Self {
        Self {
            store,
            strategy: CacheStrategy::WriteThrough,
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour default
        }
    }
    
    /// Create with custom strategy and TTL
    pub fn with_config(
        store: Arc<dyn ContextStore>,
        strategy: CacheStrategy,
        default_ttl: Option<Duration>,
    ) -> Self {
        Self {
            store,
            strategy,
            default_ttl,
        }
    }
    
    /// Get a context from cache
    pub async fn get(&self, session_id: &str) -> Result<Option<DistributedContext>> {
        match self.strategy {
            CacheStrategy::ReadThrough | CacheStrategy::WriteThrough => {
                self.store.load(session_id).await
            }
            CacheStrategy::CacheAside | CacheStrategy::WriteBehind => {
                self.store.load(session_id).await
            }
        }
    }
    
    /// Put a context into cache
    pub async fn put(&self, context: &DistributedContext) -> Result<()> {
        self.put_with_ttl(context, self.default_ttl).await
    }
    
    /// Put a context with custom TTL
    pub async fn put_with_ttl(
        &self,
        context: &DistributedContext,
        ttl: Option<Duration>,
    ) -> Result<()> {
        match self.strategy {
            CacheStrategy::WriteThrough => {
                // Write immediately to store
                self.store.save(context, ttl).await
            }
            CacheStrategy::WriteBehind => {
                // Write to cache immediately, async write to store
                let store = self.store.clone();
                let ctx = context.clone();
                tokio::spawn(async move {
                    let _ = store.save(&ctx, ttl).await;
                });
                Ok(())
            }
            CacheStrategy::ReadThrough | CacheStrategy::CacheAside => {
                self.store.save(context, ttl).await
            }
        }
    }
    
    /// Delete a context from cache
    pub async fn delete(&self, session_id: &str) -> Result<()> {
        self.store.delete(session_id).await
    }
    
    /// Check if context exists in cache
    pub async fn exists(&self, session_id: &str) -> Result<bool> {
        self.store.exists(session_id).await
    }
    
    /// Invalidate (delete) a context
    pub async fn invalidate(&self, session_id: &str) -> Result<()> {
        self.delete(session_id).await
    }
    
    /// Batch get multiple contexts
    pub async fn get_many(&self, session_ids: &[String]) -> Result<Vec<Option<DistributedContext>>> {
        let mut results = Vec::new();
        
        for session_id in session_ids {
            let context = self.get(session_id).await?;
            results.push(context);
        }
        
        Ok(results)
    }
    
    /// Batch put multiple contexts
    pub async fn put_many(&self, contexts: &[DistributedContext]) -> Result<()> {
        for context in contexts {
            self.put(context).await?;
        }
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            total_contexts: self.store.list_sessions().await.unwrap_or_default().len(),
            strategy: self.strategy,
            default_ttl: self.default_ttl,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_contexts: usize,
    pub strategy: CacheStrategy,
    pub default_ttl: Option<Duration>,
}

/// Cache warming utility
pub struct CacheWarmer {
    cache: Arc<DistributedCache>,
}

impl CacheWarmer {
    /// Create a new cache warmer
    pub fn new(cache: Arc<DistributedCache>) -> Self {
        Self { cache }
    }
    
    /// Warm cache with contexts
    pub async fn warm(&self, contexts: Vec<DistributedContext>) -> Result<()> {
        self.cache.put_many(&contexts).await
    }
    
    /// Warm cache with session IDs (loads from source)
    pub async fn warm_from_source(
        &self,
        session_ids: Vec<String>,
        source: Arc<dyn ContextStore>,
    ) -> Result<()> {
        for session_id in session_ids {
            if let Some(context) = source.load(&session_id).await? {
                self.cache.put(&context).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::store::InMemoryStore;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_cache_put_and_get() {
        let store = Arc::new(InMemoryStore::new());
        let cache = DistributedCache::new(store);
        
        let mut ctx = DistributedContext::new("test-session");
        ctx.set("key1", json!("value1"));
        
        cache.put(&ctx).await.unwrap();
        
        let loaded = cache.get("test-session").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().get("key1"), Some(&json!("value1")));
    }
    
    #[tokio::test]
    async fn test_cache_delete() {
        let store = Arc::new(InMemoryStore::new());
        let cache = DistributedCache::new(store);
        
        let ctx = DistributedContext::new("test-session");
        cache.put(&ctx).await.unwrap();
        
        assert!(cache.exists("test-session").await.unwrap());
        
        cache.delete("test-session").await.unwrap();
        
        assert!(!cache.exists("test-session").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let store = Arc::new(InMemoryStore::new());
        let cache = DistributedCache::new(store);
        
        let ctx1 = DistributedContext::new("session-1");
        let ctx2 = DistributedContext::new("session-2");
        
        cache.put_many(&[ctx1, ctx2]).await.unwrap();
        
        let results = cache.get_many(&[
            "session-1".to_string(),
            "session-2".to_string(),
        ]).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
    }
    
    #[tokio::test]
    async fn test_cache_warmer() {
        let store = Arc::new(InMemoryStore::new());
        let cache = Arc::new(DistributedCache::new(store));
        let warmer = CacheWarmer::new(cache.clone());
        
        let ctx1 = DistributedContext::new("session-1");
        let ctx2 = DistributedContext::new("session-2");
        
        warmer.warm(vec![ctx1, ctx2]).await.unwrap();
        
        assert!(cache.exists("session-1").await.unwrap());
        assert!(cache.exists("session-2").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let store = Arc::new(InMemoryStore::new());
        let cache = DistributedCache::with_config(
            store,
            CacheStrategy::WriteThrough,
            Some(Duration::from_secs(300)),
        );
        
        let ctx1 = DistributedContext::new("session-1");
        let ctx2 = DistributedContext::new("session-2");
        
        cache.put(&ctx1).await.unwrap();
        cache.put(&ctx2).await.unwrap();
        
        let stats = cache.stats().await;
        assert_eq!(stats.total_contexts, 2);
        assert_eq!(stats.strategy, CacheStrategy::WriteThrough);
    }
}
