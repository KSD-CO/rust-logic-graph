use rust_logic_graph::{Graph, Executor, NodeType, Context};
use rust_logic_graph::node::Node;
use rust_logic_graph::rule::RuleResult;
use serde_json::{json, Value};
use async_trait::async_trait;
use tonic::Request;
use purchasing_models::*;

use crate::graph_config::GraphConfig;
use crate::oms::{self, oms_service_client::OmsServiceClient};
use crate::inventory::{self, inventory_service_client::InventoryServiceClient};
use crate::supplier::{self, supplier_service_client::SupplierServiceClient};
use crate::uom::{self, uom_service_client::UomServiceClient};
use crate::rule_engine::{self, rule_engine_service_client::RuleEngineServiceClient};
use crate::po::{self, po_service_client::PoServiceClient};

/// gRPC Node that queries OMS service
pub struct OmsGrpcNode {
    grpc_url: String,
    product_id: String,
}

impl OmsGrpcNode {
    pub fn new(grpc_url: String, product_id: String) -> Self {
        Self { grpc_url, product_id }
    }
}

#[async_trait]
impl Node for OmsGrpcNode {
    fn id(&self) -> &str { "oms_grpc" }
    fn node_type(&self) -> NodeType { NodeType::GrpcNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📊 OMS gRPC Node: Fetching history for {}", self.product_id);
        
        let mut client = OmsServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("OMS gRPC connect failed: {}", e)))?;
        
        let request = Request::new(oms::HistoryRequest {
            product_id: self.product_id.clone(),
        });
        
        let response = client.get_history(request).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("OMS gRPC call failed: {}", e)))?;
        
        let grpc_data = response.into_inner();
        
        let result = json!({
            "product_id": grpc_data.product_id,
            "avg_daily_demand": grpc_data.avg_daily_demand,
            "trend": grpc_data.trend,
        });
        
        ctx.data.insert("oms_data".to_string(), result.clone());
        Ok(result)
    }
}

/// gRPC Node that queries Inventory service
pub struct InventoryGrpcNode {
    grpc_url: String,
    product_id: String,
}

impl InventoryGrpcNode {
    pub fn new(grpc_url: String, product_id: String) -> Self {
        Self { grpc_url, product_id }
    }
}

#[async_trait]
impl Node for InventoryGrpcNode {
    fn id(&self) -> &str { "inventory_grpc" }
    fn node_type(&self) -> NodeType { NodeType::GrpcNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📦 Inventory gRPC Node: Fetching levels for {}", self.product_id);
        
        let mut client = InventoryServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Inventory gRPC connect failed: {}", e)))?;
        
        let request = Request::new(inventory::LevelsRequest {
            product_id: self.product_id.clone(),
        });
        
        let response = client.get_levels(request).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Inventory gRPC call failed: {}", e)))?;
        
        let grpc_data = response.into_inner();
        
        let result = json!({
            "product_id": grpc_data.product_id,
            "warehouse_id": "WH-001",
            "current_qty": grpc_data.available_qty + grpc_data.reserved_qty,
            "reserved_qty": grpc_data.reserved_qty,
            "available_qty": grpc_data.available_qty,
        });
        
        ctx.data.insert("inventory_data".to_string(), result.clone());
        Ok(result)
    }
}

/// gRPC Node that queries Supplier service
pub struct SupplierGrpcNode {
    grpc_url: String,
    product_id: String,
}

impl SupplierGrpcNode {
    pub fn new(grpc_url: String, product_id: String) -> Self {
        Self { grpc_url, product_id }
    }
}

#[async_trait]
impl Node for SupplierGrpcNode {
    fn id(&self) -> &str { "supplier_grpc" }
    fn node_type(&self) -> NodeType { NodeType::GrpcNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🏭 Supplier gRPC Node: Fetching info for {}", self.product_id);
        
        let mut client = SupplierServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Supplier gRPC connect failed: {}", e)))?;
        
        let request = Request::new(supplier::InfoRequest {
            product_id: self.product_id.clone(),
        });
        
        let response = client.get_info(request).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Supplier gRPC call failed: {}", e)))?;
        
        let grpc_data = response.into_inner();
        
        let result = json!({
            "supplier_id": grpc_data.supplier_name,
            "product_id": grpc_data.product_id,
            "moq": grpc_data.moq,
            "lead_time_days": grpc_data.lead_time_days,
            "unit_price": grpc_data.unit_price,
            "is_active": true,
        });
        
        ctx.data.insert("supplier_data".to_string(), result.clone());
        Ok(result)
    }
}

/// gRPC Node that queries UOM service
pub struct UomGrpcNode {
    grpc_url: String,
    product_id: String,
}

impl UomGrpcNode {
    pub fn new(grpc_url: String, product_id: String) -> Self {
        Self { grpc_url, product_id }
    }
}

#[async_trait]
impl Node for UomGrpcNode {
    fn id(&self) -> &str { "uom_grpc" }
    fn node_type(&self) -> NodeType { NodeType::GrpcNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📏 UOM gRPC Node: Fetching conversion for {}", self.product_id);
        
        let mut client = UomServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("UOM gRPC connect failed: {}", e)))?;
        
        let request = Request::new(uom::ConversionRequest {
            product_id: self.product_id.clone(),
        });
        
        let response = client.get_conversion(request).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("UOM gRPC call failed: {}", e)))?;
        
        let grpc_data = response.into_inner();
        
        let result = json!({
            "product_id": grpc_data.product_id,
            "from_uom": grpc_data.base_unit,
            "to_uom": "case",
            "conversion_factor": grpc_data.case_qty,
        });
        
        ctx.data.insert("uom_data".to_string(), result.clone());
        Ok(result)
    }
}

/// gRPC Node that calls Rule Engine service
pub struct RuleEngineGrpcNode {
    grpc_url: String,
}

impl RuleEngineGrpcNode {
    pub fn new(grpc_url: String) -> Self {
        Self { grpc_url }
    }
}

#[async_trait]
impl Node for RuleEngineGrpcNode {
    fn id(&self) -> &str { "rule_engine_grpc" }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("🧠 Rule Engine gRPC Node: Evaluating business rules");
        
        let oms_data = ctx.data.get("oms_data").cloned().unwrap_or(json!({}));
        let inventory_data = ctx.data.get("inventory_data").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_data").cloned().unwrap_or(json!({}));
        
        let mut client = RuleEngineServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Rule Engine gRPC connect failed: {}", e)))?;
        
        let rule_request = rule_engine::EvaluateRequest {
            oms_data: Some(rule_engine::OmsData {
                product_id: oms_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                avg_daily_demand: oms_data.get("avg_daily_demand").and_then(|v| v.as_f64()).unwrap_or(0.0),
                trend: oms_data.get("trend").and_then(|v| v.as_str()).unwrap_or("stable").to_string(),
            }),
            inventory_data: Some(rule_engine::InventoryData {
                product_id: inventory_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                warehouse_id: inventory_data.get("warehouse_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                current_qty: inventory_data.get("current_qty").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                reserved_qty: inventory_data.get("reserved_qty").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                available_qty: inventory_data.get("available_qty").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
            }),
            supplier_data: Some(rule_engine::SupplierData {
                supplier_id: supplier_data.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                product_id: supplier_data.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                moq: supplier_data.get("moq").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32,
                lead_time_days: supplier_data.get("lead_time_days").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                unit_price: supplier_data.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                is_active: supplier_data.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true),
            }),
        };
        
        let grpc_response = client.evaluate(Request::new(rule_request)).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("Rule Engine gRPC call failed: {}", e)))?;
        
        let rule_result_grpc = grpc_response.into_inner();
        
        let result = json!({
            "need_reorder": rule_result_grpc.need_reorder,
            "shortage": rule_result_grpc.shortage,
            "order_qty": rule_result_grpc.order_qty,
            "total_amount": rule_result_grpc.total_amount,
            "requires_approval": rule_result_grpc.requires_approval,
            "approval_status": rule_result_grpc.approval_status,
            "should_create_po": rule_result_grpc.should_create_po,
            "should_send_po": rule_result_grpc.should_send_po,
            "po_status": rule_result_grpc.po_status,
            "send_method": rule_result_grpc.send_method,
        });
        
        ctx.data.insert("rule_result".to_string(), result.clone());
        Ok(result)
    }
}

/// gRPC Node that creates and sends PO
pub struct PoGrpcNode {
    grpc_url: String,
    product_id: String,
}

impl PoGrpcNode {
    pub fn new(grpc_url: String, product_id: String) -> Self {
        Self { grpc_url, product_id }
    }
}

#[async_trait]
impl Node for PoGrpcNode {
    fn id(&self) -> &str { "po_grpc" }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        tracing::info!("📝 PO gRPC Node: Creating/sending purchase order");
        
        let rule_result = ctx.data.get("rule_result").cloned().unwrap_or(json!({}));
        let supplier_data = ctx.data.get("supplier_data").cloned().unwrap_or(json!({}));
        
        let should_create_po = rule_result.get("should_create_po").and_then(|v| v.as_bool()).unwrap_or(false);
        
        if !should_create_po {
            tracing::info!("ℹ️ No PO creation needed (rules decided: should_create_po=false)");
            ctx.data.insert("po".to_string(), Value::Null);
            return Ok(Value::Null);
        }
        
        let order_qty = rule_result.get("order_qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let total_amount = rule_result.get("total_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let unit_price = supplier_data.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let supplier_id = supplier_data.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let should_send_po = rule_result.get("should_send_po").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let mut client = PoServiceClient::connect(self.grpc_url.clone()).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("PO gRPC connect failed: {}", e)))?;
        
        // Create PO
        let create_po_request = po::CreateRequest {
            product_id: self.product_id.clone(),
            supplier_id: supplier_id.to_string(),
            qty: order_qty as i64,
            unit_price,
            total_amount,
        };
        
        let po_grpc_response = client.create(Request::new(create_po_request)).await
            .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("PO create gRPC call failed: {}", e)))?;
        
        let created_po_grpc = po_grpc_response.into_inner().po
            .ok_or_else(|| rust_logic_graph::rule::RuleError::Eval("PO not found in create response".to_string()))?;
        
        tracing::info!("✅ PO created: {}", created_po_grpc.po_id);
        
        // Send PO if rules decided
        let final_po = if should_send_po {
            tracing::info!("📤 Sending PO (rules decided: should_send_po=true)");
            
            let send_po_request = po::SendRequest {
                po_id: created_po_grpc.po_id.clone(),
            };
            
            let sent_po_grpc_response = client.send(Request::new(send_po_request)).await
                .map_err(|e| rust_logic_graph::rule::RuleError::Eval(format!("PO send gRPC call failed: {}", e)))?;
            
            let sent_po_grpc = sent_po_grpc_response.into_inner().po
                .ok_or_else(|| rust_logic_graph::rule::RuleError::Eval("PO not found in send response".to_string()))?;
            
            tracing::info!("✅ PO sent: {}", sent_po_grpc.po_id);
            sent_po_grpc
        } else {
            tracing::info!("ℹ️ PO created but not sent (rules decided: should_send_po=false)");
            created_po_grpc
        };
        
        let po_json = json!({
            "po_id": final_po.po_id,
            "product_id": final_po.product_id,
            "supplier_id": final_po.supplier_id,
            "qty": final_po.qty,
            "unit_price": final_po.unit_price,
            "total_amount": final_po.total_amount,
            "status": final_po.status,
            "created_at": final_po.created_at,
            "sent_at": if final_po.sent_at.is_empty() { Value::Null } else { json!(final_po.sent_at) },
        });
        
        ctx.data.insert("po".to_string(), po_json.clone());
        Ok(po_json)
    }
}

/// Graph executor for orchestrator service (using gRPC nodes)
pub struct OrchestratorGraphExecutor {
    oms_grpc_url: String,
    inventory_grpc_url: String,
    supplier_grpc_url: String,
    uom_grpc_url: String,
    rule_engine_grpc_url: String,
    po_grpc_url: String,
}

impl OrchestratorGraphExecutor {
    pub fn new(
        oms_grpc_url: String,
        inventory_grpc_url: String,
        supplier_grpc_url: String,
        uom_grpc_url: String,
        rule_engine_grpc_url: String,
        po_grpc_url: String,
    ) -> Self {
        Self {
            oms_grpc_url,
            inventory_grpc_url,
            supplier_grpc_url,
            uom_grpc_url,
            rule_engine_grpc_url,
            po_grpc_url,
        }
    }
    
    pub async fn execute(&mut self, product_id: &str) -> anyhow::Result<PurchasingFlowResponse> {
        self.execute_with_config(product_id, "purchasing_flow_graph.yaml").await
    }
    
    /// Execute with custom graph configuration file
    pub async fn execute_with_config(&mut self, product_id: &str, config_path: &str) -> anyhow::Result<PurchasingFlowResponse> {
        tracing::info!("🎯 Orchestrator Graph Executor: Starting purchasing flow for {}", product_id);
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
        for (node_id, _node_config) in &graph_def.nodes {
            let node: Box<dyn Node> = match node_id.as_str() {
                "oms_grpc" => Box::new(OmsGrpcNode::new(
                    self.oms_grpc_url.clone(),
                    product_id.to_string(),
                )),
                "inventory_grpc" => Box::new(InventoryGrpcNode::new(
                    self.inventory_grpc_url.clone(),
                    product_id.to_string(),
                )),
                "supplier_grpc" => Box::new(SupplierGrpcNode::new(
                    self.supplier_grpc_url.clone(),
                    product_id.to_string(),
                )),
                "uom_grpc" => Box::new(UomGrpcNode::new(
                    self.uom_grpc_url.clone(),
                    product_id.to_string(),
                )),
                "rule_engine_grpc" => Box::new(RuleEngineGrpcNode::new(
                    self.rule_engine_grpc_url.clone(),
                )),
                "po_grpc" => Box::new(PoGrpcNode::new(
                    self.po_grpc_url.clone(),
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
        graph.context.set("product_id", json!(product_id));
        
        executor.execute(&mut graph).await?;
        
        tracing::info!("✅ Orchestrator graph execution completed");
        
        // Extract results from context
        let po_data = graph.context.data.get("po").cloned();
        let rule_result_data = graph.context.data.get("rule_result").cloned();
        
        let po = if let Some(po_json) = po_data {
            if po_json.is_null() {
                None
            } else {
                Some(PurchaseOrder {
                    po_id: po_json.get("po_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    product_id: po_json.get("product_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    supplier_id: po_json.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    qty: po_json.get("qty").and_then(|v| v.as_i64()).unwrap_or(0),
                    unit_price: po_json.get("unit_price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    total_amount: po_json.get("total_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    status: po_json.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    created_at: po_json.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    sent_at: po_json.get("sent_at").and_then(|v| {
                        if v.is_null() { None } else { v.as_str().map(|s| s.to_string()) }
                    }),
                })
            }
        } else {
            None
        };
        
        let calculation = if let Some(rule_json) = rule_result_data {
            Some(RuleEngineResponse {
                need_reorder: rule_json.get("need_reorder").and_then(|v| v.as_bool()).unwrap_or(false),
                shortage: rule_json.get("shortage").and_then(|v| v.as_f64()).unwrap_or(0.0),
                order_qty: rule_json.get("order_qty").and_then(|v| v.as_i64()).unwrap_or(0),
                total_amount: rule_json.get("total_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                requires_approval: rule_json.get("requires_approval").and_then(|v| v.as_bool()).unwrap_or(false),
                approval_status: rule_json.get("approval_status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            })
        } else {
            None
        };
        
        Ok(PurchasingFlowResponse {
            success: true,
            po,
            calculation,
            message: "Purchasing flow completed via Graph Executor".to_string(),
        })
    }
}
