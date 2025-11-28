pub mod circuit_breaker;
pub mod health;
pub mod failover;
pub mod degradation;

pub use circuit_breaker::{CircuitBreaker, CircuitState, CircuitConfig};
pub use health::{HealthMonitor, HealthStatus, HealthCheck};
pub use failover::{FailoverManager, ServiceEndpoint};
pub use degradation::{degrade_on_failure, FallbackHandler};
