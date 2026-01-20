pub mod bench_helpers;
pub mod cache;
pub mod core;
pub mod distributed;
pub mod error;
pub mod fault_tolerance;
pub mod integrations;
pub mod io;
pub mod memory;
pub mod multi_db;
pub mod node;
pub mod orchestrator;
pub mod parallel;
pub mod rule;
pub mod saga;
pub mod streaming;

// Re-export main types
pub use cache::{CacheConfig, CacheManager, EvictionPolicy};
pub use core::{
    Context, Edge, ExecutionMetrics, Executor, Graph, GraphDef, NodeConfig, NodeExecutionStats,
};
pub use error::{ErrorCategory, ErrorContext, Result as RLGResult, RustLogicGraphError};
pub use fault_tolerance::{
    CircuitBreaker, CircuitConfig, CircuitState, FailoverManager, HealthMonitor, HealthStatus,
    ServiceEndpoint,
};
pub use io::GraphIO;
pub use memory::{AllocationTracker, ContextPool, MemoryMetrics, PoolConfig};
pub use multi_db::{
    DistributedTransaction, JoinStrategy, ParallelDBExecutor, QueryCorrelator,
    TransactionCoordinator,
};
pub use node::{
    AINode, DBNode, DatabaseExecutor, GrpcNode, MockDatabaseExecutor, Node, NodeType, RuleNode,
};
pub use orchestrator::Orchestrator;
pub use rule::{Rule, RuleEngine, RuleError, RuleResult};
pub use saga::*;

// Re-export rust-rule-engine types for advanced usage
pub use rule::{EngineConfig, Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value as RuleValue};
