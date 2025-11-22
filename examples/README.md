# Examples

This folder contains runnable example flows that demonstrate how to use the
rust-logic-graph library.

## 📋 Example Categories

### Basic Examples
- **`simple_flow.rs`** - Basic 3-node pipeline
- **`advanced_flow.rs`** - Complex 6-node workflow
- **`parallel_execution.rs`** - Parallel node execution

### Integration Examples
- **`postgres_flow.rs`** - PostgreSQL integration
- **`openai_flow.rs`** - OpenAI GPT integration
- **`streaming_flow.rs`** - Streaming with backpressure

### GRL (Business Rules) Examples
- **`grl_rules.rs`** - GRL rule examples
- **`grl_graph_flow.rs`** - GRL + Graph integration
- **`purchasing_flow.rs`** - Complete purchasing pipeline with rules

### Advanced Control Flow Examples (v0.9.0) 🆕

#### **`subgraph_flow.rs`** - Reusable Subgraphs
Demonstrates how to create modular, reusable workflow components using YAML configuration.

**YAML Files:**
- `discount_subgraph.yaml` - Reusable discount calculation logic (3 nodes)
- `order_with_subgraph.yaml` - Main workflow that calls the subgraph (5 nodes)

**Key Concepts:**
- Input/output mapping between parent and child contexts
- Encapsulation of business logic
- Reusability across multiple workflows

```bash
cargo run --example subgraph_flow

# Inspect the YAML files
cat examples/discount_subgraph.yaml
cat examples/order_with_subgraph.yaml
```

**Benefits:**
- ✅ No recompilation needed for workflow changes
- ✅ Clear separation of concerns
- ✅ Easy to version control and review
- ✅ Reusable components

#### **`conditional_flow.rs`** - If/Else Routing
Route execution based on runtime conditions.

```bash
cargo run --example conditional_flow
```

#### **`loop_flow.rs`** - Foreach and While Loops
Iterate over arrays or use while loops with safety limits.

```bash
cargo run --example loop_flow
```

#### **`retry_flow.rs`** - Exponential Backoff
Retry failed operations with exponential backoff.

```bash
cargo run --example retry_flow
```

#### **`error_handling_flow.rs`** - Try/Catch/Finally
Handle errors gracefully with try/catch patterns.

```bash
cargo run --example error_handling_flow
```

#### **`circuit_breaker_flow.rs`** - Circuit Breaker
Protect services with circuit breaker pattern.

```bash
cargo run --example circuit_breaker_flow
```

### Error Handling Examples (v0.10.0) 🆕

#### **`error_messages_demo.rs`** - Rich Error Messages
Demonstrates comprehensive error handling with:
- Unique error codes (E001-E012)
- Error classification (Retryable/Permanent/Transient)
- Actionable suggestions
- Rich context propagation
- Documentation links

```bash
cargo run --example error_messages_demo
```

**Key Features:**
- 12 predefined error types with unique codes

### Multi-Database Orchestration (v0.10.0) 🆕

#### **`real_multi_db_orchestration.rs`** - Real Database Demo 🔥
Uses actual PostgreSQL databases from case study purchasing flow with YAML-driven configuration:

**Prerequisites:**
```bash
# Setup databases (creates 4 PostgreSQL databases)
cd case_study
./scripts/setup_multi_databases.sh

# Configure .env with database credentials
cp .env.example .env
```

**Demo 1: Parallel Query Execution (YAML-based)**
- Loads queries from `multi_db_graph.yaml` configuration
- Queries real data from OMS, Inventory, Supplier, and UOM databases
- Fetches actual purchasing flow data for a product
- Demonstrates declarative configuration approach

**Demo 2: Aggregated Dashboard Query**
- Builds purchasing dashboard with metrics from 3 databases
- Total products, inventory levels, supplier statistics
- Demonstrates real-time decision making

```bash
cargo run --example real_multi_db_orchestration
```

**Database Schema:**
- `oms_db`: Order Management System (demand history, trends)
- `inventory_db`: Inventory levels across warehouses
- `supplier_db`: Supplier catalog (MOQ, lead time, pricing)
- `uom_db`: Unit of measurement conversions

**Key Concepts:**
- `ParallelDBExecutor` - Concurrent query execution (see `src/multi_db/parallel.rs`)
- `QueryCorrelator` - SQL-like JOIN operations (see `src/multi_db/correlation.rs`)
- `DistributedTransaction` - 2PC transaction coordination (see `src/multi_db/transaction.rs`)
- YAML-driven configuration - Single source of truth for queries
- Database pool registry pattern

**YAML Configuration:** See `multi_db_graph.yaml` for the active configuration that drives Demo 1 
> Full YAML-based execution requires GraphConfig integration with multi-database routing.

**Use Cases:**
- Aggregating data from multiple databases (OMS + Inventory + Supplier + UOM)
- Real-time purchasing dashboards with metrics from distributed databases
- Parallel query execution for performance optimization
- Multi-tenant systems with database-per-tenant architecture
- Error context chain: node → graph → step → service
- Automatic retry strategy based on error category
- Links to troubleshooting documentation
- Metadata support for debugging

See [docs/ERRORS.md](../docs/ERRORS.md) for complete error reference.

## 🚀 Purchasing Flow Example

Purchasing flow (high-level)
- Data Collection: `oms_history`, `inventory_levels`, `supplier_info`, `uom_conversion` are modeled as DB nodes that return mock data and populate the graph context.
- Rule Engine: `rule_engine` node evaluates business rules and writes flags into context.
- Calculate Order Quantity: `calc_order_qty` computes the order quantity using context values.
- Create PO: `create_po` builds a purchase order object and stores it in context.
- Send PO: `send_po` marks the PO as sent (mock).

Assumptions
- DB nodes are mocked (they do not connect to real databases). They insert synthetic JSON objects into the graph context.
- The rule engine node in the example is a simple `RuleNode` placeholder (`condition = "true"`) to show integration points; replace it with your GRL/Rule engine nodes for production rules.
- Quantity calculation is a simple heuristic for demo purposes: `order_qty = max(0, avg_demand - stock)`, rounded by the supplier MOQ.

How to run
1. Build and run the example:

```bash
cargo run --example purchasing_flow
```

2. The example prints the final `graph.context` to stdout; inspect it to see the computed `po` and `po_sent` entries.

3. To adapt the example to real systems:
- Replace `DBNode` mocks with real DB integration nodes (see `src/integrations/*`).
- Implement rule logic in `rule_engine` using GRL or the `RuleEngine` API.

## Production-Grade Case Study

For a complete, production-ready implementation with real MySQL databases, comprehensive documentation, and advanced monitoring, see the **[case_study/](../case_study/)** directory.

The case study includes:
- 3 versions: Mock, Real DB, and Advanced (with monitoring)
- 4 separate MySQL databases (microservices architecture)
- Complete documentation suite (7 files)
- Helper scripts for easy execution
- Performance benchmarks
- Standalone Rust project

Quick start:
```bash
cd case_study
./scripts/run_mock.sh
```

See [case_study/QUICKSTART_STANDALONE.md](../case_study/QUICKSTART_STANDALONE.md) for details.

Contributing
- If you add real integrations or expand the example, keep data shapes explicit and document expected keys inserted into `graph.context`.
