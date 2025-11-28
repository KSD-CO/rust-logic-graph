#[cfg(feature = "http-health")]
use crate::fault_tolerance::health::{HealthCheck, HealthStatus};
#[cfg(feature = "http-health")]
use async_trait::async_trait;

#[cfg(feature = "http-health")]
pub struct HttpHealthCheck {
    url: String,
    timeout_ms: u64,
}

#[cfg(feature = "http-health")]
impl HttpHealthCheck {
    pub fn new(url: impl Into<String>, timeout_ms: u64) -> Self {
        Self { url: url.into(), timeout_ms }
    }
}

#[cfg(feature = "http-health")]
#[async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn check(&self) -> HealthStatus {
        let client = reqwest::Client::new();
        let resp = client.get(&self.url).timeout(std::time::Duration::from_millis(self.timeout_ms)).send().await;
        match resp {
            Ok(r) => {
                if r.status().is_success() {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unhealthy
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        }
    }
}
