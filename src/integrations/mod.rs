//! Integration modules for databases and AI services
//!
//! This module provides integrations with various external services:
//! - Database: PostgreSQL, MySQL, Redis, MongoDB
//! - AI/LLM: OpenAI, Anthropic Claude, Ollama

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "redis-cache")]
pub mod redis;

#[cfg(feature = "mongodb-db")]
pub mod mongodb;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "claude")]
pub mod claude;

#[cfg(feature = "ollama")]
pub mod ollama;

// Re-export for convenience
#[cfg(feature = "postgres")]
pub use postgres::PostgresNode;

#[cfg(feature = "mysql")]
pub use mysql::MySqlNode;

#[cfg(feature = "redis-cache")]
pub use redis::RedisNode;

#[cfg(feature = "mongodb-db")]
pub use mongodb::MongoNode;

#[cfg(feature = "openai")]
pub use openai::OpenAINode;

#[cfg(feature = "claude")]
pub use claude::ClaudeNode;

#[cfg(feature = "ollama")]
pub use ollama::OllamaNode;
