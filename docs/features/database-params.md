# DBNode Parameters Feature

## Overview

The params feature allows DBNode to extract parameter values from the execution context and pass them to SQL queries. This enables dynamic, data-driven database queries without hardcoding values.

## Problem Solved

**Before**: DBNode queries were static with no way to use dynamic values:
```rust
// ❌ Query always fetches the same product
DBNode::new("fetch_product", "SELECT * FROM products WHERE id = 'PROD-001'")
```

**After**: DBNode can extract values from context:
```rust
// ✅ Query uses dynamic product_id from context
DBNode::with_params(
    "fetch_product", 
    "SELECT * FROM products WHERE id = $1",
    vec!["product_id".to_string()]
)
```

## Usage

### 1. Programmatic API

#### Single Parameter
```rust
use rust_logic_graph::{NodeConfig, Graph, GraphDef, Executor};
use std::collections::HashMap;

let mut nodes = HashMap::new();

// Create DBNode with one parameter
nodes.insert(
    "fetch_user".to_string(),
    NodeConfig::db_node_with_params(
        "SELECT * FROM users WHERE user_id = $1",
        vec!["user_id".to_string()],
    ),
);

let def = GraphDef { nodes, edges: vec![] };
let mut graph = Graph::new(def);

// Set the parameter value in context
graph.context.set("user_id", serde_json::json!("USER-123"));

// Execute
let mut executor = Executor::from_graph_def(&graph.def)?;
executor.execute(&mut graph).await?;
```

#### Multiple Parameters
```rust
// DBNode with multiple parameters
nodes.insert(
    "fetch_orders".to_string(),
    NodeConfig::db_node_with_params(
        "SELECT * FROM orders WHERE user_id = $1 AND status = $2",
        vec!["user_id".to_string(), "order_status".to_string()],
    ),
);

// Set multiple parameters
graph.context.set("user_id", serde_json::json!("USER-123"));
graph.context.set("order_status", serde_json::json!("pending"));
```

### 2. JSON Configuration

```json
{
  "nodes": {
    "fetch_user": {
      "node_type": "DBNode",
      "query": "SELECT * FROM users WHERE user_id = $1",
      "params": ["user_id"]
    },
    "fetch_orders": {
      "node_type": "DBNode",
      "query": "SELECT * FROM orders WHERE user_id = $1 AND status = $2",
      "params": ["user_id", "order_status"]
    }
  },
  "edges": [
    {
      "from": "fetch_user",
      "to": "fetch_orders"
    }
  ]
}
```

### 3. YAML Configuration

```yaml
nodes:
  fetch_user:
    node_type: DBNode
    query: "SELECT * FROM users WHERE user_id = $1"
    params:
      - user_id
  
  fetch_orders:
    node_type: DBNode
    query: "SELECT * FROM orders WHERE user_id = $1 AND status = $2"
    params:
      - user_id
      - order_status

edges:
  - from: fetch_user
    to: fetch_orders
```

## How It Works

1. **Configuration**: DBNode is configured with a list of context keys
2. **Extraction**: During execution, DBNode extracts values from context using the keys
3. **Conversion**: JSON values are converted to strings for SQL binding
4. **Binding**: Parameters are bound to SQL query placeholders (`$1`, `$2`, etc.)

### Parameter Extraction Logic

```rust
// Extract params from context based on param_keys
let params: Vec<String> = if let Some(keys) = &self.param_keys {
    keys.iter()
        .filter_map(|key| {
            ctx.get(key).map(|value| {
                // Convert JSON value to string for SQL binding
                match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => value.to_string(),
                }
            })
        })
        .collect()
} else {
    vec![]
};
```

## Supported Value Types

The feature automatically converts JSON types to SQL-compatible strings:

| JSON Type | Conversion | Example |
|-----------|------------|---------|
| String | Direct use | `"USER-123"` → `"USER-123"` |
| Number | `.to_string()` | `42.5` → `"42.5"` |
| Boolean | `.to_string()` | `true` → `"true"` |
| Null | `"null"` | `null` → `"null"` |
| Object/Array | `.to_string()` | `{"a": 1}` → `"{\"a\":1}"` |

## Database Compatibility

### PostgreSQL
Uses `$1`, `$2`, `$3` placeholders:
```rust
NodeConfig::db_node_with_params(
    "SELECT * FROM users WHERE id = $1 AND status = $2",
    vec!["user_id".to_string(), "status".to_string()],
)
```

### MySQL
Uses `?` placeholders (converted internally by executor):
```rust
NodeConfig::db_node_with_params(
    "SELECT * FROM users WHERE id = ? AND status = ?",
    vec!["user_id".to_string(), "status".to_string()],
)
```

## Error Handling

### Missing Parameters
If a parameter key doesn't exist in context, it's **skipped** (not an error):
```rust
// Context has: user_id = "USER-123"
// Config expects: ["user_id", "missing_key"]
// Result: Only user_id is extracted, missing_key is ignored
```

### Empty Parameters
If no params are configured or all are missing:
```rust
// Empty params list is passed to executor
executor.execute(&self.query, &[]).await
```

## Best Practices

### 1. Clear Naming Convention
```rust
// ✅ Good: Clear parameter names
params: vec!["user_id", "product_id", "order_status"]

// ❌ Bad: Generic names
params: vec!["param1", "param2", "param3"]
```

### 2. Set Parameters Before Execution
```rust
// ✅ Good: Set all params before execute
graph.context.set("user_id", json!("USER-123"));
graph.context.set("product_id", json!("PROD-001"));
executor.execute(&mut graph).await?;

// ⚠️ Warning: Missing params are silently skipped
// Set params in earlier nodes or initial context
```

### 3. Use Consistent Parameter Keys
```rust
// ✅ Good: Same key used across nodes
nodes.insert("fetch_user", NodeConfig::db_node_with_params(
    "SELECT * FROM users WHERE id = $1",
    vec!["user_id"],
));
nodes.insert("fetch_orders", NodeConfig::db_node_with_params(
    "SELECT * FROM orders WHERE user_id = $1",
    vec!["user_id"], // Same key
));
```

### 4. Document Required Parameters
```yaml
# ✅ Good: Document what parameters are needed
nodes:
  fetch_product:
    node_type: DBNode
    query: "SELECT * FROM products WHERE id = $1"
    params:
      - product_id  # Required: Must be set in initial context
```

## Complete Example

```rust
use rust_logic_graph::{Graph, GraphDef, NodeConfig, Edge, Executor};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut nodes = HashMap::new();
    
    // Step 1: Fetch user data
    nodes.insert(
        "fetch_user".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM users WHERE user_id = $1",
            vec!["user_id".to_string()],
        ),
    );
    
    // Step 2: Fetch user's orders
    nodes.insert(
        "fetch_orders".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM orders WHERE user_id = $1 AND status = $2",
            vec!["user_id".to_string(), "order_status".to_string()],
        ),
    );
    
    // Step 3: Calculate total
    nodes.insert(
        "calculate_total".to_string(),
        NodeConfig::rule_node("true"),
    );

    let edges = vec![
        Edge {
            from: "fetch_user".to_string(),
            to: "fetch_orders".to_string(),
            rule: None,
        },
        Edge {
            from: "fetch_orders".to_string(),
            to: "calculate_total".to_string(),
            rule: None,
        },
    ];

    let def = GraphDef { nodes, edges };
    let mut graph = Graph::new(def);

    // Set initial parameters
    graph.context.set("user_id", serde_json::json!("USER-123"));
    graph.context.set("order_status", serde_json::json!("pending"));

    // Execute graph
    let mut executor = Executor::from_graph_def(&graph.def)?;
    executor.execute(&mut graph).await?;

    println!("Execution complete!");
    println!("Results: {:?}", graph.context.data);

    Ok(())
}
```

## Migration Guide

### From Static Queries
```rust
// Before
DBNode::new("fetch_user", "SELECT * FROM users WHERE id = 'USER-123'")

// After
DBNode::with_params(
    "fetch_user",
    "SELECT * FROM users WHERE id = $1",
    vec!["user_id".to_string()]
)
// And set context
graph.context.set("user_id", json!("USER-123"));
```

### From Hardcoded Values
```rust
// Before: Query constructed with string formatting
let query = format!("SELECT * FROM users WHERE id = '{}'", user_id);
DBNode::new("fetch_user", query)

// After: Use parameterized query
DBNode::with_params(
    "fetch_user",
    "SELECT * FROM users WHERE id = $1",
    vec!["user_id".to_string()]
)
```

## Testing

See `tests/db_params_tests.rs` for comprehensive test examples:
- Single parameter extraction
- Multiple parameter extraction
- Missing parameters (graceful degradation)
- Different value types (string, number, boolean)
- JSON serialization/deserialization
- Backward compatibility (nodes without params)

## API Reference

### NodeConfig
```rust
impl NodeConfig {
    /// Create DBNode with query parameters from context
    pub fn db_node_with_params(
        query: impl Into<String>, 
        params: Vec<String>
    ) -> Self;
}
```

### DBNode
```rust
impl DBNode {
    /// Create DBNode with parameter keys to extract from context
    pub fn with_params(
        id: impl Into<String>,
        query: impl Into<String>,
        param_keys: Vec<String>,
    ) -> Self;
    
    /// Create DBNode with custom executor and parameter keys
    pub fn with_executor_and_params(
        id: impl Into<String>,
        query: impl Into<String>,
        executor: Arc<dyn DatabaseExecutor>,
        param_keys: Vec<String>,
    ) -> Self;
}
```

## See Also

- [examples/db_params_flow.rs](../examples/db_params_flow.rs) - Complete working example
- [examples/db_params_graph.yaml](../examples/db_params_graph.yaml) - JSON configuration example
- [tests/db_params_tests.rs](../tests/db_params_tests.rs) - Test suite
- [case_study/monolithic/](../case_study/monolithic/) - Real-world usage in purchasing flow
