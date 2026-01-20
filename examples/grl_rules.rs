use rust_logic_graph::RuleEngine;
use serde_json::json;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    println!("=== Rust Logic Graph - GRL Rules Example ===\n");

    // Example 1: Simple GRL Rule
    println!("Example 1: Simple Age Verification");
    println!("-----------------------------------");

    let mut context1 = HashMap::new();
    context1.insert("age".to_string(), json!(25));
    context1.insert("verified".to_string(), json!(false));

    let mut engine1 = RuleEngine::new();
    let grl1 = r#"
rule "AgeVerification" {
    when
        age >= 18
    then
        verified = true;
}
"#;

    engine1.add_grl_rule(grl1)?;

    match engine1.evaluate(&context1) {
        Ok(result) => println!("✓ Rule executed: {:?}", result),
        Err(e) => println!("✗ Rule failed: {}", e),
    }

    // Example 2: Complex Business Rules
    println!("\nExample 2: E-commerce Discount Rules");
    println!("-------------------------------------");

    let mut context2 = HashMap::new();
    context2.insert("cart_total".to_string(), json!(150.0));
    context2.insert("is_member".to_string(), json!(true));
    context2.insert("discount".to_string(), json!(0.0));

    let grl2 = r#"
rule "MemberDiscount" salience 10 {
    when
        is_member == true && cart_total >= 100.0
    then
        discount = 0.15;
}

rule "RegularDiscount" salience 5 {
    when
        cart_total >= 100.0 && discount == 0.0
    then
        discount = 0.10;
}
"#;

    let mut engine2 = RuleEngine::new();
    engine2.add_grl_rule(grl2)?;

    match engine2.evaluate(&context2) {
        Ok(result) => println!("✓ Discount rules executed: {:?}", result),
        Err(e) => println!("✗ Rules failed: {}", e),
    }

    // Example 3: Using from_simple helper
    println!("\nExample 3: Simple Rule Helper");
    println!("------------------------------");

    let mut context3 = HashMap::new();
    context3.insert("temperature".to_string(), json!(35.0));
    context3.insert("alert".to_string(), json!(false));

    // Use a small GRL snippet and the RuleEngine for this helper example
    let rule3_grl = r#"
rule "temperature_alert" {
    when
        temperature > 30.0
    then
        alert = true;
}
"#;

    println!("Generated GRL:");
    println!("{}", rule3_grl);

    let mut engine3 = RuleEngine::new();
    engine3.add_grl_rule(rule3_grl)?;

    match engine3.evaluate(&context3) {
        Ok(result) => println!("✓ Temperature alert: {:?}", result),
        Err(e) => println!("✗ Rule failed: {}", e),
    }

    // Example 4: Multiple Conditions
    println!("\nExample 4: Loan Approval Rules");
    println!("-------------------------------");

    let mut context4 = HashMap::new();
    context4.insert("credit_score".to_string(), json!(720));
    context4.insert("income".to_string(), json!(75000));
    context4.insert("debt_ratio".to_string(), json!(0.3));
    context4.insert("approved".to_string(), json!(false));

    let grl4 = r#"
rule "LoanApproval" {
    when
        credit_score >= 700 &&
        income >= 50000 &&
        debt_ratio < 0.4
    then
        approved = true;
}
"#;

    let mut engine4 = RuleEngine::new();
    engine4.add_grl_rule(grl4)?;

    match engine4.evaluate(&context4) {
        Ok(result) => println!("✓ Loan approval processed: {:?}", result),
        Err(e) => println!("✗ Approval failed: {}", e),
    }

    println!("\n=== All GRL Rules Executed Successfully! ===");
    Ok(())
}
