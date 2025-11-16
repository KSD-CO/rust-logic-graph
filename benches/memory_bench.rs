use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_logic_graph::{ContextPool, PoolConfig, Context, MemoryMetrics};

fn bench_context_without_pool(c: &mut Criterion) {
    c.bench_function("context_without_pool", |b| {
        b.iter(|| {
            let mut ctx = Context::new();
            ctx.data.insert("key1".to_string(), black_box(serde_json::json!(100)));
            ctx.data.insert("key2".to_string(), black_box(serde_json::json!("value")));
            black_box(ctx);
        });
    });
}

fn bench_context_with_pool(c: &mut Criterion) {
    let pool = ContextPool::new();

    c.bench_function("context_with_pool", |b| {
        b.iter(|| {
            let mut ctx = pool.acquire();
            ctx.data.insert("key1".to_string(), black_box(serde_json::json!(100)));
            ctx.data.insert("key2".to_string(), black_box(serde_json::json!("value")));
            pool.release(ctx);
        });
    });
}

fn bench_pool_acquire_release(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_operations");

    for pool_size in [10, 50, 100] {
        let config = PoolConfig {
            max_pooled: pool_size,
            initial_capacity: 16,
            track_stats: true,
        };
        let pool = ContextPool::with_config(config);

        // Pre-fill pool
        for _ in 0..pool_size {
            pool.release(Context::new());
        }

        group.bench_with_input(
            BenchmarkId::new("acquire_release", pool_size),
            &pool,
            |b, pool| {
                b.iter(|| {
                    let ctx = pool.acquire();
                    pool.release(ctx);
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_metrics(c: &mut Criterion) {
    c.bench_function("memory_metrics_record", |b| {
        let metrics = MemoryMetrics::new();
        b.iter(|| {
            metrics.record_alloc(black_box(1024));
            metrics.record_dealloc(black_box(512));
        });
    });
}

fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Benchmark: Many small allocations without pool
    group.bench_function("many_small_without_pool", |b| {
        b.iter(|| {
            let mut contexts = Vec::new();
            for i in 0..100 {
                let mut ctx = Context::new();
                ctx.data.insert("key".to_string(), black_box(serde_json::json!(i)));
                contexts.push(ctx);
            }
            black_box(contexts);
        });
    });

    // Benchmark: Many small allocations with pool
    group.bench_function("many_small_with_pool", |b| {
        let pool = ContextPool::new();
        b.iter(|| {
            let mut contexts = Vec::new();
            for i in 0..100 {
                let mut ctx = pool.acquire();
                ctx.data.insert("key".to_string(), black_box(serde_json::json!(i)));
                contexts.push(ctx);
            }
            for ctx in contexts {
                pool.release(ctx);
            }
        });
    });

    group.finish();
}

fn bench_context_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_reuse");

    let pool = ContextPool::new();

    // Benchmark different reuse rates
    for reuse_pct in [0, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("reuse_rate", reuse_pct),
            &reuse_pct,
            |b, &pct| {
                // Pre-populate pool based on reuse percentage
                let pool_size = (100 * pct) / 100;
                for _ in 0..pool_size {
                    pool.release(Context::new());
                }

                b.iter(|| {
                    let mut ctx = pool.acquire();
                    ctx.data.insert("test".to_string(), black_box(serde_json::json!(42)));
                    pool.release(ctx);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_context_without_pool,
    bench_context_with_pool,
    bench_pool_acquire_release,
    bench_memory_metrics,
    bench_allocation_patterns,
    bench_context_reuse,
);

criterion_main!(benches);
