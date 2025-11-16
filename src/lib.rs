
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

// Re-export main types
pub use core::{Graph, GraphDef, Edge, Context, Executor};
pub use node::{Node, NodeType, RuleNode, DBNode, AINode};
pub use rule::{Rule, RuleResult, RuleError, RuleEngine, GrlRule};
pub use orchestrator::Orchestrator;
pub use io::GraphIO;
pub use cache::{CacheManager, CacheConfig, EvictionPolicy};
pub use memory::{ContextPool, PoolConfig, MemoryMetrics, AllocationTracker};
