# 🧠 Rust Logic Graph

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
- Saga pattern for distributed transactions

### Example Use Cases

```rust
// Financial risk assessment across multiple data sources
let risk_engine = Graph::new()
    .add_node("credit_history", DBNode::postgres(...))
    .add_node("transaction_analysis", DBNode::mongodb(...))
    .add_node("fraud_check", AINode::openai(...))
    .add_node("risk_rules", RuleNode::grl("risk_assessment.grl"))
    .add_node("decision", ConditionalNode::new(...));

// Multi-step AI reasoning with tool calling
let ai_agent = Graph::new()
    .add_node("understand_query", AINode::claude(...))
    .add_node("search_knowledge", SubgraphNode::new(rag_pipeline))
    .add_node("reason", AINode::openai_gpt4(...))
    .add_node("validate", RuleNode::grl("validation.grl"))
    .add_retry("reason", max_attempts: 3);

// Microservice coordination with fault tolerance
let order_flow = Graph::new()
    .add_node("inventory", GrpcNode::new("inventory-service"))
    .add_node("payment", GrpcNode::new("payment-service"))
    .add_node("shipping", GrpcNode::new("shipping-service"))
    .add_circuit_breaker("payment", threshold: 5)
    .add_saga_compensation(...);
```

---

## ✨ Key Features

- 🔥 **GRL Support** - [rust-rule-engine v0.17](https://crates.io/crates/rust-rule-engine)
- 🔄 **Topological Execution** - Automatic DAG-based node ordering
- ⚡ **Async Runtime** - Built on Tokio for high concurrency
- ⚡ **Parallel Execution** - Automatic parallel execution of independent nodes (v0.5.0)
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

---

## 🚀 Quick Start

### Installation

```toml
[dependencies]
rust-logic-graph = "0.10.0-alpha.1"

# With specific integrations
rust-logic-graph = { version = "0.10.0-alpha.1", features = ["postgres", "openai"] }

# With all integrations
rust-logic-graph = { version = "0.10.0-alpha.1", features = ["all-integrations"] }
```

## 🏢 Real-World Case Study: Purchasing Flow System

Complete production implementation: **[case_study/](case_study/)** - Purchasing automation with **two architectures** (Monolithic vs Microservices).

**Problem**: Automate purchasing decisions across products, warehouses, suppliers.  
**Solution**: 15 GRL business rules + rust-logic-graph orchestration.

### Architecture Comparison

| | **Monolithic** | **Microservices** |
|---|---|---|
| **Performance** | ~10ms (in-process) | ~56ms (gRPC overhead) |
| **Resources** | 50MB RAM, 1 CPU | 500MB RAM, 7 containers |
| **Best For** | <1K req/min, small teams (1-5 devs) | >10K req/min, large teams (15+ devs) |
| **Deployment** | Single binary | Docker Compose / Kubernetes |
| **Scaling** | Vertical only | Horizontal scaling |
| **Complexity** | Simple | Distributed tracing required |

### Quick Start

**Monolithic** (single HTTP service, 4 PostgreSQL DBs):
```bash
cd case_study/monolithic && cargo run --release
curl -X POST http://localhost:8080/purchasing/flow -d '{"product_id": "PROD-001"}'
```

**Microservices** (7 gRPC services):
```bash
cd case_study/microservices && docker compose up -d
curl -X POST http://localhost:8080/api/purchasing/flow -d '{"product_id": "PROD-001"}'
```

### Key Features

**✅ Same Business Logic** - 15 GRL rules shared across both architectures  
**✅ YAML Configuration** - Change workflows without recompilation  
**✅ Multi-Database** (Monolithic) - 4 separate PostgreSQL databases  
**✅ Dynamic Field Mapping** - Zero hardcoded field names  
**✅ Graph Executor Pattern** - Declarative node topology

### Architecture Diagrams

**Microservices Communication Flow:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLIENT (HTTP REST)                          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ POST /api/purchasing/flow
                                 ▼
        ┌────────────────────────────────────────────────────────────┐
        │      Orchestrator Service (Port 8080) - main.rs            │
        │                                                            │
        │  HTTP Endpoint → OrchestratorGraphExecutor                 │
        │                                                            │
        │  ┌──────────────────────────────────────────────────────┐  │
        │  │    rust-logic-graph Graph Executor                   │  │
        │  │    (graph_executor.rs)                               │  │
        │  │                                                      │  │
        │  │  Creates Graph with 6 Custom gRPC Nodes:             │  │
        │  │  ┌────────────────────────────────────────────────┐  │  │
        │  │  │ OmsGrpcNode                                    │  │  │
        │  │  │ • impl Node trait from rust-logic-graph        │  │  │
        │  │  │ • async fn run() → gRPC call to :50051         │  │  │
        │  │  │ • Returns JSON to Context                      │  │  │
        │  │  └────────────────────────────────────────────────┘  │  │
        │  │  ┌────────────────────────────────────────────────┐  │  │
        │  │  │ InventoryGrpcNode → gRPC :50052                │  │  │
        │  │  │ SupplierGrpcNode → gRPC :50053                 │  │  │
        │  │  │ UomGrpcNode → gRPC :50054                      │  │  │
        │  │  │ RuleEngineGrpcNode → gRPC :50056               │  │  │
        │  │  │ PoGrpcNode → gRPC :50055                       │  │  │
        │  │  └────────────────────────────────────────────────┘  │  │
        │  │                                                      │  │
        │  │  Graph Topology (hardcoded in graph_executor.rs):    │  │
        │  │  OMS ───────┐                                        │  │
        │  │  Inventory ─┼─→ RuleEngine ──→ PO                    │  │
        │  │  Supplier ──┤                                        │  │
        │  │  UOM ───────┘                                        │  │
        │  │                                                      │  │
        │  │  Executor runs in topological order                  │  │
        │  └──────────────────────────────────────────────────────┘  │
        └────────┬───────────────────────────────────────────────────┘
                 │
   ┌─────────────┼──────────────────┬────────────────┬──────────────┐
   │ (Parallel)  │  (Parallel)      │   (Parallel)   │  (Parallel)  │
   ▼             ▼                  ▼                ▼              │
┌──────────┐  ┌────────────┐  ┌─────────────┐  ┌───────────┐        │
│OMS :50051│  │Inventory   │  │Supplier     │  │UOM :50054 │        │
│  (gRPC)  │  │:50052      │  │:50053       │  │  (gRPC)   │        │
│          │  │ (gRPC)     │  │ (gRPC)      │  │           │        │
│• History │  │• Levels    │  │• Pricing    │  │• Convert  │        │
│• Demand  │  │• Available │  │• Lead Time  │  │• Factors  │        │
│          │  │            │  │• MOQ        │  │           │        │
└────┬─────┘  └─────┬──────┘  └──────┬──────┘  └─────┬─────┘        │
     │              │                │               │              │
     └──────────────┴────────────────┴───────────────┘              │
                          │                                         │
                          │ Data stored in Graph Context            │
                          ▼                                         │
                   ┌─────────────────┐                              │
                   │ Rule Engine     │ (Port 50056 - gRPC)          │
                   │     :50056      │                              │
                   │   (gRPC)        │                              │
                   │                 │                              │
                   │ • Loads GRL     │ • Evaluates 15 rules         │
                   │   rules from    │ • Returns decision flags     │
                   │   .grl file     │ • NO side effects            │
                   │ • Pure function │ • Calculations + flags       │
                   └────────┬────────┘                              │
                            │                                       │
                            │ Flags stored in Graph Context         │
                            ▼                                       │
                   ┌─────────────────┐                              │
                   │ PO Service      │ (Port 50055 - gRPC)          │
                   │    :50055       │◄─────────────────────────────┘
                   │   (gRPC)        │
                   │                 │
                   │ • Create PO     │ • Reads flags from context
                   │ • Send to       │ • Executes based on rules
                   │   Supplier      │ • Email/API delivery
                   └─────────────────┘
```

**Monolithic Clean Architecture:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   HTTP REST API (Port 8080)                             │
│                 POST /purchasing/flow {product_id}                      │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                                   ▼
        ┌───────────────────────────────────────────────────────────────┐
        │        PurchasingGraphExecutor (executors/graph_executor.rs)  │
        │                      (Clean Architecture)                      │
        │  ┌─────────────────────────────────────────────────────────┐  │
        │  │      rust-logic-graph Graph/Executor Engine              │  │
        │  │                                                          │  │
        │  │  execute_with_config(product_id, "purchasing_flow.yaml") │  │
        │  │                                                          │  │
        │  │  1. GraphConfig::from_yaml_file("purchasing_flow.yaml")  │  │
        │  │  2. Parse nodes + edges + field_mappings                 │  │
        │  │  3. For each node in YAML:                               │  │
        │  │     • Create DynamicDBNode (with database routing)       │  │
        │  │     • Create DynamicRuleNode (with field_mappings)       │  │
        │  │  4. Register all nodes to Executor                       │  │
        │  │  5. Execute graph in topological order                   │  │
        │  │                                                          │  │
        │  │  Graph Topology (from YAML):                             │  │
        │  │  oms_history ────────┐                                   │  │
        │  │  inventory_levels ───┼──→ rule_engine ──→ create_po      │  │
        │  │  supplier_info ──────┤                                   │  │
        │  │  uom_conversion ─────┘                                   │  │
        │  └─────────────────────────────────────────────────────────┘  │
        └──────────────┬────────────────────────────────────────────────┘
                       │
    ┌──────────────────┼──────────────────┬──────────────────┬───────────┐
    │ (Parallel DBs)   │  (Parallel DBs)  │  (Parallel DBs)  │ (Parallel)│
    ▼                  ▼                  ▼                  ▼           │
┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  oms_db      │  │ inventory_db │  │ supplier_db  │  │   uom_db     │   │
│ PostgreSQL   │  │ PostgreSQL   │  │ PostgreSQL   │  │ PostgreSQL   │   │
│  :5433       │  │  :5434       │  │  :5435       │  │  :5436       │   │
│              │  │              │  │              │  │              │   │
│ • history    │  │ • levels     │  │ • info       │  │ • conversion │   │
│ • demand     │  │ • available  │  │ • pricing    │  │ • factors    │   │
│ • trends     │  │ • reserved   │  │ • lead_time  │  │              │   │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘   │
       │                 │                 │                 │           │
       │ DynamicDBNode   │ DynamicDBNode   │ DynamicDBNode   │ Dynamic   │
       │ database:"oms"  │ database:"inv"  │ database:"sup"  │ DB Node   │
       └─────────────────┴─────────────────┴─────────────────┴───────────┘
                                    │
                         Data stored in Graph Context
                         with path notation (e.g., "oms_history.avg_daily_demand")
                                    │
                                    ▼
                  ┌──────────────────────────────────────────┐
                  │        DynamicRuleNode (rule_engine)     │
                  │                                          │
                  │  YAML field_mappings config:             │
                  │  ┌────────────────────────────────────┐  │
                  │  │ avg_daily_demand:                  │  │
                  │  │   "oms_history.avg_daily_demand"   │  │
                  │  │ available_qty:                     │  │
                  │  │   "inventory_levels.available_qty" │  │
                  │  │ lead_time:                         │  │
                  │  │   "supplier_info.lead_time"        │  │
                  │  │ ... (9 total mappings)             │  │
                  │  └────────────────────────────────────┘  │
                  │                                          │
                  │  extract_rule_inputs() loop:             │
                  │  • Reads field_mappings from YAML        │
                  │  • Uses get_value_by_path() for parsing  │
                  │  • Returns HashMap<String, Value>        │
                  │  • ZERO hardcoded field names!           │
                  └──────────────┬───────────────────────────┘
                                 │
                                 ▼
                  ┌──────────────────────────────────────────┐
                  │      RuleEngineService (In-Process)      │
                  │                                          │
                  │  evaluate(HashMap<String, Value>)        │
                  │                                          │
                  │  • Loads purchasing_rules.grl            │
                  │  • 15 business rules (GRL)               │
                  │  • Accepts dynamic HashMap input         │
                  │  • No struct, no hardcoded fields        │
                  │  • Pure functional evaluation            │
                  │                                          │
                  │  Rules calculate:                        │
                  │  ✓ shortage = required_qty - available   │
                  │  ✓ order_qty (respects MOQ)              │
                  │  ✓ total_amount with discounts           │
                  │  ✓ requires_approval flag                │
                  │  ✓ should_create_po flag                 │
                  └──────────────┬───────────────────────────┘
                                 │
                      Decision flags returned to Context
                                 │
                                 ▼
                  ┌──────────────────────────────────────────┐
                  │    DynamicRuleNode (create_po)           │
                  │                                          │
                  │  YAML field_mappings config:             │
                  │  ┌────────────────────────────────────┐  │
                  │  │ should_order:                      │  │
                  │  │   "rule_engine.should_order"       │  │
                  │  │ recommended_qty:                   │  │
                  │  │   "rule_engine.recommended_qty"    │  │
                  │  │ product_id:                        │  │
                  │  │   "supplier_info.product_id"       │  │
                  │  │ ... (6 total mappings)             │  │
                  │  └────────────────────────────────────┘  │
                  │                                          │
                  │  • Reads rule_engine output from context │
                  │  • Dynamic field extraction via YAML     │
                  │  • Creates PO if should_order == true    │
                  │  • Returns PO JSON or null               │
                  └──────────────────────────────────────────┘
```

**Documentation**: [Case Study Docs](case_study/docs/README.md) • [YAML Config Guide](case_study/YAML_CONFIGURATION_SUMMARY.md)

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

### Advanced Control Flow Usage (v0.9.0) 🆕

#### Conditional Branching

Route execution based on conditions:

```rust
use rust_logic_graph::{Graph, NodeConfig, Edge, Context};

let mut graph = Graph::new();

// Add nodes
graph.add_node("check_inventory", NodeConfig::default());
graph.add_node("process_order", NodeConfig::default());
graph.add_node("notify_supplier", NodeConfig::default());

// Add conditional routing
graph.add_node("route_decision", NodeConfig {
    node_type: NodeType::Conditional {
        condition: "available_qty > 100".to_string(),
        true_branch: "process_order".to_string(),
        false_branch: "notify_supplier".to_string(),
    },
    ..Default::default()
});

graph.add_edge(Edge::new("check_inventory", "route_decision"));
graph.add_edge(Edge::new("route_decision", "process_order"));
graph.add_edge(Edge::new("route_decision", "notify_supplier"));

// Execute
let result = graph.execute().await?;
```

#### Loops

Iterate over collections or use while loops:

```rust
// Foreach loop over products
graph.add_node("process_products", NodeConfig {
    node_type: NodeType::Loop {
        loop_type: LoopType::Foreach {
            items_key: "products".to_string(),
            item_var: "current_product".to_string(),
            body_node: "process_single_product".to_string(),
        },
        max_iterations: Some(100),
    },
    ..Default::default()
});

// While loop with condition
graph.add_node("retry_until_success", NodeConfig {
    node_type: NodeType::Loop {
        loop_type: LoopType::While {
            condition: "status != 'success'".to_string(),
            body_node: "attempt_operation".to_string(),
        },
        max_iterations: Some(10),
    },
    ..Default::default()
});
```

#### Error Handling

Try/catch patterns for resilient workflows:

```rust
graph.add_node("safe_operation", NodeConfig {
    node_type: NodeType::TryCatch {
        try_node: "risky_operation".to_string(),
        catch_node: Some("handle_error".to_string()),
        finally_node: Some("cleanup".to_string()),
    },
    ..Default::default()
});
```

#### Retry Logic

Exponential backoff for transient failures:

```rust
graph.add_node("api_call", NodeConfig {
    node_type: NodeType::Retry {
        target_node: "external_api".to_string(),
        max_attempts: 3,
        backoff_ms: 100,
        exponential: true,
    },
    ..Default::default()
});
```

#### Circuit Breaker

Fault tolerance for unstable services:

```rust
graph.add_node("protected_service", NodeConfig {
    node_type: NodeType::CircuitBreaker {
        target_node: "unstable_service".to_string(),
        failure_threshold: 5,
        timeout_ms: 60000,
    },
    ..Default::default()
});
```

#### Subgraphs

Nested graph execution with input/output mapping:

```rust
graph.add_node("payment_flow", NodeConfig {
    node_type: NodeType::Subgraph {
        graph_def: payment_graph_def,
        input_mapping: vec![("order_id", "id"), ("amount", "total")],
        output_key: "payment_result".to_string(),
    },
    ..Default::default()
});
```

**See [examples/](examples/) for complete working examples.**

---


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

**Version**: 0.8.8 (Latest)
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
| `examples/sample_graph.json` | Linear workflow with 5 nodes |
| `examples/cyclic_graph.json` | Graph with cycle for testing |
| `examples/sample_context.json` | Sample input data |

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

## 📝 Changelog

### v0.10.0-alpha.1 (2025-11-22) - Rich Error Messages 🆕

**New Features:**
- 🚨 **Production-Grade Error Handling** - Comprehensive error system
  - 12 predefined error types with unique codes (E001-E012)
  - Error classification: Retryable, Permanent, Transient, Configuration
  - Rich context propagation: node → graph → step → service → metadata
  - Actionable suggestions for every error
  - Automatic documentation links
  - Source error chaining support

**Error Types:**
- E001: Node execution error
- E002: Database connection error
- E003: Rule evaluation error
- E004: Configuration error
- E005: Timeout error
- E006: Graph validation error
- E007: Serialization error
- E008: AI/LLM error
- E009: Cache error
- E010: Context error
- E011: Distributed system error
- E012: Transaction coordination error

**API:**
```rust
use rust_logic_graph::error::{RustLogicGraphError, ErrorContext};

let err = RustLogicGraphError::database_connection_error("...")
    .with_context(
        ErrorContext::new()
            .with_node("fetch_data")
            .with_graph("order_flow")
            .add_metadata("database", "orders_db")
    );

// Automatic retry strategy
if err.is_retryable() {
    retry_with_backoff(operation).await?;
}
```

**Documentation:**
- Complete error reference: `docs/ERRORS.md` (600+ lines)
- Example: `examples/error_messages_demo.rs`
- Summary: `docs/BETTER_ERROR_MESSAGES_SUMMARY.md`

**Testing:**
- 5 unit tests in `src/error/mod.rs`
- All tests passing (44/44 total)

**Impact:**
- 10x faster debugging with clear error messages
- Production-ready error handling
- Foundation for distributed systems (v0.10.0)

---

### v0.8.9 (2025-11-22) - DBNode Parameters Feature

**New Features:**
- 🔧 **DBNode Context Parameters** - Dynamic query parameter extraction
  - Extract SQL parameters from execution context
  - `NodeConfig::db_node_with_params()` for parameterized queries
  - Support for `$1`, `$2` (PostgreSQL) and `?` (MySQL) placeholders
  - Automatic type conversion (String, Number, Boolean, Null)
  - Graceful handling of missing parameters
  - See [DB Parameters Guide](docs/DB_PARAMS.md)

**API Additions:**
```rust
// Create DBNode with context parameter extraction
NodeConfig::db_node_with_params(
    "SELECT * FROM users WHERE id = $1",
    vec!["user_id".to_string()]
)

// Set parameters in context
graph.context.set("user_id", json!("USER-123"));
```

**Configuration Support:**
```yaml
nodes:
  fetch_user:
    node_type: DBNode
    query: "SELECT * FROM users WHERE user_id = $1"
    params:
      - user_id  # Extract from context
```

**Testing:**
- 7 new integration tests in `tests/db_params_tests.rs`
- Single/multiple parameter extraction
- Missing parameter handling
- Type conversion tests
- JSON/YAML serialization tests

**Documentation:**
- Complete guide in `docs/DB_PARAMS.md`
- Example: `examples/db_params_flow.rs`
- JSON example: `examples/db_params_graph.json`

**Compatibility:**
- Fully backward compatible
- Existing DBNodes work without changes
- Optional feature (params default to None)

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
- 🏗️ **Monolithic Clean Architecture** (NEW)
  - Multi-database architecture with 4 PostgreSQL databases
  - Dynamic field mapping via YAML configuration
  - Zero hardcoded field names in code
  - Database routing per node via config
  - `field_mappings` for flexible data extraction
  - `RuleEngineService` accepts `HashMap<String, Value>`
  - Config-driven `DynamicDBNode` and `DynamicRuleNode`
- 📚 **Comprehensive Documentation**
  - YAML configuration guide with examples
  - Before/After comparison showing improvements
  - Multiple workflow examples
  - Integration guides for both architectures
  - Clean architecture patterns documentation

**Improvements:**
- Monolithic and Microservices both support YAML configs
- Reduced boilerplate code by 70% in executors
- Better separation of concerns (config vs. code)
- Easier testing with multiple configurations
- No recompilation needed for workflow changes
- Complete flexibility in field naming and mapping

**Examples:**
```yaml
# Monolithic with multi-database
nodes:
  oms_history:
    database: "oms_db"
    query: "SELECT ..."
  rule_engine:
    field_mappings:
      avg_daily_demand: "oms_history.avg_daily_demand"
```

```rust
// Dynamic field extraction (no hardcoding)
let inputs = self.extract_rule_inputs(ctx);
rule_service.evaluate(inputs)?;  // HashMap<String, Value>
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
