//! Streaming processing module
//!
//! Provides stream-based node execution with backpressure handling

use crate::core::Context;
use crate::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use std::pin::Pin;

pub mod operators;
pub mod stream_node;

pub use operators::{StreamOperator, MapOperator, FilterOperator, FoldOperator};
pub use stream_node::StreamNode;

/// Stream item type
pub type StreamItem = Result<Value, RuleError>;

/// Stream type alias
pub type ValueStream = Pin<Box<dyn Stream<Item = StreamItem> + Send>>;

/// Backpressure configuration
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Buffer size for bounded channels
    pub buffer_size: usize,
    /// Maximum concurrent operations
    pub max_concurrent: usize,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            max_concurrent: 10,
        }
    }
}

/// Chunk configuration for large datasets
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Size of each chunk
    pub chunk_size: usize,
    /// Overlap between chunks (for sliding windows)
    pub overlap: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            overlap: 0,
        }
    }
}

/// Stream processor trait
#[async_trait]
pub trait StreamProcessor: Send + Sync {
    /// Process a single item from the stream
    async fn process_item(&self, item: Value, ctx: &Context) -> RuleResult;

    /// Process a chunk of items (for batch operations)
    async fn process_chunk(&self, items: Vec<Value>, ctx: &Context) -> Result<Vec<Value>, RuleError> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let result = self.process_item(item, ctx).await?;
            results.push(result);
        }
        Ok(results)
    }
}

/// Create a stream from a vector with backpressure
pub fn create_stream_from_vec(
    data: Vec<Value>,
    config: BackpressureConfig,
) -> ValueStream {
    let (tx, rx) = mpsc::channel(config.buffer_size);

    tokio::spawn(async move {
        for item in data {
            if tx.send(Ok(item)).await.is_err() {
                break;
            }
        }
    });

    Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
}

/// Create a chunked stream from a large dataset
pub fn create_chunked_stream(
    data: Vec<Value>,
    config: ChunkConfig,
) -> Pin<Box<dyn Stream<Item = Result<Vec<Value>, RuleError>> + Send>> {
    let chunks: Vec<Vec<Value>> = data
        .chunks(config.chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    Box::pin(tokio_stream::iter(chunks.into_iter().map(Ok)))
}

/// Apply backpressure to a stream
/// Note: This is a placeholder. For full backpressure,
/// use StreamNode with BackpressureConfig
pub fn apply_backpressure(
    stream: ValueStream,
    _config: BackpressureConfig,
) -> ValueStream {
    // Simply return the stream as-is
    // Backpressure is handled by the bounded channels
    stream
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_create_stream_from_vec() {
        let data = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Number(3.into()),
        ];

        let config = BackpressureConfig::default();
        let mut stream = create_stream_from_vec(data, config);

        let mut count = 0;
        while let Some(Ok(_)) = stream.next().await {
            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_chunked_stream() {
        let data: Vec<Value> = (0..10)
            .map(|i| Value::Number(i.into()))
            .collect();

        let config = ChunkConfig {
            chunk_size: 3,
            overlap: 0,
        };

        let mut stream = create_chunked_stream(data, config);

        let mut chunk_count = 0;
        while let Some(Ok(chunk)) = stream.next().await {
            chunk_count += 1;
            assert!(chunk.len() <= 3);
        }

        assert_eq!(chunk_count, 4); // 3, 3, 3, 1
    }
}
