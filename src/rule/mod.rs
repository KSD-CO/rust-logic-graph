mod engine;

// Re-export the convenience wrapper
pub use engine::RuleEngine;

// Re-export rust-rule-engine types for advanced usage
pub use engine::{EngineConfig, Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Rule evaluation failed: {0}")]
    Eval(String),

    #[error("Missing variable in context: {0}")]
    MissingVariable(String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

pub type RuleResult = Result<JsonValue, RuleError>;

/// Simple rule implementation (backward compatible)
/// For advanced features, use RuleEngine directly
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub condition: String,
}

impl Rule {
    pub fn new(id: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            condition: condition.into(),
        }
    }

    /// Evaluate the rule against provided data context
    pub fn evaluate(&self, context: &HashMap<String, JsonValue>) -> RuleResult {
        let condition = self.condition.trim();

        // Handle simple boolean literals
        if condition == "true" {
            return Ok(JsonValue::Bool(true));
        }
        if condition == "false" {
            return Ok(JsonValue::Bool(false));
        }

        // Handle variable lookup (e.g., "user_active")
        if !condition.contains(' ') && !condition.contains(['>', '<', '=', '!']) {
            return context
                .get(condition)
                .cloned()
                .ok_or_else(|| RuleError::MissingVariable(condition.to_string()));
        }

        // Handle comparisons (e.g., "age > 18", "status == active")
        if let Some(result) = self.evaluate_comparison(condition, context) {
            return result;
        }

        // Handle logical operations (e.g., "active && verified")
        if condition.contains("&&") || condition.contains("||") {
            return self.evaluate_logical(condition, context);
        }

        // Default to true if we can't parse (permissive)
        Ok(JsonValue::Bool(true))
    }

    fn evaluate_comparison(
        &self,
        expr: &str,
        context: &HashMap<String, JsonValue>,
    ) -> Option<RuleResult> {
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some((left, right)) = expr.split_once(op) {
                let left = left.trim();
                let right = right.trim();

                let left_val = self.get_value(left, context).ok()?;
                let right_val = self.get_value(right, context).ok()?;

                let result = match op {
                    "==" => left_val == right_val,
                    "!=" => left_val != right_val,
                    ">" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater),
                    "<" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less),
                    ">=" => {
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater)
                            || left_val == right_val
                    }
                    "<=" => {
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less)
                            || left_val == right_val
                    }
                    _ => false,
                };

                return Some(Ok(JsonValue::Bool(result)));
            }
        }
        None
    }

    fn evaluate_logical(&self, expr: &str, context: &HashMap<String, JsonValue>) -> RuleResult {
        if let Some((left, right)) = expr.split_once("&&") {
            let left_result = Rule::new("temp", left.trim()).evaluate(context)?;
            let right_result = Rule::new("temp", right.trim()).evaluate(context)?;

            if let (JsonValue::Bool(l), JsonValue::Bool(r)) = (left_result, right_result) {
                return Ok(JsonValue::Bool(l && r));
            }
        }

        if let Some((left, right)) = expr.split_once("||") {
            let left_result = Rule::new("temp", left.trim()).evaluate(context)?;
            let right_result = Rule::new("temp", right.trim()).evaluate(context)?;

            if let (JsonValue::Bool(l), JsonValue::Bool(r)) = (left_result, right_result) {
                return Ok(JsonValue::Bool(l || r));
            }
        }

        Err(RuleError::InvalidExpression(expr.to_string()))
    }

    fn get_value(&self, s: &str, context: &HashMap<String, JsonValue>) -> RuleResult {
        // Try to parse as number
        if let Ok(num) = s.parse::<i64>() {
            return Ok(JsonValue::Number(num.into()));
        }

        // Try to parse as float
        if let Ok(num) = s.parse::<f64>() {
            if let Some(n) = serde_json::Number::from_f64(num) {
                return Ok(JsonValue::Number(n));
            }
        }

        // Try to parse as boolean
        if s == "true" {
            return Ok(JsonValue::Bool(true));
        }
        if s == "false" {
            return Ok(JsonValue::Bool(false));
        }

        // Try string literal (quoted)
        if s.starts_with('"') && s.ends_with('"') {
            return Ok(JsonValue::String(s[1..s.len() - 1].to_string()));
        }

        // Otherwise, look up in context
        context
            .get(s)
            .cloned()
            .ok_or_else(|| RuleError::MissingVariable(s.to_string()))
    }

    fn compare_values(
        &self,
        left: &JsonValue,
        right: &JsonValue,
        ordering: std::cmp::Ordering,
    ) -> bool {
        match (left, right) {
            (JsonValue::Number(l), JsonValue::Number(r)) => {
                if let (Some(l), Some(r)) = (l.as_f64(), r.as_f64()) {
                    return l.partial_cmp(&r) == Some(ordering);
                }
            }
            (JsonValue::String(l), JsonValue::String(r)) => {
                return l.cmp(r) == ordering;
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_boolean() {
        let rule = Rule::new("r1", "true");
        let context = HashMap::new();
        assert_eq!(rule.evaluate(&context).unwrap(), JsonValue::Bool(true));
    }

    #[test]
    fn test_comparison() {
        let mut context = HashMap::new();
        context.insert("age".to_string(), JsonValue::Number(25.into()));

        let rule = Rule::new("r1", "age > 18");
        assert_eq!(rule.evaluate(&context).unwrap(), JsonValue::Bool(true));

        let rule = Rule::new("r2", "age < 20");
        assert_eq!(rule.evaluate(&context).unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_logical_and() {
        let mut context = HashMap::new();
        context.insert("active".to_string(), JsonValue::Bool(true));
        context.insert("verified".to_string(), JsonValue::Bool(true));

        let rule = Rule::new("r1", "active && verified");
        assert_eq!(rule.evaluate(&context).unwrap(), JsonValue::Bool(true));
    }
}
