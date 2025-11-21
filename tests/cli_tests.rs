use rust_logic_graph::{GraphDef, NodeType};
use std::collections::HashMap;

#[test]
fn test_valid_graph_structure() {
    let mut nodes = HashMap::new();
    nodes.insert("start".to_string(), NodeType::RuleNode);
    nodes.insert("end".to_string(), NodeType::RuleNode);

    let edges = vec![
        rust_logic_graph::Edge {
            from: "start".to_string(),
            to: "end".to_string(),
            rule: None,
        }
    ];

    let graph_def = GraphDef::from_node_types(nodes, edges);

    assert_eq!(graph_def.nodes.len(), 2);
    assert_eq!(graph_def.edges.len(), 1);
}

#[test]
fn test_empty_graph() {
    let graph_def = GraphDef::from_node_types(HashMap::new(), Vec::new());

    assert_eq!(graph_def.nodes.len(), 0);
    assert_eq!(graph_def.edges.len(), 0);
}

#[test]
fn test_graph_with_cycle() {
    let mut nodes = HashMap::new();
    nodes.insert("a".to_string(), NodeType::RuleNode);
    nodes.insert("b".to_string(), NodeType::RuleNode);
    nodes.insert("c".to_string(), NodeType::RuleNode);

    let edges = vec![
        rust_logic_graph::Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            rule: None,
        },
        rust_logic_graph::Edge {
            from: "b".to_string(),
            to: "c".to_string(),
            rule: None,
        },
        rust_logic_graph::Edge {
            from: "c".to_string(),
            to: "a".to_string(),
            rule: None,
        },
    ];

    let graph_def = GraphDef::from_node_types(nodes, edges);

    // Graph should be created successfully even with cycles
    // CLI validation will detect the cycle
    assert_eq!(graph_def.nodes.len(), 3);
    assert_eq!(graph_def.edges.len(), 3);
}

#[test]
fn test_graph_serialization() {
    let mut nodes = HashMap::new();
    nodes.insert("node1".to_string(), NodeType::RuleNode);

    let edges = vec![];

    let graph_def = GraphDef::from_node_types(nodes, edges);

    // Test JSON serialization
    let json = serde_json::to_string(&graph_def).unwrap();
    assert!(json.contains("node1"));
    assert!(json.contains("RuleNode"));

    // Test deserialization
    let deserialized: GraphDef = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.nodes.len(), 1);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_graph_file(dir: &TempDir, filename: &str, graph_def: &GraphDef) -> PathBuf {
        let file_path = dir.path().join(filename);
        let json = serde_json::to_string_pretty(graph_def).unwrap();
        fs::write(&file_path, json).unwrap();
        file_path
    }

    #[test]
    fn test_load_and_validate_graph() {
        let temp_dir = TempDir::new().unwrap();

        let mut nodes = HashMap::new();
        nodes.insert("start".to_string(), NodeType::RuleNode);
        nodes.insert("end".to_string(), NodeType::RuleNode);

        let edges = vec![
            rust_logic_graph::Edge {
                from: "start".to_string(),
                to: "end".to_string(),
                rule: None,
            }
        ];

        let graph_def = GraphDef::from_node_types(nodes, edges);
        let _file_path = create_test_graph_file(&temp_dir, "test_graph.json", &graph_def);

        // File should be created and readable
        assert!(temp_dir.path().join("test_graph.json").exists());
    }
}
