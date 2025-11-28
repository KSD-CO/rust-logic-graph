use crate::distributed::store::ContextStore;
use crate::distributed::InMemoryStore;
use crate::fault_tolerance::health::{HealthCheck, HealthStatus};
use std::sync::Arc;
use tokio::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open { until: Instant },
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub probe_interval: Duration,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            probe_interval: Duration::from_secs(5),
        }
    }
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<RwLock<u32>>,
    config: CircuitConfig,
    store: Arc<dyn ContextStore>,
    key: String,
    health_check: Option<Arc<dyn HealthCheck>>,
}

impl CircuitBreaker {
    pub fn new(key: impl Into<String>, store: Option<Arc<dyn ContextStore>>, config: Option<CircuitConfig>) -> Self {
        let cfg = config.unwrap_or_default();
        let st: Arc<dyn ContextStore> = match store {
            Some(s) => s,
            None => Arc::new(InMemoryStore::new()),
        };

        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(RwLock::new(0)),
            config: cfg,
            store: st,
            key: key.into(),
            health_check: None,
        }
    }

    pub fn with_health_check(mut self, check: Arc<dyn HealthCheck>) -> Self {
        self.health_check = Some(check);
        self
    }

    pub async fn record_success(&self) {
        // Update failures and state, then persist without holding locks
        {
            let mut f = self.failures.write().await;
            *f = 0;
        }

        let state_debug = {
            let mut s = self.state.write().await;
            *s = CircuitState::Closed;
            format!("{:?}", *s)
        };

        let failures_val = *self.failures.read().await;
        let _ = self.persist_state_payload(state_debug, failures_val).await;
    }

    pub async fn record_failure(&self) {
        {
            let mut f = self.failures.write().await;
            *f += 1;
        }

        let failures_val = *self.failures.read().await;
        if failures_val >= self.config.failure_threshold {
            let until = Instant::now() + self.config.recovery_timeout;
            let state_debug = {
                let mut s = self.state.write().await;
                *s = CircuitState::Open { until };
                format!("{:?}", *s)
            };
            let _ = self.persist_state_payload(state_debug, failures_val).await;
        }
    }

    pub async fn is_allowed(&self) -> bool {
        let mut s = self.state.write().await;
        match &*s {
            CircuitState::Closed => true,
            CircuitState::Open { until } => {
                if Instant::now() >= *until {
                    *s = CircuitState::HalfOpen;
                    let state_debug = format!("{:?}", *s);
                    let failures_val = *self.failures.read().await;
                    let _ = self.persist_state_payload(state_debug, failures_val).await;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub async fn probe_loop(self: Arc<Self>) {
        loop {
            tokio::time::sleep(self.config.probe_interval).await;

            // If a health check was provided, run it to decide state
            if let Some(check) = &self.health_check {
                match check.check().await {
                    HealthStatus::Healthy => {
                        let mut s = self.state.write().await;
                        if let CircuitState::HalfOpen = &*s {
                            *s = CircuitState::Closed;
                            let mut f = self.failures.write().await;
                            *f = 0;
                            let _ = self.persist_state().await;
                        }
                    }
                    HealthStatus::Unhealthy => {
                        // If unhealthy, open the circuit for recovery_timeout
                        let until = Instant::now() + self.config.recovery_timeout;
                        let mut s = self.state.write().await;
                        *s = CircuitState::Open { until };
                        let _ = self.persist_state().await;
                    }
                    HealthStatus::Unknown => {
                        // leave state alone
                    }
                }
            } else {
                // Fallback simple behavior
                let allowed = self.is_allowed().await;
                if allowed {
                    let mut s = self.state.write().await;
                    if let CircuitState::HalfOpen = &*s {
                        *s = CircuitState::Closed;
                        let mut f = self.failures.write().await;
                        *f = 0;
                        let _ = self.persist_state().await;
                    }
                }
            }
        }
    }

    async fn persist_state_payload(&self, state_debug: String, failures: u32) -> anyhow::Result<()> {
        use serde_json::json;
        let payload = json!({"state": state_debug, "failures": failures, "version": self.config.failure_threshold});
        let mut ctx = crate::distributed::DistributedContext::new(format!("cb:{}", self.key));
        ctx.set("payload", serde_json::Value::String(payload.to_string()));
        let _ = self.store.save(&ctx, None).await;
        Ok(())
    }

    async fn persist_state(&self) -> anyhow::Result<()> {
        let state_debug = {
            let s = self.state.read().await;
            format!("{:?}", *s)
        };
        let failures = *self.failures.read().await;
        self.persist_state_payload(state_debug, failures).await
    }

    pub async fn current_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::InMemoryStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_circuit_transitions() {
        let store = Arc::new(InMemoryStore::new());
        let cb = Arc::new(CircuitBreaker::new("svc-a", Some(store), Some(CircuitConfig { failure_threshold: 2, recovery_timeout: Duration::from_millis(100), probe_interval: Duration::from_millis(50) })));

        assert!(cb.is_allowed().await);
        cb.record_failure().await;
        assert!(cb.is_allowed().await);
        cb.record_failure().await;
        assert!(!cb.is_allowed().await);

        // wait for recovery to elapse
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(cb.is_allowed().await); // moves to HalfOpen then Closed by probe loop not running here
        cb.record_success().await;
        assert_eq!(cb.current_state().await, CircuitState::Closed);
    }
}
