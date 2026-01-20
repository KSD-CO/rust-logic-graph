//! Stream-based node implementation

use crate::core::Context;
use crate::node::{Node, NodeType};
use crate::rule::{RuleError, RuleResult};
use crate::streaming::{
    apply_backpressure, create_chunked_stream, create_stream_from_vec, BackpressureConfig,
    ChunkConfig, StreamProcessor,
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{error, info};

/// Stream node for processing data streams
#[derive(Clone)]
pub struct StreamNode {
    pub id: String,
    pub processor: Arc<dyn StreamProcessor>,
    pub backpressure_config: BackpressureConfig,
    pub chunk_config: Option<ChunkConfig>,
    pub collect_results: bool,
}

impl StreamNode {
    /// Create a new stream node
    pub fn new(id: impl Into<String>, processor: Arc<dyn StreamProcessor>) -> Self {
        Self {
            id: id.into(),
            processor,
            backpressure_config: BackpressureConfig::default(),
            chunk_config: None,
            collect_results: true,
        }
    }

    /// Configure backpressure
    pub fn with_backpressure(mut self, config: BackpressureConfig) -> Self {
        self.backpressure_config = config;
        self
    }

    /// Enable chunked processing for large datasets
    pub fn with_chunking(mut self, config: ChunkConfig) -> Self {
        self.chunk_config = Some(config);
        self
    }

    /// Set whether to collect all results (default: true)
    /// If false, only the last result is stored
    pub fn collect_results(mut self, collect: bool) -> Self {
        self.collect_results = collect;
        self
    }

    /// Process a stream of data
    pub async fn process_stream(&self, data: Vec<Value>, ctx: &Context) -> RuleResult {
        info!("StreamNode[{}]: Processing {} items", self.id, data.len());

        if let Some(chunk_config) = &self.chunk_config {
            // Chunked processing for large datasets
            self.process_chunked(data, chunk_config.clone(), ctx).await
        } else {
            // Regular streaming processing
            self.process_regular(data, ctx).await
        }
    }

    /// Regular stream processing
    async fn process_regular(&self, data: Vec<Value>, ctx: &Context) -> RuleResult {
        let stream = create_stream_from_vec(data, self.backpressure_config.clone());
        let stream = apply_backpressure(stream, self.backpressure_config.clone());

        let mut stream = Box::pin(stream);
        let mut results = Vec::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(value) => match self.processor.process_item(value, ctx).await {
                    Ok(result) => results.push(result),
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }

        info!("StreamNode[{}]: Processed {} items", self.id, results.len());

        if self.collect_results {
            Ok(Value::Array(results))
        } else {
            results
                .last()
                .cloned()
                .ok_or_else(|| RuleError::Eval("No results produced".to_string()))
        }
    }

    /// Chunked stream processing for large datasets
    async fn process_chunked(
        &self,
        data: Vec<Value>,
        chunk_config: ChunkConfig,
        ctx: &Context,
    ) -> RuleResult {
        info!(
            "StreamNode[{}]: Processing {} items in chunks of {}",
            self.id,
            data.len(),
            chunk_config.chunk_size
        );

        let mut stream = create_chunked_stream(data, chunk_config);
        let mut all_results = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    info!(
                        "StreamNode[{}]: Processing chunk of {} items",
                        self.id,
                        chunk.len()
                    );
                    match self.processor.process_chunk(chunk, ctx).await {
                        Ok(results) => {
                            if self.collect_results {
                                all_results.extend(results);
                            } else {
                                if let Some(last) = results.last() {
                                    all_results = vec![last.clone()];
                                }
                            }
                        }
                        Err(e) => {
                            error!("StreamNode[{}]: Chunk processing failed: {}", self.id, e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    error!("StreamNode[{}]: Stream error: {}", self.id, e);
                    return Err(e);
                }
            }
        }

        info!(
            "StreamNode[{}]: Total processed {} items",
            self.id,
            all_results.len()
        );

        if self.collect_results {
            Ok(Value::Array(all_results))
        } else {
            all_results
                .last()
                .cloned()
                .ok_or_else(|| RuleError::Eval("No results produced".to_string()))
        }
    }
}

impl std::fmt::Debug for StreamNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamNode")
            .field("id", &self.id)
            .field("backpressure_config", &self.backpressure_config)
            .field("chunk_config", &self.chunk_config)
            .field("collect_results", &self.collect_results)
            .finish()
    }
}

#[async_trait]
impl Node for StreamNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode // Using AINode as it's for processing
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        info!("StreamNode[{}]: Starting stream execution", self.id);

        // Get input data from context
        let input_key = format!("{}_input", self.id);
        let data = ctx
            .data
            .get(&input_key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| RuleError::Eval(format!("No input data found for key: {}", input_key)))?
            .clone();

        let result = self.process_stream(data, ctx).await?;

        // Store result in context
        ctx.data
            .insert(format!("{}_result", self.id), result.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestProcessor;

    #[async_trait]
    impl StreamProcessor for TestProcessor {
        async fn process_item(&self, item: Value, _ctx: &Context) -> RuleResult {
            if let Some(n) = item.as_i64() {
                Ok(Value::Number((n * 2).into()))
            } else {
                Ok(item)
            }
        }
    }

    #[tokio::test]
    async fn test_stream_node_basic() {
        let processor = Arc::new(TestProcessor);
        let node = StreamNode::new("test", processor);

        let data: Vec<Value> = (1..=5).map(|i| Value::Number(i.into())).collect();

        let ctx = Context {
            data: HashMap::new(),
        };

        let result = node.process_stream(data, &ctx).await.unwrap();

        if let Value::Array(results) = result {
            assert_eq!(results.len(), 5);
            assert_eq!(results[0], Value::Number(2.into()));
            assert_eq!(results[4], Value::Number(10.into()));
        } else {
            panic!("Expected array result");
        }
    }

    #[tokio::test]
    async fn test_stream_node_chunked() {
        let processor = Arc::new(TestProcessor);
        let node = StreamNode::new("test", processor).with_chunking(ChunkConfig {
            chunk_size: 3,
            overlap: 0,
        });

        let data: Vec<Value> = (1..=10).map(|i| Value::Number(i.into())).collect();

        let ctx = Context {
            data: HashMap::new(),
        };

        let result = node.process_stream(data, &ctx).await.unwrap();

        if let Value::Array(results) = result {
            assert_eq!(results.len(), 10);
        } else {
            panic!("Expected array result");
        }
    }
}
