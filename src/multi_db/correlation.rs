use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::error::{ErrorContext, RustLogicGraphError};

/// Strategy for joining query results from different databases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStrategy {
    /// Inner join: Only include rows that exist in both datasets
    Inner,
    /// Left join: Include all rows from left dataset, matching rows from right
    Left,
    /// Right join: Include all rows from right dataset, matching rows from left
    Right,
    /// Full outer join: Include all rows from both datasets
    Full,
}

/// Query result correlator for joining data from multiple databases
///
/// Enables joining and correlating query results from different databases
/// using common keys, similar to SQL JOINs but across distributed databases.
///
/// # Example
/// ```no_run
/// use rust_logic_graph::multi_db::{QueryCorrelator, JoinStrategy};
/// use serde_json::json;
///
/// let mut correlator = QueryCorrelator::new();
///
/// // Data from OMS database
/// let users = json!([
///     {"user_id": 1, "name": "Alice"},
///     {"user_id": 2, "name": "Bob"},
/// ]);
///
/// // Data from Orders database
/// let orders = json!([
///     {"order_id": 101, "user_id": 1, "amount": 50.0},
///     {"order_id": 102, "user_id": 1, "amount": 75.0},
///     {"order_id": 103, "user_id": 3, "amount": 100.0},
/// ]);
///
/// let result = correlator.join(
///     &users,
///     &orders,
///     "user_id",
///     "user_id",
///     JoinStrategy::Inner
/// ).unwrap();
///
/// println!("Joined {} rows", result.as_array().unwrap().len());
/// ```
pub struct QueryCorrelator {
    /// Optional prefix for disambiguating column names from left dataset
    pub left_prefix: Option<String>,
    /// Optional prefix for disambiguating column names from right dataset
    pub right_prefix: Option<String>,
}

impl QueryCorrelator {
    /// Create a new query correlator
    pub fn new() -> Self {
        Self {
            left_prefix: None,
            right_prefix: None,
        }
    }

    /// Set prefix for left dataset columns (e.g., "user_")
    pub fn with_left_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.left_prefix = Some(prefix.into());
        self
    }

    /// Set prefix for right dataset columns (e.g., "order_")
    pub fn with_right_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.right_prefix = Some(prefix.into());
        self
    }

    /// Join two datasets on specified keys
    ///
    /// # Arguments
    /// * `left` - Left dataset (JSON array of objects)
    /// * `right` - Right dataset (JSON array of objects)
    /// * `left_key` - Key field name in left dataset
    /// * `right_key` - Key field name in right dataset
    /// * `strategy` - Join strategy (Inner, Left, Right, Full)
    ///
    /// # Returns
    /// JSON array containing joined rows
    pub fn join(
        &self,
        left: &Value,
        right: &Value,
        left_key: &str,
        right_key: &str,
        strategy: JoinStrategy,
    ) -> Result<Value, RustLogicGraphError> {
        info!(
            "🔗 Query Correlator: Joining datasets on {}.{} = {}.{} ({:?})",
            self.left_prefix.as_deref().unwrap_or("left"),
            left_key,
            self.right_prefix.as_deref().unwrap_or("right"),
            right_key,
            strategy
        );

        // Validate inputs are arrays
        let left_arr = left.as_array().ok_or_else(|| {
            RustLogicGraphError::graph_validation_error("Left dataset must be an array")
                .with_context(ErrorContext::new().add_metadata("type", &format!("{:?}", left)))
        })?;

        let right_arr = right.as_array().ok_or_else(|| {
            RustLogicGraphError::graph_validation_error("Right dataset must be an array")
                .with_context(ErrorContext::new().add_metadata("type", &format!("{:?}", right)))
        })?;

        debug!(
            "Left dataset: {} rows, Right dataset: {} rows",
            left_arr.len(),
            right_arr.len()
        );

        // Build index for right dataset for efficient lookup
        let mut right_index: HashMap<String, Vec<&Value>> = HashMap::new();
        for right_row in right_arr {
            if let Some(key_value) = right_row.get(right_key) {
                let key_str = self.value_to_string(key_value);
                right_index
                    .entry(key_str)
                    .or_insert_with(Vec::new)
                    .push(right_row);
            }
        }

        let mut result = Vec::new();
        let mut matched_right_indices = std::collections::HashSet::new();

        // Process left dataset
        for left_row in left_arr {
            let left_obj = left_row.as_object().ok_or_else(|| {
                RustLogicGraphError::graph_validation_error("Left row must be an object")
            })?;

            if let Some(key_value) = left_row.get(left_key) {
                let key_str = self.value_to_string(key_value);

                if let Some(matching_rights) = right_index.get(&key_str) {
                    // Found matches - create joined rows
                    for right_row in matching_rights {
                        let joined = self.merge_rows(left_obj, right_row.as_object().unwrap())?;
                        result.push(joined);
                        matched_right_indices.insert(key_str.clone());
                    }
                } else if strategy == JoinStrategy::Left || strategy == JoinStrategy::Full {
                    // No match - include left row with nulls for right columns (LEFT or FULL join)
                    let mut joined = self.prefix_object(left_obj, &self.left_prefix);
                    if let Some(prefix) = &self.right_prefix {
                        // Add null columns from right dataset
                        if let Some(sample_right) = right_arr.first().and_then(|v| v.as_object()) {
                            for key in sample_right.keys() {
                                joined.insert(format!("{}{}", prefix, key), Value::Null);
                            }
                        }
                    }
                    result.push(Value::Object(joined));
                }
            }
        }

        // Handle RIGHT or FULL join - include unmatched right rows
        if strategy == JoinStrategy::Right || strategy == JoinStrategy::Full {
            for right_row in right_arr {
                if let Some(key_value) = right_row.get(right_key) {
                    let key_str = self.value_to_string(key_value);
                    if !matched_right_indices.contains(&key_str) {
                        // Unmatched right row
                        let right_obj = right_row.as_object().unwrap();
                        let mut joined = self.prefix_object(right_obj, &self.right_prefix);

                        // Add null columns from left dataset
                        if let Some(prefix) = &self.left_prefix {
                            if let Some(sample_left) = left_arr.first().and_then(|v| v.as_object())
                            {
                                for key in sample_left.keys() {
                                    joined.insert(format!("{}{}", prefix, key), Value::Null);
                                }
                            }
                        }
                        result.push(Value::Object(joined));
                    }
                }
            }
        }

        info!("✅ Query Correlator: Joined {} rows", result.len());
        Ok(Value::Array(result))
    }

    /// Merge two JSON objects with optional prefixes
    fn merge_rows(
        &self,
        left: &serde_json::Map<String, Value>,
        right: &serde_json::Map<String, Value>,
    ) -> Result<Value, RustLogicGraphError> {
        let mut merged = self.prefix_object(left, &self.left_prefix);
        let right_prefixed = self.prefix_object(right, &self.right_prefix);
        merged.extend(right_prefixed);
        Ok(Value::Object(merged))
    }

    /// Add prefix to all keys in an object
    fn prefix_object(
        &self,
        obj: &serde_json::Map<String, Value>,
        prefix: &Option<String>,
    ) -> serde_json::Map<String, Value> {
        if let Some(p) = prefix {
            obj.iter()
                .map(|(k, v)| (format!("{}{}", p, k), v.clone()))
                .collect()
        } else {
            obj.clone()
        }
    }

    /// Convert a JSON value to string for indexing
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => value.to_string(),
        }
    }
}

impl Default for QueryCorrelator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_inner_join() {
        let correlator = QueryCorrelator::new();

        let users = json!([
            {"user_id": 1, "name": "Alice"},
            {"user_id": 2, "name": "Bob"},
        ]);

        let orders = json!([
            {"order_id": 101, "user_id": 1, "amount": 50.0},
            {"order_id": 102, "user_id": 3, "amount": 100.0},
        ]);

        let result = correlator
            .join(&users, &orders, "user_id", "user_id", JoinStrategy::Inner)
            .unwrap();
        let arr = result.as_array().unwrap();

        // Only user_id=1 matches
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["order_id"], 101);
    }

    #[test]
    fn test_left_join() {
        let correlator = QueryCorrelator::new();

        let users = json!([
            {"user_id": 1, "name": "Alice"},
            {"user_id": 2, "name": "Bob"},
        ]);

        let orders = json!([
            {"order_id": 101, "user_id": 1, "amount": 50.0},
        ]);

        let result = correlator
            .join(&users, &orders, "user_id", "user_id", JoinStrategy::Left)
            .unwrap();
        let arr = result.as_array().unwrap();

        // Both users included (user_id=2 has null order fields)
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_with_prefixes() {
        let correlator = QueryCorrelator::new()
            .with_left_prefix("user_")
            .with_right_prefix("order_");

        let users = json!([{"id": 1, "name": "Alice"}]);
        let orders = json!([{"id": 101, "user_id": 1}]);

        let result = correlator
            .join(&users, &orders, "id", "user_id", JoinStrategy::Inner)
            .unwrap();
        let arr = result.as_array().unwrap();

        // Check prefixed column names
        assert!(arr[0].get("user_id").is_some());
        assert!(arr[0].get("user_name").is_some());
        assert!(arr[0].get("order_id").is_some());
    }
}
