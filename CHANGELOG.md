# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.5] - 2025-11-20

### Added
- **YAML Configuration Support** - Declarative graph definitions
  - `GraphConfig` module for parsing YAML configurations (`serde_yaml`)
  - Load graph structure from external YAML files
  - Support for both JSON and YAML formats
  - Dynamic node registration from configuration files
- **Enhanced Graph Executor API**
  - `execute()` method - uses default configuration
  - `execute_with_config(config_path)` method - loads custom YAML config
  - Automatic node creation from YAML definitions
- **Multiple Workflow Support**
  - Easy creation of workflow variants (standard, simplified, urgent, approval)
  - Config-driven workflow selection
  - A/B testing support with different configs
- **Case Study Updates**
  - Monolithic version with YAML configuration
  - Microservices orchestrator with YAML configuration
  - Example configs: `purchasing_flow_graph.yaml`, `simplified_flow_graph.yaml`, `urgent_flow_graph.yaml`
- **Comprehensive Documentation**
  - [YAML Configuration Summary](case_study/YAML_CONFIGURATION_SUMMARY.md)
  - [Graph Config README](case_study/microservices/services/orchestrator-service/GRAPH_CONFIG_README.md)
  - [Comparison: Before/After](case_study/monolithic/COMPARISON_BEFORE_AFTER.md)
  - Usage examples and best practices

### Changed
- Graph executors now support both hardcoded and YAML-based configurations
- Improved separation of concerns (configuration vs. implementation)
- Reduced boilerplate code by ~70% in graph executors

### Dependencies
- Added `serde_yaml = "0.9"` for YAML parsing

### Benefits
- 📉 **70% less code** - Graph definitions move from code to YAML
- 🚀 **No recompile** - Change workflows without rebuilding
- 🔄 **Multiple workflows** - Easy variant creation
- 📖 **Better readability** - Clear, declarative structure
- 🧪 **Easier testing** - Test with different configurations

### Example
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

## [0.7.0] - 2025-11-16

### Added
- **Memory Optimization** - Comprehensive memory management features
  - Context pooling for reduced allocations
  - Memory metrics and tracking
  - Allocation profiling utilities
  - RAII guards for automatic cleanup
- **New Module**: `memory` module with pooling and metrics
- **Documentation**: [Memory Optimization Guide](docs/MEMORY_OPTIMIZATION.md)
- **Benchmarks**: Memory performance benchmarks in `benches/memory_bench.rs`

### Dependencies
- Added `parking_lot = "0.12"` for efficient synchronization
- Added `once_cell = "1"` for lazy static initialization

### Performance
- 2-3x faster context creation with pooling
- 50-98% reduction in allocations depending on reuse rate
- Memory usage tracking and profiling capabilities

### Features
- `ContextPool` - Object pool for Context reuse
- `PoolConfig` - Configurable pool parameters
- `MemoryMetrics` - Global allocation tracking
- `AllocationTracker` - Scoped memory profiling
- `MemoryProfiler` - Function-level profiling
- Pool statistics and reuse rate monitoring

## [0.5.0] - 2025-11-16

### Breaking Changes
- **Upgraded `rust-rule-engine`** from v0.10 to v0.14.0
  - Now uses RETE-UL algorithm (2-24x faster than previous version)
  - Improved rule matching performance
  - Better memory efficiency
  - 98% Drools compatible (up from 97%)
  - API is 100% backward compatible
  - See [Migration Guide](docs/MIGRATION_GUIDE.md)

### Added
- **CLI Tool** (`rlg` binary) for developer productivity
  - Graph validation with comprehensive checks (cycles, unreachable nodes, edge references)
  - Dry-run execution mode with execution order visualization
  - Performance profiling with min/max/avg statistics and throughput calculation
  - ASCII graph visualization with tree structure
- **Caching Layer** for high-performance result caching
  - TTL-based expiration
  - Multiple eviction policies (LRU, LFU, FIFO)
  - Memory limits and usage tracking
  - Cache statistics and hit rates
- **Parallel Execution** improvements
  - Automatic layer detection
  - Concurrent execution of independent nodes
  - Parallelism statistics
- **Documentation**
  - [CLI Tool Guide](docs/CLI_TOOL.md) - Complete CLI documentation
  - [Cache Implementation Guide](docs/CACHE_IMPLEMENTATION.md) - Caching documentation
  - [Migration Guide](docs/MIGRATION_GUIDE.md) - v0.14.0 upgrade guide
  - [CHANGELOG.md](CHANGELOG.md) - Project changelog
- **Example Files**
  - `examples/sample_graph.json` - Sample linear workflow
  - `examples/cyclic_graph.json` - Cyclic graph for testing
  - `examples/sample_context.json` - Sample input data
- **Tests**
  - CLI validation tests
  - Graph serialization tests
  - Integration tests for file loading
  - Total: 32 tests passing

### Dependencies
- Added `clap = "4"` for CLI argument parsing
- Added `colored = "2"` for terminal colors
- Added `tempfile = "3"` (dev) for testing

### Performance
- 2-24x faster rule matching with RETE-UL
- Better memory efficiency with cache management
- Parallel execution of independent nodes
- Automatic parallel execution of independent graph layers
- Result caching to avoid redundant computations
- Configurable eviction policies (LRU, LFU, FIFO)

## [0.4.0] - 2025-11-06

### Added
- Parallel execution support
- Automatic layer identification for parallel processing
- Performance benchmarks

## [0.3.0] - Previous

### Added
- Streaming processing support
- Stream-based node execution
- Backpressure handling
- Kubernetes deployment guides

## [0.2.0] - Previous

### Added
- Database integrations (PostgreSQL, MySQL, Redis, MongoDB)
- AI/LLM integrations (OpenAI, Claude, Ollama)

## [0.1.0] - Initial Release

### Added
- Core graph execution engine
- Topological ordering
- Rule-based execution
- GRL support
- Basic node types (RuleNode, DBNode, AINode)

[Unreleased]: https://github.com/yourusername/rust-logic-graph/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/yourusername/rust-logic-graph/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/yourusername/rust-logic-graph/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/yourusername/rust-logic-graph/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yourusername/rust-logic-graph/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/rust-logic-graph/releases/tag/v0.1.0
