//! Redis integration for caching and pub/sub
//!
//! Provides async cache operations using redis-rs

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use serde_json::Value;
use tracing::{info, error};

/// Redis cache node
#[derive(Debug, Clone)]
pub struct RedisNode {
    pub id: String,
    pub operation: RedisOperation,
    pub key: String,
    pub value: Option<String>,
    pub ttl: Option<u64>,
    pub client: Option<Client>,
}

#[derive(Debug, Clone)]
pub enum RedisOperation {
    Get,
    Set,
    Delete,
    Exists,
}

impl RedisNode {
    /// Create a new Redis node for GET operation
    pub fn get(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            operation: RedisOperation::Get,
            key: key.into(),
            value: None,
            ttl: None,
            client: None,
        }
    }

    /// Create a new Redis node for SET operation
    pub fn set(id: impl Into<String>, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            operation: RedisOperation::Set,
            key: key.into(),
            value: Some(value.into()),
            ttl: None,
            client: None,
        }
    }

    /// Set TTL for cache entries (in seconds)
    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Initialize with Redis client
    pub fn with_client(mut self, redis_url: &str) -> Result<Self, RuleError> {
        let client = Client::open(redis_url)
            .map_err(|e| RuleError::Eval(format!("Failed to connect to Redis: {}", e)))?;
        self.client = Some(client);
        Ok(self)
    }

    /// Execute Redis operation
    async fn execute_operation(&self, ctx: &Context) -> Result<Value, RuleError> {
        let client = self.client.as_ref()
            .ok_or_else(|| RuleError::Eval("Redis client not initialized".to_string()))?;

        let mut conn = client.get_async_connection()
            .await
            .map_err(|e| RuleError::Eval(format!("Failed to get Redis connection: {}", e)))?;

        let key = self.process_key(&self.key, ctx);

        match &self.operation {
            RedisOperation::Get => {
                info!("RedisNode[{}]: GET key: {}", self.id, key);
                let result: Option<String> = conn.get(&key)
                    .await
                    .map_err(|e| RuleError::Eval(format!("GET failed: {}", e)))?;

                match result {
                    Some(val) => {
                        // Try to parse as JSON, otherwise return as string
                        if let Ok(json) = serde_json::from_str::<Value>(&val) {
                            Ok(json)
                        } else {
                            Ok(Value::String(val))
                        }
                    }
                    None => Ok(Value::Null),
                }
            }

            RedisOperation::Set => {
                let value = self.value.as_ref()
                    .ok_or_else(|| RuleError::Eval("SET operation requires value".to_string()))?;
                let processed_value = self.process_value(value, ctx);

                info!("RedisNode[{}]: SET key: {} = {}", self.id, key, processed_value);

                if let Some(ttl) = self.ttl {
                    let _: () = conn.set_ex(&key, &processed_value, ttl)
                        .await
                        .map_err(|e| RuleError::Eval(format!("SET with TTL failed: {}", e)))?;
                } else {
                    let _: () = conn.set(&key, &processed_value)
                        .await
                        .map_err(|e| RuleError::Eval(format!("SET failed: {}", e)))?;
                }

                Ok(Value::Bool(true))
            }

            RedisOperation::Delete => {
                info!("RedisNode[{}]: DELETE key: {}", self.id, key);
                let deleted: i32 = conn.del(&key)
                    .await
                    .map_err(|e| RuleError::Eval(format!("DELETE failed: {}", e)))?;

                Ok(Value::Bool(deleted > 0))
            }

            RedisOperation::Exists => {
                info!("RedisNode[{}]: EXISTS key: {}", self.id, key);
                let exists: bool = conn.exists(&key)
                    .await
                    .map_err(|e| RuleError::Eval(format!("EXISTS failed: {}", e)))?;

                Ok(Value::Bool(exists))
            }
        }
    }

    fn process_key(&self, key: &str, ctx: &Context) -> String {
        let mut processed = key.to_string();

        for (k, v) in &ctx.data {
            let placeholder = format!("{{{{{}}}}}", k);
            if processed.contains(&placeholder) {
                let replacement = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => continue,
                };
                processed = processed.replace(&placeholder, &replacement);
            }
        }

        processed
    }

    fn process_value(&self, value: &str, ctx: &Context) -> String {
        let mut processed = value.to_string();

        for (k, v) in &ctx.data {
            let placeholder = format!("{{{{{}}}}}", k);
            if processed.contains(&placeholder) {
                let replacement = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                };
                processed = processed.replace(&placeholder, &replacement);
            }
        }

        processed
    }
}

#[async_trait]
impl Node for RedisNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("RedisNode[{}]: Starting {:?} operation", self.id, self.operation);

        match self.execute_operation(ctx).await {
            Ok(result) => {
                info!("RedisNode[{}]: Operation successful", self.id);
                ctx.data.insert(format!("{}_result", self.id), result.clone());
                Ok(result)
            }
            Err(e) => {
                error!("RedisNode[{}]: Operation failed: {}", self.id, e);
                Err(e)
            }
        }
    }
}
