use rust_rule_engine::{RustRuleEngine, Facts, KnowledgeBase, GRLParser, Value as RRValue, };
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

use super::{RuleError, RuleResult};

/// Advanced rule engine powered by rust-rule-engine
pub struct RuleEngine {
    engine: RustRuleEngine,
}

impl RuleEngine {
    /// Create a new rule engine with default knowledge base
    pub fn new() -> Self {
        let kb = KnowledgeBase::new("LogicGraph");
        Self {
            engine: RustRuleEngine::new(kb),
        }
    }

    /// Add a rule using GRL (Grule Rule Language) syntax
    pub fn add_grl_rule(&mut self, grl_content: &str) -> Result<(), RuleError> {
        let rules = GRLParser::parse_rules(grl_content)
            .map_err(|e| RuleError::Eval(format!("Failed to parse GRL: {}", e)))?;

        for rule in rules {
            self.engine.knowledge_base().add_rule(rule)
                .map_err(|e| RuleError::Eval(format!("Failed to add rule: {}", e)))?;
        }

        Ok(())
    }

    /// Evaluate all rules against the given context
    pub fn evaluate(&mut self, context: &HashMap<String, Value>) -> RuleResult {
        // Convert context to Facts
        let mut facts = Facts::new();

        for (key, value) in context {
            // Convert serde_json::Value to rust_rule_engine::Value
            let rr_value = match value {
                Value::Bool(b) => RRValue::Boolean(*b),
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        RRValue::Number(f)
                    } else {
                        continue;
                    }
                }
                Value::String(s) => RRValue::String(s.clone()),
                _ => {
                    debug!("Skipping unsupported value type for key: {}", key);
                    continue;
                }
            };

            facts.set(&key, rr_value);
        }

        // Execute rules
        match self.engine.execute(&mut facts) {
            Ok(_) => {
                debug!("Rules executed successfully");
                // Return success indicator
                Ok(Value::Bool(true))
            }
            Err(e) => {
                warn!("Rule execution failed: {}", e);
                Err(RuleError::Eval(format!("Rule execution failed: {}", e)))
            }
        }
    }

    /// Create a rule engine from GRL script content
    pub fn from_grl(grl_script: &str) -> Result<Self, RuleError> {
        let mut engine = Self::new();
        engine.add_grl_rule(grl_script)?;
        Ok(engine)
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced rule with GRL support
#[derive(Debug, Clone)]
pub struct GrlRule {
    pub id: String,
    pub grl_content: String,
}

impl GrlRule {
    pub fn new(id: impl Into<String>, grl_content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            grl_content: grl_content.into(),
        }
    }

    /// Evaluate the GRL rule
    pub fn evaluate(&self, context: &HashMap<String, Value>) -> RuleResult {
        let mut engine = RuleEngine::new();
        engine.add_grl_rule(&self.grl_content)?;
        engine.evaluate(context)
    }

    /// Create GRL rule from simple condition
    /// Example: GrlRule::from_simple("age_check", "age >= 18", "eligible = true")
    pub fn from_simple(id: impl Into<String>, condition: &str, action: &str) -> Self {
        let id = id.into();

        // Convert to GRL format
        let grl_content = format!(
            r#"
rule "{}" {{
    when
        {}
    then
        {};
}}
"#,
            id, condition, action
        );

        Self {
            id,
            grl_content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_engine_creation() {
        let _engine = RuleEngine::new();
        // Just test creation works
    }

    #[test]
    fn test_grl_rule_creation() {
        let rule = GrlRule::new("test", "rule test { when true then }");
        assert_eq!(rule.id, "test");
    }

    #[test]
    fn test_rule_from_simple() {
        let rule = GrlRule::from_simple("age_check", "age >= 18", "eligible = true");
        assert!(rule.grl_content.contains("age >= 18"));
        assert!(rule.grl_content.contains("eligible = true"));
    }
}
