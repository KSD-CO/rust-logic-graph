use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_logic_graph::rule::RuleEngine;
use std::fs;

fn benchmark_grl_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("grl_parsing");
    group.sample_size(100); // More samples for accuracy
    
    // Load the purchasing rules
    let rules_content = std::fs::read_to_string("./examples/purchasing_rules.grl")
        .or_else(|_| std::fs::read_to_string("examples/purchasing_rules.grl"))
        .expect("Failed to read purchasing_rules.grl");
    
    group.bench_function("parse_purchasing_rules", |b| {
        b.iter(|| {
            let mut engine = RuleEngine::new();
            let result = engine.add_grl_rule(black_box(&rules_content));
            assert!(result.is_ok(), "Failed to parse rules");
        });
    });
    
    // Test parsing simple rule
    let simple_rule = r#"
    rule "test_rule" {
        when
            amount > 100
        then
            result = true;
    }
    "#;
    
    group.bench_function("parse_simple_rule", |b| {
        b.iter(|| {
            let mut engine = RuleEngine::new();
            let result = engine.add_grl_rule(black_box(simple_rule));
            assert!(result.is_ok(), "Failed to parse simple rule");
        });
    });
    
    // Test parsing complex rule
    let complex_rule = r#"
    rule "complex_rule" {
        salience 100
        when
            order_amount > 100 &&
            customer_type != "" &&
            discount_percentage >= 0 &&
            discount_percentage <= 50
        then
            if (order_amount > 1000 && discount_percentage > 20) {
                applicable_discount = discount_percentage * 1.2;
            } else {
                applicable_discount = discount_percentage;
            }
    }
    "#;
    
    group.bench_function("parse_complex_rule", |b| {
        b.iter(|| {
            let mut engine = RuleEngine::new();
            let result = engine.add_grl_rule(black_box(complex_rule));
            assert!(result.is_ok(), "Failed to parse complex rule");
        });
    });
    
    group.finish();
}

fn benchmark_grl_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("grl_execution");
    group.sample_size(50);
    
    // Setup rule engine with parsed rules
    let rules_content = std::fs::read_to_string("./examples/purchasing_rules.grl")
        .or_else(|_| std::fs::read_to_string("examples/purchasing_rules.grl"))
        .expect("Failed to read purchasing_rules.grl");
    
    group.bench_function("execute_purchasing_rules", |b| {
        b.iter_batched(
            || {
                let mut engine = RuleEngine::new();
                engine.add_grl_rule(&rules_content)
                    .expect("Failed to parse rules");
                engine
            },
            |mut engine| {
                let mut context = std::collections::HashMap::new();
                context.insert("amount".to_string(), serde_json::json!(1500.0));
                context.insert("customer_type".to_string(), serde_json::json!("premium"));
                context.insert("discount_percentage".to_string(), serde_json::json!(15.0));
                
                engine.evaluate(black_box(&context))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_grl_parsing,
    benchmark_grl_execution
);
criterion_main!(benches);
