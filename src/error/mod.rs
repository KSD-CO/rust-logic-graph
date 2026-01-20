//! Rich error handling for Rust Logic Graph
//!
//! This module provides comprehensive error types with:
//! - Unique error codes for documentation lookup
//! - Error classification (Retryable, Permanent, Transient)
//! - Actionable suggestions for fixing errors
//! - Rich context propagation across distributed systems
//! - Links to troubleshooting documentation

use std::fmt;

/// Error classification for retry strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Error can be retried (temporary network issues, rate limits)
    Retryable,
    /// Error is permanent (invalid configuration, syntax errors)
    Permanent,
    /// Error is transient (database deadlock, temporary unavailability)
    Transient,
    /// Error in configuration (missing required fields, invalid values)
    Configuration,
}

/// Context about where the error occurred in the graph execution
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Node ID where error occurred
    pub node_id: Option<String>,
    /// Graph name being executed
    pub graph_name: Option<String>,
    /// Execution step/phase
    pub execution_step: Option<String>,
    /// Service name (for distributed systems)
    pub service_name: Option<String>,
    /// Additional context key-value pairs
    pub metadata: Vec<(String, String)>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            node_id: None,
            graph_name: None,
            execution_step: None,
            service_name: None,
            metadata: Vec::new(),
        }
    }

    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_graph(mut self, graph_name: impl Into<String>) -> Self {
        self.graph_name = Some(graph_name.into());
        self
    }

    pub fn with_step(mut self, step: impl Into<String>) -> Self {
        self.execution_step = Some(step.into());
        self
    }

    pub fn with_service(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main error type for Rust Logic Graph
#[derive(Debug)]
pub struct RustLogicGraphError {
    /// Unique error code (e.g., "E001", "E002")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Error classification for retry logic
    pub category: ErrorCategory,
    /// Actionable suggestion for fixing the error
    pub suggestion: Option<String>,
    /// Link to documentation/troubleshooting
    pub doc_link: Option<String>,
    /// Rich context about where error occurred
    pub context: ErrorContext,
    /// Underlying cause (if any)
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl RustLogicGraphError {
    /// Create a new error with code and message
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        category: ErrorCategory,
    ) -> Self {
        let code = code.into();
        let doc_link = Some(format!("https://docs.rust-logic-graph.dev/errors/{}", code));

        Self {
            code,
            message: message.into(),
            category,
            suggestion: None,
            doc_link,
            context: ErrorContext::new(),
            source: None,
        }
    }

    /// Add an actionable suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add error context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Add underlying source error
    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.category,
            ErrorCategory::Retryable | ErrorCategory::Transient
        )
    }

    /// Check if error is permanent
    pub fn is_permanent(&self) -> bool {
        matches!(
            self.category,
            ErrorCategory::Permanent | ErrorCategory::Configuration
        )
    }
}

impl fmt::Display for RustLogicGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;

        // Add context information
        if let Some(ref graph) = self.context.graph_name {
            write!(f, "\n  Graph: {}", graph)?;
        }
        if let Some(ref node) = self.context.node_id {
            write!(f, "\n  Node: {}", node)?;
        }
        if let Some(ref step) = self.context.execution_step {
            write!(f, "\n  Step: {}", step)?;
        }
        if let Some(ref service) = self.context.service_name {
            write!(f, "\n  Service: {}", service)?;
        }

        // Add metadata
        for (key, value) in &self.context.metadata {
            write!(f, "\n  {}: {}", key, value)?;
        }

        // Add suggestion
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n\n💡 Suggestion: {}", suggestion)?;
        }

        // Add documentation link
        if let Some(ref link) = self.doc_link {
            write!(f, "\n📖 Documentation: {}", link)?;
        }

        // Add source error
        if let Some(ref source) = self.source {
            write!(f, "\n\nCaused by: {}", source)?;
        }

        Ok(())
    }
}

impl std::error::Error for RustLogicGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

// Convenience constructors for common error types

impl RustLogicGraphError {
    /// Node execution error
    pub fn node_execution_error(node_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new("E001", message, ErrorCategory::Retryable)
            .with_context(ErrorContext::new().with_node(node_id))
            .with_suggestion(
                "Check node configuration and input data. Verify all dependencies are available.",
            )
    }

    /// Database connection error
    pub fn database_connection_error(message: impl Into<String>) -> Self {
        Self::new("E002", message, ErrorCategory::Retryable)
            .with_suggestion("Verify database connection string, credentials, and network connectivity. Check if database server is running.")
    }

    /// Rule evaluation error
    pub fn rule_evaluation_error(message: impl Into<String>) -> Self {
        Self::new("E003", message, ErrorCategory::Permanent)
            .with_suggestion("Check rule syntax and ensure all required facts are present. Verify rule logic is correct.")
    }

    /// Configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new("E004", message, ErrorCategory::Configuration)
            .with_suggestion("Review configuration file for missing or invalid values. Check against schema documentation.")
    }

    /// Timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::new("E005", message, ErrorCategory::Transient)
            .with_suggestion("Increase timeout duration or investigate performance bottlenecks. Check for slow downstream services.")
    }

    /// Graph validation error
    pub fn graph_validation_error(message: impl Into<String>) -> Self {
        Self::new("E006", message, ErrorCategory::Permanent)
            .with_suggestion("Verify graph structure is valid. Check for cycles, missing nodes, or invalid edge connections.")
    }

    /// Serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::new("E007", message, ErrorCategory::Permanent)
            .with_suggestion("Check data format and ensure all required fields are present. Verify JSON/YAML syntax is valid.")
    }

    /// AI/LLM error
    pub fn ai_error(message: impl Into<String>) -> Self {
        Self::new("E008", message, ErrorCategory::Retryable)
            .with_suggestion("Verify API key and model availability. Check rate limits and quota. Review prompt for issues.")
    }

    /// Cache error
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::new("E009", message, ErrorCategory::Transient).with_suggestion(
            "Check cache configuration and connectivity. Verify cache backend is operational.",
        )
    }

    /// Context error
    pub fn context_error(message: impl Into<String>) -> Self {
        Self::new("E010", message, ErrorCategory::Permanent)
            .with_suggestion("Verify context data structure. Ensure required keys are present and values are correct types.")
    }

    /// Distributed system error
    pub fn distributed_error(message: impl Into<String>, service: impl Into<String>) -> Self {
        Self::new("E011", message, ErrorCategory::Retryable)
            .with_context(ErrorContext::new().with_service(service))
            .with_suggestion("Check service health and network connectivity. Verify service discovery and load balancing configuration.")
    }

    /// Transaction coordination error
    pub fn transaction_error(message: impl Into<String>) -> Self {
        Self::new("E012", message, ErrorCategory::Transient)
            .with_suggestion("Review transaction logic and compensation handlers. Check for deadlocks or isolation issues.")
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, RustLogicGraphError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RustLogicGraphError::node_execution_error("node_1", "Failed to execute node");
        assert_eq!(err.code, "E001");
        assert_eq!(err.category, ErrorCategory::Retryable);
        assert!(err.is_retryable());
        assert!(!err.is_permanent());
    }

    #[test]
    fn test_error_with_context() {
        let context = ErrorContext::new()
            .with_node("node_1")
            .with_graph("my_graph")
            .with_step("execution");

        let err = RustLogicGraphError::new("E001", "Test error", ErrorCategory::Retryable)
            .with_context(context);

        assert_eq!(err.context.node_id, Some("node_1".to_string()));
        assert_eq!(err.context.graph_name, Some("my_graph".to_string()));
    }

    #[test]
    fn test_error_display() {
        let err = RustLogicGraphError::database_connection_error("Connection timeout");
        let display = format!("{}", err);

        assert!(display.contains("[E002]"));
        assert!(display.contains("Connection timeout"));
        assert!(display.contains("💡 Suggestion:"));
        assert!(display.contains("📖 Documentation:"));
    }

    #[test]
    fn test_error_categories() {
        assert!(RustLogicGraphError::database_connection_error("test").is_retryable());
        assert!(RustLogicGraphError::configuration_error("test").is_permanent());
        assert!(RustLogicGraphError::timeout_error("test").is_retryable());
        assert!(RustLogicGraphError::graph_validation_error("test").is_permanent());
    }

    #[test]
    fn test_error_with_metadata() {
        let context = ErrorContext::new()
            .add_metadata("user_id", "123")
            .add_metadata("request_id", "req_456");

        let err = RustLogicGraphError::new("E001", "Test", ErrorCategory::Retryable)
            .with_context(context);

        assert_eq!(err.context.metadata.len(), 2);
    }
}
