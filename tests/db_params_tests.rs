/// Integration tests for DBNode params feature
use rust_logic_graph::{Executor, Graph, GraphDef, NodeConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_db_node_with_single_param() {
    let mut nodes = HashMap::new();

    // DBNode with one parameter from context
    nodes.insert(
        "db_query".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM table WHERE id = $1",
            vec!["item_id".to_string()],
        ),
    );

    let def = GraphDef {
        nodes,
        edges: vec![],
    };
    let mut graph = Graph::new(def);

    // Set the parameter in context
    graph.context.set("item_id", serde_json::json!("ID-123"));

    let mut executor = Executor::from_graph_def(&graph.def).unwrap();
    let result = executor.execute(&mut graph).await;

    assert!(result.is_ok());
    assert!(graph.context.contains_key("db_query_result"));
}

#[tokio::test]
async fn test_db_node_with_multiple_params() {
    let mut nodes = HashMap::new();

    // DBNode with multiple parameters
    nodes.insert(
        "db_query".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM orders WHERE user_id = $1 AND status = $2",
            vec!["user_id".to_string(), "status".to_string()],
        ),
    );

    let def = GraphDef {
        nodes,
        edges: vec![],
    };
    let mut graph = Graph::new(def);

    // Set parameters in context
    graph.context.set("user_id", serde_json::json!("USER-456"));
    graph.context.set("status", serde_json::json!("active"));

    let mut executor = Executor::from_graph_def(&graph.def).unwrap();
    let result = executor.execute(&mut graph).await;

    assert!(result.is_ok());
    assert!(graph.context.contains_key("db_query_result"));
}

#[tokio::test]
async fn test_db_node_without_params() {
    let mut nodes = HashMap::new();

    // DBNode without params (backward compatibility)
    nodes.insert(
        "db_query".to_string(),
        NodeConfig::db_node("SELECT * FROM table"),
    );

    let def = GraphDef {
        nodes,
        edges: vec![],
    };
    let mut graph = Graph::new(def);

    let mut executor = Executor::from_graph_def(&graph.def).unwrap();
    let result = executor.execute(&mut graph).await;

    assert!(result.is_ok());
    assert!(graph.context.contains_key("db_query_result"));
}

#[tokio::test]
async fn test_db_node_with_missing_context_param() {
    let mut nodes = HashMap::new();

    // DBNode expects a parameter that won't be in context
    nodes.insert(
        "db_query".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM table WHERE id = $1",
            vec!["missing_param".to_string()],
        ),
    );

    let def = GraphDef {
        nodes,
        edges: vec![],
    };
    let mut graph = Graph::new(def);

    // Don't set the parameter - should still execute but with empty params
    let mut executor = Executor::from_graph_def(&graph.def).unwrap();
    let result = executor.execute(&mut graph).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_db_node_with_different_value_types() {
    let mut nodes = HashMap::new();

    nodes.insert(
        "db_query".to_string(),
        NodeConfig::db_node_with_params(
            "SELECT * FROM products WHERE id = $1 AND price = $2 AND active = $3",
            vec![
                "product_id".to_string(),
                "price".to_string(),
                "is_active".to_string(),
            ],
        ),
    );

    let def = GraphDef {
        nodes,
        edges: vec![],
    };
    let mut graph = Graph::new(def);

    // Test with different JSON value types
    graph
        .context
        .set("product_id", serde_json::json!("PROD-001"));
    graph.context.set("price", serde_json::json!(99.99));
    graph.context.set("is_active", serde_json::json!(true));

    let mut executor = Executor::from_graph_def(&graph.def).unwrap();
    let result = executor.execute(&mut graph).await;

    assert!(result.is_ok());
}

#[test]
fn test_json_serialization_with_params() {
    let config = NodeConfig::db_node_with_params(
        "SELECT * FROM users WHERE id = $1",
        vec!["user_id".to_string()],
    );

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("params"));
    assert!(json.contains("user_id"));

    let deserialized: NodeConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.params.is_some());
    assert_eq!(deserialized.params.unwrap(), vec!["user_id"]);
}

#[test]
fn test_json_deserialization_without_params() {
    let json = r#"{
        "node_type": "DBNode",
        "query": "SELECT * FROM users"
    }"#;

    let config: NodeConfig = serde_json::from_str(json).unwrap();
    assert!(config.params.is_none());
}
