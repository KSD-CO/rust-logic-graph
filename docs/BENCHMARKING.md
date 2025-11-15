# Benchmarking Guide

This document explains how to run benchmarks and performance tests for Rust Logic Graph.

## Goals

- Provide reproducible microbenchmarks (Criterion)
- Provide a simple load test example
- Provide a perf-regression test to run in CI manually
- Compare caching vs no-cache, executor performance, and load characteristics

## Running Criterion benchmarks

Criterion is added as a dev-dependency. To run benches:

```bash
# Run all criterion benches
cargo bench
```

On macOS this will produce `target/criterion` output with charts and measurements.

### Benchmark details
- `benches/bench_cache.rs` compares executor runtime with and without the caching layer.
- Benchmarks use an async tokio runtime and the `ExpensiveComputeNode` from `examples/cache_flow.rs`.

## Load testing

There's an example load test in `examples/bench_load.rs` that runs many concurrent graphs using a semaphore for concurrency control.

```bash
cargo run --example bench_load
```

Tweak `concurrency` and `total` in the example to model different load profiles.

## Perf regression test

A long-running performance regression test is available under `tests/perf_regression.rs`. It is ignored by default.

Run it with:

```bash
cargo test -- --ignored --nocapture
```

This will run many sequential executions and print average execution time. Adjust threshold in the test to match your environment.

## Comparing with alternatives

To compare with other engines, run the same graph workloads with each system and compare mean/median latency, throughput, and memory usage. Capture results in CSV and plot.

## CI recommendations

- Run `cargo bench` in a dedicated performance CI runner (large runners, pinned CPUs).
- Run `tests/perf_regression.rs` manually or on a schedule, not on every PR.
- Store benchmark artifacts (criterion reports) as job artifacts for analysis.

## Notes

- Criterion highlights noisy runs — run multiple times and take medians.
- Use CPU pinning and consistent hardware for fair comparisons.
- Consider Docker images with fixed environments for reproducible results.
