use rust_logic_graph::{Graph, Executor, NodeType};
use rust_logic_graph::node::Node;
use serde_json::json;
use sqlx::{MySqlPool, PgPool};
use std::collections::HashMap;

use crate::config::GraphConfig;
use crate::models::PurchaseOrder;
use crate::utils;
use crate::nodes::{DynamicDBNode, DynamicRuleNode, DatabasePool};

/// Graph executor using YAML config
pub struct PurchasingGraphExecutor {
    pools: HashMap<String, DatabasePool>,
    default_pool: DatabasePool,
}

impl PurchasingGraphExecutor {
    pub fn new(default_pool: DatabasePool) -> Self {
        Self { 
            pools: HashMap::new(),
            default_pool,
        }
    }
    
    /// Add a database pool for a specific database name
    pub fn add_pool(&mut self, db_name: String, pool: DatabasePool) {
        self.pools.insert(db_name, pool);
    }
    
    pub fn from_mysql(pool: MySqlPool) -> Self {
        Self::new(DatabasePool::from_mysql(pool))
    }
    
    pub fn from_postgres(pool: PgPool) -> Self {
        Self::new(DatabasePool::from_postgres(pool))
    }
    
    /// Get pool for a specific database, fallback to default
    fn get_pool(&self, db_name: Option<&str>) -> DatabasePool {
        db_name
            .and_then(|name| self.pools.get(name))
            .cloned()
            .unwrap_or_else(|| self.default_pool.clone())
    }
    
    pub async fn execute(&mut self, product_id: &str) -> anyhow::Result<Option<PurchaseOrder>> {
        eprintln!("\n🎯🎯🎯 EXECUTOR.EXECUTE called with product_id: {} 🎯🎯🎯", product_id);
        self.execute_with_config(product_id, "purchasing_flow_graph.yaml").await
    }
    
    pub async fn execute_with_config(&mut self, product_id: &str, config_path: &str) -> anyhow::Result<Option<PurchaseOrder>> {
        tracing::info!("🎯 Graph Executor: Starting purchasing flow for {}", product_id);
        tracing::info!("📋 Loading graph config from: {}", config_path);
        
        // Load graph configuration from YAML
        let graph_config = GraphConfig::from_yaml_file(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load graph config: {}", e))?;
        
        // Convert to GraphDef
        let graph_def = graph_config.to_graph_def()?;
        
        tracing::info!("✅ Graph loaded: {} nodes, {} edges", graph_def.nodes.len(), graph_def.edges.len());
        
        // Build executor and register nodes dynamically from config
        // Note: Cache is disabled because each request has different product_id
        // and results vary per product, making cache ineffective for this use case
        let mut executor = Executor::new();
        
        for (node_id, node_config) in &graph_def.nodes {
            let node: Box<dyn Node> = match node_config.node_type {
                NodeType::DBNode => {
                    let query = node_config.query.clone()
                        .ok_or_else(|| anyhow::anyhow!("DBNode '{}' missing query field", node_id))?;
                    
                    let node_cfg = graph_config.nodes.get(node_id);
                    
                    // Check if node has custom connection string
                    let pool = if let Some(conn_str) = node_cfg.and_then(|nc| nc.connection.as_deref()) {
                        // Create a new pool for this specific connection
                        tracing::info!("📡 Node '{}' using custom connection: {}", node_id, 
                            conn_str.split('@').last().unwrap_or("***"));
                        
                        let pg_pool = utils::database::create_postgres_pool(conn_str).await
                            .map_err(|e| anyhow::anyhow!("Failed to create pool for {}: {}", node_id, e))?;
                        
                        DatabasePool::from_postgres(pg_pool)
                    } else {
                        // Use database name routing (existing behavior)
                        let db_name = node_cfg.and_then(|nc| nc.database.as_deref());
                        let pool = self.get_pool(db_name);
                        
                        if let Some(db) = db_name {
                            tracing::info!("📦 Node '{}' will use database: {}", node_id, db);
                        } else {
                            tracing::info!("📦 Node '{}' will use default database", node_id);
                        }
                        
                        pool
                    };
                    
                    // Get params from node config, default to empty vec
                    let params = node_config.params.clone().unwrap_or_default();
                    
                    if !params.is_empty() {
                        tracing::info!("🔑 Node '{}' will extract params from context: {:?}", node_id, params);
                    }
                    
                    Box::new(DynamicDBNode::new(
                        node_id.clone(),
                        query,
                        pool,
                        params,
                    ))
                }
                NodeType::RuleNode => {
                    let condition = node_config.condition.clone()
                        .unwrap_or_else(|| "true".to_string());
                    
                    let node_cfg = graph_config.nodes.get(node_id);
                    
                    // Get inputs and field_mappings from config
                    let (inputs, field_mappings) = if let Some(cfg) = node_cfg {
                        (cfg.inputs.clone(), cfg.field_mappings.clone())
                    } else {
                        (HashMap::new(), HashMap::new())
                    };
                    
                    if !field_mappings.is_empty() {
                        tracing::info!("📋 Node '{}' using config-driven mappings: {} fields", 
                            node_id, field_mappings.len());
                    }
                    
                    Box::new(DynamicRuleNode::with_mappings(
                        node_id.clone(),
                        condition,
                        inputs,
                        field_mappings,
                    ))
                }
                _ => {
                    tracing::warn!("⚠️  Unsupported node type for {}", node_id);
                    continue;
                }
            };
            
            executor.register_node(node);
        }
        
        // Execute graph
        let mut graph = Graph::new(graph_def);
        graph.context.set("product_id", json!(product_id));
        
        executor.execute(&mut graph).await?;
        
        tracing::info!("✅ Graph execution completed");
        
        // Extract PO from context
        let po_data = graph.context.data.get("po").cloned();
        
        if let Some(po_json) = po_data {
            if po_json.is_null() {
                return Ok(None);
            }
            
            let po = PurchaseOrder {
                product_id: po_json.get("product_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                order_qty: po_json.get("order_qty").and_then(|v| v.as_f64()).unwrap_or(0.0),
                order_unit: po_json.get("order_unit").and_then(|v| v.as_str()).unwrap_or("units").to_string(),
                supplier_id: po_json.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                expected_delivery_date: po_json.get("expected_delivery_date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                total_cost: po_json.get("total_cost").and_then(|v| v.as_f64()).unwrap_or(0.0),
            };
            
            Ok(Some(po))
        } else {
            Ok(None)
        }
    }
}
