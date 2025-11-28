//! Distributed Context Sharing Example
//!
//! Demonstrates context serialization, state sharing between microservices,
//! distributed caching, and conflict resolution.
//!
//! This example shows distributed context features that can be used with
//! the YAML graph configuration at: examples/distributed_context_graph.yaml
//!
//! Usage: cargo run --example distributed_context

use rust_logic_graph::distributed::{
    DistributedContext, SharedContext, InMemoryStore,
    DistributedCache, CacheStrategy, VersionedContext, ConflictResolution,
    ThreeWayMerge,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Distributed Context Sharing Example ===");
    println!("📄 Graph Configuration: examples/distributed_context_graph.yaml\n");
    
    // Example 1: Context Serialization
    example_serialization()?;
    
    // Example 2: State Sharing Between Services
    example_state_sharing().await?;
    
    // Example 3: Distributed Caching
    example_distributed_caching().await?;
    
    // Example 4: Conflict Resolution
    example_conflict_resolution()?;
    
    // Example 5: Three-Way Merge
    example_three_way_merge()?;
    
    println!("\n💡 Integration with YAML Graph:");
    println!("   The YAML config (distributed_context_graph.yaml) defines a complete");
    println!("   order processing workflow that demonstrates:");
    println!("   • Context sharing between validation → inventory → payment → notification");
    println!("   • Each node can read/write to the shared distributed context");
    println!("   • Use SharedContext for thread-safe access across async nodes");
    println!("   • Use DistributedCache for performance optimization");
    println!("   • Context is automatically serialized when crossing service boundaries");
    
    Ok(())
}

/// Example 1: Context Serialization for Remote Execution
fn example_serialization() -> anyhow::Result<()> {
    println!("📦 Example 1: Context Serialization\n");
    
    // Create a context with business data
    let mut context = DistributedContext::new("order-12345");
    context.set("customer_id", json!("CUST-789"));
    context.set("order_total", json!(1499.99));
    context.set("items", json!([
        {"sku": "LAPTOP-001", "qty": 1, "price": 1299.99},
        {"sku": "MOUSE-042", "qty": 2, "price": 100.00}
    ]));
    context.add_tag("order");
    context.add_tag("high-value");
    context.set_modified_by("order-service");
    
    println!("Created context:");
    println!("  Session ID: {}", context.session_id);
    println!("  Version: {}", context.metadata.version);
    println!("  Data keys: {:?}", context.data.keys().collect::<Vec<_>>());
    println!("  Tags: {:?}", context.metadata.tags);
    
    // Serialize to binary (MessagePack)
    let binary = context.serialize()?;
    println!("\n✓ Serialized to {} bytes (MessagePack)", binary.len());
    
    // Serialize to JSON (human-readable)
    let json_str = context.to_json()?;
    println!("✓ Serialized to {} bytes (JSON)", json_str.len());
    
    // Deserialize from binary
    let restored = DistributedContext::deserialize(&binary)?;
    println!("\n✓ Deserialized successfully");
    println!("  Customer ID: {}", restored.get("customer_id").unwrap());
    println!("  Order Total: {}", restored.get("order_total").unwrap());
    
    println!("\n{}\n", "─".repeat(80));
    Ok(())
}

/// Example 2: State Sharing Between Microservices
async fn example_state_sharing() -> anyhow::Result<()> {
    println!("🌐 Example 2: State Sharing Between Microservices\n");
    
    // Simulate Order Service
    let order_context = SharedContext::new("session-abc123");
    order_context.set("user_id", json!("user-456")).await;
    order_context.set("cart_total", json!(299.99)).await;
    order_context.set("created_by", json!("order-service")).await;
    
    println!("Order Service created context:");
    println!("  Version: {}", order_context.version().await);
    
    // Serialize and "transmit" to Inventory Service
    let serialized = order_context.serialize().await?;
    println!("  Transmitted {} bytes to Inventory Service", serialized.len());
    
    // Inventory Service receives and deserializes
    let inventory_context = DistributedContext::deserialize(&serialized)?;
    println!("\nInventory Service received context:");
    println!("  User ID: {}", inventory_context.get("user_id").unwrap());
    println!("  Cart Total: {}", inventory_context.get("cart_total").unwrap());
    
    // Inventory Service adds data
    let mut inventory_context = inventory_context;
    inventory_context.set("items_checked", json!(true));
    inventory_context.set("stock_available", json!(true));
    inventory_context.set_modified_by("inventory-service");
    
    println!("\nInventory Service modified context:");
    println!("  Version: {}", inventory_context.metadata.version);
    println!("  Modified by: {}", inventory_context.metadata.modified_by.as_ref().unwrap());
    
    // Send to Payment Service
    let payment_serialized = inventory_context.serialize()?;
    let payment_context = DistributedContext::deserialize(&payment_serialized)?;
    
    println!("\nPayment Service received final context:");
    println!("  All keys: {:?}", payment_context.data.keys().collect::<Vec<_>>());
    println!("  ✓ Complete state shared across 3 services");
    
    println!("\n{}\n", "─".repeat(80));
    Ok(())
}

/// Example 3: Distributed Caching with Redis/Memcached
async fn example_distributed_caching() -> anyhow::Result<()> {
    println!("💾 Example 3: Distributed Caching\n");
    
    // Create in-memory store (simulating Redis)
    let store = Arc::new(InMemoryStore::new());
    
    // Create distributed cache with write-through strategy
    let cache = DistributedCache::with_config(
        store.clone(),
        CacheStrategy::WriteThrough,
        Some(Duration::from_secs(3600)),
    );
    
    println!("Cache configuration:");
    println!("  Strategy: WriteThrough");
    println!("  Default TTL: 3600 seconds");
    
    // Service A writes to cache
    let mut ctx_a = DistributedContext::new("user-session-xyz");
    ctx_a.set("authenticated", json!(true));
    ctx_a.set("role", json!("admin"));
    ctx_a.set("preferences", json!({"theme": "dark", "language": "en"}));
    
    cache.put(&ctx_a).await?;
    println!("\n✓ Service A cached context for user-session-xyz");
    
    // Service B reads from cache
    let ctx_b = cache.get("user-session-xyz").await?;
    assert!(ctx_b.is_some());
    let ctx_b = ctx_b.unwrap();
    
    println!("✓ Service B retrieved context from cache:");
    println!("  Role: {}", ctx_b.get("role").unwrap());
    println!("  Authenticated: {}", ctx_b.get("authenticated").unwrap());
    
    // Batch operations
    let mut ctx1 = DistributedContext::new("batch-1");
    ctx1.set("data", json!("value1"));
    
    let mut ctx2 = DistributedContext::new("batch-2");
    ctx2.set("data", json!("value2"));
    
    cache.put_many(&[ctx1, ctx2]).await?;
    println!("\n✓ Batch cached 2 contexts");
    
    let batch_results = cache.get_many(&[
        "batch-1".to_string(),
        "batch-2".to_string(),
    ]).await?;
    
    println!("✓ Batch retrieved {} contexts", batch_results.len());
    
    // Cache stats
    let stats = cache.stats().await;
    println!("\nCache Statistics:");
    println!("  Total contexts: {}", stats.total_contexts);
    println!("  Strategy: {:?}", stats.strategy);
    
    println!("\n{}\n", "─".repeat(80));
    Ok(())
}

/// Example 4: Conflict Resolution Strategies
fn example_conflict_resolution() -> anyhow::Result<()> {
    println!("⚔️  Example 4: Conflict Resolution\n");
    
    // Strategy 1: Last Write Wins
    println!("Strategy 1: Last Write Wins");
    let mut vctx = VersionedContext::with_config(
        "conflict-test",
        5,
        ConflictResolution::LastWriteWins,
    );
    
    let mut ctx1 = DistributedContext::new("conflict-test");
    ctx1.set("price", json!(100));
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    vctx.update(ctx1)?;
    
    let mut ctx2 = DistributedContext::new("conflict-test");
    ctx2.set("price", json!(150));
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    vctx.merge_with_resolution(&ctx2)?;
    
    println!("  Version 1: price = 100");
    println!("  Version 2: price = 150 (newer timestamp)");
    println!("  Result: price = {}", vctx.current.get("price").unwrap());
    println!("  ✓ Newer value wins\n");
    
    // Strategy 2: Higher Version Wins
    println!("Strategy 2: Higher Version Wins");
    let mut vctx2 = VersionedContext::with_config(
        "version-test",
        5,
        ConflictResolution::HigherVersionWins,
    );
    
    let mut ctx_v1 = DistributedContext::new("version-test");
    ctx_v1.set("status", json!("pending"));
    ctx_v1.metadata.version = 5;
    
    vctx2.update(ctx_v1)?;
    
    let mut ctx_v2 = DistributedContext::new("version-test");
    ctx_v2.set("status", json!("approved"));
    ctx_v2.metadata.version = 10;
    
    vctx2.merge_with_resolution(&ctx_v2)?;
    
    println!("  Version 5: status = pending");
    println!("  Version 10: status = approved");
    println!("  Result: status = {}", vctx2.current.get("status").unwrap());
    println!("  ✓ Higher version wins\n");
    
    // Strategy 3: Merge All
    println!("Strategy 3: Merge All");
    let mut vctx3 = VersionedContext::with_config(
        "merge-test",
        5,
        ConflictResolution::MergeAll,
    );
    
    let mut ctx_local = DistributedContext::new("merge-test");
    ctx_local.set("local_key", json!("local_value"));
    vctx3.update(ctx_local)?;
    
    let mut ctx_remote = DistributedContext::new("merge-test");
    ctx_remote.set("remote_key", json!("remote_value"));
    
    vctx3.merge_with_resolution(&ctx_remote)?;
    
    println!("  Local: local_key = local_value");
    println!("  Remote: remote_key = remote_value");
    println!("  Result: Both keys preserved");
    println!("    local_key = {}", vctx3.current.get("local_key").unwrap());
    println!("    remote_key = {}", vctx3.current.get("remote_key").unwrap());
    println!("  ✓ All changes merged\n");
    
    println!("{}\n", "─".repeat(80));
    Ok(())
}

/// Example 5: Three-Way Merge for Complex Conflicts
fn example_three_way_merge() -> anyhow::Result<()> {
    println!("🔀 Example 5: Three-Way Merge\n");
    
    // Base version (common ancestor)
    let mut base = DistributedContext::new("merge-scenario");
    base.set("name", json!("Product A"));
    base.set("price", json!(100));
    base.set("stock", json!(50));
    let base_snapshot = base.snapshot();
    
    println!("Base version:");
    println!("  name: Product A");
    println!("  price: 100");
    println!("  stock: 50");
    
    // Local changes (Service A)
    let mut local = base.clone();
    local.set("price", json!(120));  // Price increase
    local.set("description", json!("Updated description"));  // New field
    let local_snapshot = local.snapshot();
    
    println!("\nLocal changes (Service A):");
    println!("  price: 100 → 120");
    println!("  description: (added)");
    
    // Remote changes (Service B)
    let mut remote = base.clone();
    remote.set("stock", json!(45));  // Stock update
    remote.set("category", json!("Electronics"));  // New field
    let remote_snapshot = remote.snapshot();
    
    println!("\nRemote changes (Service B):");
    println!("  stock: 50 → 45");
    println!("  category: (added)");
    
    // Perform three-way merge
    let merger = ThreeWayMerge::new(base_snapshot, local_snapshot, remote_snapshot);
    let merged = merger.merge()?;
    
    println!("\n✓ Three-way merge result:");
    println!("  name: {}", merged.get("name").unwrap());
    println!("  price: {} (from local)", merged.get("price").unwrap());
    println!("  stock: {} (from remote)", merged.get("stock").unwrap());
    println!("  description: {} (from local)", merged.get("description").unwrap());
    println!("  category: {} (from remote)", merged.get("category").unwrap());
    println!("\n✓ All non-conflicting changes merged successfully!");
    
    println!("\n{}\n", "─".repeat(80));
    Ok(())
}
