# Purchasing Flow: Mock vs Real Database Comparison

## Overview

| Feature | Mock Version | Real DB Version |
|---------|--------------|-----------------|
| File | `purchasing_flow.rs` | `purchasing_flow_realdb.rs` |
| Data Source | Hardcoded JSON | MySQL Database |
| Databases | None | 4 separate databases |
| Dependencies | Basic | `sqlx`, `chrono` |
| Cargo Features | None | `--features mysql` |
| Setup Required | None | Database setup needed |
| Realistic | Demo only | Production-like |

## Architecture Comparison

### Mock Version (purchasing_flow.rs)

```
┌─────────────┐
│   DBNode    │  Returns: json!({"avg": 10})
│ (mock data) │  Connection: None
└─────────────┘
```

**Characteristics:**
- No external dependencies
- Instant execution
- Predictable results
- Good for demos/testing
- No setup required

### Real DB Version (purchasing_flow_realdb.rs)

```
┌─────────────┐
│ MySQLDBNode │  Query: SELECT * FROM oms_history...
│  (oms_db)   │  Connection: Pool<MySql>
└─────────────┘
       │
       ▼
┌─────────────┐
│  MySQL DB   │  Host: 171.244.10.40:6033
│   oms_db    │  User: lune_dev
└─────────────┘
```

**Characteristics:**
- Real database connections
- Network I/O involved
- Dynamic data
- Production patterns
- Requires setup

## Code Comparison

### Mock DBNode (Original)

```rust
// Simple DBNode that returns mock data
exec.register_node(Box::new(
    rust_logic_graph::node::DBNode::new(
        "oms_history",
        "SELECT * FROM oms_history"
    )
));

// Returns hardcoded data:
// {"avg": 10}
```

### Real DBNode (New)

```rust
// Custom node with real MySQL connection
struct MySQLDBNode {
    id: String,
    query: String,
    pool: Pool<MySql>,  // Real connection pool
    db_name: String,
}

exec.register_node(Box::new(
    MySQLDBNode::new(
        "oms_history",
        "SELECT product_id, avg_daily_demand, trend
         FROM oms_history WHERE product_id = 'PROD-001'",
        OMS_DB  // Connects to oms_db
    ).await?
));

// Returns real data from database:
// {"product_id": "PROD-001", "avg_daily_demand": 15.5, "trend": "increasing"}
```

## Data Flow Comparison

### Mock Version

```
User Input
    ↓
Graph Execution
    ↓
DBNode.run() → returns json!({"avg": 10})  [instant]
    ↓
Rule Engine
    ↓
Calculate Order Qty (using mock data)
    ↓
Create PO
    ↓
Send PO
```

### Real DB Version

```
User Input
    ↓
Graph Execution
    ↓
MySQLDBNode.run()
    ↓
Connect to oms_db (171.244.10.40:6033)  [network I/O]
    ↓
Execute SQL: SELECT ... FROM oms_history
    ↓
Parse results → JSON
    ↓
Rule Engine (combines data from 4 DBs)
    ↓
Calculate Order Qty (using real data)
    ↓
Create PO
    ↓
Send PO
```

## Dependencies

### Mock Version (Cargo.toml)

```toml
[dependencies]
rust-logic-graph = "0.7.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
async-trait = "0.1"
anyhow = "1"

# No build command needed
# cargo run --example purchasing_flow
```

### Real DB Version (Cargo.toml)

```toml
[dependencies]
rust-logic-graph = "0.7.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
async-trait = "0.1"
anyhow = "1"
chrono = "0.4"
sqlx = { version = "0.7", features = ["mysql"] }

# Requires mysql feature
# cargo run --example purchasing_flow_realdb --features mysql
```

## Setup Comparison

### Mock Version

```bash
# No setup needed
cargo run --example purchasing_flow
```

✅ Runs immediately
✅ No external dependencies
✅ Consistent results

### Real DB Version

```bash
# Step 1: Setup databases (one-time)
./examples/setup_databases.sh

# Step 2: Run example
cargo run --example purchasing_flow_realdb --features mysql
```

✅ Realistic behavior
✅ Tests real integrations
✅ Production-ready patterns
⚠️ Requires database access
⚠️ Setup needed

## Performance Comparison

| Metric | Mock | Real DB |
|--------|------|---------|
| Startup | Instant | ~500ms (connection) |
| Query | 0ms | ~10-50ms per query |
| Total Time | ~1ms | ~200-500ms |
| Memory | Minimal | Connection pool |
| Network | None | Required |

## Use Cases

### When to Use Mock Version

✅ Learning the framework
✅ Quick demos
✅ Unit testing logic
✅ No database available
✅ Rapid prototyping
✅ Documentation examples

### When to Use Real DB Version

✅ Integration testing
✅ Performance testing
✅ Production simulation
✅ Multi-database patterns
✅ Real data validation
✅ End-to-end testing
✅ Demonstrating architecture

## Output Comparison

### Mock Version Output

```
Final context: {
  "oms_history": {"avg": 10},
  "inventory_levels": {"qty": 3},
  "supplier_info": {"moq": 1},
  "calc_order_qty_result": {"order_qty": 7},
  "po": {"po_id": "PO-12345", "qty": 7},
  "po_sent": {"po_id": "PO-12345", "qty": 7, "sent": true}
}
```

### Real DB Version Output

```
=== Purchasing Flow with Real MySQL Databases ===
Each node connects to a separate database:
  - OMS Node        -> oms_db
  - Inventory Node  -> inventory_db
  - Supplier Node   -> supplier_db
  - UOM Node        -> uom_db

Creating database connections for each node...
  [oms_history] Connecting to oms_db...
  [inventory_levels] Connecting to inventory_db...
  [supplier_info] Connecting to supplier_db...
  [uom_conversion] Connecting to uom_db...

[oms_history] Database: oms_db | Executing query: SELECT...
[oms_history] Result: {"product_id":"PROD-001","avg_daily_demand":15.5,"trend":"increasing"}

[inventory_levels] Database: inventory_db | Executing query: SELECT...
[inventory_levels] Result: {"product_id":"PROD-001","warehouse_id":"WH-001","current_qty":25,"reserved_qty":5,"available_qty":20}

[supplier_info] Database: supplier_db | Executing query: SELECT...
[supplier_info] Result: {"supplier_id":"SUP-001","product_id":"PROD-001","moq":20,"lead_time_days":7,"unit_price":15.99}

[calc_order_qty] Calculating order quantity...
[calc_order_qty] Result: {"order_qty":100,"avg_demand":15.5,"available_qty":20,"demand_during_lead_time":108.5,"shortage":88.5,"moq":20}

[create_po] Creating purchase order...
[create_po] PO created: {"po_id":"PO-1731715200","product_id":"PROD-001","supplier_id":"SUP-001","qty":100,"unit_price":15.99,"total_amount":1599.0,"status":"draft"}

[send_po] Sending purchase order...
[send_po] PO sent: {"po_id":"PO-1731715200","product_id":"PROD-001","supplier_id":"SUP-001","qty":100,"unit_price":15.99,"total_amount":1599.0,"status":"sent"}
```

## Error Handling Comparison

### Mock Version

```rust
// Simple, no real errors
async fn run(&self, ctx: &mut Context) -> RuleResult {
    let data = json!({"avg": 10});
    ctx.data.insert(self.id.clone(), data.clone());
    Ok(data)
}
```

### Real DB Version

```rust
// Comprehensive error handling
async fn run(&self, ctx: &mut Context) -> RuleResult {
    let rows = sqlx::query(&self.query)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RuleError::Eval(
            format!("Database query error on {}: {}", self.db_name, e)
        ))?;

    // Handle connection errors
    // Handle query errors
    // Handle parsing errors
    // ...
}
```

## Summary Matrix

| Aspect | Mock | Real DB | Winner |
|--------|------|---------|--------|
| **Ease of Use** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Mock |
| **Setup Speed** | ⭐⭐⭐⭐⭐ | ⭐⭐ | Mock |
| **Realism** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Real DB |
| **Production Ready** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Real DB |
| **Testing Value** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Real DB |
| **Learning Curve** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Mock |
| **Architecture Demo** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Real DB |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Mock |

## Recommendation

**Start with Mock** (`purchasing_flow.rs`)
- Learning the framework
- Understanding the flow
- Quick experiments

**Graduate to Real DB** (`purchasing_flow_realdb.rs`)
- Building production systems
- Integration testing
- Demonstrating to stakeholders
- Performance testing

Both versions are valuable and serve different purposes!
