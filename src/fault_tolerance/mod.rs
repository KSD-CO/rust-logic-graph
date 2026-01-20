pub mod circuit_breaker;
pub mod degradation;
pub mod failover;
pub mod health;

pub use circuit_breaker::{CircuitBreaker, CircuitConfig, CircuitState};
pub use degradation::{degrade_on_failure, FallbackHandler};
pub use failover::{FailoverManager, ServiceEndpoint};
pub use health::{HealthCheck, HealthMonitor, HealthStatus};
