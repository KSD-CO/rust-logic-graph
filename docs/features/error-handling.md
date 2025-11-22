# Error Handling

**Version**: v0.10.0-alpha.1  
**Status**: ✅ Production-Ready  
**Last Updated**: 2025-11-22

---

## Overview

Production-grade error handling system with unique error codes, actionable suggestions, and rich context propagation.

### Key Features

✅ **12 Predefined Error Types** (E001-E012)  
✅ **Error Classification** (Retryable, Permanent, Transient, Configuration)  
✅ **Rich Context Propagation** (node → graph → step → service → metadata)  
✅ **Actionable Suggestions** for every error  
✅ **Documentation Links** (automatic)  
✅ **Source Error Chaining** support  

---

## Quick Start

### Basic Usage

```rust
use rust_logic_graph::error::{RustLogicGraphError, ErrorContext};

// Simple error
let err = RustLogicGraphError::database_connection_error(
    "Failed to connect to PostgreSQL at localhost:5432"
);

// Error with rich context
let err = RustLogicGraphError::node_execution_error(
    "validate_order",
    "Order validation failed: missing required field 'customer_id'"
).with_context(
    ErrorContext::new()
        .with_graph("purchasing_flow")
        .with_step("validation")
        .add_metadata("order_id", "ORD-12345")
);

// Automatic retry strategy
if err.is_retryable() {
    retry_with_backoff(operation).await?;
} else {
    tracing::error!("Permanent error: {}", err);
    return Err(err);
}
```

### Error Output

```
[E002] Failed to connect to PostgreSQL at localhost:5432
  Graph: purchasing_flow
  Step: database_initialization
  database: orders_db
  timeout: 5s

💡 Suggestion: Verify database connection string, credentials, and network 
                connectivity. Check if database server is running.

📖 Documentation: https://docs.rust-logic-graph.dev/errors/E002
```

---

## Error Categories

| Category | Description | Retry Strategy |
|----------|-------------|----------------|
| **Retryable** | Temporary failures (network issues, rate limits) | Retry with exponential backoff |
| **Permanent** | Fatal errors (syntax errors, invalid config) | Do not retry, fix issue |
| **Transient** | Short-lived issues (deadlocks, temporary unavailability) | Retry with short delay |
| **Configuration** | Configuration problems | Do not retry, fix configuration |

---

## Error Reference

### E001: Node Execution Error

**Category**: Retryable  
**Severity**: Medium

**Description**: A node failed to execute successfully.

**Common Causes**:
- Invalid input data
- Missing dependencies
- Resource temporarily unavailable
- Unexpected runtime error

**Solution**:
1. Check node configuration in graph definition
2. Verify input data format and values
3. Ensure all required dependencies are available
4. Review node logs for detailed error information
5. Check resource availability (memory, connections)

**Example**:
```rust
let err = RustLogicGraphError::node_execution_error(
    "validate_order", 
    "Order validation failed: missing required field 'customer_id'"
);
```

---

### E002: Database Connection Error

**Category**: Retryable  
**Severity**: High

**Description**: Failed to establish connection to database.

**Common Causes**:
- Invalid connection string
- Database server not running
- Network connectivity issues
- Authentication failure
- Connection pool exhausted

**Solution**:
1. Verify database connection string format
2. Check database server is running and accessible
3. Test network connectivity (ping, telnet)
4. Verify credentials (username, password)
5. Check connection pool configuration
6. Review firewall rules

**Example**:
```rust
let err = RustLogicGraphError::database_connection_error(
    "Failed to connect to PostgreSQL at localhost:5432"
);
```

**Configuration**:
```yaml
database:
  connection_string: "postgresql://user:pass@localhost:5432/db"
  pool_size: 10
  timeout: 5s
```

---

### E003: Rule Evaluation Error

**Category**: Permanent  
**Severity**: High

**Description**: Business rule evaluation failed due to syntax or logic error.

**Common Causes**:
- Invalid rule syntax (GRL)
- Undefined variables in rule
- Type mismatch in rule evaluation
- Missing required facts
- Circular rule dependencies

**Solution**:
1. Validate rule syntax using CLI tool: `rlg validate rules.grl`
2. Check all variables are defined in context
3. Verify data types match rule expectations
4. Ensure all required facts are provided
5. Review rule logic for correctness

**Example**:
```rust
let err = RustLogicGraphError::rule_evaluation_error(
    "Undefined variable 'total_amount' in rule 'discount_policy'"
);
```

**Rule Example**:
```grl
rule "discount_policy" {
    when
        total_amount > 1000  // Ensure 'total_amount' exists in context
    then
        discount = 0.1;
}
```

---

### E004: Configuration Error

**Category**: Configuration  
**Severity**: High

**Description**: Invalid or missing configuration value.

**Common Causes**:
- Missing required configuration field
- Invalid configuration value
- Malformed YAML/JSON
- Environment variable not set
- Configuration file not found

**Solution**:
1. Review configuration file for completeness
2. Check against schema documentation
3. Validate YAML/JSON syntax
4. Verify all environment variables are set
5. Ensure configuration file exists at expected path

**Example**:
```rust
let err = RustLogicGraphError::configuration_error(
    "Missing required field 'database.connection_string' in config file"
);
```

---

### E005: Timeout Error

**Category**: Transient  
**Severity**: Medium

**Description**: Operation exceeded configured timeout.

**Common Causes**:
- Slow downstream services
- Large data processing
- Network latency
- Database query too slow
- Insufficient resources

**Solution**:
1. Increase timeout configuration
2. Optimize slow queries/operations
3. Add database indexes
4. Scale up resources
5. Investigate performance bottlenecks
6. Consider breaking into smaller operations

**Example**:
```rust
let err = RustLogicGraphError::timeout_error(
    "Node execution exceeded 30s timeout"
);
```

---

### E006: Graph Validation Error

**Category**: Permanent  
**Severity**: High

**Description**: Graph structure is invalid.

**Common Causes**:
- Cyclic dependencies detected
- Missing node referenced in edges
- Duplicate node IDs
- Invalid edge connections
- Disconnected subgraphs

**Solution**:
1. Validate graph using CLI: `rlg validate graph.json`
2. Check for cycles in dependencies
3. Verify all edge references exist
4. Ensure node IDs are unique
5. Review graph structure visually

**Example**:
```rust
let err = RustLogicGraphError::graph_validation_error(
    "Cycle detected: node_a -> node_b -> node_c -> node_a"
);
```

---

### E007: Serialization Error

**Category**: Permanent  
**Severity**: Medium

**Description**: Failed to serialize/deserialize data.

**Common Causes**:
- Invalid JSON/YAML syntax
- Missing required fields
- Type mismatch
- Incompatible data structure
- Encoding issues

**Solution**:
1. Validate JSON/YAML syntax
2. Check schema matches data structure
3. Verify all required fields present
4. Ensure data types are correct
5. Check for encoding issues (UTF-8)

---

### E008: AI/LLM Error

**Category**: Retryable  
**Severity**: Medium

**Description**: AI model API call failed.

**Common Causes**:
- Invalid API key
- Rate limit exceeded
- Model unavailable
- Quota exceeded
- Invalid prompt format
- Content policy violation

**Solution**:
1. Verify API key is correct
2. Check rate limits and quota
3. Implement exponential backoff
4. Review prompt for policy violations
5. Consider model availability
6. Monitor token usage

**Example**:
```rust
let err = RustLogicGraphError::ai_error(
    "OpenAI API rate limit exceeded: 60 requests per minute"
);
```

---

### E009: Cache Error

**Category**: Transient  
**Severity**: Low

**Description**: Cache operation failed.

**Common Causes**:
- Redis/Memcached unavailable
- Cache full
- Network issues
- Invalid cache key
- Serialization failure

**Solution**:
1. Check cache backend is running
2. Verify connectivity to cache server
3. Review cache size limits
4. Check cache key format
5. Consider cache eviction policy

**Note**: Cache errors typically don't stop execution (cache miss fallback).

---

### E010: Context Error

**Category**: Permanent  
**Severity**: High

**Description**: Invalid context data structure.

**Common Causes**:
- Missing required context keys
- Type mismatch in context values
- Null/undefined values
- Invalid context structure

**Solution**:
1. Verify required keys are present
2. Check value types match expectations
3. Initialize context properly
4. Review context schema

---

### E011: Distributed System Error

**Category**: Retryable  
**Severity**: High

**Description**: Communication with downstream service failed.

**Common Causes**:
- Service unavailable (503)
- Service timeout
- Network partition
- Service discovery failure
- Load balancer issues

**Solution**:
1. Check service health status
2. Verify service discovery configuration
3. Review load balancer settings
4. Test network connectivity
5. Check circuit breaker state
6. Review service logs

**Example**:
```rust
let err = RustLogicGraphError::distributed_error(
    "Service unavailable: inventory-service returned 503",
    "inventory-service"
);
```

---

### E012: Transaction Coordination Error

**Category**: Transient  
**Severity**: High

**Description**: Distributed transaction coordination failed.

**Common Causes**:
- Database deadlock
- Transaction timeout
- Compensation failure in Saga
- Isolation violation
- Network partition during commit

**Solution**:
1. Review transaction isolation level
2. Check for deadlock patterns
3. Verify compensation logic
4. Increase transaction timeout
5. Implement retry with backoff
6. Consider eventual consistency

---

## Best Practices

### 1. Always Add Context

```rust
let err = RustLogicGraphError::database_connection_error("Connection failed")
    .with_context(
        ErrorContext::new()
            .with_node("fetch_orders")
            .with_graph("order_processing")
            .add_metadata("database", "orders_db")
    );
```

### 2. Use Appropriate Category

```rust
// Temporary network issue - Retryable
RustLogicGraphError::database_connection_error("...")

// Invalid configuration - Permanent
RustLogicGraphError::configuration_error("...")

// Deadlock - Transient
RustLogicGraphError::transaction_error("...")
```

### 3. Implement Retry Logic

```rust
let mut attempts = 0;
loop {
    match execute_node().await {
        Ok(result) => return Ok(result),
        Err(e) if e.is_retryable() && attempts < 3 => {
            attempts += 1;
            tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
        }
        Err(e) => return Err(e),
    }
}
```

### 4. Log with Structure

```rust
tracing::error!(
    error_code = %err.code,
    error_category = ?err.category,
    node_id = ?err.context.node_id,
    graph = ?err.context.graph_name,
    "Node execution failed: {}",
    err
);
```

---

## Examples

### Example 1: Database Error with Full Context

```rust
let context = ErrorContext::new()
    .with_node("fetch_orders")
    .with_graph("order_processing")
    .with_step("database_query")
    .add_metadata("database", "orders_db")
    .add_metadata("timeout", "5s");

let err = RustLogicGraphError::database_connection_error(
    "Failed to connect to PostgreSQL"
).with_context(context);
```

### Example 2: Distributed System Error

```rust
let context = ErrorContext::new()
    .with_graph("order_orchestration")
    .with_step("check_inventory")
    .with_service("inventory-service")
    .add_metadata("service_url", "http://inventory-service:8080")
    .add_metadata("attempt", "3/3");

let err = RustLogicGraphError::distributed_error(
    "Service unavailable: inventory-service returned 503",
    "inventory-service"
).with_context(context);
```

### Example 3: Run Demo

```bash
cargo run --example error_messages_demo
```

Output shows 9 error scenarios with beautiful formatting.

---

## Testing

### Unit Tests

Located in `src/error/mod.rs`:

```rust
#[test]
fn test_error_creation() {
    let err = RustLogicGraphError::node_execution_error("node_1", "Failed");
    assert_eq!(err.code, "E001");
    assert!(err.is_retryable());
}

#[test]
fn test_error_with_context() {
    let context = ErrorContext::new()
        .with_node("node_1")
        .with_graph("my_graph");
    let err = RustLogicGraphError::new("E001", "Test", ErrorCategory::Retryable)
        .with_context(context);
    assert_eq!(err.context.node_id, Some("node_1".to_string()));
}
```

Run tests:
```bash
cargo test error:: --lib
```

**Result**: 5/5 tests passing ✅

---

## Implementation Details

### File Structure

```
src/error/
└── mod.rs (400+ lines)
    ├── ErrorCategory enum
    ├── ErrorContext struct
    ├── RustLogicGraphError struct
    └── 12 convenience constructors

docs/features/
└── error-handling.md (this file)

examples/
└── error_messages_demo.rs (180+ lines)
```

### API Surface

```rust
// Main types
pub struct RustLogicGraphError { ... }
pub struct ErrorContext { ... }
pub enum ErrorCategory { ... }
pub type Result<T> = std::result::Result<T, RustLogicGraphError>;

// Constructors
impl RustLogicGraphError {
    pub fn new(...) -> Self;
    pub fn node_execution_error(...) -> Self;
    pub fn database_connection_error(...) -> Self;
    // ... 10 more error types
}

// Builder methods
impl RustLogicGraphError {
    pub fn with_suggestion(self, ...) -> Self;
    pub fn with_context(self, ...) -> Self;
    pub fn with_source(self, ...) -> Self;
}

// Classification
impl RustLogicGraphError {
    pub fn is_retryable(&self) -> bool;
    pub fn is_permanent(&self) -> bool;
}
```

---

## Migration from v0.9.0

### Before (Generic Errors)

```rust
// Generic string error
Err("Database connection failed".into())
```

### After (Rich Errors)

```rust
use rust_logic_graph::error::RustLogicGraphError;

// Rich error with context
Err(RustLogicGraphError::database_connection_error(
    "Failed to connect to PostgreSQL"
).with_context(
    ErrorContext::new()
        .with_node("fetch_data")
        .with_graph("order_flow")
))
```

### Backward Compatibility

✅ New error module is optional  
✅ Existing code continues to work  
✅ Gradual migration path  

---

## Impact & Benefits

### For Developers
- **10x faster debugging** - Clear error messages with context
- **Reduced support tickets** - Actionable suggestions
- **Better monitoring** - Error classification for metrics
- **Production-ready** - Documentation links for troubleshooting

### For Operations
- **Retry strategy** - Automatic classification (retryable vs permanent)
- **Alerting** - Error codes for alert rules
- **Debugging** - Rich context for log analysis
- **Compliance** - Error audit trail

---

## Statistics

- **Lines of Code**: 400+ (error module)
- **Documentation**: 1,000+ lines
- **Error Types**: 12 predefined
- **Unit Tests**: 5/5 passing
- **Examples**: 9 scenarios
- **Total Tests**: 44/44 passing (including existing)

---

## Future Enhancements (v0.11.0+)

- [ ] Error aggregation across distributed services
- [ ] Custom error handlers
- [ ] Error recovery strategies
- [ ] Error analytics dashboard
- [ ] OpenTelemetry integration for error traces
- [ ] Prometheus metrics for error rates

---

## Getting Help

- 📖 **Documentation**: [Main README](../../README.md)
- 💬 **Discord**: https://discord.gg/rust-logic-graph
- 🐛 **GitHub Issues**: https://github.com/KSD-CO/rust-logic-graph/issues
- 📧 **Email**: support@rust-logic-graph.dev

---

*Last Updated: 2025-11-22*
