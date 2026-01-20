// Multi-Database Query Orchestration
//
// This module provides capabilities for executing queries across multiple databases
// in parallel, correlating results, and managing distributed transactions.

pub mod correlation;
pub mod parallel;
pub mod transaction;

pub use correlation::{JoinStrategy, QueryCorrelator};
pub use parallel::ParallelDBExecutor;
pub use transaction::{DistributedTransaction, TransactionCoordinator};
