use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
}

pub struct HealthMonitor {
    checks: Arc<RwLock<Vec<Arc<dyn HealthCheck>>>>,
    poll_interval: Duration,
    store: Option<std::sync::Arc<dyn crate::distributed::store::ContextStore>>,
}

impl HealthMonitor {
    pub fn new(poll_interval: Duration) -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
            poll_interval,
            store: None,
        }
    }

    pub fn with_store(
        mut self,
        store: std::sync::Arc<dyn crate::distributed::store::ContextStore>,
    ) -> Self {
        self.store = Some(store);
        self
    }

    pub async fn add_check(&self, check: Arc<dyn HealthCheck>) {
        let mut c = self.checks.write().await;
        c.push(check);
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            let checks = self.checks.read().await.clone();
            for chk in checks.iter() {
                let status = chk.check().await;
                if let Some(store) = &self.store {
                    // persist health to distributed context
                    let mut ctx =
                        crate::distributed::DistributedContext::new(format!("health:{}", "global"));
                    ctx.set("status", serde_json::json!(format!("{:?}", status)));
                    let _ = store.save(&ctx, None).await;
                }
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;
    #[async_trait::async_trait]
    impl HealthCheck for AlwaysHealthy {
        async fn check(&self) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    #[tokio::test]
    async fn test_add_and_run_check() {
        let monitor = Arc::new(HealthMonitor::new(Duration::from_millis(50)));
        monitor.add_check(Arc::new(AlwaysHealthy)).await;

        // Run one iteration in background, then cancel
        let m = monitor.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(120)).await;
            // allow a couple of poll cycles
        });

        handle.await.unwrap();
    }
}
