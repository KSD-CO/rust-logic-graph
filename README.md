# 🧠 Rust Logic Graph

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/github-KSD--CO%2Frust--logic--graph-blue.svg)](https://github.com/KSD-CO/rust-logic-graph)
[![CI](https://github.com/KSD-CO/rust-logic-graph/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-logic-graph/actions)

A high-performance **reasoning graph framework** for Rust with **GRL (Grule Rule Language)** support. Build complex workflows with conditional execution, topological ordering, and async processing.

---

## ✨ Key Features

- 🔥 **GRL Support** - [rust-rule-engine v0.14.0](https://crates.io/crates/rust-rule-engine) with RETE-UL algorithm (2-24x faster)
- 🔄 **Topological Execution** - Automatic DAG-based node ordering
- ⚡ **Async Runtime** - Built on Tokio for high concurrency
- ⚡ **Parallel Execution** - Automatic parallel execution of independent nodes (v0.5.0)
- 💾 **Caching Layer** - High-performance result caching with TTL, eviction policies, and memory limits (v0.5.0)
- 🧠 **Memory Optimization** - Context pooling and allocation tracking (v0.7.0)
- 🛠️ **CLI Developer Tools** - Graph validation, dry-run, profiling, and visualization (v0.5.0)
- 🎨 **Web Graph Editor** - Next.js visual editor with drag-and-drop interface (v0.8.0)
- � **YAML Configuration** - Declarative graph definitions with external config files (v0.8.5)
- �📊 **Multiple Node Types** - RuleNode, DBNode, AINode
- 📝 **JSON/YAML Configuration** - Simple workflow definitions
- 🎯 **98% Drools Compatible** - Easy migration from Java
- 🌊 **Streaming Processing** - Stream-based execution with backpressure (v0.3.0)
- 🗄️ **Database Integrations** - PostgreSQL, MySQL, Redis, MongoDB (v0.2.0)
- 🤖 **AI/LLM Integrations** - OpenAI, Claude, Ollama (v0.2.0)

---

## 🚀 Quick Start

### Installation

```toml
[dependencies]
rust-logic-graph = "0.8.5"

# With specific integrations
rust-logic-graph = { version = "0.8.5", features = ["postgres", "openai"] }

# With all integrations
rust-logic-graph = { version = "0.8.5", features = ["all-integrations"] }
```

### Simple Example

```rust
use rust_logic_graph::{RuleEngine, GrlRule};

let grl = r#"
rule "Discount" {
    when
        cart_total > 100 && is_member == true
    then
        discount = 0.15;
}
"#;

let mut engine = RuleEngine::new();
engine.add_grl_rule(grl)?;
```

## 🏢 Real-World Case Study: Purchasing Flow System

See a complete production implementation in **[case_study/](case_study/)** - A full-featured purchasing automation system built with Rust Logic Graph.

### 📊 System Overview

**Problem**: Automate purchasing decisions for inventory replenishment across multiple products, warehouses, and suppliers.

**Solution**: Business rules in GRL decide when/how much to order. Orchestrator executes the workflows.

### 🎯 Two Architecture Implementations

**1. Microservices (v4.0)** - 7 services with gRPC
- Orchestrator (port 8080) - Workflow coordination
- OMS Service (port 50051) - Order management data
- Inventory Service (port 50052) - Stock levels
- Supplier Service (port 50053) - Supplier information
- UOM Service (port 50054) - Unit conversions
- Rule Engine (port 50055) - GRL business rules
- PO Service (port 50056) - Purchase order management

**2. Monolithic** - Single HTTP service
- Same business logic as microservices
- Single process on port 8080
- Shared GRL rules file
- Direct function calls instead of gRPC

### 🔥 GRL Business Rules (15 Rules)

```grl
rule "CalculateShortage" salience 120 no-loop {
  when
    required_qty > 0
  then
    Log("Calculating shortage...");
    shortage = required_qty - available_qty;
    Log("Shortage calculated");
}

rule "OrderMOQWhenShortageIsLess" salience 110 no-loop {
  when
    shortage > 0 && shortage < moq && is_active == true
  then
    Log("Shortage less than MOQ, ordering MOQ");
    order_qty = moq;
}
```

**See full rules**: [purchasing_rules.grl](case_study/microservices/services/rule-engine-service/rules/purchasing_rules.grl)

### YAML Configuration (NEW in v0.8.5)

Both Monolithic and Microservices implementations now support **YAML-based graph configuration**:

```yaml
# purchasing_flow_graph.yaml
nodes:
  oms_grpc:
    type: DBNode
    description: "Fetch order management data"
  
  inventory_grpc:
    type: DBNode
    description: "Fetch inventory levels"
  
  rule_engine_grpc:
    type: RuleNode
    description: "Evaluate business rules"
    dependencies:
      - oms_grpc
      - inventory_grpc

edges:
  - from: oms_grpc
    to: rule_engine_grpc
  - from: inventory_grpc
    to: rule_engine_grpc
```

**Benefits:**
- ✅ **70% less code** - Graph definition moves from code to YAML
- ✅ **No recompile** - Change workflows without rebuilding
- ✅ **Multiple workflows** - Easy to create variants (urgent, standard, approval)
- ✅ **Better readability** - Clear, declarative graph structure
- ✅ **Easy testing** - Test with different configurations

**Usage:**
```rust
// Default config
executor.execute("PROD-001").await?;

// Custom workflow
executor.execute_with_config("PROD-001", "urgent_flow.yaml").await?;
```

**Documentation**: See [YAML_CONFIGURATION_SUMMARY.md](case_study/YAML_CONFIGURATION_SUMMARY.md)

### Microservices Communication Flow

After v0.8.0 refactor, the Orchestrator now uses **rust-logic-graph's Graph/Executor pattern** to coordinate microservices:

- The Orchestrator receives a purchasing request (HTTP) and creates a **Graph** with 6 custom **gRPC Nodes**.
- Each Node wraps a gRPC call to a service: `OmsGrpcNode`, `InventoryGrpcNode`, `SupplierGrpcNode`, `UomGrpcNode`, `RuleEngineGrpcNode`, `PoGrpcNode`.
- The **Executor** runs the graph in topological order:
  1. **Data Collection Phase** (parallel): OMS, Inventory, Supplier, UOM nodes execute simultaneously via gRPC
  2. **Rule Evaluation Phase**: RuleEngineGrpcNode waits for all data, then evaluates GRL rules
  3. **Execution Phase**: PoGrpcNode creates/sends PO based on rule decisions
- All business logic (decision flags, calculations) comes from GRL rules. The Orchestrator is a pure executor.

**Graph Topology**:
```
OMS Node ────┐
             │
Inventory ───┼──→ RuleEngine Node ──→ PO Node
             │
Supplier ────┤
             │
UOM Node ────┘
```

**Benefits of Graph/Executor Pattern**:
- ✅ **Declarative**: Define workflow as nodes + edges instead of imperative code
- ✅ **Parallel Execution**: Data nodes run concurrently automatically
- ✅ **Type Safety**: Custom Node implementations with Rust's type system
- ✅ **Testable**: Each node can be tested in isolation
- ✅ **Consistent**: Same pattern used in monolithic and microservices

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT (HTTP REST)                          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ POST /purchasing/flow
                                 ▼
        ┌────────────────────────────────────────────────────────────┐
        │            Orchestrator Service (Port 8080)                │
        │  ┌─────────────────────────────────────────────────────┐   │
        │  │          rust-logic-graph Graph Executor            │   │
        │  │                                                     │   │
        │  │  Creates Graph with 6 gRPC Nodes:                   │   │
        │  │  • OmsGrpcNode      → gRPC to OMS :50051            │   │
        │  │  • InventoryGrpcNode → gRPC to Inventory :50052     │   │
        │  │  • SupplierGrpcNode → gRPC to Supplier :50053       │   │
        │  │  • UomGrpcNode      → gRPC to UOM :50054            │   │
        │  │  • RuleEngineGrpcNode → gRPC to Rules :50055        │   │
        │  │  • PoGrpcNode       → gRPC to PO :50056             │   │
        │  │                                                     │   │
        │  │  Graph Topology:                                    │   │
        │  │  OMS ───────┐                                       │   │
        │  │  Inventory ─┼─→ RuleEngine ──→ PO                   │   │
        │  │  Supplier ──┤                                       │   │
        │  │  UOM ───────┘                                       │   │
        │  └─────────────────────────────────────────────────────┘   │
        └────────┬───────────────────────────────────────────────────┘
                 │
   ┌─────────────┼──────────────────┬────────────────┬──────────────┐
   │ (Parallel)  │  (Parallel)      │   (Parallel)   │  (Parallel)  │
   ▼             ▼                  ▼                ▼              │
┌──────────┐  ┌────────────┐  ┌─────────────┐  ┌───────────┐        │
│OMS :50051│  │Inventory   │  │Supplier     │  │UOM :50054 │        │
│          │  │:50052      │  │:50053       │  │           │        │
│• History │  │• Levels    │  │• Pricing    │  │• Convert  │        │
│• Demand  │  │• Available │  │• Lead Time  │  │• Factors  │        │
└────┬─────┘  └─────┬──────┘  └──────┬──────┘  └─────┬─────┘        │
     │              │                │               │              │
     └──────────────┴────────────────┴───────────────┘              │
                          │                                         │
                          │ Data stored in Graph Context            │
                          ▼                                         │
                   ┌─────────────────┐                              │
                   │ Rule Engine     │ (Port 50055 - gRPC)          │
                   │     :50055      │                              │
                   │                 │                              │
                   │ • GRL Rules     │ • Evaluates 15 rules         │
                   │ • Calculations  │ • Returns decision flags     │
                   │ • Decision Flags│ • NO side effects            │
                   └────────┬────────┘                              │
                            │                                       │
                            │ Flags stored in Graph Context         │
                            ▼                                       │
                   ┌─────────────────┐                              │
                   │ PO Service      │ (Port 50056 - gRPC)          │
                   │    :50056       │◄─────────────────────────────┘
                   │                 │
                   │ • Create PO     │ • Reads flags from context
                   │ • Send to       │ • Executes based on rules
                   │   Supplier      │ • Email/API delivery
                   └─────────────────┘
```
**Note**: The Rule Engine service returns decision flags and calculations to the Graph Context. The PoGrpcNode then reads these flags from the context to determine whether to create/send the PO.

### Where rust-logic-graph is Used

**Monolithic App** (`case_study/monolithic/`):
- Uses `Graph`, `Executor`, and custom `Node` implementations
- 6 DB nodes query local MySQL databases directly
- `RuleEngineNode` calls in-process `RuleEngine`
- Single process, no network calls

**Orchestrator Microservice** (`case_study/microservices/services/orchestrator-service/`):
- Uses `Graph`, `Executor`, and custom gRPC `Node` implementations  
- 6 gRPC nodes make network calls to remote services
- Same graph topology as monolithic
- Distributed across multiple processes

**Rule Engine Service** (`case_study/microservices/services/rule-engine-service/`):
- Uses `RuleEngine` for GRL evaluation
- Exposed via gRPC endpoint
- Stateless service (no graph execution)

**Other Microservices** (OMS, Inventory, Supplier, UOM, PO):
- Standard gRPC services with database access
- Do NOT use rust-logic-graph directly
- Called by Orchestrator's Graph Executor

### Web Graph Editor (NEW in v0.8.0)

**🌐 Online Editor**: [https://logic-graph-editor.amalthea.cloud/](https://logic-graph-editor.amalthea.cloud/)

Try the visual graph editor online - no installation required! Create workflows, define rules, and visualize your logic graphs with drag-and-drop.

### CLI Tools (v0.5.0)

```bash
# Build the CLI tool
cargo build --release --bin rlg

# Validate a graph
./target/release/rlg validate --file examples/sample_graph.json

# Visualize graph structure
./target/release/rlg visualize --file examples/sample_graph.json --details

# Profile performance
./target/release/rlg profile --file examples/sample_graph.json --iterations 100

# Dry-run without execution
./target/release/rlg dry-run --file examples/sample_graph.json --verbose
```

**[Full CLI Documentation →](docs/CLI_TOOL.md)**

### Run Examples

```bash
# Basic workflow
cargo run --example simple_flow

# GRL rules
cargo run --example grl_rules

# Advanced integration
cargo run --example grl_graph_flow
```

---


---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[🏢 Case Study: Purchasing Flow](case_study/docs/README.md)** | Real production system with microservices & monolithic implementations |
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

## 🏗️ Architecture

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

- **RETE-UL Algorithm**: Advanced pattern matching with unlinking (v0.14.0)
- **2-24x Faster**: Than v0.10 at 50+ rules
- **98% Drools Compatible**: Easy migration path
- **Async by Default**: High concurrency support
- **Parallel Execution**: Automatic layer-based parallelism
- **Smart Caching**: Result caching with TTL and eviction policies

---

## 🧪 Testing & CLI Tools

```bash
# Run all tests
cargo test

# Build CLI tool
cargo build --release --bin rlg

# Validate graph
./target/release/rlg validate --file examples/sample_graph.json

# Visualize graph structure
./target/release/rlg visualize --file examples/sample_graph.json

# Profile performance
./target/release/rlg profile --file examples/sample_graph.json --iterations 100

# Dry-run execution
./target/release/rlg dry-run --file examples/sample_graph.json --verbose
```

**Test Results**: ✅ 32/32 tests passing

**[Learn more about CLI tools →](docs/CLI_TOOL.md)**

---

## 📦 Project Status

**Version**: 0.8.5 (Latest)
**Status**: Production-ready with YAML configuration, web graph editor, and real-world case study

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

### CLI Tool Examples (v0.5.0)

| File | Description |
|------|-------------|
| `examples/sample_graph.json` | Linear workflow with 5 nodes |
| `examples/cyclic_graph.json` | Graph with cycle for testing |
| `examples/sample_context.json` | Sample input data |

**See [CLI_TOOL.md](docs/CLI_TOOL.md) for usage examples**

---

## 🌟 Why Rust Logic Graph?

### vs. Traditional Rule Engines
- ✅ **Async by default** - No blocking I/O
- ✅ **Type safety** - Rust's type system
- ✅ **Modern syntax** - GRL support
- ✅ **Graph-based** - Complex workflows

### vs. Workflow Engines
- ✅ **Embedded** - No external services
- ✅ **Fast** - Compiled Rust code
- ✅ **Flexible** - Custom nodes
- ✅ **Rule-based** - Business logic in rules

---

## 📝 Changelog

### v0.8.5 (2025-11-20) - YAML Configuration Release

**New Features:**
- 📋 **YAML Configuration Support** - Declarative graph definitions
  - Load graph structure from YAML files instead of hardcoded
  - `GraphConfig` module for parsing YAML configurations
  - Support for both JSON and YAML formats
  - 70% code reduction in graph executors
  - See [YAML Configuration Guide](case_study/YAML_CONFIGURATION_SUMMARY.md)
- 🔧 **Enhanced Graph Executor API**
  - `execute()` - Use default configuration
  - `execute_with_config(config_path)` - Load custom YAML config
  - Dynamic node registration from config
- 📝 **Multiple Workflow Support**
  - Standard flow (full process)
  - Simplified flow (skip optional steps)
  - Urgent flow (fast-track)
  - Easy to create custom workflows
- 📚 **Comprehensive Documentation**
  - YAML configuration guide with examples
  - Before/After comparison showing improvements
  - Multiple workflow examples
  - Integration guides for both architectures

**Improvements:**
- Monolithic and Microservices both support YAML configs
- Reduced boilerplate code by 70% in executors
- Better separation of concerns (config vs. code)
- Easier testing with multiple configurations
- No recompilation needed for workflow changes

**Examples:**
```yaml
# purchasing_flow_graph.yaml
nodes:
  oms_grpc:
    type: DBNode
    description: "Fetch OMS data"
  rule_engine_grpc:
    type: RuleNode
    dependencies: [oms_grpc]

edges:
  - from: oms_grpc
    to: rule_engine_grpc
```

```rust
// Use default config
executor.execute("PROD-001").await?;

// Use custom config
executor.execute_with_config("PROD-001", "urgent_flow.yaml").await?;
```

**Compatibility:**
- All tests passing
- API backward compatible
- Existing hardcoded graphs still work

### v0.8.0 (2025-11-20) - Web Editor & Production Case Study Release

**New Features:**
- 🎨 **Web Graph Editor** - Next.js visual editor with drag-and-drop
  - Online version: https://logic-graph-editor.amalthea.cloud/
  - React Flow-based graph visualization
  - Real-time node editing and validation
  - Export/import JSON workflows
  - See [Graph Editor Guide](graph-editor/README.md)
- 🏢 **Production Case Study** - Complete purchasing flow system
  - Microservices architecture (7 services with gRPC)
  - Monolithic architecture (single HTTP service)
  - 15 GRL business rules for purchasing decisions
  - Kubernetes deployment manifests
  - Docker Compose for local development
  - Shared GRL rules proving portability
  - See [Case Study Documentation](case_study/docs/README.md)

**Improvements:**
- Updated README with case study section
- Added online graph editor link
- Comprehensive production examples

**Compatibility:**
- All tests passing
- API backward compatible

### v0.5.0 (2025-11-06) - Performance & Developer Tools Release

**Breaking Changes:**
- ⚡ **Upgraded rust-rule-engine** from v0.10 → v0.14.0
  - Now uses RETE-UL algorithm (2-24x faster)
  - Better memory efficiency
  - Improved conflict resolution
  - See [Migration Guide](docs/MIGRATION_GUIDE.md)

**New Features:**
- 🛠️ **CLI Developer Tools** (`rlg` binary)
  - Graph validation with comprehensive checks
  - Dry-run execution mode
  - Performance profiling with statistics
  - ASCII graph visualization
  - See [CLI Tool Guide](docs/CLI_TOOL.md)
- 💾 **Caching Layer** - High-performance result caching
  - TTL-based expiration
  - Multiple eviction policies (LRU, LFU, FIFO)
  - Memory limits and statistics
  - See [Cache Guide](docs/CACHE_IMPLEMENTATION.md)
- ⚡ **Parallel Node Execution** - Automatic detection and parallel execution
  - Layer detection algorithm using topological sort
  - Concurrent execution within layers
  - Parallelism analysis and statistics
- 📊 **ParallelExecutor** - New executor with parallel capabilities
- 📝 **New Examples** - CLI examples and test graphs
- ✅ **32 Tests** - Comprehensive test coverage

**Improvements:**
- Updated documentation with CLI tools, caching, and migration guides
- Performance benchmarking utilities
- Example graph files for testing

**Compatibility:**
- All 32 tests passing
- API is backward compatible (100%)
- Performance: 2-24x faster rule matching

### v0.3.0 (2025-11-03) - Streaming & Performance Release

**New Features:**
- 🌊 **Streaming Processing** - Stream-based node execution
  - Backpressure handling with bounded channels
  - Large dataset support with chunking
  - Stream operators (map, filter, fold, async map)
- 📝 **New Example** - `streaming_flow.rs` with 6 demonstrations
- ✅ **8 New Tests** - Streaming module testing

**Performance:**
- Processed 10,000 items in chunks
- ~432 items/sec throughput with backpressure

### v0.2.0 (2025-11-02) - Integrations Release

**New Features:**
- 🗄️ **Database Integrations** - PostgreSQL, MySQL, Redis, MongoDB
- 🤖 **AI/LLM Integrations** - OpenAI GPT-4, Claude 3.5, Ollama
- 📝 **Integration Examples** - `postgres_flow.rs`, `openai_flow.rs`
- 📚 **INTEGRATIONS.md** - Comprehensive integration guide
- 🎛️ **Feature Flags** - Optional dependencies for integrations

### v0.1.0 (2025-11-01) - Initial Release

**Core Features:**
- 🧠 Core graph execution engine
- 🔥 GRL (Grule Rule Language) integration
- 🔄 Topological sorting
- ⚡ Async execution with Tokio
- 📊 Three node types (Rule, DB, AI)
- 📝 JSON I/O for graphs
- 📚 4 working examples
- ✅ 6/6 tests passing

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
- [rust-rule-engine v0.14.0](https://crates.io/crates/rust-rule-engine) - GRL support with RETE-UL
- [Tokio](https://tokio.rs/) - Async runtime
- [Petgraph](https://github.com/petgraph/petgraph) - Graph algorithms
- [Serde](https://serde.rs/) - Serialization
- [Clap](https://github.com/clap-rs/clap) - CLI framework

---

<div align="center">

**⭐ Star us on GitHub if you find this useful! ⭐**

[Documentation](docs/) • [Examples](examples/) • [Use Cases](docs/USE_CASES.md) • [YAML Config Guide](case_study/YAML_CONFIGURATION_SUMMARY.md)

</div>
