# 🧠 Rust Logic Graph

**Current Version:** 0.12.0

> **Reasoning Engine for Distributed Backend & AI Orchestration**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/github-KSD--CO%2Frust--logic--graph-blue.svg)](https://github.com/KSD-CO/rust-logic-graph)
[![CI](https://github.com/KSD-CO/rust-logic-graph/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-logic-graph/actions)

A high-performance **reasoning engine** for distributed backend systems and AI orchestration. Build complex decision workflows, coordinate multiple services, and create intelligent agent systems with **GRL (Grule Rule Language)** support.

**Not a no-code automation tool** - Rust Logic Graph is an embeddable library for developers building distributed reasoning systems, not a UI-first workflow platform like n8n or Zapier.

---

## 🎯 What is Rust Logic Graph?

Rust Logic Graph is a **reasoning engine library** for building intelligent backend systems:

### Core Capabilities

**🧠 Distributed Reasoning**
- Connect decisions across multiple databases and services
- Build complex decision trees with business rules (GRL)
- Maintain context as data flows through your system
- Explain how decisions were reached

**🤖 AI Agent Orchestration**
- Coordinate multiple LLMs in reasoning chains
- Build RAG (Retrieval-Augmented Generation) pipelines
- Create multi-agent systems with tool calling
- Native support for OpenAI, Claude, Ollama, and custom models

**⚡ High-Performance Execution**
- Sub-millisecond latency (embedded library, not service)
- Automatic parallel execution of independent operations
- Memory-efficient context pooling
- Async/await throughout

**🔧 Production-Ready Patterns**
- Circuit breakers for unstable services
- Retry logic with exponential backoff
- Try/catch error handling
- Saga pattern for distributed transactions (v0.12.0)

## ✨ Key Features

- 🔥 **GRL Support** - [rust-rule-engine v1.18.0-alpha](https://crates.io/crates/rust-rule-engine) with **50-100x faster parsing**
- 🔄 **Topological Execution** - Automatic DAG-based node ordering
- ⚡ **Async Runtime** - Built on Tokio for high concurrency
- ⚡ **Parallel Execution** - Automatic parallel execution of independent nodes (v0.5.0)
- 🗄️ **Multi-Database Orchestration** - Parallel queries, correlation, distributed transactions (v0.10.0) 🆕
- 💾 **Caching Layer** - High-performance result caching with TTL, eviction policies, and memory limits (v0.5.0)
- 🧠 **Memory Optimization** - Context pooling and allocation tracking (v0.7.0)
- 🛠️ **CLI Developer Tools** - Graph validation, dry-run, profiling, and visualization (v0.5.0)
- 🎨 **Web Graph Editor** - Next.js visual editor with drag-and-drop interface (v0.8.0)
- 📋 **YAML Configuration** - Declarative graph definitions with external config files (v0.8.5)
- 🎯 **Advanced Control Flow** - Subgraphs, conditionals, loops, error handling (v0.9.0)
- 🚨 **Rich Error Messages** - Unique error codes, actionable suggestions, full context (v0.10.0) 🆕
- 📊 **Multiple Node Types** - RuleNode, DBNode, AINode, ConditionalNode, LoopNode, TryCatchNode, RetryNode, CircuitBreakerNode
- 📝 **JSON/YAML Configuration** - Simple workflow definitions
- 🎯 **98% Drools Compatible** - Easy migration from Java
- 🌊 **Streaming Processing** - Stream-based execution with backpressure (v0.3.0)
- 🗄️ **Database Integrations** - PostgreSQL, MySQL, Redis, MongoDB (v0.2.0)
- 🤖 **AI/LLM Integrations** - OpenAI, Claude, Ollama (v0.2.0)
- 🛡️ **Saga Pattern** - Distributed transaction coordinator, compensation, state persistence, timeout/deadline (v0.12.0)
- 🛒 **E-commerce Saga Example** - Real-world order flow with compensation and rollback (v0.12.0)

---

## 🚀 Quick Start

### Installation

```toml
[dependencies]
rust-logic-graph = "0.12.0"

# With specific integrations
rust-logic-graph = { version = "0.12.0", features = ["postgres", "openai"] }

# With all integrations
rust-logic-graph = { version = "0.12.0", features = ["all-integrations"] }
```

### Example Use Cases

**Saga Pattern (E-commerce order flow):**
```rust
let mut saga = SagaCoordinator::new(Some(Duration::from_secs(10)));
saga.add_step(SagaStep {
    id: "reserve_inventory".to_string(),
    action: Box::new(|ctx| { /* ... */ Ok(()) }),
    compensation: Some(Box::new(|ctx| { /* ... */ Ok(()) })),
    status: SagaStepStatus::Pending,
    timeout: Some(Duration::from_secs(3)),
});
// ... more steps (charge_payment, create_shipment, send_confirmation)
saga.execute()?;
```

**Financial risk assessment across multiple data sources**
```rust
let risk_engine = Graph::new()
    .add_node("credit_history", DBNode::postgres(...))
    .add_node("transaction_analysis", DBNode::mongodb(...))
    .add_node("fraud_check", AINode::openai(...))
    .add_node("risk_rules", RuleNode::grl("risk_assessment.grl"))
    .add_node("decision", ConditionalNode::new(...));
```

**Multi-step AI reasoning with tool calling**
```rust
let ai_agent = Graph::new()
    .add_node("understand_query", AINode::claude(...))
    .add_node("search_knowledge", SubgraphNode::new(rag_pipeline))
    .add_node("reason", AINode::openai_gpt4(...))
    .add_node("validate", RuleNode::grl("validation.grl"))
    .add_retry("reason", max_attempts: 3);
```

**Microservice coordination with fault tolerance**
```rust
let order_flow = Graph::new()
    .add_node("inventory", GrpcNode::new("inventory-service"))
    .add_node("payment", GrpcNode::new("payment-service"))
    .add_node("shipping", GrpcNode::new("shipping-service"))
    .add_circuit_breaker("payment", threshold: 5)
    .add_saga_compensation(...);
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[�🏢 Case Study: Purchasing Flow](case_study/docs/README.md)** | Real production system with microservices & monolithic implementations |
| **[📋 YAML Configuration Guide](case_study/YAML_CONFIGURATION_SUMMARY.md)** | Declarative graph configuration with YAML (NEW in v0.8.5) |
| **[Graph Editor Guide](graph-editor/README.md)** | Visual web-based graph editor with Next.js (NEW in v0.8.0) |
| **[Memory Optimization Guide](docs/MEMORY_OPTIMIZATION.md)** | Context pooling and allocation tracking (v0.7.0) |
| **[CLI Tool Guide](docs/CLI_TOOL.md)** | Developer tools for validation, profiling, and visualization (v0.5.0) |
| **[Cache Guide](docs/CACHE_IMPLEMENTATION.md)** | Caching layer with TTL and eviction policies (v0.5.0) |
| **[Migration Guide](docs/MIGRATION_GUIDE.md)** | Upgrade guide to v0.14.0 with RETE-UL (v0.5.0) |
| **[Integrations Guide](docs/INTEGRATIONS.md)** | Database & AI integrations (v0.2.0) |
| **[GRL Guide](docs/GRL.md)** | Complete GRL syntax and examples |
| **[Use Cases](docs/USE_CASES.md)** | 33+ real-world applications |
| **[Extending](docs/EXTENDING.md)** | Create custom nodes and integrations |
| **[Implementation](docs/IMPLEMENTATION_SUMMARY.md)** | Technical details |

---

## 🎯 Use Cases

Rust Logic Graph powers applications in:

- 💰 **Finance** - Loan approval, fraud detection, risk assessment
- 🛒 **E-commerce** - Dynamic pricing, recommendations, fulfillment
- 🏥 **Healthcare** - Patient triage, clinical decisions, monitoring
- 🏭 **Manufacturing** - Predictive maintenance, QC automation
- 🛡️ **Insurance** - Claims processing, underwriting
- 📊 **Marketing** - Lead scoring, campaign optimization
- ⚖️ **Compliance** - AML monitoring, GDPR automation

**[View all 33+ use cases →](docs/USE_CASES.md)**

---

## 🏗️ Architecture Patterns

### Pattern 1: Multi-Database Reasoning
Query multiple databases, apply business rules, make decisions:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  PostgreSQL │───▶│   MongoDB   │───▶│    Redis    │
│  (Users)    │    │ (Analytics) │    │   (Cache)   │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┴───────────────────┘
                           │
                   ┌───────▼───────┐
                   │  Rule Engine  │◀─── GRL Rules
                   │  (Decision)   │
                   └───────┬───────┘
                           │
                   ┌───────▼───────┐
                   │    Actions    │
                   └───────────────┘
```

### Pattern 2: AI Agent with Tools
LLM orchestration with tool calling and validation:

```
┌─────────────┐
│ User Query  │
└──────┬──────┘
       │
┌──────▼──────────┐
│ LLM (Claude)    │──────┐
│ Understand      │      │
└──────┬──────────┘      │
       │              Tool Calls
┌──────▼──────────┐      │
│ RAG Subgraph    │◀─────┤
│ (Vector Search) │      │
└──────┬──────────┘      │
       │              ┌───▼──────┐
┌──────▼──────────┐  │ Database │
│ LLM (GPT-4)     │◀─┤ Query    │
│ Reason          │  └──────────┘
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Validate (GRL)  │
│ Business Rules  │
└──────┬──────────┘
       │
┌──────▼──────────┐
│ Response        │
└─────────────────┘
```

### Pattern 3: Saga Pattern for Distributed Transactions
Coordinate microservices with compensation logic:

```
Order Service ──▶ Inventory Service ──▶ Payment Service ──▶ Shipping Service
     │                   │                    │                    │
   Success            Success             Success              Success
     │                   │                    │                    │
     └───────────────────┴────────────────────┴────────────────────┘
                                  │
                           ┌──────▼──────┐
                           │  Complete   │
                           └─────────────┘

If Payment Fails:
     │                   │                    ✗
     │                   │              Compensation
     │                   │◀───────────────────┘
     │            Release Inventory
     │◀──────────────────┘
  Cancel Order
```

**[View 6 complete architecture patterns →](docs/ARCHITECTURE_PATTERNS.md)**

---

## 🏗️ System Architecture

![Rust Logic Graph architecture diagram](https://raw.githubusercontent.com/KSD-CO/rust-logic-graph/main/docs/images/rust-logic-graph-architect.png)

---

## 🔥 GRL Example

```grl
rule "HighValueLoan" salience 100 {
    when
        loan_amount > 100000 &&
        credit_score < 750
    then
        requires_manual_review = true;
        approval_tier = "senior";
}

rule "AutoApproval" salience 50 {
    when
        credit_score >= 700 &&
        income >= loan_amount * 3 &&
        debt_ratio < 0.4
    then
        auto_approve = true;
        interest_rate = 3.5;
}
```

**[Learn more about GRL →](docs/GRL.md)**

---

## 📊 Performance

### GRL Parsing Benchmark (v1.18.0-alpha)

| Benchmark | v1.18.0-alpha | v1.3.0 | Speedup |
|-----------|---------------|--------|---------|
| Simple Rule (1 rule) | **2.95 µs** | 298 µs | **101x faster** |
| Complex Rule (1 rule) | **10.27 µs** | 305 µs | **30x faster** |
| Purchasing Rules (11 rules) | **71.6 µs** | 3,500 µs | **49x faster** |
| Rule Execution (11 rules) | **32.66 µs** | - | - |

### Graph Execution Benchmark (vs dagrs)

| Chain Size | rust-logic-graph | dagrs | Speedup |
|------------|------------------|-------|---------|
| 5 nodes | **271 µs** | 1.7 ms | **6.3x faster** |
| 10 nodes | **526 µs** | 1.8 ms | **3.5x faster** |
| 20 nodes | **996 µs** | 2.1 ms | **2.1x faster** |

### Key Performance Features

- **RETE-UL Algorithm**: Advanced pattern matching with unlinking
- **No-Regex Parser**: v1.18.0-alpha uses hand-written parser (50-100x faster)
- **98% Drools Compatible**: Easy migration path
- **Async by Default**: High concurrency support
- **Parallel Execution**: Automatic layer-based parallelism
- **Smart Caching**: Result caching with TTL and eviction policies

---

## 🧪 Testing & CLI Tools

```bash
# Run all tests
cargo test

# Build CLI tool (YAML-only support)
cargo build --release --bin rlg

# Validate graph
./target/release/rlg validate --file examples/sample_graph.yaml

# Visualize graph structure
./target/release/rlg visualize --file examples/sample_graph.yaml --details

# Profile performance
./target/release/rlg profile --file examples/sample_graph.yaml --iterations 100

# Dry-run execution
./target/release/rlg dry-run --file examples/sample_graph.yaml --verbose
```

**Test Results**: ✅ 74/74 tests passing  
**CLI Format**: YAML only (`.yaml` or `.yml` files)

**[Learn more about CLI tools →](docs/CLI_TOOL.md)**

---

## 📦 Project Status

**Version**: 0.11.0 (Latest)
**Status**: Production-ready with YAML-driven multi-database orchestration

### What's Working
- ✅ Core graph execution engine
- ✅ **RETE-UL algorithm** (v0.14.0) - 2-24x faster
- ✅ Three node types (Rule, DB, AI)
- ✅ Topological sorting
- ✅ Async execution
- ✅ JSON I/O
- ✅ **Database integrations** (PostgreSQL, MySQL, Redis, MongoDB)
- ✅ **AI integrations** (OpenAI, Claude, Ollama)
- ✅ **Streaming processing** with backpressure and chunking
- ✅ **Parallel execution** with automatic layer detection
- ✅ **Caching layer** with TTL, eviction policies, memory limits (v0.5.0)
- ✅ **Memory optimization** with context pooling (v0.7.0)
- ✅ **CLI Developer Tools** - validate, profile, visualize, dry-run (v0.5.0)
- ✅ **Web Graph Editor** - Next.js visual editor with drag-and-drop (v0.8.0)
- ✅ **Production Case Study** - Purchasing flow with microservices & monolithic (v0.8.0)
- ✅ **YAML Configuration** - Declarative graph definitions (v0.8.5)
 - ✅ **Distributed Context Sharing** - MessagePack serialization, shared context, versioning and stores (InMemory/Redis) (v0.11.0)
 - 🛡️ ✅ **Fault Tolerance** - Circuit breakers, health monitoring, failover, graceful degradation (v0.11.0)
- ✅ Stream operators (map, filter, fold)
- ✅ Comprehensive documentation

### Roadmap
- [x] Streaming processing (v0.3.0) - COMPLETED ✅
- [x] Parallel node execution (v0.4.0) - COMPLETED ✅
- [x] Caching layer (v0.5.0) - COMPLETED ✅
- [x] CLI Developer Tools (v0.5.0) - COMPLETED ✅
- [x] RETE-UL upgrade (v0.5.0) - COMPLETED ✅
- [x] Memory Optimization (v0.7.0) - COMPLETED ✅
- [x] Web Graph Editor (v0.8.0) - COMPLETED ✅
- [x] Production Case Study (v0.8.0) - COMPLETED ✅
- [x] YAML Configuration (v0.8.5) - COMPLETED ✅
- [ ] GraphQL API (v0.9.0)
- [ ] Production release (v1.0.0)

**See [ROADMAP.md](ROADMAP.md) for details**

---

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create your feature branch
3. Write tests for new features
4. Submit a pull request

---

## 📖 Examples

### Core Examples

| Example | Description | Lines |
|---------|-------------|-------|
| `simple_flow.rs` | Basic 3-node pipeline | 36 |
| `advanced_flow.rs` | Complex 6-node workflow | 120 |
| `grl_rules.rs` | GRL rule examples | 110 |
| `grl_graph_flow.rs` | GRL + Graph integration | 140 |
| `postgres_flow.rs` | PostgreSQL integration | 100 |
| `openai_flow.rs` | OpenAI GPT integration | 150 |
| `streaming_flow.rs` | Streaming with backpressure | 200 |
| `parallel_execution.rs` | Parallel node execution | 250 |

### Advanced Control Flow Examples (v0.9.0) 🆕

| Example | Description | Features Demonstrated |
|---------|-------------|----------------------|
| `conditional_flow.rs` | If/else routing based on conditions | ConditionalNode, branch selection |
| `loop_flow.rs` | Foreach and while loop patterns | LoopNode, iteration over arrays |
| `retry_flow.rs` | Exponential backoff retry logic | RetryNode, configurable attempts |
| `error_handling_flow.rs` | Try/catch/finally patterns | TryCatchNode, error recovery |
| `circuit_breaker_flow.rs` | Circuit breaker fault tolerance | CircuitBreakerNode, failure thresholds |
| `subgraph_flow.rs` | Nested graph execution | SubgraphNode, input/output mapping |

### Multi-Database Orchestration (v0.10.0) 🆕

| Example | Description | Features Demonstrated |
|---------|-------------|----------------------|
| `real_multi_db_orchestration.rs` | Query multiple databases in parallel with real data | ParallelDBExecutor, QueryCorrelator, DistributedTransaction |

**Demo 1: Parallel Queries** - Execute queries across 4 databases concurrently  
**Demo 2: Query Correlation** - JOIN results from different databases (Inner/Left/Right/Full)  
**Demo 3: Distributed Transactions** - Two-Phase Commit (2PC) for atomic operations

```rust
use rust_logic_graph::multi_db::{ParallelDBExecutor, QueryCorrelator, JoinStrategy};

// Execute queries in parallel across multiple databases
let mut executor = ParallelDBExecutor::new();
executor
    .add_query("oms_db", "get_user", || async { /* query */ })
    .add_query("orders_db", "get_orders", || async { /* query */ });

let results = executor.execute_all().await?;

// Correlate results with SQL-like JOINs
let correlator = QueryCorrelator::new();
let joined = correlator.join(
    &users_data, 
    &orders_data,
    "user_id", 
    "user_id",
    JoinStrategy::Inner
)?;
```

**Run examples:**
```bash
# Conditional routing
cargo run --example conditional_flow

# Loop over products
cargo run --example loop_flow

# Retry with backoff
cargo run --example retry_flow

# Error handling
cargo run --example error_handling_flow

# Circuit breaker
cargo run --example circuit_breaker_flow

# Nested subgraphs
cargo run --example subgraph_flow

# Rich error messages (v0.10.0) 🆕
cargo run --example error_messages_demo

# Multi-database orchestration (v0.10.0) 🆕
cargo run --example multi_db_orchestration
```

### Error Handling (v0.10.0) 🆕

Production-grade error messages with unique codes, actionable suggestions, and full context:

```rust
use rust_logic_graph::error::{RustLogicGraphError, ErrorContext};

// Rich error with context
let err = RustLogicGraphError::database_connection_error(
    "Failed to connect to PostgreSQL"
).with_context(
    ErrorContext::new()
        .with_node("fetch_orders")
        .with_graph("order_processing")
        .add_metadata("database", "orders_db")
);

// Output:
// [E002] Failed to connect to PostgreSQL
//   Graph: order_processing
//   Node: fetch_orders
//   database: orders_db
//
// 💡 Suggestion: Verify database connection string, credentials, 
//                and network connectivity.
// 📖 Documentation: https://docs.rust-logic-graph.dev/errors/E002

// Automatic retry strategy
if err.is_retryable() {
    retry_with_backoff(operation).await?;
}
```

**12 Error Types**: Node execution (E001), Database (E002), Rules (E003), Config (E004), Timeout (E005), Validation (E006), Serialization (E007), AI (E008), Cache (E009), Context (E010), Distributed (E011), Transaction (E012)

**See [docs/ERRORS.md](docs/ERRORS.md) for complete error reference**

### CLI Tool Examples (v0.5.0)

| File | Description |
|------|-------------|
| `examples/sample_graph.yaml` | Linear workflow with 5 nodes |
| `examples/cyclic_graph.yaml` | Graph with cycle for testing |
| `examples/sample_context.yaml` | Sample input data |

**See [CLI_TOOL.md](docs/CLI_TOOL.md) for usage examples**

---

## 🌟 What Makes Rust Logic Graph Unique?

### 🧠 Reasoning-First Architecture
Traditional workflow engines execute tasks. Rust Logic Graph **reasons** through decisions:
- **Business Rule Engine** - GRL integration for complex decision logic
- **Context-Aware Execution** - Decisions based on accumulated knowledge
- **Multi-Step Reasoning** - Chain decisions across multiple nodes
- **Explainable Decisions** - Trace how conclusions were reached

### 🌐 Built for Distributed Systems
Not a monolithic workflow runner - designed for microservices from day one:
- **Multi-Database Orchestration** - Query PostgreSQL, MySQL, MongoDB, Redis in one flow
- **Service Coordination** - Orchestrate gRPC, REST, and internal services
- **Fault Tolerance** - Circuit breakers, retries, saga patterns
- **Distributed Context** - Share state across services seamlessly

### 🤖 AI-Native Orchestration
LLMs are first-class citizens, not afterthoughts:
- **Multi-Model Workflows** - Combine OpenAI, Claude, Ollama in one reasoning chain
- **RAG Pipeline Ready** - Vector DB integration, embedding generation
- **Agent Coordination** - Build multi-agent systems with shared context
- **Tool Calling Framework** - LLMs can invoke graph nodes as tools

### ⚡ Performance Without Compromise
Embedded library architecture means zero network overhead:
- **Sub-Millisecond Latency** - Direct function calls, not HTTP
- **Memory Efficient** - Context pooling, zero-copy where possible
- **Parallel by Default** - Automatic detection of independent operations
- **Async Everything** - Built on Tokio for maximum concurrency

### 🔧 Developer Experience
Designed for developers who write code, not click buttons:
- **Type-Safe** - Rust's type system catches errors at compile time
- **YAML + Code** - Declarative when possible, programmatic when needed
- **Embeddable** - Library, not service - runs in your process
- **Testable** - Unit test your workflows like any other code

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- **Repository**: https://github.com/KSD-CO/rust-logic-graph
- **rust-rule-engine**: https://crates.io/crates/rust-rule-engine
- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/KSD-CO/rust-logic-graph/issues)

---

## 👥 Authors

**James Vu** - Initial work

---

## 🙏 Acknowledgments

Built with:
- [rust-rule-engine v1.18.0-alpha](https://crates.io/crates/rust-rule-engine) - GRL support with no-regex parser (50-100x faster)
- [Tokio](https://tokio.rs/) - Async runtime
- [Petgraph](https://github.com/petgraph/petgraph) - Graph algorithms
- [Serde](https://serde.rs/) - Serialization
- [Clap](https://github.com/clap-rs/clap) - CLI framework

---

<div align="center">

**⭐ Star us on GitHub if you find this useful! ⭐**

[Documentation](docs/) • [Examples](examples/) • [Use Cases](docs/USE_CASES.md) • [YAML Config Guide](case_study/YAML_CONFIGURATION_SUMMARY.md)

</div>
