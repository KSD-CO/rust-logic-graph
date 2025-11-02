# 🧠 Rust Logic Graph

A modular **reasoning graph framework** built in Rust, designed to orchestrate rule-based and AI-based nodes across a distributed system. Build complex workflows with conditional execution, topological ordering, and async node processing.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ Features

- 🔄 **Topological Execution** - Automatic DAG-based node ordering
- 📊 **Multiple Node Types** - RuleNode, DBNode, AINode
- 🎯 **Rule Engine** - Built-in expression evaluator with comparisons and logic
- ⚡ **Async Runtime** - Built on Tokio for concurrent execution
- 📝 **JSON Configuration** - Define graphs in simple JSON format
- 🔍 **Rich Context** - Share data between nodes seamlessly
- 🪵 **Tracing Support** - Built-in logging with `tracing`

---

## ⚙️ Architecture Overview

```
 +------------------+
 | Logic Graph Core |
 |   - GraphDef     |
 |   - Context      |
 |   - Executor     |
 +---------+--------+
           |
           v
 +------------------+
 | Node Layer       |
 | - RuleNode       | → Evaluates conditions
 | - DBNode         | → Database operations
 | - AINode         | → AI/LLM processing
 +---------+--------+
           |
           v
 +------------------+
 | Orchestrator     |
 | Async flow & rule |
 | evaluation engine |
 +------------------+
```

---

## 🧩 Module Summary

| Module | Description | Key Types |
|---------|-------------|-----------|
| `core` | Graph structure and execution | `Graph`, `GraphDef`, `Edge`, `Context`, `Executor` |
| `node` | Node implementations | `Node` trait, `RuleNode`, `DBNode`, `AINode` |
| `rule` | Conditional logic evaluation | `Rule`, `RuleResult`, `RuleError` |
| `orchestrator` | Workflow coordination | `Orchestrator` |
| `io` | Graph serialization | `GraphIO` |

---

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-logic-graph = "0.1.0"
```

### Basic Usage

```rust
use rust_logic_graph::{Graph, Orchestrator, GraphIO};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load graph from JSON
    let def = GraphIO::load_from_file("graph.json")?;

    // Create and execute graph
    let mut graph = Graph::new(def);
    Orchestrator::execute_graph(&mut graph).await?;

    // Access results
    println!("{:?}", graph.context.data);
    Ok(())
}
```

### Define a Graph (JSON)

```json
{
  "nodes": {
    "validate": "RuleNode",
    "fetch_data": "DBNode",
    "process": "AINode"
  },
  "edges": [
    { "from": "validate", "to": "fetch_data", "rule": "is_valid" },
    { "from": "fetch_data", "to": "process", "rule": "has_data" }
  ]
}
```

---

## 📚 Examples

### Simple Flow

```bash
cargo run --example simple_flow
```

A basic 3-node pipeline: `RuleNode → DBNode → AINode`

### Advanced Flow

```bash
cargo run --example advanced_flow
```

Complex workflow with:
- Input validation
- Permission checks
- Conditional branching
- Analytics generation
- Notification system

---

## 🔧 Node Types

### RuleNode

Evaluates conditions and transforms data.

```rust
use rust_logic_graph::RuleNode;

let node = RuleNode::new("check_age", "age > 18");
```

**Supported Operators:**
- Comparisons: `>`, `<`, `>=`, `<=`, `==`, `!=`
- Logical: `&&`, `||`
- Literals: numbers, strings (quoted), booleans

### DBNode

Simulates database operations.

```rust
use rust_logic_graph::DBNode;

let node = DBNode::new("fetch_users", "SELECT * FROM users");
```

Returns mock data with async delay (configurable).

### AINode

Simulates AI/LLM processing.

```rust
use rust_logic_graph::AINode;

let node = AINode::new("summarize", "Summarize the data");
```

Returns mock AI responses with context awareness.

---

## 🎯 Rule Evaluation

The built-in rule engine supports:

```rust
use rust_logic_graph::Rule;
use std::collections::HashMap;
use serde_json::json;

let mut context = HashMap::new();
context.insert("age".to_string(), json!(25));
context.insert("verified".to_string(), json!(true));

// Comparisons
let rule = Rule::new("age_check", "age > 18");
assert!(rule.evaluate(&context).is_ok());

// Logical operations
let rule = Rule::new("check", "age > 18 && verified");
assert!(rule.evaluate(&context).is_ok());

// Equality
let rule = Rule::new("exact", "age == 25");
assert!(rule.evaluate(&context).is_ok());
```

---

## 🔄 Execution Flow

1. **Graph Definition** - Load from JSON or build programmatically
2. **Node Registration** - Executor creates node instances
3. **Topological Sort** - Determines execution order
4. **Rule Evaluation** - Checks edge conditions
5. **Node Execution** - Runs nodes asynchronously
6. **Context Updates** - Nodes store results in shared context

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_comparison
```

---

## 📊 Roadmap

- [x] Core graph structure
- [x] Basic node types (Rule, DB, AI)
- [x] Topological execution
- [x] Rule evaluation engine
- [x] JSON I/O
- [x] Async execution
- [ ] Real database integration
- [ ] Real AI/LLM integration (OpenAI, Anthropic)
- [ ] GraphQL API
- [ ] Web UI for graph visualization
- [ ] Plugin system
- [ ] Distributed execution
- [ ] Performance optimizations

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

## 👤 Author

**James Vu** - [GitHub](https://github.com/jamesvu)

---

## 🙏 Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Petgraph](https://github.com/petgraph/petgraph) - Graph data structures
- [Serde](https://serde.rs/) - Serialization
- [Tracing](https://tracing.rs/) - Logging
