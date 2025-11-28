use rust_logic_graph::rule::RuleEngine;
use std::fs;
use std::time::Instant;

fn main() {
    println!("📊 GRL DETAILED PERFORMANCE TEST");
    println!("Testing case_study/monolithic/purchasing_rules.grl\n");
    
    // Read the complex rule from case_study
    let rules_content = fs::read_to_string("case_study/monolithic/purchasing_rules.grl")
        .or_else(|_| fs::read_to_string("./case_study/monolithic/purchasing_rules.grl"))
        .expect("Failed to read purchasing_rules.grl");
    
    println!("Rule file: case_study/monolithic/purchasing_rules.grl");
    println!("File size: {} bytes", rules_content.len());
    println!("Rule count: {}", rules_content.matches("rule \"").count());
    println!();
    
    // Test parsing 10 times
    println!("Parsing Performance (10 samples):");
    println!("{}", "─".repeat(50));
    
    let mut times = Vec::new();
    let mut errors = 0;
    
    for i in 1..=10 {
        let start = Instant::now();
        let mut engine = RuleEngine::new();
        let result = engine.add_grl_rule(&rules_content);
        let elapsed = start.elapsed();
        
        match result {
            Ok(_) => {
                let ms = elapsed.as_secs_f64() * 1000.0;
                times.push(ms);
                println!("Sample {:2}: ✅ {:.3} ms", i, ms);
            },
            Err(e) => {
                println!("Sample {:2}: ❌ Error: {}", i, e);
                errors += 1;
            }
        }
    }
    
    println!();
    if errors == 0 {
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = times[0];
        let max = times[times.len() - 1];
        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let p50 = times[times.len() / 2];
        let p95 = times[(times.len() * 95) / 100];
        
        println!("📈 Statistics:");
        println!("  Min:    {:.3} ms", min);
        println!("  P50:    {:.3} ms", p50);
        println!("  P95:    {:.3} ms", p95);
        println!("  Max:    {:.3} ms", max);
        println!("  Avg:    {:.3} ms", avg);
        println!();
        println!("✅ All samples successful!");
    } else {
        println!("❌ {} errors out of 10 samples", errors);
    }
}
