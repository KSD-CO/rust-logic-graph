# 🧠 Rust Logic Graph

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/github-KSD--CO%2Frust--logic--graph-blue.svg)](https://github.com/KSD-CO/rust-logic-graph)
[![CI](https://github.com/KSD-CO/rust-logic-graph/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-logic-graph/actions)

A high-performance **reasoning graph framework** for Rust with **GRL (Grule Rule Language)** support. Build complex workflows with conditional execution, topological ordering, and async processing.

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
rust-logic-graph = "0.8.8"

# With specific integrations
rust-logic-graph = { version = "0.8.8", features = ["postgres", "openai"] }

# With all integrations
rust-logic-graph = { version = "0.8.8", features = ["all-integrations"] }
```

## 🏢 Real-World Case Study: Purchasing Flow System

See a complete production implementation in **[case_study/](case_study/)** - A full-featured purchasing automation system built with Rust Logic Graph.

### 📊 System Overview

**Problem**: Automate purchasing decisions for inventory replenishment across multiple products, warehouses, and suppliers.

**Solution**: Business rules in GRL decide when/how much to order. Orchestrator executes the workflows.

### 🎯 Two Architecture Implementations

The purchasing flow system demonstrates the same business logic implemented in **two different architectures** - showcasing rust-logic-graph's flexibility for different deployment scenarios.

#### **Architecture Comparison: Pros, Cons & Use Cases**

| Aspect | 🏢 **Monolithic** | 🌐 **Microservices** |
|--------|------------------|---------------------|
| **✅ Advantages** | • **Fast development** - Single codebase, quick iterations<br>• **Low latency** - In-process calls (~10ms)<br>• **Simple deployment** - Single binary<br>• **Easy debugging** - Single process, simple logs<br>• **Low cost** - ~50MB RAM, 1 CPU<br>• **YAML flexibility** - Change workflows without rebuild | • **Horizontal scaling** - Scale services independently<br>• **Team autonomy** - Separate service ownership<br>• **Fault isolation** - Service failure ≠ system failure<br>• **Tech flexibility** - Different languages per service<br>• **Independent deploys** - Update without full restart<br>• **Production proven** - Battle-tested at scale |
| **❌ Disadvantages** | • **Vertical scaling only** - Limited by single machine<br>• **Single point of failure** - Process crash = full outage<br>• **Tight coupling** - All code in one repo<br>• **Resource competition** - Services share CPU/RAM<br>• **Deployment risk** - One deploy affects everything | • **Network overhead** - gRPC calls (~56ms, 5.6x slower)<br>• **Complex setup** - Docker, K8s, service mesh<br>• **High resource usage** - ~500MB RAM, 7 containers<br>• **Debugging complexity** - Distributed tracing needed<br>• **Development friction** - Slower build/test cycles<br>• **Infrastructure cost** - More servers required |
| **🎯 Best Use Cases** | ✅ **Startups** - MVP, validate quickly<br>✅ **Small teams** (1-5 devs)<br>✅ **Low-medium traffic** (<1K req/min)<br>✅ **Cost-sensitive** projects<br>✅ **Frequent changes** - Business logic evolves<br>✅ **Simple ops** - Limited DevOps resources | ✅ **High scale** (>10K req/min)<br>✅ **Large teams** (15+ devs, multiple teams)<br>✅ **Critical uptime** - 99.99% SLA<br>✅ **Independent services** - Different release cycles<br>✅ **Polyglot needs** - Mix languages/frameworks<br>✅ **Regulatory** - Service isolation required |
| **⚠️ Anti-patterns** | ❌ Don't use if:<br>• Need >10K requests/min<br>• Team >15 developers<br>• Services need independent scaling<br>• Require 99.99% uptime | ❌ Don't use if:<br>• Team <5 developers<br>• Traffic <1K requests/min<br>• Premature optimization<br>• No DevOps expertise |
| **🏗️ Architecture** | Single HTTP service (Port 8080)<br>4 PostgreSQL DBs (multi-database)<br>YAML-driven graph execution | 7 services (gRPC + HTTP)<br>4 PostgreSQL DBs (service-owned)<br>Hardcoded gRPC graph topology |
| **📊 Performance** | ~10ms latency (in-process)<br>~50MB RAM, 1 CPU core | ~56ms latency (network calls)<br>~500MB RAM, 7 containers |

#### **When to Use Each Architecture**

**✅ Use Monolithic When:**
- 🚀 **Early stage startup** - Fast iteration, quick deployments
- 💰 **Limited resources** - Small team, limited infrastructure budget
- 📊 **Low-medium traffic** - <1000 requests/minute
- 🎯 **MVP/Prototype** - Need to validate business logic quickly
- 🛠️ **Simple operations** - Single deployment, easy monitoring
- 👥 **Small team** - 1-5 developers, full-stack ownership
- 🔧 **Frequent changes** - Business logic changes often, need flexibility
- 💵 **Cost-sensitive** - Minimize cloud costs, fewer resources

**Monolithic Example (Port 8080):**
```bash
cd case_study/monolithic
cargo run --release
curl -X POST http://localhost:8080/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

**✅ Use Microservices When:**
- 📈 **High scale** - >10,000 requests/minute, need horizontal scaling
- 👥 **Large team** - Multiple teams, service ownership per team
- 🔧 **Independent deployments** - Deploy services independently
- 🛡️ **Fault isolation** - Service failure shouldn't crash entire system
- 🌍 **Polyglot needs** - Different services in different languages
- 🔄 **Different SLAs** - Critical services need higher availability
- 📊 **Complex monitoring** - Distributed tracing, service mesh
- 💰 **Budget for infrastructure** - Can afford Kubernetes, service mesh

**Microservices Example (7 Services):**
```bash
cd case_study/microservices
docker compose up -d
curl -X POST http://localhost:8080/api/purchasing/flow \
  -H "Content-Type: application/json" \
  -d '{"product_id": "PROD-001"}'
```

#### **Migration Path: Start Monolithic → Scale to Microservices**

1. **Phase 1: Start Monolithic**
   - Build and validate business logic
   - Use YAML config for flexibility
   - Deploy single binary

2. **Phase 2: Extract Critical Services**
   - Identify bottlenecks (e.g., Rule Engine)
   - Extract to separate service
   - Keep rest monolithic

3. **Phase 3: Full Microservices**
   - Split all services when scale demands
   - Add service mesh, observability
   - Use Kubernetes for orchestration

**Both implementations use:**
- ✅ Same GRL business rules (15 rules in `purchasing_rules.grl`)
- ✅ Same graph topology (OMS → Inventory → Supplier → UOM → RuleEngine → PO)
- ✅ rust-logic-graph's Graph/Executor pattern
- ✅ Clean architecture principles

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

Both Monolithic and Microservices implementations support **YAML-based graph configuration**, but with different approaches:

**Monolithic YAML Example** (`purchasing_flow_graph.yaml`):
```yaml
nodes:
  oms_history:
    type: DBNode
    database: "oms_db"  # Multi-database routing
    query: "SELECT product_id, avg_daily_demand::float8, trend FROM oms_history WHERE product_id = $1"
  
  inventory_levels:
    type: DBNode
    database: "inventory_db"
    query: "SELECT product_id, available_qty::float8, reserved_qty::float8 FROM inventory WHERE product_id = $1"
  
  rule_engine:
    type: RuleNode
    description: "Evaluate business rules with dynamic field mapping"
    dependencies:
      - oms_history
      - inventory_levels
      - supplier_info
      - uom_conversion
    field_mappings:  # Dynamic field extraction (NEW)
      avg_daily_demand: "oms_history.avg_daily_demand"
      available_qty: "inventory_levels.available_qty"
      lead_time: "supplier_info.lead_time"
      moq: "supplier_info.moq"

  create_po:
    type: RuleNode
    dependencies:
      - rule_engine
    field_mappings:
      should_order: "rule_engine.should_order"
      recommended_qty: "rule_engine.recommended_qty"
      product_id: "supplier_info.product_id"

edges:
  - from: oms_history
    to: rule_engine
  - from: inventory_levels
    to: rule_engine
  - from: rule_engine
    to: create_po
```

**Microservices YAML Example** (`purchasing_flow_graph.yaml`):
```yaml
nodes:
  oms_grpc:
    type: GrpcNode
    query: "http://localhost:50051#GetOrderHistory"
    description: "Fetch order management data via gRPC"
  
  inventory_grpc:
    type: GrpcNode
    query: "http://localhost:50052#GetInventoryLevels"
    description: "Fetch inventory levels via gRPC"
  
  supplier_grpc:
    type: GrpcNode
    query: "http://localhost:50053#GetSupplierInfo"
    description: "Fetch supplier information via gRPC"
  
  uom_grpc:
    type: GrpcNode
    query: "http://localhost:50054#ConvertUnits"
    description: "Fetch UOM conversions via gRPC"
  
  rule_engine_grpc:
    type: RuleNode
    description: "Evaluate business rules"
    dependencies:
      - oms_grpc
      - inventory_grpc
      - supplier_grpc
      - uom_grpc
  
  po_grpc:
    type: RuleNode
    description: "Create purchase order"
    dependencies:
      - rule_engine_grpc

edges:
  - from: oms_grpc
    to: rule_engine_grpc
  - from: inventory_grpc
    to: rule_engine_grpc
  - from: supplier_grpc
    to: rule_engine_grpc
  - from: uom_grpc
    to: rule_engine_grpc
  - from: rule_engine_grpc
    to: po_grpc
```

**Key Differences:**

| Feature | Monolithic YAML | Microservices YAML |
|---------|----------------|-------------------|
| **Node Type** | `DBNode` (direct SQL) | `GrpcNode` (service calls) |
| **Query** | SQL queries | gRPC endpoint URLs |
| **Database Routing** | `database: "oms_db"` | No database (delegates to services) |
| **Field Mappings** | ✅ Dynamic via YAML | ❌ Hardcoded in Node implementations |
| **Flexibility** | 100% config-driven | Hybrid (topology in YAML, logic in code) |

**Benefits:**
- ✅ **70% less code** - Graph definition moves from Rust to YAML
- ✅ **No recompile** - Change workflows without rebuilding
- ✅ **Dynamic field mapping** (Monolithic only) - Zero hardcoded field names
- ✅ **Multi-database routing** (Monolithic only) - Each node specifies its database
- ✅ **Service URLs** (Microservices only) - Configure gRPC endpoints
- ✅ **Better readability** - Clear, declarative graph structure
- ✅ **Easy testing** - Test with different configurations

**Key Architecture Differences:**

| Aspect | Monolithic | Microservices |
|--------|-----------|---------------|
| **Service Count** | 1 service | 7 services (Orchestrator, OMS, Inventory, Supplier, UOM, RuleEngine, PO) |
| **Ports** | Single port 8080 | Orchestrator: 8080, Services: 50051-50056 (gRPC) |
| **Database Access** | Direct SQL queries to 4 DBs | gRPC calls to service APIs |
| **Field Mapping** | YAML `field_mappings` config | Hardcoded in gRPC node implementations |
| **Rule Engine** | In-process RuleEngine call | gRPC to rule-engine-service :50055 |
| **Communication** | Function calls (0 network) | gRPC (network overhead) |
| **Graph Executor** | `PurchasingGraphExecutor` | `OrchestratorExecutor` with gRPC nodes |
| **Node Types** | `DynamicDBNode`, `DynamicRuleNode` | `OmsGrpcNode`, `InventoryGrpcNode`, etc. |
| **Configuration** | 100% YAML-driven | Partially hardcoded gRPC contracts |
| **Flexibility** | Change workflow via YAML only | Need code changes for new services |
| **Dependencies** | rust-logic-graph + sqlx | rust-logic-graph + tonic + prost |
| **Deployment** | `cargo run` or single binary | `docker compose up` (11 containers) |
| **Development** | Hot reload, fast compile | Rebuild multiple containers |
| **Production Ready** | ✅ Yes (single binary) | ✅ Yes (Docker/K8s) |

**Example Response Time Comparison:**

```
Monolithic (in-process):
┌─────────────────────────────────────┐
│ HTTP Request → Graph Executor       │ ~2ms
│ ├─ DB Query (oms_db)                │ ~1ms
│ ├─ DB Query (inventory_db)          │ ~1ms
│ ├─ DB Query (supplier_db)           │ ~1ms
│ ├─ DB Query (uom_db)                │ ~1ms
│ ├─ Rule Engine (in-process)         │ ~2ms
│ └─ Create PO (in-process)           │ ~2ms
│ Total: ~10ms                        │
└─────────────────────────────────────┘

Microservices (network calls):
┌─────────────────────────────────────┐
│ HTTP Request → Orchestrator         │ ~2ms
│ ├─ gRPC OMS Service (50051)         │ ~8ms (network + DB)
│ ├─ gRPC Inventory (50052)           │ ~8ms (network + DB)
│ ├─ gRPC Supplier (50053)            │ ~8ms (network + DB)
│ ├─ gRPC UOM (50054)                 │ ~8ms (network + DB)
│ ├─ gRPC Rule Engine (50055)         │ ~12ms (network + rules)
│ └─ gRPC PO Service (50056)          │ ~10ms (network + create)
│ Total: ~56ms (5.6x slower)          │
└─────────────────────────────────────┘
```

**Trade-offs Summary:**

| Consideration | Monolithic Wins | Microservices Wins |
|---------------|----------------|-------------------|
| **Performance** | ✅ 5-10x faster | ❌ Network overhead |
| **Simplicity** | ✅ Single process | ❌ Complex setup |
| **Resource Usage** | ✅ ~50MB RAM | ❌ ~500MB RAM |
| **Development Speed** | ✅ Faster iteration | ❌ Slower builds |
| **Scalability** | ❌ Vertical only | ✅ Horizontal scale |
| **Team Autonomy** | ❌ Shared codebase | ✅ Independent teams |
| **Fault Isolation** | ❌ Single point of failure | ✅ Service isolation |
| **Deployment** | ✅ Single binary | ❌ Multi-container |
| **Monitoring** | ✅ Simple logs | ❌ Distributed tracing |
| **Cost** | ✅ Lower infra cost | ❌ Higher infra cost |

**Real-World Recommendation:**

```
Traffic Level          | Recommended Architecture
-----------------------|-------------------------
< 100 req/min          | Monolithic (overkill to use microservices)
100-1,000 req/min      | Monolithic (scales easily vertically)
1,000-10,000 req/min   | Monolithic or Hybrid (extract bottlenecks)
> 10,000 req/min       | Microservices (horizontal scaling needed)

Team Size              | Recommended Architecture
-----------------------|-------------------------
1-5 developers         | Monolithic (single codebase)
5-15 developers        | Monolithic or Hybrid
15-50 developers       | Microservices (team per service)
> 50 developers        | Microservices (clear boundaries)
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
        │  │  │ RuleEngineGrpcNode → gRPC :50055               │  │  │
        │  │  │ PoGrpcNode → gRPC :50056                       │  │  │
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
                   │ Rule Engine     │ (Port 50055 - gRPC)          │
                   │     :50055      │                              │
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
                   │ PO Service      │ (Port 50056 - gRPC)          │
                   │    :50056       │◄─────────────────────────────┘
                   │   (gRPC)        │
                   │                 │
                   │ • Create PO     │ • Reads flags from context
                   │ • Send to       │ • Executes based on rules
                   │   Supplier      │ • Email/API delivery
                   └─────────────────┘
```
**Note**: The Orchestrator uses **rust-logic-graph's Graph/Executor pattern** - each gRPC service call is wrapped in a custom `Node` implementation. The Rule Engine returns decision flags to the Graph Context, and the PoGrpcNode reads these flags to determine whether to create/send the PO.

### Where rust-logic-graph is Used

**Monolithic App** (`case_study/monolithic/`):
- Uses `Graph`, `Executor`, and custom `Node` implementations
- **Multi-database architecture**: 4 separate PostgreSQL databases (oms_db, inventory_db, supplier_db, uom_db)
- **Dynamic field mapping**: YAML-configured field extraction with zero hardcoded field names
- **Config-driven nodes**: `DynamicDBNode` and `DynamicRuleNode` read behavior from YAML
- Database routing via `database` field in YAML (e.g., `database: "oms_db"`)
- Field mappings via `field_mappings` in YAML (e.g., `avg_daily_demand: "oms_history.avg_daily_demand"`)
- `RuleEngineService` accepts `HashMap<String, Value>` for complete flexibility
- Graph structure defined in `purchasing_flow_graph.yaml`
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

**Architecture Highlights:**

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

**Key Design Principles:**

1. **Multi-Database Routing** - Each node specifies its database in YAML:
   ```yaml
   oms_history:
     database: "oms_db"  # Routes to oms_db pool
   ```

2. **Dynamic Field Mapping** - Zero hardcoded fields in Rust code:
   ```yaml
   field_mappings:
     avg_daily_demand: "oms_history.avg_daily_demand"
   ```
   ```rust
   // Code is 100% generic
   for (key, path) in &self.field_mappings {
       inputs.insert(key.clone(), get_value_by_path(ctx, path));
   }
   ```

3. **Config-Driven Execution** - Graph structure in YAML, not Rust:
   ```rust
   executor.execute_with_config("PROD-001", "purchasing_flow_graph.yaml")?;
   ```

4. **HashMap-Based RuleEngine** - Accepts any fields:
   ```rust
   pub fn evaluate(&mut self, inputs: HashMap<String, Value>) -> Result<Output>
   ```

**Microservices Communication Flow**

1. **Multi-Database Routing** (`graph_executor.rs`):
```rust
// YAML config specifies database per node
oms_history:
  database: "oms_db"
  query: "SELECT ..."

// Executor routes to correct pool
let pool = self.get_pool(node_config.database.as_deref());
```

2. **Dynamic Field Mapping** (`graph_executor.rs`):
```rust
// YAML config defines field mappings
field_mappings:
  avg_daily_demand: "oms_history.avg_daily_demand"
  available_qty: "inventory_levels.available_qty"

// Code extracts dynamically (zero hardcoding)
fn extract_inputs(&self, ctx: &Context) -> HashMap<String, Value> {
    for (key, path) in &self.field_mappings {
        if let Some(value) = self.get_value_by_path(ctx, path) {
            inputs.insert(key.clone(), value);
        }
    }
}
```

3. **Config-Driven RuleEngine** (`rule_service.rs`):
```rust
// Accepts HashMap instead of struct - 100% flexible
pub fn evaluate(&mut self, inputs: HashMap<String, Value>) -> Result<Output> {
    // Uses any fields present in HashMap
    // No hardcoded field requirements
}
```

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
