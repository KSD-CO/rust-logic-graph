//! Cache Flow Example
//!
//! Demonstrates the caching layer features:
//! - Node result caching
//! - Cache invalidation strategies (LRU, FIFO, LFU)
//! - TTL support with automatic expiration
//! - Memory limits with automatic eviction
//! - Cache statistics and monitoring
//!
//! Run with:
//! ```bash
//! cargo run --example cache_flow
//! ```

use rust_logic_graph::{
    cache::{CacheConfig, CacheManager, EvictionPolicy},
    rule::RuleError,
    Context, Executor, Graph, GraphDef, Node, NodeType,
};
use serde_json::json;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber;

/// Custom node that simulates expensive computation
struct ExpensiveComputeNode {
    id: String,
    computation_time_ms: u64,
}

impl ExpensiveComputeNode {
    fn new(id: impl Into<String>, computation_time_ms: u64) -> Self {
        Self {
            id: id.into(),
            computation_time_ms,
        }
    }
}

#[async_trait::async_trait]
impl Node for ExpensiveComputeNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::RuleNode
    }

    async fn run(&self, context: &mut Context) -> Result<serde_json::Value, RuleError> {
        info!(
            "Node '{}': Starting expensive computation ({} ms)...",
            self.id, self.computation_time_ms
        );

        // Simulate expensive computation
        tokio::time::sleep(Duration::from_millis(self.computation_time_ms)).await;

        // Get input value
        let input = context
            .data
            .get("input")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Compute result (factorial)
        let result = (1..=input).product::<i64>();

        // Store in context
        context
            .data
            .insert(format!("{}_result", self.id), json!(result));

        info!(
            "Node '{}': Computation complete. Result: {}",
            self.id, result
        );

        Ok(json!(result))
    }
}

async fn demo_basic_caching() -> anyhow::Result<()> {
    println!("\n=== Demo 1: Basic Caching ===\n");

    // Create cache with default configuration
    let cache_config = CacheConfig {
        max_entries: 100,
        max_memory_bytes: 10 * 1024 * 1024, // 10MB
        default_ttl: None,                  // No expiration
        eviction_policy: EvictionPolicy::LRU,
        enable_background_cleanup: false,
    };

    let cache = CacheManager::new(cache_config).await?;

    // Create executor with cache
    let mut executor = Executor::with_cache(cache.clone());

    // Register expensive computation node
    executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 1000)));

    // Create simple graph
    let graph_def = GraphDef::from_node_types(
        vec![("compute".to_string(), NodeType::RuleNode)]
            .into_iter()
            .collect(),
        vec![],
    );

    // First execution - cache miss
    info!("First execution (cache miss):");
    let mut graph1 = Graph::new(graph_def.clone());
    graph1.context.set("input", json!(5));

    let start = std::time::Instant::now();
    executor.execute(&mut graph1).await?;
    let duration1 = start.elapsed();

    println!("First execution time: {:?}", duration1);
    println!("Cache stats: {:?}", cache.stats());

    // Second execution - cache hit
    info!("\nSecond execution (cache hit):");
    let mut graph2 = Graph::new(graph_def.clone());
    graph2.context.set("input", json!(5)); // Same input

    let start = std::time::Instant::now();
    executor.execute(&mut graph2).await?;
    let duration2 = start.elapsed();

    println!("Second execution time: {:?}", duration2);
    println!("Cache stats: {:?}", cache.stats());
    println!(
        "Speedup: {:.2}x",
        duration1.as_secs_f64() / duration2.as_secs_f64()
    );

    // Third execution with different input - cache miss
    info!("\nThird execution with different input (cache miss):");
    let mut graph3 = Graph::new(graph_def);
    graph3.context.set("input", json!(10)); // Different input

    let start = std::time::Instant::now();
    executor.execute(&mut graph3).await?;
    let duration3 = start.elapsed();

    println!("Third execution time: {:?}", duration3);
    println!("Final cache stats: {:?}", cache.stats());
    println!("Hit rate: {:.1}%", cache.stats().hit_rate());

    Ok(())
}

async fn demo_ttl_expiration() -> anyhow::Result<()> {
    println!("\n\n=== Demo 2: TTL Expiration ===\n");

    // Create cache with short TTL
    let cache_config = CacheConfig {
        max_entries: 100,
        max_memory_bytes: 10 * 1024 * 1024,
        default_ttl: Some(Duration::from_secs(2)), // 2 second TTL
        eviction_policy: EvictionPolicy::LRU,
        enable_background_cleanup: true,
    };

    let cache = CacheManager::new(cache_config).await?;
    let mut executor = Executor::with_cache(cache.clone());
    executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 500)));

    let graph_def = GraphDef::from_node_types(
        vec![("compute".to_string(), NodeType::RuleNode)]
            .into_iter()
            .collect(),
        vec![],
    );

    // First execution
    info!("First execution:");
    let mut graph1 = Graph::new(graph_def.clone());
    graph1.context.set("input", json!(7));
    executor.execute(&mut graph1).await?;
    println!("Cache stats after first execution: {:?}", cache.stats());

    // Immediate second execution - should hit cache
    info!("\nImmediate second execution (within TTL):");
    let mut graph2 = Graph::new(graph_def.clone());
    graph2.context.set("input", json!(7));
    executor.execute(&mut graph2).await?;
    println!("Cache stats: {:?}", cache.stats());

    // Wait for TTL to expire
    println!("\nWaiting for TTL to expire (3 seconds)...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Third execution after TTL - should miss cache
    info!("\nThird execution (after TTL expiration):");
    let mut graph3 = Graph::new(graph_def);
    graph3.context.set("input", json!(7));
    executor.execute(&mut graph3).await?;
    println!("Cache stats after expiration: {:?}", cache.stats());

    Ok(())
}

async fn demo_eviction_policies() -> anyhow::Result<()> {
    println!("\n\n=== Demo 3: Eviction Policies ===\n");

    for policy in [
        EvictionPolicy::LRU,
        EvictionPolicy::FIFO,
        EvictionPolicy::LFU,
    ] {
        println!("\n--- Testing {:?} Policy ---", policy);

        let cache_config = CacheConfig {
            max_entries: 3, // Small limit to trigger eviction
            max_memory_bytes: 10 * 1024 * 1024,
            default_ttl: None,
            eviction_policy: policy,
            enable_background_cleanup: false,
        };

        let cache = CacheManager::new(cache_config).await?;
        let mut executor = Executor::with_cache(cache.clone());
        executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 100)));

        let graph_def = GraphDef::from_node_types(
            vec![("compute".to_string(), NodeType::RuleNode)]
                .into_iter()
                .collect(),
            vec![],
        );

        // Add 4 entries (should evict 1)
        for i in 1..=4 {
            let mut graph = Graph::new(graph_def.clone());
            graph.context.set("input", json!(i));
            executor.execute(&mut graph).await?;
            println!("Added entry {}, cache size: {}", i, cache.len());
        }

        let stats = cache.stats();
        println!(
            "Final stats: entries={}, evictions={}",
            stats.current_entries, stats.evictions
        );
    }

    Ok(())
}

async fn demo_memory_limits() -> anyhow::Result<()> {
    println!("\n\n=== Demo 4: Memory Limits ===\n");

    // Create cache with small memory limit
    let cache_config = CacheConfig {
        max_entries: 1000,
        max_memory_bytes: 1024, // Only 1KB
        default_ttl: None,
        eviction_policy: EvictionPolicy::LRU,
        enable_background_cleanup: false,
    };

    let cache = CacheManager::new(cache_config).await?;
    let mut executor = Executor::with_cache(cache.clone());
    executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 50)));

    let graph_def = GraphDef::from_node_types(
        vec![("compute".to_string(), NodeType::RuleNode)]
            .into_iter()
            .collect(),
        vec![],
    );

    // Add entries until memory limit is reached
    for i in 1..=20 {
        let mut graph = Graph::new(graph_def.clone());
        graph.context.set("input", json!(i));
        graph.context.set("large_data", json!(vec![i; 100])); // Add some bulk
        executor.execute(&mut graph).await?;

        let stats = cache.stats();
        println!(
            "Entry {}: size={} entries, memory={} bytes, evictions={}",
            i, stats.current_entries, stats.current_memory_bytes, stats.evictions
        );
    }

    let stats = cache.stats();
    println!(
        "\nFinal memory usage: {} bytes (limit: 1024 bytes)",
        stats.current_memory_bytes
    );
    println!("Total evictions: {}", stats.evictions);

    Ok(())
}

async fn demo_cache_invalidation() -> anyhow::Result<()> {
    println!("\n\n=== Demo 5: Cache Invalidation ===\n");

    let cache = CacheManager::new(CacheConfig::default()).await?;
    let mut executor = Executor::with_cache(cache.clone());
    executor.register_node(Box::new(ExpensiveComputeNode::new("compute", 200)));

    let graph_def = GraphDef::from_node_types(
        vec![("compute".to_string(), NodeType::RuleNode)]
            .into_iter()
            .collect(),
        vec![],
    );

    // Execute multiple times with different inputs
    for i in 1..=5 {
        let mut graph = Graph::new(graph_def.clone());
        graph.context.set("input", json!(i));
        executor.execute(&mut graph).await?;
    }

    println!("Cache populated with {} entries", cache.len());
    println!("Cache stats: {:?}", cache.stats());

    // Invalidate specific node
    println!("\nInvalidating all entries for node 'compute'...");
    let invalidated = cache.invalidate_node("compute");
    println!("Invalidated {} entries", invalidated);
    println!("Cache size after invalidation: {}", cache.len());

    // Re-execute - should be cache miss
    info!("\nRe-executing after invalidation:");
    let mut graph = Graph::new(graph_def);
    graph.context.set("input", json!(1));
    executor.execute(&mut graph).await?;
    println!("Final cache stats: {:?}", cache.stats());

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║         Rust Logic Graph - Cache Layer Demo             ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    demo_basic_caching().await?;
    demo_ttl_expiration().await?;
    demo_eviction_policies().await?;
    demo_memory_limits().await?;
    demo_cache_invalidation().await?;

    println!("\n\n✅ All cache demos completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Node result caching with automatic key generation");
    println!("  ✓ Cache hit/miss tracking and statistics");
    println!("  ✓ TTL-based expiration with background cleanup");
    println!("  ✓ Multiple eviction policies (LRU, FIFO, LFU)");
    println!("  ✓ Memory limits with automatic eviction");
    println!("  ✓ Manual cache invalidation");

    Ok(())
}
