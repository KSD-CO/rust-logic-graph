//! Stream transformation operators

use crate::core::Context;
use crate::rule::{RuleError, RuleResult};
use crate::streaming::StreamProcessor;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Stream operator trait
#[async_trait]
pub trait StreamOperator: StreamProcessor {
    /// Get operator name
    fn name(&self) -> &str;
}

/// Map operator - transforms each item
pub struct MapOperator<F>
where
    F: Fn(Value) -> Value + Send + Sync,
{
    pub name: String,
    pub func: Arc<F>,
}

impl<F> MapOperator<F>
where
    F: Fn(Value) -> Value + Send + Sync,
{
    pub fn new(name: impl Into<String>, func: F) -> Self {
        Self {
            name: name.into(),
            func: Arc::new(func),
        }
    }
}

#[async_trait]
impl<F> StreamProcessor for MapOperator<F>
where
    F: Fn(Value) -> Value + Send + Sync,
{
    async fn process_item(&self, item: Value, _ctx: &Context) -> RuleResult {
        Ok((self.func)(item))
    }
}

#[async_trait]
impl<F> StreamOperator for MapOperator<F>
where
    F: Fn(Value) -> Value + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }
}

/// Filter operator - filters items based on predicate
pub struct FilterOperator<F>
where
    F: Fn(&Value) -> bool + Send + Sync,
{
    pub name: String,
    pub predicate: Arc<F>,
}

impl<F> FilterOperator<F>
where
    F: Fn(&Value) -> bool + Send + Sync,
{
    pub fn new(name: impl Into<String>, predicate: F) -> Self {
        Self {
            name: name.into(),
            predicate: Arc::new(predicate),
        }
    }
}

#[async_trait]
impl<F> StreamProcessor for FilterOperator<F>
where
    F: Fn(&Value) -> bool + Send + Sync,
{
    async fn process_item(&self, item: Value, _ctx: &Context) -> RuleResult {
        if (self.predicate)(&item) {
            Ok(item)
        } else {
            Err(RuleError::Eval("Filtered out".to_string()))
        }
    }
}

#[async_trait]
impl<F> StreamOperator for FilterOperator<F>
where
    F: Fn(&Value) -> bool + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }
}

/// Fold operator - accumulates values
pub struct FoldOperator<F, T>
where
    F: Fn(T, Value) -> T + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    pub name: String,
    pub initial: T,
    pub func: Arc<F>,
}

impl<F, T> FoldOperator<F, T>
where
    F: Fn(T, Value) -> T + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    pub fn new(name: impl Into<String>, initial: T, func: F) -> Self {
        Self {
            name: name.into(),
            initial,
            func: Arc::new(func),
        }
    }
}

#[async_trait]
impl<F, T> StreamProcessor for FoldOperator<F, T>
where
    F: Fn(T, Value) -> T + Send + Sync,
    T: Clone + Send + Sync + Into<Value> + 'static,
{
    async fn process_item(&self, item: Value, _ctx: &Context) -> RuleResult {
        // For fold, we need to process in chunks
        Ok(item)
    }

    async fn process_chunk(
        &self,
        items: Vec<Value>,
        _ctx: &Context,
    ) -> Result<Vec<Value>, RuleError> {
        let result = items
            .into_iter()
            .fold(self.initial.clone(), |acc, item| (self.func)(acc, item));

        Ok(vec![result.into()])
    }
}

#[async_trait]
impl<F, T> StreamOperator for FoldOperator<F, T>
where
    F: Fn(T, Value) -> T + Send + Sync,
    T: Clone + Send + Sync + Into<Value> + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }
}

/// Async map operator - transforms each item asynchronously
pub struct AsyncMapOperator<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Value> + Send,
{
    pub name: String,
    pub func: Arc<F>,
}

impl<F, Fut> AsyncMapOperator<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Value> + Send,
{
    pub fn new(name: impl Into<String>, func: F) -> Self {
        Self {
            name: name.into(),
            func: Arc::new(func),
        }
    }
}

#[async_trait]
impl<F, Fut> StreamProcessor for AsyncMapOperator<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Value> + Send,
{
    async fn process_item(&self, item: Value, _ctx: &Context) -> RuleResult {
        Ok((self.func)(item).await)
    }
}

#[async_trait]
impl<F, Fut> StreamOperator for AsyncMapOperator<F, Fut>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Value> + Send,
{
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_map_operator() {
        let op = MapOperator::new("double", |v: Value| {
            if let Some(n) = v.as_i64() {
                Value::Number((n * 2).into())
            } else {
                v
            }
        });

        let ctx = Context {
            data: HashMap::new(),
        };

        let result = op
            .process_item(Value::Number(5.into()), &ctx)
            .await
            .unwrap();
        assert_eq!(result, Value::Number(10.into()));
    }

    #[tokio::test]
    async fn test_filter_operator() {
        let op = FilterOperator::new("even_only", |v: &Value| {
            v.as_i64().map(|n| n % 2 == 0).unwrap_or(false)
        });

        let ctx = Context {
            data: HashMap::new(),
        };

        let result = op.process_item(Value::Number(4.into()), &ctx).await;
        assert!(result.is_ok());

        let result = op.process_item(Value::Number(5.into()), &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fold_operator() {
        let op = FoldOperator::new("sum", 0i64, |acc: i64, v: Value| {
            acc + v.as_i64().unwrap_or(0)
        });

        let ctx = Context {
            data: HashMap::new(),
        };

        let items: Vec<Value> = (1..=5).map(|i| Value::Number(i.into())).collect();
        let results = op.process_chunk(items, &ctx).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], Value::Number(15.into()));
    }

    #[tokio::test]
    async fn test_async_map_operator() {
        let op = AsyncMapOperator::new("async_double", |v: Value| async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            if let Some(n) = v.as_i64() {
                Value::Number((n * 2).into())
            } else {
                v
            }
        });

        let ctx = Context {
            data: HashMap::new(),
        };

        let result = op
            .process_item(Value::Number(5.into()), &ctx)
            .await
            .unwrap();
        assert_eq!(result, Value::Number(10.into()));
    }
}
