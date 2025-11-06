
pub mod core;
pub mod node;
pub mod rule;
pub mod orchestrator;
pub mod io;
pub mod integrations;
pub mod streaming;
pub mod parallel;

// Re-export main types
pub use core::{Graph, GraphDef, Edge, Context, Executor};
pub use node::{Node, NodeType, RuleNode, DBNode, AINode};
pub use rule::{Rule, RuleResult, RuleError, RuleEngine, GrlRule};
pub use orchestrator::Orchestrator;
pub use io::GraphIO;
