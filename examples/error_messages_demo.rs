//! Demonstration of rich error messages in Rust Logic Graph
//!
//! This example shows how to use the error handling system with:
//! - Error codes and categories
//! - Actionable suggestions
//! - Rich context propagation
//! - Documentation links

use rust_logic_graph::error::{RustLogicGraphError, ErrorCategory, ErrorContext};

fn main() {
    println!("🧪 Rust Logic Graph - Rich Error Messages Demo\n");
    println!("{}", "=".repeat(60));

    // Example 1: Node execution error with context
    println!("\n1️⃣ Node Execution Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::node_execution_error(
        "validate_order", 
        "Order validation failed: missing required field 'customer_id'"
    );
    println!("{}", err);

    // Example 2: Database connection error
    println!("\n\n2️⃣ Database Connection Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::database_connection_error(
        "Failed to connect to PostgreSQL at localhost:5432"
    ).with_context(
        ErrorContext::new()
            .with_graph("purchasing_flow")
            .with_step("database_initialization")
            .add_metadata("database", "orders_db")
            .add_metadata("timeout", "5s")
    );
    println!("{}", err);

    // Example 3: Rule evaluation error
    println!("\n\n3️⃣ Rule Evaluation Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::rule_evaluation_error(
        "Undefined variable 'total_amount' in rule 'discount_policy'"
    ).with_context(
        ErrorContext::new()
            .with_node("apply_discount_rules")
            .with_graph("pricing_engine")
    );
    println!("{}", err);

    // Example 4: Configuration error
    println!("\n\n4️⃣ Configuration Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::configuration_error(
        "Missing required field 'database.connection_string' in config file"
    );
    println!("{}", err);

    // Example 5: Timeout error
    println!("\n\n5️⃣ Timeout Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::timeout_error(
        "Node execution exceeded 30s timeout"
    ).with_context(
        ErrorContext::new()
            .with_node("fetch_supplier_data")
            .with_graph("supply_chain_flow")
            .add_metadata("timeout_ms", "30000")
            .add_metadata("elapsed_ms", "30124")
    );
    println!("{}", err);

    // Example 6: AI/LLM error
    println!("\n\n6️⃣ AI/LLM Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::ai_error(
        "OpenAI API rate limit exceeded: 60 requests per minute"
    ).with_context(
        ErrorContext::new()
            .with_node("generate_product_description")
            .add_metadata("model", "gpt-4")
            .add_metadata("retry_after", "45s")
    );
    println!("{}", err);

    // Example 7: Distributed system error
    println!("\n\n7️⃣ Distributed System Error:");
    println!("{}", "─".repeat(60));
    let err = RustLogicGraphError::distributed_error(
        "Service unavailable: inventory-service returned 503",
        "inventory-service"
    ).with_context(
        ErrorContext::new()
            .with_graph("order_orchestration")
            .with_step("check_inventory")
            .add_metadata("service_url", "http://inventory-service:8080")
            .add_metadata("attempt", "3/3")
    );
    println!("{}", err);

    // Example 8: Custom error with full context
    println!("\n\n8️⃣ Custom Error with Full Context:");
    println!("{}", "─".repeat(60));
    let context = ErrorContext::new()
        .with_node("process_payment")
        .with_graph("checkout_flow")
        .with_step("payment_processing")
        .with_service("payment-gateway-service")
        .add_metadata("order_id", "ORD-12345")
        .add_metadata("amount", "$150.00")
        .add_metadata("payment_method", "credit_card")
        .add_metadata("trace_id", "abc123def456");

    let err = RustLogicGraphError::new(
        "E999",
        "Payment gateway returned insufficient funds error",
        ErrorCategory::Permanent
    )
    .with_context(context)
    .with_suggestion("Notify customer about payment failure and suggest alternative payment method.");
    println!("{}", err);

    // Example 9: Error classification
    println!("\n\n9️⃣ Error Classification:");
    println!("{}", "─".repeat(60));
    let errors = vec![
        ("Database connection", RustLogicGraphError::database_connection_error("test")),
        ("Configuration", RustLogicGraphError::configuration_error("test")),
        ("Timeout", RustLogicGraphError::timeout_error("test")),
        ("Graph validation", RustLogicGraphError::graph_validation_error("test")),
    ];

    for (name, err) in errors {
        println!("{}: {} ({})", 
            name,
            if err.is_retryable() { "✅ Retryable" } else { "❌ Permanent" },
            format!("{:?}", err.category)
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("\n✨ Error handling demo complete!");
    println!("\n💡 Key Features:");
    println!("   • Unique error codes (E001-E012)");
    println!("   • Error classification (Retryable/Permanent/Transient/Configuration)");
    println!("   • Actionable suggestions for every error");
    println!("   • Rich context (node, graph, service, metadata)");
    println!("   • Documentation links for troubleshooting");
    println!("   • Source error chaining");
}
