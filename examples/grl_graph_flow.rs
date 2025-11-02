use rust_logic_graph::{Graph, GraphIO, Executor, RuleNode, DBNode, AINode, RuleEngine};
use tracing_subscriber;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Rust Logic Graph - GRL Integration Example ===");
    println!("Scenario: Loan Application with Advanced GRL Rules\n");

    // Load graph definition
    let def = GraphIO::load_from_file("examples/grl_graph_flow.json")?;

    // Create custom executor with GRL-powered nodes
    let mut executor = Executor::new();

    // Input validation with GRL
    executor.register_node(Box::new(RuleNode::new(
        "input_validation",
        "loan_amount > 0 && loan_amount <= 1000000"
    )));

    // Fetch customer data
    executor.register_node(Box::new(DBNode::new(
        "fetch_customer",
        "SELECT * FROM customers WHERE id = ?"
    )));

    // Risk assessment with complex GRL rules
    executor.register_node(Box::new(RuleNode::new(
        "risk_assessment",
        "credit_score >= 600 && income >= loan_amount * 3"
    )));

    // Fraud detection AI
    executor.register_node(Box::new(AINode::new(
        "fraud_detection",
        "Analyze transaction patterns for fraud indicators"
    )));

    // Final approval decision
    executor.register_node(Box::new(RuleNode::new(
        "approval_decision",
        "risk_score < 50 && fraud_score < 30"
    )));

    // Notification
    executor.register_node(Box::new(AINode::new(
        "notification",
        "Generate approval/rejection notification email"
    )));

    // Create graph with initial context
    let mut graph = Graph::new(def);

    // Set application data
    graph.context.data.insert("loan_amount".to_string(), json!(50000));
    graph.context.data.insert("credit_score".to_string(), json!(720));
    graph.context.data.insert("income".to_string(), json!(180000));
    graph.context.data.insert("customer_id".to_string(), json!(12345));

    println!("Application Data:");
    println!("  Loan Amount: $50,000");
    println!("  Credit Score: 720");
    println!("  Annual Income: $180,000");
    println!("  Customer ID: 12345\n");

    // Execute the graph
    println!("Processing loan application through GRL-powered workflow...\n");
    executor.execute(&mut graph).await?;

    // Display results
    println!("\n=== Application Results ===\n");

    if let Some(validation) = graph.context.data.get("input_validation_result") {
        println!("✓ Input Validation: {}", validation);
    }

    if let Some(customer) = graph.context.data.get("fetch_customer_result") {
        println!("✓ Customer Data Retrieved");
    }

    if let Some(risk) = graph.context.data.get("risk_assessment_result") {
        println!("✓ Risk Assessment: {}", risk);
    }

    if let Some(fraud) = graph.context.data.get("fraud_detection_result") {
        println!("✓ Fraud Detection Completed");
    }

    if let Some(decision) = graph.context.data.get("approval_decision_result") {
        println!("✓ Approval Decision: {}", decision);
    }

    println!("\n=== GRL-Powered Workflow Complete ===");

    // Demonstrate standalone GRL engine
    println!("\n=== Bonus: Advanced GRL Rules ===\n");

    let mut grl_engine = RuleEngine::new();

    let advanced_rules = r#"
rule "HighValueLoan" salience 100 {
    when
        loan_amount > 100000 && credit_score < 750
    then
        requires_manual_review = true;
        approval_tier = "senior";
}

rule "StandardApproval" salience 50 {
    when
        loan_amount <= 100000 && credit_score >= 650
    then
        auto_approve = true;
        approval_tier = "standard";
}

rule "RiskMitigation" salience 25 {
    when
        debt_to_income_ratio > 0.4
    then
        requires_collateral = true;
        interest_rate_adjustment = 1.5;
}
"#;

    grl_engine.add_grl_rule(advanced_rules)?;

    println!("Advanced GRL rules loaded:");
    println!("  - High Value Loan Review");
    println!("  - Standard Approval Process");
    println!("  - Risk Mitigation Measures");

    println!("\n✅ All systems operational with rust-rule-engine integration!");

    Ok(())
}
