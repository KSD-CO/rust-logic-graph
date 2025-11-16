# Examples

This folder contains runnable example flows that demonstrate how to use the
rust-logic-graph library. The key example added is `purchasing_flow.rs`, which
models a simple purchasing pipeline (data collection, rule engine, order
calculation, PO creation and send).

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
