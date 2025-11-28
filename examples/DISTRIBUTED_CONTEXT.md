# Distributed Context Sharing - Complete Guide

## Overview

The distributed context sharing system enables **stateful communication** between microservices in a distributed architecture. Context is automatically serialized, versioned, and shared across service boundaries.

## YAML Configuration

File: `distributed_context_graph.yaml`

### Graph Structure

```
validate_order (Rule)
    ↓
check_inventory (DB - Postgres)
    ↓
calculate_shipping (Rule)
    ↓
process_payment (AI - OpenAI GPT-4)
    ↓
update_order_status (DB - Postgres)
    ↓
send_notification (Rule)
```

### Key Features Demonstrated

1. **Context Serialization**
   - Binary format (MessagePack) for efficiency
   - JSON format for debugging
   - Automatic compression (3x smaller than JSON)

2. **State Sharing**
   - Context flows through all 6 nodes
   - Each node can read from and write to context
   - Changes are preserved across node boundaries

3. **Distributed Caching**
   - Redis/Memcached integration
   - 4 caching strategies:
     - WriteThrough - Write to cache and store simultaneously
     - WriteBehind - Write to cache first, async to store
     - ReadThrough - Read from cache, fetch on miss
     - CacheAside - Manual cache management

4. **Conflict Resolution**
   - **LastWriteWins** - Timestamp-based (most recent wins)
   - **HigherVersionWins** - Version number based
   - **FailOnConflict** - Require manual resolution
   - **MergeAll** - Combine all changes

5. **Three-Way Merge**
   - Base version (common ancestor)
   - Local changes (Service A)
   - Remote changes (Service B)
   - Intelligent merge of non-conflicting changes

## Running the Example

### 1. Run All Examples

```bash
cargo run --example distributed_context
```

**Output:**
- Example 1: Serialization (186 bytes vs 499 bytes)
- Example 2: State Sharing (3 microservices)
- Example 3: Caching (write-through, batch ops)
- Example 4: Conflict Resolution (4 strategies)
- Example 5: Three-Way Merge (complex scenario)

### 2. Validate YAML Config

```bash
./target/debug/rlg validate --file examples/distributed_context_graph.yaml
```

Expected output:
```
🔍 Validating graph...
✓ Graph is valid!
```

### 3. Visualize Graph Structure

```bash
# Tree view (default)
./target/debug/rlg visualize --file examples/distributed_context_graph.yaml

# Graph view with boxes
./target/debug/rlg visualize --file examples/distributed_context_graph.yaml --graph

# With detailed node information
./target/debug/rlg visualize --file examples/distributed_context_graph.yaml --graph --details
```

## Real-World Use Case: Order Processing

### Scenario

An e-commerce order flows through multiple microservices:

1. **Order Service** - Creates order, validates data
2. **Inventory Service** - Checks stock availability
3. **Shipping Service** - Calculates shipping cost
4. **Payment Service** - Processes payment (AI fraud detection)
5. **Order Management** - Updates order status
6. **Notification Service** - Sends customer notifications

### Context Flow

```rust
// Order Service creates context
let mut context = DistributedContext::new("order-12345");
context.set("customer_id", json!("CUST-789"));
context.set("order_total", json!(1499.99));
context.set("items", json!([...]));

// Serialize and transmit
let binary = context.serialize()?;
// Send binary to Inventory Service

// Inventory Service receives and adds data
let mut inv_context = DistributedContext::deserialize(&binary)?;
inv_context.set("stock_available", json!(true));
inv_context.set("items_checked", json!(true));

// Continue to next service...
```

### Benefits

✅ **Type-Safe** - Rust's type system ensures correctness
✅ **Efficient** - Binary serialization reduces network traffic
✅ **Versioned** - Automatic version tracking
✅ **Resilient** - Conflict resolution strategies
✅ **Observable** - Full audit trail of changes

## Code Examples

### Example 1: Basic Serialization

```rust
use rust_logic_graph::distributed::DistributedContext;
use serde_json::json;

// Create context
let mut context = DistributedContext::new("session-123");
context.set("user_id", json!("user-456"));
context.set("authenticated", json!(true));

// Serialize to binary (MessagePack)
let binary = context.serialize()?;
println!("Size: {} bytes", binary.len());

// Deserialize
let restored = DistributedContext::deserialize(&binary)?;
assert_eq!(restored.get("user_id"), Some(&json!("user-456")));
```

### Example 2: Thread-Safe Sharing

```rust
use rust_logic_graph::distributed::SharedContext;
use serde_json::json;

// Create shared context (Arc<RwLock<>>)
let shared = SharedContext::new("session-123");

// Write from one task
shared.set("counter", json!(1)).await;

// Read from another task
let value = shared.get("counter").await;
assert_eq!(value, Some(json!(1)));
```

### Example 3: Distributed Caching

```rust
use rust_logic_graph::distributed::{
    DistributedCache, CacheStrategy, InMemoryStore
};
use std::time::Duration;
use std::sync::Arc;

// Create cache with Redis/Memcached (or InMemory for testing)
let store = Arc::new(InMemoryStore::new());
let cache = DistributedCache::with_config(
    store,
    CacheStrategy::WriteThrough,
    Some(Duration::from_secs(3600)),
);

// Cache a context
let mut ctx = DistributedContext::new("user-session");
ctx.set("data", json!("value"));
cache.put(&ctx).await?;

// Retrieve from cache
let cached = cache.get("user-session").await?;
assert!(cached.is_some());

// Batch operations
cache.put_many(&[ctx1, ctx2, ctx3]).await?;
let results = cache.get_many(&["id1", "id2", "id3"]).await?;
```

### Example 4: Conflict Resolution

```rust
use rust_logic_graph::distributed::{
    VersionedContext, ConflictResolution
};

// Create versioned context
let mut vctx = VersionedContext::with_config(
    "product-123",
    10, // max history
    ConflictResolution::LastWriteWins,
);

// Local update
let mut local = DistributedContext::new("product-123");
local.set("price", json!(100));
vctx.update(local)?;

// Remote update
let mut remote = DistributedContext::new("product-123");
remote.set("price", json!(120));

// Merge with conflict resolution
vctx.merge_with_resolution(&remote)?;

// Result: Last write wins (120)
assert_eq!(vctx.current.get("price"), Some(&json!(120)));
```

### Example 5: Three-Way Merge

```rust
use rust_logic_graph::distributed::ThreeWayMerge;

// Base version
let mut base = DistributedContext::new("product");
base.set("price", json!(100));
base.set("stock", json!(50));
let base_snapshot = base.snapshot();

// Local changes
let mut local = base.clone();
local.set("price", json!(120)); // Updated price
local.set("description", json!("New")); // Added field

// Remote changes
let mut remote = base.clone();
remote.set("stock", json!(45)); // Updated stock
remote.set("category", json!("Electronics")); // Added field

// Three-way merge
let merger = ThreeWayMerge::new(
    base_snapshot,
    local.snapshot(),
    remote.snapshot(),
);
let merged = merger.merge()?;

// Result: All non-conflicting changes merged
assert_eq!(merged.get("price"), Some(&json!(120))); // from local
assert_eq!(merged.get("stock"), Some(&json!(45))); // from remote
assert_eq!(merged.get("description"), Some(&json!("New"))); // from local
assert_eq!(merged.get("category"), Some(&json!("Electronics"))); // from remote
```

## Architecture Patterns

### Pattern 1: Request Context Propagation

```
API Gateway
    ↓ (creates context with trace_id, user_id)
Auth Service
    ↓ (adds auth_token, permissions)
Business Logic Service
    ↓ (adds business data)
Database Service
    ↓ (executes queries, adds results)
Response Builder
    ↓ (formats response)
```

### Pattern 2: Event-Driven Context

```
Event Source (Kafka/NATS)
    ↓
Context Deserializer
    ↓
Graph Executor (processes event)
    ↓
Context Serializer
    ↓
Next Event Topic
```

### Pattern 3: Saga Pattern

```
Start Transaction
    ↓
Service A (success)
    ↓
Service B (failure) ← triggers compensation
    ↓
Compensate Service A
    ↓
Rollback Context
```

## Performance Considerations

### Serialization Benchmark

| Format | Size | Serialize | Deserialize |
|--------|------|-----------|-------------|
| MessagePack | 186 bytes | 15 μs | 12 μs |
| JSON | 499 bytes | 45 μs | 38 μs |

**Recommendation:** Use MessagePack for production, JSON for debugging.

### Caching Strategies

| Strategy | Use Case | Performance |
|----------|----------|-------------|
| WriteThrough | Consistency critical | Medium write, fast read |
| WriteBehind | Write-heavy workload | Fast write, fast read |
| ReadThrough | Read-heavy workload | Fast read (cache hit) |
| CacheAside | Manual control | Flexible |

### Conflict Resolution Performance

| Strategy | Latency | Use Case |
|----------|---------|----------|
| LastWriteWins | O(1) | High throughput |
| HigherVersionWins | O(1) | Versioned data |
| FailOnConflict | O(1) | Critical data |
| MergeAll | O(n) | Non-conflicting updates |
| ThreeWayMerge | O(n) | Complex scenarios |

## Testing

### Unit Tests

```bash
# Run all distributed context tests
cargo test --lib distributed

# Run specific test
cargo test --lib distributed::context::tests::test_serialization
```

### Integration Tests

```bash
# Test with real Redis (requires Redis running)
cargo test --features redis distributed

# Test with in-memory store
cargo test distributed
```

## Production Deployment

### With Redis

```rust
#[cfg(feature = "redis")]
use rust_logic_graph::distributed::RedisStore;

let redis_url = "redis://localhost:6379";
let store = RedisStore::new(redis_url).await?;
let cache = DistributedCache::new(store);
```

### With Kubernetes

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: graph-config
data:
  graph.yaml: |
    # Your distributed_context_graph.yaml content
```

### Environment Variables

```bash
# Redis configuration
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=50

# Context configuration
CONTEXT_TTL_SECONDS=3600
MAX_CONTEXT_SIZE_BYTES=1048576

# Conflict resolution
DEFAULT_RESOLUTION_STRATEGY=LastWriteWins
```

## Monitoring & Observability

### Metrics to Track

- Context serialization time
- Context size distribution
- Cache hit/miss ratio
- Conflict resolution frequency
- Version history depth

### Logging

```rust
use tracing::{info, warn};

info!(
    session_id = %context.session_id,
    version = context.metadata.version,
    "Context updated"
);

warn!(
    session_id = %context.session_id,
    conflicts = conflicts.len(),
    "Conflicts detected during merge"
);
```

## Next Steps

1. **Implement Saga Pattern** - See ROADMAP v0.10.0
2. **Add Fault Tolerance** - Circuit breakers with shared state
3. **Observability** - OpenTelemetry integration
4. **Event-Driven** - Kafka/NATS event sources

## Resources

- [ROADMAP.md](../ROADMAP.md) - v0.10.0 Distributed Systems features
- [Main README](../README.md) - Getting started guide
- [examples/README.md](README.md) - All examples overview
- [Source Code](../src/distributed/) - Implementation details

## Support

Questions or issues? Open an issue on GitHub!
