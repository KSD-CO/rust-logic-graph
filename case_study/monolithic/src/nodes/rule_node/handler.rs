use rust_logic_graph::{NodeType, Context};
use rust_logic_graph::node::Node;
use rust_logic_graph::rule::{RuleResult, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;

use super::evaluator::{evaluate_rule_engine, create_purchase_order};

/// Rule node for business logic evaluation
pub struct DynamicRuleNode {
    id: String,
    condition: String,
    inputs: HashMap<String, String>,
    field_mappings: HashMap<String, String>,
}

impl DynamicRuleNode {
    pub fn new(id: String, condition: String) -> Self {
        Self { 
            id, 
            condition,
            inputs: HashMap::new(),
            field_mappings: HashMap::new(),
        }
    }
    
    pub fn with_mappings(
        id: String, 
        condition: String,
        inputs: HashMap<String, String>,
        field_mappings: HashMap<String, String>,
    ) -> Self {
        Self { id, condition, inputs, field_mappings }
    }
}

#[async_trait]
impl Node for DynamicRuleNode {
    fn id(&self) -> &str { &self.id }
    fn node_type(&self) -> NodeType { NodeType::RuleNode }
    
    async fn run(&self, ctx: &mut Context) -> RuleResult {
        eprintln!("🧠🧠🧠 RuleNode[{}]: STARTING run() 🧠🧠🧠", self.id);
        tracing::info!("🧠 RuleNode[{}]: Evaluating condition from YAML", self.id);
        tracing::debug!("Condition: {}", self.condition);
        
        let result = match self.id.as_str() {
            "rule_engine" => {
                eprintln!("🔍 Calling evaluate_rule_engine...");
                evaluate_rule_engine(ctx, &self.id, &self.field_mappings)
            }
            "create_po" => {
                eprintln!("🔍 Calling create_purchase_order...");
                create_purchase_order(ctx, &self.field_mappings)
            }
            _ => Err(RuleError::Eval(format!("Unknown rule node: {}", self.id)))
        };
        
        eprintln!("🧠🧠🧠 RuleNode[{}]: FINISHED run() with result: {:?} 🧠🧠🧠", self.id, result.is_ok());
        result
    }
}
