//! Health check integration for circuit breakers.
//!
//! Provides a [`HealthCheck`] trait and a built-in [`AlwaysHealthy`] check
//! for cases where downstream health monitoring is not needed.

/// Result of a health check probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// The downstream service is healthy.
    Healthy,
    /// The downstream service is unhealthy.
    Unhealthy,
    /// Health check could not be performed.
    Unknown,
}

/// A trait for health checking downstream services.
///
/// Implement this trait to integrate with your service discovery or
/// liveness probe mechanism.
pub trait HealthCheck {
    /// Perform a health check and return the status.
    fn check(&self) -> HealthStatus;
}

/// A health check that always returns [`HealthStatus::Healthy`].
///
/// Useful for testing or when health checking is handled externally.
#[derive(Debug, Clone, Default)]
pub struct AlwaysHealthy;

impl HealthCheck for AlwaysHealthy {
    fn check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// A health check that always returns [`HealthStatus::Unhealthy`].
///
/// Useful for testing failure scenarios.
#[derive(Debug, Clone, Default)]
pub struct AlwaysUnhealthy;

impl HealthCheck for AlwaysUnhealthy {
    fn check(&self) -> HealthStatus {
        HealthStatus::Unhealthy
    }
}

/// A health check that returns a configurable result.
#[derive(Debug, Clone)]
pub struct StaticHealthCheck {
    status: HealthStatus,
}

impl StaticHealthCheck {
    /// Create a new static health check with the given status.
    pub fn new(status: HealthStatus) -> Self {
        Self { status }
    }
}

impl HealthCheck for StaticHealthCheck {
    fn check(&self) -> HealthStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_healthy() {
        let check = AlwaysHealthy;
        assert_eq!(check.check(), HealthStatus::Healthy);
    }

    #[test]
    fn always_unhealthy() {
        let check = AlwaysUnhealthy;
        assert_eq!(check.check(), HealthStatus::Unhealthy);
    }

    #[test]
    fn static_health_check_healthy() {
        let check = StaticHealthCheck::new(HealthStatus::Healthy);
        assert_eq!(check.check(), HealthStatus::Healthy);
    }

    #[test]
    fn static_health_check_unhealthy() {
        let check = StaticHealthCheck::new(HealthStatus::Unhealthy);
        assert_eq!(check.check(), HealthStatus::Unhealthy);
    }

    #[test]
    fn static_health_check_unknown() {
        let check = StaticHealthCheck::new(HealthStatus::Unknown);
        assert_eq!(check.check(), HealthStatus::Unknown);
    }

    #[test]
    fn health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }
}
