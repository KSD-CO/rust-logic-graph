# Rule Engine Service with Rete Algorithm

This service implements the business rules processor using the **Rete Algorithm** for efficient pattern matching and incremental rule evaluation.

## Overview

The Rule Engine Service uses `rust-logic-graph`'s IncrementalEngine, which implements the Rete algorithm - a highly efficient pattern-matching algorithm for production rule systems.

## Rete Algorithm Benefits

### 1. **Incremental Matching**
- Rules are compiled into a Rete network once
- Facts are matched incrementally as they're added/modified
- No need to re-evaluate all rules from scratch

### 2. **Pattern Sharing**
- Common patterns across rules share the same nodes
- Reduces redundant condition evaluations
- Optimizes memory usage

### 3. **Performance**
- O(1) to O(log n) matching complexity per fact change
- Significantly faster than naive rule engines for large rulesets
- Ideal for real-time systems

### 4. **Working Memory**
- Facts persist in working memory
- Rules fire based on current state
- Supports complex multi-fact patterns

## Architecture

```
GRL Rules File
      │
      ▼
GrlReteLoader ──► IncrementalEngine (Rete Network)
                        │
                        ├─► Alpha Network (fact filtering)
                        ├─► Beta Network (join operations)
                        └─► Production Nodes (rule activations)
                        
Facts ──► insert_with_template() ──► Working Memory
                                           │
                                           ▼
                                      fire_all()
                                           │
                                           ▼
                                      Rule Actions
                                           │
                                           ▼
                                     Updated Facts
```

## Template System

Facts are validated against templates for type safety:

```rust
let purchasing_template = TemplateBuilder::new("PurchasingData")
    .float_field("avg_daily_demand")
    .string_field("trend")
    .float_field("available_qty")
    .float_field("moq")
    .float_field("lead_time_days")
    .float_field("unit_price")
    .bool_field("is_active")
    .float_field("demand_lead_time")
    .bool_field("need_reorder")
    .float_field("shortage")
    .float_field("order_qty")
    .float_field("total_amount")
    .bool_field("requires_approval")
    .string_field("approval_status")
    .build();
```

## Usage

### 1. Insert Facts

```rust
let mut facts = TypedFacts::new();
facts.set("avg_daily_demand", FactValue::Float(10.0));
facts.set("trend", FactValue::String("stable".to_string()));
facts.set("available_qty", FactValue::Float(50.0));
// ... more fields

let handle = engine.insert_with_template("PurchasingData", facts)?;
```

### 2. Fire Rules

```rust
engine.reset();
let fired = engine.fire_all();
println!("Fired {} rule activations", fired.len());
```

### 3. Query Results

```rust
if let Some(result) = engine.working_memory().get(&handle) {
    let order_qty = result.data.get("order_qty");
    println!("Order quantity: {:?}", order_qty);
}
```

## GRL Rules Integration

Rules are written in GRL (Generic Rule Language) and loaded into the Rete network:

```grl
rule "Check Reorder Needed"
when
    available_qty < demand_lead_time
then
    set need_reorder = true
    set shortage = demand_lead_time - available_qty
end

rule "Calculate Order Quantity"
when
    need_reorder == true
    shortage > 0
then
    set order_qty = ceil(shortage / moq) * moq
end
```

The `GrlReteLoader` compiles these rules into Rete network nodes.

## API Endpoints

### POST /evaluate

Evaluates purchasing rules using Rete algorithm.

**Request:**
```json
{
  "oms_data": {
    "product_id": "PROD-001",
    "avg_daily_demand": 10.0,
    "trend": "stable"
  },
  "inventory_data": {
    "product_id": "PROD-001",
    "warehouse_id": "WH-001",
    "current_qty": 100,
    "reserved_qty": 25,
    "available_qty": 75
  },
  "supplier_data": {
    "supplier_id": "SUP-001",
    "product_id": "PROD-001",
    "moq": 50,
    "lead_time_days": 7,
    "unit_price": 12.50,
    "is_active": true
  }
}
```

**Response:**
```json
{
  "need_reorder": true,
  "shortage": 70.0,
  "order_qty": 100,
  "total_amount": 1250.00,
  "requires_approval": false,
  "approval_status": "auto_approved"
}
```

### GET /health

Health check endpoint.

## Performance Characteristics

### Rete vs Naive Engine

| Metric | Naive Engine | Rete Engine |
|--------|-------------|-------------|
| Rule Compilation | None | O(R × C) one-time |
| Fact Insertion | O(R × C) | O(log N) |
| Pattern Matching | O(R × F × C) | O(log N) |
| Memory Usage | Low | Higher (network nodes) |
| Best For | Few rules | Many rules, frequent updates |

Where:
- R = Number of rules
- F = Number of facts
- C = Conditions per rule
- N = Network node count

### When to Use Rete

✅ **Use Rete When:**
- You have 10+ rules
- Rules share common patterns
- Facts change frequently
- Need sub-millisecond response times
- Complex multi-condition rules

❌ **Consider Alternatives When:**
- Very few rules (< 5)
- Simple if-then logic
- Memory is extremely constrained
- Rules change more often than facts

## Environment Variables

- `PORT` - Service port (default: 8085)
- Rules file location (searches in order):
  - `/app/rules/purchasing_rules.grl` (Docker)
  - `rules/purchasing_rules.grl` (local)
  - `../../rules/purchasing_rules.grl` (relative)

## Building

```bash
cargo build --release
```

## Running

```bash
# Local
cargo run

# Docker
docker build -t rule-engine-service .
docker run -p 8085:8085 rule-engine-service
```

## Testing

```bash
# Health check
curl http://localhost:8085/health

# Evaluate rules
curl -X POST http://localhost:8085/evaluate \
  -H "Content-Type: application/json" \
  -d '{
    "oms_data": {...},
    "inventory_data": {...},
    "supplier_data": {...}
  }'
```

## Rete Algorithm Details

### Alpha Network
- Filters facts based on single conditions
- One alpha node per unique condition
- Example: `available_qty < 100`

### Beta Network
- Joins facts from multiple alpha nodes
- Implements complex patterns
- Example: `available_qty < demand_lead_time AND is_active == true`

### Working Memory
- Stores all active facts
- Facts are referenced by handles
- Efficient lookup and modification

### Agenda
- Queue of rule activations
- Conflict resolution strategies
- Ensures deterministic execution

## Advanced Features

### Incremental Updates
```rust
// Modify existing fact
engine.retract(&handle);
facts.set("available_qty", FactValue::Float(60.0));
engine.insert_with_template("PurchasingData", facts)?;
engine.fire_all();
```

### Multiple Fact Types
```rust
// Define multiple templates
engine.templates_mut().register(order_template);
engine.templates_mut().register(product_template);

// Insert different fact types
engine.insert_with_template("Order", order_facts)?;
engine.insert_with_template("Product", product_facts)?;
```

### Rule Priorities
GRL supports priorities for conflict resolution:
```grl
rule "High Priority Rule" priority 10
when
    ...
then
    ...
end
```

## Monitoring

Logs include Rete-specific metrics:
- Number of rules fired
- Working memory size
- Pattern matching time
- Network compilation time

```
INFO Fired 5 rule activations
INFO Rete engine evaluation complete: order_qty=100, total_amount=$1250.00, rules_fired=5
```

## References

- [Rete Algorithm Paper](http://www.csl.sri.com/users/mwfong/Technical/RETE%20Matching%20Algorithm%20-%20Forgy%20OCR.pdf)
- [rust-logic-graph Documentation](https://github.com/yourusername/rust-logic-graph)
- [Production Rule Systems](https://en.wikipedia.org/wiki/Production_rule_system)

## License

MIT - See main project LICENSE file
