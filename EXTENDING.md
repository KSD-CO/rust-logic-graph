# 🔧 Extending Rust Logic Graph

Guide to extend và customize Rust Logic Graph framework.

---

## 📋 Table of Contents

1. [Creating Custom Node Types](#creating-custom-node-types)
2. [Adding New Rule Operators](#adding-new-rule-operators)
3. [Custom Graph Loaders](#custom-graph-loaders)
4. [Integrating Real Services](#integrating-real-services)
5. [Performance Optimization](#performance-optimization)

---

## 1. Creating Custom Node Types

### Step 1: Define Your Node Struct

```rust
use rust_logic_graph::{Node, NodeType, Context, RuleResult};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct HttpNode {
    pub id: String,
    pub url: String,
    pub method: String,
}

impl HttpNode {
    pub fn new(id: impl Into<String>, url: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            method: method.into(),
        }
    }
}
```

### Step 2: Implement the Node Trait

```rust
#[async_trait]
impl Node for HttpNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::HttpNode  // You'll need to add this to the enum
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        // Your HTTP logic here
        let client = reqwest::Client::new();
        let response = match self.method.as_str() {
            "GET" => client.get(&self.url).send().await,
            "POST" => client.post(&self.url).send().await,
            _ => return Err(RuleError::Eval("Unsupported method".into())),
        };

        let json = response
            .map_err(|e| RuleError::Eval(e.to_string()))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| RuleError::Eval(e.to_string()))?;

        ctx.data.insert(format!("{}_result", self.id), json.clone());
        Ok(json)
    }
}
```

### Step 3: Register with Executor

```rust
let mut executor = Executor::new();
executor.register_node(Box::new(HttpNode::new(
    "fetch_api",
    "https://api.example.com/data",
    "GET"
)));
```

---

## 2. Adding New Rule Operators

### Extend Rule Module

Edit `src/rule/mod.rs`:

```rust
impl Rule {
    fn evaluate_comparison(&self, expr: &str, context: &HashMap<String, Value>) -> Option<RuleResult> {
        // Add new operators
        for op in ["==", "!=", ">=", "<=", ">", "<", "contains", "starts_with"] {
            if let Some((left, right)) = expr.split_once(op) {
                let left = left.trim();
                let right = right.trim();

                let left_val = self.get_value(left, context).ok()?;
                let right_val = self.get_value(right, context).ok()?;

                let result = match op {
                    // ... existing operators ...
                    "contains" => self.string_contains(&left_val, &right_val),
                    "starts_with" => self.string_starts_with(&left_val, &right_val),
                    _ => false,
                };

                return Some(Ok(Value::Bool(result)));
            }
        }
        None
    }

    fn string_contains(&self, left: &Value, right: &Value) -> bool {
        if let (Value::String(l), Value::String(r)) = (left, right) {
            return l.contains(r.as_str());
        }
        false
    }

    fn string_starts_with(&self, left: &Value, right: &Value) -> bool {
        if let (Value::String(l), Value::String(r)) = (left, right) {
            return l.starts_with(r.as_str());
        }
        false
    }
}
```

### Usage

```rust
let rule = Rule::new("email_check", "email contains \"@example.com\"");
let rule = Rule::new("prefix_check", "name starts_with \"John\"");
```

---

## 3. Custom Graph Loaders

### Load from YAML

```rust
use serde_yaml;

pub struct YamlGraphLoader;

impl YamlGraphLoader {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<GraphDef> {
        let data = fs::read_to_string(&path)?;
        let graph_def: GraphDef = serde_yaml::from_str(&data)?;
        Ok(graph_def)
    }
}
```

### Load from Database

```rust
use sqlx::PgPool;

pub struct DbGraphLoader {
    pool: PgPool,
}

impl DbGraphLoader {
    pub async fn load_graph(&self, graph_id: i32) -> Result<GraphDef> {
        let nodes = sqlx::query!("SELECT * FROM graph_nodes WHERE graph_id = $1", graph_id)
            .fetch_all(&self.pool)
            .await?;

        let edges = sqlx::query!("SELECT * FROM graph_edges WHERE graph_id = $1", graph_id)
            .fetch_all(&self.pool)
            .await?;

        // Convert to GraphDef
        // ...
    }
}
```

---

## 4. Integrating Real Services

### PostgreSQL Integration

```rust
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct PostgresNode {
    pub id: String,
    pub query: String,
    pub pool: PgPool,
}

#[async_trait]
impl Node for PostgresNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::DBNode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        let rows = sqlx::query(&self.query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RuleError::Eval(e.to_string()))?;

        // Convert rows to JSON
        let results: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                // Convert row to JSON based on your schema
                serde_json::json!({})
            })
            .collect();

        let result = serde_json::json!({
            "rows": results,
            "count": results.len()
        });

        ctx.data.insert(format!("{}_result", self.id), result.clone());
        Ok(result)
    }
}
```

### OpenAI Integration

```rust
use async_openai::{Client, types::CreateCompletionRequest};

#[derive(Debug, Clone)]
pub struct OpenAINode {
    pub id: String,
    pub prompt: String,
    pub api_key: String,
}

#[async_trait]
impl Node for OpenAINode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        let client = Client::new().with_api_key(&self.api_key);

        // Build context-aware prompt
        let context_data: String = ctx.data
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let full_prompt = format!("{}\n\nContext:\n{}", self.prompt, context_data);

        let request = CreateCompletionRequest {
            model: "gpt-4".to_string(),
            prompt: Some(full_prompt),
            max_tokens: Some(500),
            ..Default::default()
        };

        let response = client
            .completions()
            .create(request)
            .await
            .map_err(|e| RuleError::Eval(e.to_string()))?;

        let result = serde_json::json!({
            "response": response.choices[0].text,
            "model": "gpt-4",
            "tokens": response.usage.total_tokens
        });

        ctx.data.insert(format!("{}_result", self.id), result.clone());
        Ok(result)
    }
}
```

### Anthropic Claude Integration

```rust
use anthropic_sdk::{Client, types::Message};

#[derive(Debug, Clone)]
pub struct ClaudeNode {
    pub id: String,
    pub prompt: String,
    pub api_key: String,
}

#[async_trait]
impl Node for ClaudeNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::AINode
    }

    async fn run(&self, ctx: &mut Context) -> RuleResult {
        let client = Client::new(&self.api_key);

        let message = client
            .messages()
            .create(
                "claude-3-5-sonnet-20241022",
                &self.prompt,
                1024,
            )
            .await
            .map_err(|e| RuleError::Eval(e.to_string()))?;

        let result = serde_json::json!({
            "response": message.content[0].text,
            "model": "claude-3-5-sonnet",
            "stop_reason": message.stop_reason
        });

        ctx.data.insert(format!("{}_result", self.id), result.clone());
        Ok(result)
    }
}
```

---

## 5. Performance Optimization

### Parallel Node Execution

```rust
use tokio::task::JoinSet;

impl Executor {
    pub async fn execute_parallel(&self, graph: &mut Graph) -> Result<()> {
        // Group nodes by depth level
        let levels = self.compute_levels(&graph.def)?;

        for (_level, nodes) in levels {
            let mut join_set = JoinSet::new();

            for node_id in nodes {
                if let Some(node) = self.nodes.get(&node_id) {
                    let node = node.clone();
                    let mut ctx = graph.context.clone();

                    join_set.spawn(async move {
                        node.run(&mut ctx).await
                    });
                }
            }

            // Wait for all nodes in this level
            while let Some(result) = join_set.join_next().await {
                result??;
            }
        }

        Ok(())
    }
}
```

### Caching Results

```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct CachedExecutor {
    executor: Executor,
    cache: Arc<DashMap<String, serde_json::Value>>,
}

impl CachedExecutor {
    pub async fn execute(&self, graph: &mut Graph) -> Result<()> {
        for node_id in self.compute_order(&graph.def)? {
            let cache_key = format!("{}:{:?}", node_id, graph.context.data);

            if let Some(cached) = self.cache.get(&cache_key) {
                graph.context.data.insert(
                    format!("{}_result", node_id),
                    cached.clone()
                );
                continue;
            }

            // Execute and cache
            if let Some(node) = self.executor.nodes.get(&node_id) {
                let result = node.run(&mut graph.context).await?;
                self.cache.insert(cache_key, result);
            }
        }

        Ok(())
    }
}
```

### Metrics Collection

```rust
use std::time::Instant;

pub struct MetricsCollector {
    node_durations: DashMap<String, Duration>,
    execution_count: DashMap<String, u64>,
}

impl MetricsCollector {
    pub async fn execute_with_metrics(
        &self,
        node: &dyn Node,
        ctx: &mut Context,
    ) -> RuleResult {
        let start = Instant::now();
        let result = node.run(ctx).await;
        let duration = start.elapsed();

        self.node_durations.insert(node.id().to_string(), duration);
        *self.execution_count.entry(node.id().to_string()).or_insert(0) += 1;

        result
    }

    pub fn report(&self) {
        println!("=== Execution Metrics ===");
        for entry in self.node_durations.iter() {
            println!("{}: {:?}", entry.key(), entry.value());
        }
    }
}
```

---

## 🎯 Best Practices

1. **Error Handling**: Always use proper error types
2. **Logging**: Use `tracing` for debug information
3. **Testing**: Write unit tests for custom nodes
4. **Documentation**: Document your extensions
5. **Type Safety**: Leverage Rust's type system
6. **Async**: Use async/await for I/O operations

---

## 📚 Resources

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Petgraph Documentation](https://docs.rs/petgraph/)
- [Serde Guide](https://serde.rs/)

---

Happy extending! 🚀
