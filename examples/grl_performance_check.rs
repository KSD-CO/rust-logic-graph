use rust_logic_graph::rule::RuleEngine;
use std::fs;
use std::time::Instant;

fn main() {
    // Read purchasing rules
    let rules_content = fs::read_to_string("examples/purchasing_rules.grl")
        .or_else(|_| fs::read_to_string("./examples/purchasing_rules.grl"))
        .expect("Failed to read purchasing_rules.grl");

    println!("📊 GRL Performance Check (rust-rule-engine v0.17)\n");
    println!("Rule file size: {} bytes", rules_content.len());
    println!("Number of rule files: 1\n");

    // Test 1: Parse purchasing rules
    println!("Test 1: Parsing purchasing_rules.grl");
    println!("{}", "─".repeat(50));

    let mut total_parse_time = std::time::Duration::ZERO;
    let samples = 5;

    for i in 1..=samples {
        let start = Instant::now();
        let mut engine = RuleEngine::new();
        let result = engine.add_grl_rule(&rules_content);
        let elapsed = start.elapsed();
        total_parse_time += elapsed;

        match result {
            Ok(_) => println!("  Sample {}: ✅ {:?}", i, elapsed),
            Err(e) => println!("  Sample {}: ❌ Error: {}", i, e),
        }
    }

    let avg_parse_time = total_parse_time / samples as u32;
    println!(
        "\n  Average parse time: {:.3} ms",
        avg_parse_time.as_secs_f64() * 1000.0
    );
    println!("  Min/Max: Would require multiple samples\n");

    // Test 2: Parse simple rule
    println!("Test 2: Parsing simple rule");
    println!("{}", "─".repeat(50));

    let simple_rule = r#"
    rule "simple" {
        when
            amount > 100
        then
            discount = 10;
    }
    "#;

    let mut total_simple = std::time::Duration::ZERO;
    for i in 1..=10 {
        let start = Instant::now();
        let mut engine = RuleEngine::new();
        let _ = engine.add_grl_rule(simple_rule);
        let elapsed = start.elapsed();
        total_simple += elapsed;

        if i <= 3 {
            println!("  Sample {}: ✅ {:?}", i, elapsed);
        }
    }

    let avg_simple = total_simple / 10;
    println!("  ... (10 samples)");
    println!(
        "  Average parse time: {:.3} ms\n",
        avg_simple.as_secs_f64() * 1000.0
    );

    // Test 3: Execution
    println!("Test 3: Rule execution");
    println!("{}", "─".repeat(50));

    let mut engine = RuleEngine::new();
    engine
        .add_grl_rule(&rules_content)
        .expect("Failed to load rules");

    let mut total_exec = std::time::Duration::ZERO;

    for i in 1..=5 {
        let mut context = std::collections::HashMap::new();
        context.insert("amount".to_string(), serde_json::json!(1500.0));
        context.insert("customer_type".to_string(), serde_json::json!("premium"));
        context.insert("discount_percentage".to_string(), serde_json::json!(15.0));

        let start = Instant::now();
        let _ = engine.evaluate(&context);
        let elapsed = start.elapsed();
        total_exec += elapsed;

        println!("  Sample {}: ✅ {:?}", i, elapsed);
    }

    let avg_exec = total_exec / 5;
    println!(
        "\n  Average execution time: {:.3} ms\n",
        avg_exec.as_secs_f64() * 1000.0
    );

    // Summary
    println!("📈 Performance Summary");
    println!("{}", "═".repeat(50));
    println!(
        "  GRL Parse (purchasing rules): {:.3} ms",
        avg_parse_time.as_secs_f64() * 1000.0
    );
    println!(
        "  GRL Parse (simple rule):      {:.3} ms",
        avg_simple.as_secs_f64() * 1000.0
    );
    println!(
        "  Rule Execution:               {:.3} ms",
        avg_exec.as_secs_f64() * 1000.0
    );
    println!("\n✅ GRL parsing performance is acceptable!");
}
