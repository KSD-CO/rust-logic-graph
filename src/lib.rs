
pub mod core;
pub mod node;
pub mod rule;
pub mod orchestrator;
pub mod io;
pub mod integrations;
pub mod streaming;
pub mod parallel;
pub mod cache;
pub mod bench_helpers;
pub mod memory;
pub mod error;
pub mod multi_db;
pub mod distributed;
pub mod fault_tolerance;
pub mod saga;

// Re-export main types
pub use saga::*;
pub use core::{Graph, GraphDef, Edge, Context, Executor, NodeConfig, ExecutionMetrics, NodeExecutionStats};
pub use node::{Node, NodeType, RuleNode, DBNode, AINode, GrpcNode, DatabaseExecutor, MockDatabaseExecutor};
pub use rule::{Rule, RuleResult, RuleError, RuleEngine};
pub use orchestrator::Orchestrator;
pub use io::GraphIO;
pub use cache::{CacheManager, CacheConfig, EvictionPolicy};
pub use memory::{ContextPool, PoolConfig, MemoryMetrics, AllocationTracker};
pub use error::{RustLogicGraphError, ErrorCategory, ErrorContext, Result as RLGResult};
pub use multi_db::{ParallelDBExecutor, QueryCorrelator, JoinStrategy, DistributedTransaction, TransactionCoordinator};
pub use fault_tolerance::{CircuitBreaker, CircuitState, CircuitConfig, HealthMonitor, HealthStatus, FailoverManager, ServiceEndpoint};

// Re-export rust-rule-engine types for advanced usage
pub use rule::{Facts, KnowledgeBase, GRLParser, Value as RuleValue, EngineConfig, RustRuleEngine};
