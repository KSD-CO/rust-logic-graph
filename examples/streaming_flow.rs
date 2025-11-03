//! Streaming Processing Example
//!
//! This example demonstrates stream-based node execution with:
//! - Backpressure handling
//! - Large dataset chunking
//! - Stream transformation operators (map, filter, fold)
//!
//! To run: cargo run --example streaming_flow

use rust_logic_graph::{Context, Node};
use rust_logic_graph::streaming::{
    StreamNode, StreamProcessor, BackpressureConfig, ChunkConfig,
    MapOperator, FilterOperator, FoldOperator,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

// Custom stream processor
struct NumberProcessor;

#[async_trait]
impl StreamProcessor for NumberProcessor {
    async fn process_item(&self, item: Value, _ctx: &Context) -> Result<Value, rust_logic_graph::RuleError> {
        // Simulate some async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        if let Some(n) = item.as_i64() {
            Ok(Value::Number((n * 2).into()))
        } else {
            Ok(item)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("🚀 Streaming Processing Example\n");

    // Example 1: Basic streaming with backpressure
    println!("=== Example 1: Basic Streaming with Backpressure ===\n");

    let data: Vec<Value> = (1..=100).map(|i| Value::Number(i.into())).collect();

    let processor = Arc::new(NumberProcessor);
    let node = StreamNode::new("processor", processor)
        .with_backpressure(BackpressureConfig {
            buffer_size: 10,
            max_concurrent: 5,
        });

    let ctx = Context {
        data: HashMap::new(),
    };

    println!("Processing 100 numbers with backpressure...");
    let start = std::time::Instant::now();
    let result = node.process_stream(data, &ctx).await?;
    let duration = start.elapsed();

    if let Value::Array(results) = result {
        println!("✓ Processed {} items in {:?}", results.len(), duration);
        println!("  First: {:?}, Last: {:?}\n", results.first(), results.last());
    }

    // Example 2: Large dataset with chunking
    println!("=== Example 2: Large Dataset with Chunking ===\n");

    let large_data: Vec<Value> = (1..=10_000).map(|i| Value::Number(i.into())).collect();

    let processor = Arc::new(NumberProcessor);
    let node = StreamNode::new("chunked_processor", processor)
        .with_chunking(ChunkConfig {
            chunk_size: 1000,
            overlap: 0,
        })
        .with_backpressure(BackpressureConfig {
            buffer_size: 100,
            max_concurrent: 10,
        });

    let ctx = Context {
        data: HashMap::new(),
    };

    println!("Processing 10,000 numbers in chunks of 1,000...");
    let start = std::time::Instant::now();
    let result = node.process_stream(large_data, &ctx).await?;
    let duration = start.elapsed();

    if let Value::Array(results) = result {
        println!("✓ Processed {} items in {:?}", results.len(), duration);
        println!("  Throughput: {:.0} items/sec\n", results.len() as f64 / duration.as_secs_f64());
    }

    // Example 3: Map operator
    println!("=== Example 3: Map Operator ===\n");

    let data: Vec<Value> = (1..=10).map(|i| Value::Number(i.into())).collect();

    let map_op = Arc::new(MapOperator::new("square", |v: Value| {
        if let Some(n) = v.as_i64() {
            Value::Number((n * n).into())
        } else {
            v
        }
    }));

    let node = StreamNode::new("map_node", map_op);
    let ctx = Context {
        data: HashMap::new(),
    };

    println!("Squaring numbers 1-10...");
    let result = node.process_stream(data, &ctx).await?;

    if let Value::Array(results) = result {
        println!("✓ Results: {:?}\n", results);
    }

    // Example 4: Filter operator
    println!("=== Example 4: Filter Operator ===\n");

    let data: Vec<Value> = (1..=20).map(|i| Value::Number(i.into())).collect();

    let filter_op = Arc::new(FilterOperator::new("even_only", |v: &Value| {
        v.as_i64().map(|n| n % 2 == 0).unwrap_or(false)
    }));

    let node = StreamNode::new("filter_node", filter_op);
    let ctx = Context {
        data: HashMap::new(),
    };

    println!("Filtering even numbers from 1-20...");
    let result = node.process_stream(data, &ctx).await?;

    if let Value::Array(results) = result {
        println!("✓ Even numbers: {:?}\n", results);
    }

    // Example 5: Fold operator (sum)
    println!("=== Example 5: Fold Operator (Sum) ===\n");

    let data: Vec<Value> = (1..=100).map(|i| Value::Number(i.into())).collect();

    let fold_op = Arc::new(FoldOperator::new("sum", 0i64, |acc: i64, v: Value| {
        acc + v.as_i64().unwrap_or(0)
    }));

    let node = StreamNode::new("fold_node", fold_op)
        .with_chunking(ChunkConfig {
            chunk_size: 100,
            overlap: 0,
        })
        .collect_results(false); // Only return final result

    let ctx = Context {
        data: HashMap::new(),
    };

    println!("Summing numbers 1-100...");
    let result = node.process_stream(data, &ctx).await?;

    println!("✓ Sum: {:?}\n", result);

    // Example 6: Chained operations
    println!("=== Example 6: Chained Operations ===\n");

    println!("Pipeline: numbers → filter(even) → map(square) → collect");

    // First: filter even numbers
    let data: Vec<Value> = (1..=20).map(|i| Value::Number(i.into())).collect();

    let filter_op = Arc::new(FilterOperator::new("even", |v: &Value| {
        v.as_i64().map(|n| n % 2 == 0).unwrap_or(false)
    }));

    let filter_node = StreamNode::new("filter", filter_op);
    let ctx = Context {
        data: HashMap::new(),
    };

    let filtered = filter_node.process_stream(data, &ctx).await?;

    // Then: square the numbers
    if let Value::Array(filtered_data) = filtered {
        let map_op = Arc::new(MapOperator::new("square", |v: Value| {
            if let Some(n) = v.as_i64() {
                Value::Number((n * n).into())
            } else {
                v
            }
        }));

        let map_node = StreamNode::new("square", map_op);
        let result = map_node.process_stream(filtered_data, &ctx).await?;

        println!("✓ Results: {:?}\n", result);
    }

    println!("=== Benefits of Streaming Processing ===");
    println!("  • Memory efficient - processes items incrementally");
    println!("  • Backpressure handling - prevents overwhelming consumers");
    println!("  • Large dataset support - chunking for massive data");
    println!("  • Concurrent processing - multiple items in parallel");
    println!("  • Composable operators - chain transformations");
    println!("\n🎉 Example completed!");

    Ok(())
}
