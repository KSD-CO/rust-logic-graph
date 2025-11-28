use crate::fault_tolerance::circuit_breaker::CircuitBreaker;
use crate::distributed::InMemoryStore;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
}

pub struct FailoverManager {
    endpoints: Arc<RwLock<Vec<ServiceEndpoint>>>,
    cb: Arc<CircuitBreaker>,
}

impl FailoverManager {
    pub fn new(endpoints: Vec<ServiceEndpoint>, cb: Arc<CircuitBreaker>) -> Self {
        Self { endpoints: Arc::new(RwLock::new(endpoints)), cb }
    }

    pub async fn select(&self) -> Option<ServiceEndpoint> {
        let eps = self.endpoints.read().await;
        for ep in eps.iter() {
            // very simple: if circuit allows, pick first
            if self.cb.is_allowed().await {
                return Some(ep.clone());
            }
        }
        None
    }

    pub async fn add_endpoint(&self, ep: ServiceEndpoint) {
        let mut eps = self.endpoints.write().await;
        eps.push(ep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_tolerance::circuit_breaker::{CircuitConfig};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_failover_selection() {
        let cb = Arc::new(CircuitBreaker::new("svc-a", Some(Arc::new(InMemoryStore::new())), Some(CircuitConfig { failure_threshold: 10, recovery_timeout: std::time::Duration::from_secs(60), probe_interval: std::time::Duration::from_secs(5)})));
        let eps = vec![ServiceEndpoint { name: "primary".into(), url: "https://primary.local".into() }, ServiceEndpoint { name: "backup".into(), url: "https://backup.local".into() }];
        let fm = FailoverManager::new(eps, cb);
        let selected = fm.select().await;
        assert!(selected.is_some());
    }
}
