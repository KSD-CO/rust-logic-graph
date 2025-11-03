//! OpenAI Integration Example
//!
//! This example demonstrates how to use OpenAI GPT models in a logic graph.
//!
//! To run this example:
//! 1. Set OPENAI_API_KEY environment variable
//! 2. cargo run --example openai_flow --features openai

use rust_logic_graph::{Context, Executor, Node};
use std::collections::HashMap;

#[cfg(feature = "openai")]
use rust_logic_graph::integrations::OpenAINode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    #[cfg(not(feature = "openai"))]
    {
        println!("❌ This example requires the 'openai' feature");
        println!("Run with: cargo run --example openai_flow --features openai");
        return Ok(());
    }

    #[cfg(feature = "openai")]
    {
        println!("🤖 OpenAI Integration Example\n");

        // Check for API key
        let has_api_key = std::env::var("OPENAI_API_KEY").is_ok();

        if !has_api_key {
            println!("⚠️  OPENAI_API_KEY not set");
            println!("Set OPENAI_API_KEY to test with real API calls");
            println!("\nShowing mock examples:\n");
        }

        // Example 1: Simple completion
        println!("=== Example 1: Simple GPT-4 Completion ===\n");

        let node1 = OpenAINode::gpt4(
            "analyzer",
            "Analyze the following text for sentiment: {{text}}"
        )
        .with_temperature(0.7)
        .with_max_tokens(100);

        let mut ctx1 = Context {
            data: HashMap::new(),
        };
        ctx1.data.insert("text".to_string(), serde_json::json!("I love this product!"));

        println!("Prompt: Analyze the following text for sentiment: I love this product!");
        println!("Model: gpt-4");
        println!("Temperature: 0.7");
        println!("Max tokens: 100\n");

        if has_api_key {
            match node1.run(&mut ctx1).await {
                Ok(result) => {
                    println!("✓ API call successful!");
                    if let Some(content) = result.get("content") {
                        println!("Response: {}\n", content);
                    }
                }
                Err(e) => {
                    println!("✗ API call failed: {}\n", e);
                }
            }
        } else {
            println!("Expected response: 'The sentiment is positive...'\n");
        }

        // Example 2: GPT-3.5 Turbo with system prompt
        println!("=== Example 2: GPT-3.5 Turbo with System Prompt ===\n");

        let node2 = OpenAINode::gpt35_turbo(
            "assistant",
            "{{user_question}}"
        )
        .with_system_prompt("You are a helpful coding assistant specialized in Rust programming.")
        .with_temperature(0.5);

        let mut ctx2 = Context {
            data: HashMap::new(),
        };
        ctx2.data.insert(
            "user_question".to_string(),
            serde_json::json!("How do I create a HashMap in Rust?")
        );

        println!("System: You are a helpful coding assistant specialized in Rust programming.");
        println!("User: How do I create a HashMap in Rust?");
        println!("Model: gpt-3.5-turbo\n");

        if has_api_key {
            match node2.run(&mut ctx2).await {
                Ok(result) => {
                    println!("✓ API call successful!");
                    if let Some(content) = result.get("content") {
                        println!("Response: {}\n", content);
                    }
                }
                Err(e) => {
                    println!("✗ API call failed: {}\n", e);
                }
            }
        } else {
            println!("Expected response: 'To create a HashMap in Rust...'\n");
        }

        // Example 3: Workflow with multiple AI nodes
        println!("=== Example 3: Multi-Stage AI Workflow ===\n");

        println!("Workflow:");
        println!("  1. Extract key points from document");
        println!("  2. Summarize the key points");
        println!("  3. Generate action items\n");

        let mut executor = Executor::new();
        let mut ctx3 = Context {
            data: HashMap::new(),
        };

        ctx3.data.insert(
            "document".to_string(),
            serde_json::json!("Our Q4 revenue increased by 25%...")
        );

        // Stage 1: Extract key points
        println!("Stage 1: Extract key points");
        let node3a = OpenAINode::gpt4(
            "extract_points",
            "Extract the key points from this document: {{document}}"
        );
        println!("  Node: extract_points");
        println!("  Prompt: Extract the key points from this document...\n");

        // Stage 2: Summarize
        println!("Stage 2: Summarize key points");
        let node3b = OpenAINode::gpt35_turbo(
            "summarize",
            "Summarize these points in 2-3 sentences: {{extract_points_content}}"
        );
        println!("  Node: summarize");
        println!("  Prompt: Summarize these points in 2-3 sentences...\n");

        // Stage 3: Generate action items
        println!("Stage 3: Generate action items");
        let node3c = OpenAINode::gpt4(
            "action_items",
            "Based on this summary, generate 3 action items: {{summarize_content}}"
        );
        println!("  Node: action_items");
        println!("  Prompt: Based on this summary, generate 3 action items...\n");

        println!("✓ Workflow defined successfully!\n");

        println!("=== Benefits of OpenAI Integration ===");
        println!("  • Multiple model support (GPT-4, GPT-3.5)");
        println!("  • System prompts for context");
        println!("  • Temperature control");
        println!("  • Token usage tracking");
        println!("  • Context variable interpolation");
        println!("  • Chain multiple AI operations\n");

        println!("=== Available Models ===");
        println!("  • gpt-4: Most capable model");
        println!("  • gpt-4-turbo-preview: Faster GPT-4");
        println!("  • gpt-3.5-turbo: Fast and cost-effective\n");

        println!("🎉 Example completed!");

        if !has_api_key {
            println!("\n💡 Tip: Set OPENAI_API_KEY to run with real API calls");
        }
    }

    Ok(())
}
