# CLI Tool Documentation

The Rust Logic Graph CLI tool (`rlg`) provides developer utilities for validating, testing, profiling, and visualizing logic graphs.

## Installation

Build and install the CLI tool:

```bash
cargo build --release
# The binary will be at: target/release/rlg

# Optionally, install it globally:
cargo install --path .
```

## Commands

### 1. Graph Validation

Validate a graph definition file to check for common issues.

```bash
rlg validate --file examples/sample_graph.json
```

**Options:**
- `-f, --file <PATH>`: Path to the graph definition file (JSON format)
- `-v, --verbose`: Show detailed validation steps

**Checks performed:**
- Node ID uniqueness
- Edge references (all edges point to existing nodes)
- Cycle detection (warning)
- Unreachable nodes detection (warning)
- Empty graph detection

**Example output:**
```
Validating graph...
  ✓ Checking node uniqueness...
  ✓ Checking edge references...
  ✓ Checking for cycles...
  ✓ Checking node reachability...

✓ Graph is valid!
  Nodes: 5
  Edges: 4
```

### 2. Dry-Run Execution

Execute a graph in dry-run mode without performing actual operations.

```bash
rlg dry-run --file examples/sample_graph.json --context examples/sample_context.json
```

**Options:**
- `-f, --file <PATH>`: Path to the graph definition file
- `-c, --context <PATH>`: Path to the context/input data file (optional)
- `-v, --verbose`: Show execution order and details

**Example output:**
```
Running graph in dry-run mode...
  Loading context from: examples/sample_context.json

Execution Plan:
  Total nodes: 5
  Total edges: 4

Node execution order:
  1. start (Rule { rule: "check_inventory" })
  2. check_price (Rule { rule: "validate_price" })
  3. process_payment (Rule { rule: "process_payment" })
  4. send_confirmation (Rule { rule: "send_email" })
  5. end (Rule { rule: "complete_order" })

✓ Dry-run completed (no actual execution performed)
```

### 3. Performance Profiling

Profile the execution performance of a graph.

```bash
rlg profile --file examples/sample_graph.json --iterations 100
```

**Options:**
- `-f, --file <PATH>`: Path to the graph definition file
- `-c, --context <PATH>`: Path to the context/input data file (optional)
- `-i, --iterations <N>`: Number of iterations to run (default: 1)

**Example output:**
```
Profiling graph execution...
  Running 100 iteration(s)...

Performance Profile:
  Iterations: 100
  Total time: 1.234567s
  Average time: 12.345670ms
  Min time: 10.123456ms
  Max time: 15.678901ms
  Throughput: 80.99 ops/sec
```

### 4. Graph Visualization

Visualize a graph structure in ASCII format.

```bash
rlg visualize --file examples/sample_graph.json
```

**Options:**
- `-f, --file <PATH>`: Path to the graph definition file
- `-d, --details`: Show detailed node information

**Example output:**
```
Graph Visualization

Nodes:
  start
  check_price
  process_payment
  send_confirmation
  end

Edges:
  start ├─> check_price
  check_price ├─> process_payment
  process_payment ├─> send_confirmation
  send_confirmation └─> end

ASCII Graph:
  [start]
  └── [check_price]
      └── [process_payment]
          └── [send_confirmation]
              └── [end]

Statistics:
  Total nodes: 5
  Total edges: 4
  Entry points: 1 (["start"])
  Exit points: 1 (["end"])
```

## Example Graphs

### Simple Linear Graph

Located at [examples/sample_graph.json](../examples/sample_graph.json)

```json
{
  "nodes": {
    "start": { "Rule": { "rule": "check_inventory" } },
    "check_price": { "Rule": { "rule": "validate_price" } },
    "process_payment": { "Rule": { "rule": "process_payment" } },
    "send_confirmation": { "Rule": { "rule": "send_email" } },
    "end": { "Rule": { "rule": "complete_order" } }
  },
  "edges": [
    { "from": "start", "to": "check_price", "rule": null },
    { "from": "check_price", "to": "process_payment", "rule": null },
    { "from": "process_payment", "to": "send_confirmation", "rule": null },
    { "from": "send_confirmation", "to": "end", "rule": null }
  ]
}
```

### Cyclic Graph (for testing)

Located at [examples/cyclic_graph.json](../examples/cyclic_graph.json)

This graph contains a cycle and will trigger validation warnings:

```bash
rlg validate --file examples/cyclic_graph.json
```

## Context File Format

Context files contain input data for graph execution in JSON format:

```json
{
  "product_id": "12345",
  "quantity": 2,
  "price": 29.99,
  "customer_email": "customer@example.com",
  "inventory_count": 100
}
```

## Use Cases

### Development Workflow

1. **Create/Edit Graph**: Design your graph structure
2. **Validate**: Check for errors
   ```bash
   rlg validate -f my_graph.json -v
   ```
3. **Visualize**: Understand the structure
   ```bash
   rlg visualize -f my_graph.json -d
   ```
4. **Dry-Run**: Test the execution flow
   ```bash
   rlg dry-run -f my_graph.json -c my_context.json -v
   ```
5. **Profile**: Optimize performance
   ```bash
   rlg profile -f my_graph.json -i 1000
   ```

### CI/CD Integration

Add graph validation to your CI pipeline:

```bash
# Validate all graph files
for graph in graphs/*.json; do
  rlg validate --file "$graph" || exit 1
done
```

### Performance Regression Testing

```bash
# Baseline
rlg profile -f my_graph.json -i 100 > baseline.txt

# After changes
rlg profile -f my_graph.json -i 100 > current.txt

# Compare results
diff baseline.txt current.txt
```

## Troubleshooting

### Common Errors

1. **"Edge references non-existent node"**
   - Fix: Ensure all edge `from` and `to` fields reference valid node IDs

2. **"Graph contains cycles"**
   - This is a warning, not an error
   - Cycles can cause infinite loops during execution
   - Review your graph logic

3. **"Unreachable nodes found"**
   - Some nodes have no path from entry points
   - These nodes will never execute
   - Consider removing them or adding edges

### File Format Issues

- Only JSON format is currently supported
- Ensure valid JSON syntax
- Node IDs must be strings
- Edge references must match exact node IDs

## Advanced Usage

### Scripting

Use the CLI in scripts for automation:

```bash
#!/bin/bash

GRAPH="production_workflow.json"
CONTEXT="production_data.json"

# Validate
if ! rlg validate -f "$GRAPH"; then
  echo "Validation failed!"
  exit 1
fi

# Profile
rlg profile -f "$GRAPH" -c "$CONTEXT" -i 50

# Generate visualization
rlg visualize -f "$GRAPH" -d > graph_structure.txt
```

### Performance Monitoring

Track performance over time:

```bash
# Add to cron job or monitoring system
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
rlg profile -f critical_graph.json -i 100 > "perf_${TIMESTAMP}.log"
```

## Future Enhancements

Planned features:
- YAML format support
- Export visualizations to DOT/Graphviz format
- Interactive graph editing
- Real-time execution monitoring
- Performance comparison tools
- Graph diff/merge utilities

## Contributing

To add new CLI commands:

1. Add command variant to `Commands` enum in [src/bin/rlg.rs](../src/bin/rlg.rs)
2. Implement handler function
3. Add to match statement in `main()`
4. Update this documentation
5. Add tests in [tests/cli_tests.rs](../tests/cli_tests.rs)

## See Also

- [Main README](../README.md)
- [Examples](../examples/README.md)
- [API Documentation](https://docs.rs/rust-logic-graph)
