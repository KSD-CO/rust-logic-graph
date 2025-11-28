//! Distributed Context Sharing
//!
//! This module provides distributed context management for sharing state
//! across microservices in a distributed system.
//!
//! # Features
//!
//! - **Context Serialization**: Efficient serialization for remote execution
//! - **State Sharing**: Share context between microservices
//! - **Distributed Caching**: Redis/Memcached integration
//! - **Versioning**: Context versioning with conflict resolution
//!
//! # Example
//!
//! ```rust,no_run
//! use rust_logic_graph::distributed::{DistributedContext, ContextStore};
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create distributed context with versioning
//! let mut context = DistributedContext::new("session-123");
//! context.set("user_id", json!("user-456"));
//! context.set("tenant", json!("acme-corp"));
//!
//! // Serialize for transmission
//! let serialized = context.serialize()?;
//!
//! // Deserialize on remote service
//! let remote_context = DistributedContext::deserialize(&serialized)?;
//! # Ok(())
//! # }
//! ```

pub mod context;
pub mod store;
pub mod versioning;
pub mod cache;

pub use context::{DistributedContext, ContextSnapshot, SharedContext};
pub use store::{ContextStore, InMemoryStore};

#[cfg(feature = "redis")]
pub use store::RedisStore;

pub use store::MemcachedStore;
pub use versioning::{ContextVersion, ConflictResolution, VersionedContext, ThreeWayMerge};
pub use cache::{DistributedCache, CacheStrategy, CacheWarmer};
