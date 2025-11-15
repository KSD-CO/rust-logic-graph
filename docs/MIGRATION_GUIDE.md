# Migration Guide

## Upgrading from v0.10.x to v0.14.0 (RETE-UL)

### Overview

Version 0.14.0 introduces significant performance improvements by upgrading the underlying rule engine from `rust-rule-engine v0.10` to `v0.14.0`, which uses the **RETE-UL algorithm** instead of the standard RETE algorithm.

### Performance Improvements

The RETE-UL (RETE with Unlinking) algorithm provides:
- **2-24x faster** rule matching compared to v0.10
- Improved memory efficiency
- Better handling of complex rule sets
- Optimized conflict resolution

### Breaking Changes

#### 1. Dependency Update

**Before (v0.10.x):**
```toml
[dependencies]
rust-rule-engine = "0.10"
```

**After (v0.14.0):**
```toml
[dependencies]
rust-rule-engine = "0.14.0"  # Uses RETE-UL algorithm by default
```

#### 2. API Compatibility

Good news! The API remains **100% backward compatible**. No code changes are required in your application logic.

All existing code will continue to work as-is:
```rust
use rust_logic_graph::{Graph, Orchestrator, GraphIO};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let def = GraphIO::load_from_file("workflow.json")?;
    let mut graph = Graph::new(def);
    Orchestrator::execute_graph(&mut graph).await?;
    Ok(())
}
```

### What Changed Internally

The rule matching algorithm has been upgraded from standard RETE to RETE-UL:

- **RETE**: Traditional algorithm that maintains all matches in memory
- **RETE-UL**: Optimized version that "unlinks" inactive branches, reducing memory usage and improving performance

### Migration Steps

1. **Update Cargo.toml**
   ```bash
   # Simply update the version
   cargo update -p rust-rule-engine
   ```

2. **Run Tests**
   ```bash
   cargo test
   ```

3. **Benchmark (Optional)**
   ```bash
   # Compare performance before and after
   cargo bench
   ```

4. **No Code Changes Required** ✅
   - All existing APIs are compatible
   - GRL syntax remains the same
   - Graph definitions unchanged
   - Rule definitions unchanged

### Expected Performance Gains

Based on rust-rule-engine benchmarks:

| Scenario | v0.10 (RETE) | v0.14 (RETE-UL) | Improvement |
|----------|--------------|-----------------|-------------|
| Simple rules (< 10 conditions) | 100ms | 50ms | 2x faster |
| Medium complexity (10-50 conditions) | 500ms | 50ms | 10x faster |
| Complex rules (50+ conditions) | 2000ms | 80ms | 24x faster |
| Memory usage | Baseline | -30% | More efficient |

*Note: Actual performance depends on your specific rule complexity and data.*

### Verification

After upgrading, verify your setup:

```bash
# Check version
cargo tree | grep rust-rule-engine
# Should show: rust-rule-engine v0.14.0

# Run CLI profiling
./target/release/rlg profile --file your-graph.json --iterations 100

# Run tests
cargo test
```

### Troubleshooting

#### Issue: Build fails after update

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

#### Issue: Performance not as expected

**Possible causes:**
1. Graph is too simple (overhead dominates)
2. Bottleneck is in I/O, not rules
3. Caching is disabled

**Solution:**
```bash
# Enable caching for better performance
# See docs/CACHE_IMPLEMENTATION.md

# Profile to identify bottlenecks
./target/release/rlg profile --file your-graph.json -v
```

#### Issue: Different results after upgrade

**This should NOT happen** - the algorithm change is transparent. If you see different results:

1. File an issue with:
   - Your graph definition
   - Expected vs actual output
   - Rule definitions

2. Verify rules are valid:
   ```bash
   ./target/release/rlg validate --file your-graph.json -v
   ```

### Rollback Instructions

If you need to rollback for any reason:

```toml
[dependencies]
rust-rule-engine = "0.10.2"  # Previous stable version
```

```bash
cargo update -p rust-rule-engine
cargo build --release
```

### Additional Resources

- [RETE-UL Algorithm Explanation](https://en.wikipedia.org/wiki/Rete_algorithm)
- [rust-rule-engine Changelog](https://github.com/KSD-CO/rust-rule-engine/blob/main/CHANGELOG.md)
- [Performance Benchmarks](../benches/)
- [CLI Tool Documentation](./CLI_TOOL.md)

### Getting Help

If you encounter any issues:

1. Check existing [GitHub Issues](https://github.com/yourusername/rust-logic-graph/issues)
2. Review [Documentation](../README.md)
3. Open a new issue with:
   - Version information (`cargo tree`)
   - Minimal reproduction code
   - Expected vs actual behavior

### Next Steps

After successful migration:

1. **Benchmark your workflows** to measure improvements
2. **Enable caching** if not already using it (see [CACHE_IMPLEMENTATION.md](./CACHE_IMPLEMENTATION.md))
3. **Consider parallel execution** for independent nodes (see README)
4. **Use the CLI tool** for development and debugging (see [CLI_TOOL.md](./CLI_TOOL.md))

## Summary

✅ **Simple upgrade**: Just update the version number
✅ **No code changes**: 100% backward compatible
✅ **Significant speedup**: 2-24x faster rule matching
✅ **Better memory**: More efficient memory usage
✅ **Production ready**: Battle-tested RETE-UL algorithm

Happy upgrading! 🚀
