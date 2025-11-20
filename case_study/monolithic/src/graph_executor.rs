use rust_logic_graph::{Graph, GraphDef, Executor, NodeType, Context};
use rust_logic_graph::node::Node;
use rust_logic_graph::rule::RuleResult;
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;

use crate::graph_config::GraphConfig;
use crate::services::{OmsService, InventoryService, SupplierService, UomService, RuleEngineService};
use crate::models::{PurchaseOrder, PurchasingContext};

/// Custom node that queries OMS service
pub struct OmsNode {
    service: Arc<OmsService>,
    product_id: String,
}

impl OmsNode {
    pub fn new(service: Arc<OmsService>, product_id: String) -> Self {
        Self { service, product_id }
    }
}

#[async_trait]
impl Node for OmsNode {
    fn id(&self) -> &str { "oms_history" }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📊 OMS Node: Fetching history for {}", self.product_id);
        
        let data = self.service.get_oms_history(&self.product_id).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(e.to_string()))?;
        
        let result = json!({
            "product_id": data.product_id,
            "avg_daily_demand": data.avg_daily_demand,
            "trend": data.trend,
        });
        
        ctx.data.insert("oms_data".to_string(), result.clone());
        Ok(result)
    }
}

/// Custom node that queries Inventory service
pub struct InventoryNode {
    service: Arc<InventoryService>,
    product_id: String,
}

impl InventoryNode {
    pub fn new(service: Arc<InventoryService>, product_id: String) -> Self {
        Self { service, product_id }
    }
}

#[async_trait]
impl Node for InventoryNode {
    fn id(&self) -> &str { "inventory_levels" }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📦 Inventory Node: Fetching levels for {}", self.product_id);
        
        let data = self.service.get_inventory(&self.product_id).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(e.to_string()))?;
        
        let result = json!({
            "product_id": data.product_id,
            "warehouse_id": data.warehouse_id,
            "available_qty": data.available_qty,
            "reserved_qty": data.reserved_qty,
        });
        
        ctx.data.insert("inventory_data".to_string(), result.clone());
        Ok(result)
    }
}

/// Custom node that queries Supplier service
pub struct SupplierNode {
    service: Arc<SupplierService>,
    product_id: String,
}

impl SupplierNode {
    pub fn new(service: Arc<SupplierService>, product_id: String) -> Self {
        Self { service, product_id }
    }
}

#[async_trait]
impl Node for SupplierNode {
    fn id(&self) -> &str { "supplier_info" }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🏭 Supplier Node: Fetching info for {}", self.product_id);
        
        let data = self.service.get_supplier_info(&self.product_id).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(e.to_string()))?;
        
        let result = json!({
            "supplier_id": data.supplier_id,
            "product_id": data.product_id,
            "moq": data.moq,
            "lead_time": data.lead_time,
            "unit_price": data.unit_price,
        });
        
        ctx.data.insert("supplier_data".to_string(), result.clone());
        Ok(result)
    }
}

/// Custom node that queries UOM service
pub struct UomNode {
    service: Arc<UomService>,
    product_id: String,
}

impl UomNode {
    pub fn new(service: Arc<UomService>, product_id: String) -> Self {
        Self { service, product_id }
    }
}

#[async_trait]
impl Node for UomNode {
    fn id(&self) -> &str { "uom_conversion" }
    fn node_type(&self) -> NodeType { NodeType::DBNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📏 UOM Node: Fetching conversion for {}", self.product_id);
        
        let data = self.service.get_uom_conversion(&self.product_id).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(e.to_string()))?;
        
        let result = json!({
            "product_id": data.product_id,
            "from_uom": data.from_uom,
            "to_uom": data.to_uom,
            "conversion_factor": data.conversion_factor,
        });
        
        ctx.data.insert("uom_data".to_string(), result.clone());
        Ok(result)
    }
}

/// Custom node that runs GRL rule engine
pub struct RuleEngineNode {
    service: Arc<RuleEngineService>,
}

impl RuleEngineNode {
    pub fn new(service: Arc<RuleEngineService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Node for RuleEngineNode {
    fn id(&self) -> &str { "rule_engine" }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🧠 Rule Engine Node: Evaluating business rules");
        
        // Extract data from context
        let oms_data = ctx.data.get("oms_data").cloned().unwrap_or(json!({}));
        let inventory_data = ctx.data.get("inventory_data").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_data").cloned().unwrap_or(json!({}));
        let uom_data = ctx.data.get("uom_data").cloned().unwrap_or(json!({}));
        
        // Build PurchasingContext
        let purchasing_ctx = PurchasingContext {
            oms_data: crate::models::OmsHistoryData {
                product_id: oms_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                avg_daily_demand: oms_data.get("avg_daily_demand").and_then(|v| v.as_f64()).unwrap_or(0.0),
                trend: oms_data.get("trend").and_then(|v| v.as_str()).unwrap_or("stable").to_string(),
            },
            inventory_data: crate::models::InventoryData {
                product_id: inventory_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                warehouse_id: inventory_data.get("warehouse_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                available_qty: inventory_data.get("available_qty").and_then(|v| v.as_f64()).unwrap_or(0.0),
                reserved_qty: inventory_data.get("reserved_qty").and_then(|v| v.as_f64()).unwrap_or(0.0),
            },
            supplier_data: crate::models::SupplierData {
                supplier_id: supplier_data.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                product_id: supplier_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                moq: supplier_data.get("moq").and_then(|v| v.as_f64()).unwrap_or(0.0),
                lead_time: supplier_data.get("lead_time").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                unit_price: supplier_data.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
            },
            uom_data: crate::models::UomConversionData {
                product_id: uom_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                from_uom: uom_data.get("from_uom").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                to_uom: uom_data.get("to_uom").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                conversion_factor: uom_data.get("conversion_factor").and_then(|v| v.as_f64()).unwrap_or(1.0),
            },
        };
        
        // Evaluate rules
        let result = self.service.evaluate(&purchasing_ctx)
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(e.to_string()))?;
        
        let result_json = json!({
            "should_order": result.should_order,
            "recommended_qty": result.recommended_qty,
            "reason": result.reason,
        });
        
        ctx.data.insert("rule_result".to_string(), result_json.clone());
        Ok(result_json)
    }
}

/// Custom node that creates PO
pub struct CreatePoNode {
    product_id: String,
}

impl CreatePoNode {
    pub fn new(product_id: String) -> Self {
        Self { product_id }
    }
}

#[async_trait]
impl Node for CreatePoNode {
    fn id(&self) -> &str { "create_po" }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📝 Create PO Node: Creating purchase order");
        
        let rule_result = ctx.data.get("rule_result").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_data").cloned().unwrap_or(json!({}));
        
        let should_order = rule_result.get("should_order").and_then(|v| v.as_bool()).unwrap_or(false);
        
        if !should_order {
            tracing::info!("ℹ️ No PO needed - sufficient inventory");
            ctx.data.insert("po".to_string(), Value::Null);
            return Ok(Value::Null);
        }
        
        let recommended_qty = rule_result.get("recommended_qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let supplier_id = supplier_data.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let unit_price = supplier_data.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let lead_time = supplier_data.get("lead_time").and_then(|v| v.as_i64()).unwrap_or(0);
        
        let total_cost = recommended_qty * unit_price;
        let expected_delivery = chrono::Utc::now() + chrono::Duration::days(lead_time);
        
        let po = PurchaseOrder {
            product_id: self.product_id.clone(),
            order_qty: recommended_qty,
            order_unit: "units".to_string(),
            supplier_id: supplier_id.to_string(),
            expected_delivery_date: expected_delivery.format("%Y-%m-%d").to_string(),
            total_cost,
        };
        
        let po_json = json!({
            "product_id": po.product_id,
            "order_qty": po.order_qty,
            "order_unit": po.order_unit,
            "supplier_id": po.supplier_id,
            "expected_delivery_date": po.expected_delivery_date,
            "total_cost": po.total_cost,
        });
        
        ctx.data.insert("po".to_string(), po_json.clone());
        Ok(po_json)
    }
}

/// Graph executor that orchestrates the purchasing flow
pub struct PurchasingGraphExecutor {
    oms_service: Arc<OmsService>,
    inventory_service: Arc<InventoryService>,
    supplier_service: Arc<SupplierService>,
    uom_service: Arc<UomService>,
    rule_engine: Arc<RuleEngineService>,
}

impl PurchasingGraphExecutor {
    pub fn new(
        oms_service: OmsService,
        inventory_service: InventoryService,
        supplier_service: SupplierService,
        uom_service: UomService,
        rule_engine: RuleEngineService,
    ) -> Self {
        Self {
            oms_service: Arc::new(oms_service),
            inventory_service: Arc::new(inventory_service),
            supplier_service: Arc::new(supplier_service),
            uom_service: Arc::new(uom_service),
            rule_engine: Arc::new(rule_engine),
        }
    }
    
    pub async fn execute(&self, product_id: &str) -> anyhow::Result<Option<PurchaseOrder>> {
        self.execute_with_config(product_id, "purchasing_flow_graph.yaml").await
    }
    
    /// Execute with custom graph configuration file
    pub async fn execute_with_config(&self, product_id: &str, config_path: &str) -> anyhow::Result<Option<PurchaseOrder>> {
        tracing::info!("🎯 Graph Executor: Starting purchasing flow for {}", product_id);
        tracing::info!("📋 Loading graph config from: {}", config_path);
        
        // Load graph configuration from YAML
        let graph_config = GraphConfig::from_yaml_file(config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load graph config: {}", e))?;
        
        // Convert to GraphDef
        let graph_def = graph_config.to_graph_def()?;
        
        tracing::info!("✅ Graph loaded: {} nodes, {} edges", graph_def.nodes.len(), graph_def.edges.len());
        
        // Build executor and register nodes dynamically
        let mut executor = Executor::new();
        
        // Register nodes based on configuration
        for (node_id, _node_type) in &graph_def.nodes {
            let node: Box<dyn Node> = match node_id.as_str() {
                "oms_history" => Box::new(OmsNode::new(
                    self.oms_service.clone(),
                    product_id.to_string(),
                )),
                "inventory_levels" => Box::new(InventoryNode::new(
                    self.inventory_service.clone(),
                    product_id.to_string(),
                )),
                "supplier_info" => Box::new(SupplierNode::new(
                    self.supplier_service.clone(),
                    product_id.to_string(),
                )),
                "uom_conversion" => Box::new(UomNode::new(
                    self.uom_service.clone(),
                    product_id.to_string(),
                )),
                "rule_engine" => Box::new(RuleEngineNode::new(
                    self.rule_engine.clone(),
                )),
                "create_po" => Box::new(CreatePoNode::new(
                    product_id.to_string(),
                )),
                _ => {
                    tracing::warn!("⚠️ Unknown node in config: {}", node_id);
                    continue;
                }
            };
            
            executor.register_node(node);
        }
        
        // Execute graph
        let mut graph = Graph::new(graph_def);
        graph.context.set("product_id", json!(product_id))?;
        
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
