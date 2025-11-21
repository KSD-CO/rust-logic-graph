
use anyhow::Result;
use tracing::info;

use crate::core::{Graph, Executor};

pub struct Orchestrator {
    executor: Executor,
}

impl Orchestrator {
    /// Create a new orchestrator with an executor
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }

    /// Execute the graph using the internal executor
    pub async fn execute(&mut self, graph: &mut Graph) -> Result<()> {
        info!("Orchestrator: Starting orchestration...");
        self.executor.execute(graph).await?;
        info!("Orchestrator: Orchestration completed");
        Ok(())
    }

    /// Execute a graph using a default executor built from the graph definition
    pub async fn execute_graph(graph: &mut Graph) -> Result<()> {
        info!("Orchestrator: Building executor from graph definition");
        let executor = Executor::from_graph_def(&graph.def)?;
        let mut orchestrator = Self::new(executor);
        orchestrator.execute(graph).await
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new(Executor::default())
    }
}
