// Multi-Database Query Orchestration
//
// This module provides capabilities for executing queries across multiple databases
// in parallel, correlating results, and managing distributed transactions.

pub mod parallel;
pub mod correlation;
pub mod transaction;

pub use parallel::ParallelDBExecutor;
pub use correlation::{JoinStrategy, QueryCorrelator};
pub use transaction::{DistributedTransaction, TransactionCoordinator};
