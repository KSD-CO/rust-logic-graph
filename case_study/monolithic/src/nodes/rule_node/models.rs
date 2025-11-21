use serde::{Deserialize, Serialize};

/// Rule evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    pub should_order: bool,
    pub recommended_qty: f64,
    pub reason: String,
}
