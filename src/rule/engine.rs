// Re-export rust-rule-engine types for convenience
pub use rust_rule_engine::{
    engine::{facts::Facts, knowledge_base::KnowledgeBase, EngineConfig, RustRuleEngine},
    types::Value,
    GRLParser, // v1.18.0-alpha: re-exported from root (now uses grl_no_regex parser)
};

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::debug;

use super::{RuleError, RuleResult};

/// Convenience wrapper around RustRuleEngine with JSON integration
///
/// This provides a simplified API for common use cases while maintaining
/// full access to the underlying rust-rule-engine capabilities.
///
/// # Thread Safety
/// RustRuleEngine is thread-safe (Send + Sync), making it suitable for
/// use in multi-threaded web services like Axum.
///
/// # Example
/// ```no_run
/// use rust_logic_graph::RuleEngine;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// let mut engine = RuleEngine::new();
///
/// let grl = r#"
///     rule "discount_rule" {
///         salience 100
///         when
///             total > 100
///         then
///             discount = total * 0.1;
///     }
/// "#;
///
/// engine.add_grl_rule(grl).unwrap();
///
/// let mut context = HashMap::new();
/// context.insert("total".to_string(), json!(150.0));
///
/// let result = engine.evaluate(&context).unwrap();
/// ```
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

    /// Create a new rule engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        let kb = KnowledgeBase::new("LogicGraph");
        Self {
            engine: RustRuleEngine::with_config(kb, config),
        }
    }

    /// Add rules from GRL syntax
    ///
    /// # Example
    /// ```no_run
    /// use rust_logic_graph::RuleEngine;
    ///
    /// let mut engine = RuleEngine::new();
    /// let grl = r#"
    ///     rule "high_value_order" {
    ///         salience 100
    ///         when
    ///             order_amount > 1000
    ///         then
    ///             priority = "high";
    ///             requires_approval = true;
    ///     }
    /// "#;
    /// engine.add_grl_rule(grl).unwrap();
    /// ```
    pub fn add_grl_rule(&mut self, grl_content: &str) -> Result<(), RuleError> {
        let start = std::time::Instant::now();
        debug!("⏱️  [GRL Parse] Starting GRLParser::parse_rules()...");

        let parse_start = std::time::Instant::now();
        let rules = GRLParser::parse_rules(grl_content)
            .map_err(|e| RuleError::Eval(format!("Failed to parse GRL: {}", e)))?;
        let parse_elapsed = parse_start.elapsed();

        let rule_count = rules.len();
        debug!(
            "   ✅ GRLParser::parse_rules() took {:.3}s for {} rules",
            parse_elapsed.as_secs_f64(),
            rule_count
        );

        debug!(
            "⏱️  [GRL Add] Adding {} rules to knowledge_base...",
            rule_count
        );
        let add_start = std::time::Instant::now();

        for (idx, rule) in rules.into_iter().enumerate() {
            let rule_start = std::time::Instant::now();
            self.engine
                .knowledge_base()
                .add_rule(rule)
                .map_err(|e| RuleError::Eval(format!("Failed to add rule: {}", e)))?;
            let rule_elapsed = rule_start.elapsed();

            if rule_elapsed.as_millis() > 10 {
                debug!(
                    "      Rule #{} took {:.3}ms",
                    idx + 1,
                    rule_elapsed.as_secs_f64() * 1000.0
                );
            }
        }

        let add_elapsed = add_start.elapsed();
        debug!(
            "   ✅ add_rule() loop took {:.3}s",
            add_elapsed.as_secs_f64()
        );

        let total_elapsed = start.elapsed();
        debug!(
            "✅ [GRL Total] Loaded {} GRL rules in {:.3}s",
            rule_count,
            total_elapsed.as_secs_f64()
        );

        Ok(())
    }

    /// Evaluate rules with JSON context (convenience method)
    ///
    /// For more control, use `inner()` or `inner_mut()` to access the
    /// underlying RustRuleEngine directly.
    ///
    /// # Example
    /// ```no_run
    /// use rust_logic_graph::RuleEngine;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut engine = RuleEngine::new();
    /// // ... add rules ...
    ///
    /// let mut context = HashMap::new();
    /// context.insert("total".to_string(), json!(150.0));
    ///
    /// let result = engine.evaluate(&context).unwrap();
    /// ```
    pub fn evaluate(&mut self, context: &HashMap<String, JsonValue>) -> RuleResult {
        // Convert JSON context to Facts
        let facts = Facts::new();

        for (key, value) in context {
            let rr_value = match value {
                JsonValue::Bool(b) => Value::Boolean(*b),
                JsonValue::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        Value::Number(f)
                    } else {
                        continue;
                    }
                }
                JsonValue::String(s) => Value::String(s.clone()),
                _ => {
                    debug!("Skipping unsupported value type for key: {}", key);
                    continue;
                }
            };

            facts.set(&key, rr_value);
        }

        // Execute rules
        match self.engine.execute(&facts) {
            Ok(_) => {
                debug!("Rules executed successfully");

                // Helper to convert Value to JsonValue
                let convert_value = |val: &Value| -> Option<JsonValue> {
                    match val {
                        Value::Boolean(b) => Some(JsonValue::Bool(*b)),
                        Value::Number(n) => Some(JsonValue::from(*n)),
                        Value::String(s) => Some(JsonValue::String(s.clone())),
                        Value::Integer(i) => Some(JsonValue::from(*i)),
                        _ => None,
                    }
                };

                // Get ALL facts from the engine after rule execution
                // This captures all values set by rules (including Expression Evaluation results)
                let all_facts = facts.get_all_facts();

                let mut result = HashMap::new();
                for (key, value) in all_facts {
                    if let Some(json_value) = convert_value(&value) {
                        result.insert(key, json_value);
                    }
                }

                Ok(JsonValue::Object(result.into_iter().collect()))
            }
            Err(e) => Err(RuleError::Eval(format!("Rule execution failed: {}", e))),
        }
    }

    /// Create a rule engine from GRL script
    ///
    /// # Example
    /// ```no_run
    /// use rust_logic_graph::RuleEngine;
    ///
    /// let grl = r#"
    ///     rule "example" {
    ///         salience 100
    ///         when
    ///             x > 0
    ///         then
    ///             y = x * 2;
    ///     }
    /// "#;
    ///
    /// let mut engine = RuleEngine::from_grl(grl).unwrap();
    /// ```
    pub fn from_grl(grl_script: &str) -> Result<Self, RuleError> {
        let mut engine = Self::new();
        engine.add_grl_rule(grl_script)?;
        Ok(engine)
    }

    /// Get reference to underlying RustRuleEngine for advanced usage
    ///
    /// This provides full access to rust-rule-engine features:
    /// - Custom functions
    /// - Templates
    /// - Globals
    /// - Deffacts
    /// - Fine-grained control
    pub fn inner(&self) -> &RustRuleEngine {
        &self.engine
    }

    /// Get mutable reference to underlying RustRuleEngine
    pub fn inner_mut(&mut self) -> &mut RustRuleEngine {
        &mut self.engine
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rule_engine_creation() {
        let _engine = RuleEngine::new();
    }

    #[test]
    fn test_engine_evaluation() {
        let mut engine = RuleEngine::new();

        let grl = r#"
        rule "test_rule" {
            salience 100
            when
                age >= 18
            then
                eligible = true;
        }
        "#;

        engine.add_grl_rule(grl).unwrap();

        let mut context = HashMap::new();
        context.insert("age".to_string(), json!(20));

        let result = engine.evaluate(&context).unwrap();
        assert_eq!(result.get("eligible").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_from_grl() {
        let grl = r#"
        rule "test" {
            salience 100
            when
                x > 0
            then
                result = true;
                message = "x is positive";
        }
        "#;

        let mut engine = RuleEngine::from_grl(grl).unwrap();

        let mut context = HashMap::new();
        context.insert("x".to_string(), json!(5));

        let result = engine.evaluate(&context).unwrap();
        assert_eq!(result.get("result").unwrap().as_bool().unwrap(), true);
        assert_eq!(
            result.get("message").unwrap().as_str().unwrap(),
            "x is positive"
        );
    }

    #[test]
    fn test_multiple_rules_salience() {
        let mut engine = RuleEngine::new();

        let grl = r#"
        rule "high_priority" {
            salience 100
            when
                value > 100
            then
                priority = "high";
                high_rule_fired = true;
        }

        rule "medium_priority" {
            salience 50
            when
                value > 50 && value <= 100
            then
                priority = "medium";
                medium_rule_fired = true;
        }
        "#;

        engine.add_grl_rule(grl).unwrap();

        // Test with high value
        let mut context = HashMap::new();
        context.insert("value".to_string(), json!(150));

        let result = engine.evaluate(&context).unwrap();
        // Only high priority rule should fire for value > 100
        assert_eq!(result.get("priority").unwrap().as_str().unwrap(), "high");
        assert_eq!(
            result.get("high_rule_fired").unwrap().as_bool().unwrap(),
            true
        );

        // Test with medium value
        let mut context2 = HashMap::new();
        context2.insert("value".to_string(), json!(75));

        let result2 = engine.evaluate(&context2).unwrap();
        // Only medium priority rule should fire for 50 < value <= 100
        assert_eq!(result2.get("priority").unwrap().as_str().unwrap(), "medium");
        assert_eq!(
            result2.get("medium_rule_fired").unwrap().as_bool().unwrap(),
            true
        );
    }

    #[test]
    fn test_direct_engine_access() {
        let engine = RuleEngine::new();

        // Access underlying RustRuleEngine
        let inner = engine.inner();
        let kb = inner.knowledge_base();

        // Can access knowledge base directly
        assert_eq!(kb.name(), "LogicGraph");
    }
}
