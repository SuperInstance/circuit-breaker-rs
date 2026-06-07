//! Configuration for circuit breaker behavior.
//!
//! Use [`Config::builder()`] to construct a configuration with custom parameters,
//! or [`Config::default()`] for sensible defaults.

/// Configuration for a [`CircuitBreaker`](crate::CircuitBreaker).
///
/// # Defaults
///
/// | Parameter            | Default |
/// |----------------------|---------|
/// | `failure_threshold`  | 5       |
/// | `recovery_timeout`   | 60s     |
/// | `half_open_max_probes` | 1     |
/// | `success_threshold`  | 3       |
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of consecutive failures required to trip the circuit (open).
    pub failure_threshold: u32,
    /// Seconds to wait before transitioning from open to half-open.
    pub recovery_timeout_secs: u64,
    /// Maximum number of probe requests allowed in half-open state.
    pub half_open_max_probes: u32,
    /// Number of consecutive successes in half-open to close the circuit.
    pub success_threshold: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_secs: 60,
            half_open_max_probes: 1,
            success_threshold: 3,
        }
    }
}

impl Config {
    /// Create a new [`ConfigBuilder`].
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder(Config::default())
    }

    /// Validate the configuration, returning `Ok(())` if valid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.failure_threshold == 0 {
            return Err(ConfigError::InvalidParameter {
                name: "failure_threshold",
                reason: "must be greater than 0",
            });
        }
        if self.recovery_timeout_secs == 0 {
            return Err(ConfigError::InvalidParameter {
                name: "recovery_timeout_secs",
                reason: "must be greater than 0",
            });
        }
        if self.half_open_max_probes == 0 {
            return Err(ConfigError::InvalidParameter {
                name: "half_open_max_probes",
                reason: "must be greater than 0",
            });
        }
        if self.success_threshold == 0 {
            return Err(ConfigError::InvalidParameter {
                name: "success_threshold",
                reason: "must be greater than 0",
            });
        }
        Ok(())
    }
}

/// Errors that can occur during configuration validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A parameter has an invalid value.
    InvalidParameter {
        /// Parameter name.
        name: &'static str,
        /// Why the value is invalid.
        reason: &'static str,
    },
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidParameter { name, reason } => {
                write!(f, "invalid parameter '{name}': {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Builder for [`Config`].
#[derive(Debug, Clone)]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    /// Set the failure threshold.
    pub fn failure_threshold(mut self, n: u32) -> Self {
        self.0.failure_threshold = n;
        self
    }

    /// Set the recovery timeout in seconds.
    pub fn recovery_timeout_secs(mut self, secs: u64) -> Self {
        self.0.recovery_timeout_secs = secs;
        self
    }

    /// Set the maximum number of probe requests in half-open state.
    pub fn half_open_max_probes(mut self, n: u32) -> Self {
        self.0.half_open_max_probes = n;
        self
    }

    /// Set the success threshold to close the circuit from half-open.
    pub fn success_threshold(mut self, n: u32) -> Self {
        self.0.success_threshold = n;
        self
    }

    /// Build the configuration, validating all parameters.
    pub fn build(self) -> Result<Config, ConfigError> {
        self.0.validate()?;
        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.failure_threshold, 5);
        assert_eq!(c.recovery_timeout_secs, 60);
        assert_eq!(c.half_open_max_probes, 1);
        assert_eq!(c.success_threshold, 3);
    }

    #[test]
    fn builder_custom() {
        let c = Config::builder()
            .failure_threshold(10)
            .recovery_timeout_secs(120)
            .half_open_max_probes(3)
            .success_threshold(5)
            .build()
            .unwrap();
        assert_eq!(c.failure_threshold, 10);
        assert_eq!(c.recovery_timeout_secs, 120);
        assert_eq!(c.half_open_max_probes, 3);
        assert_eq!(c.success_threshold, 5);
    }

    #[test]
    fn validate_zero_failure_threshold() {
        let c = Config {
            failure_threshold: 0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_recovery_timeout() {
        let c = Config {
            recovery_timeout_secs: 0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_half_open_probes() {
        let c = Config {
            half_open_max_probes: 0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_zero_success_threshold() {
        let c = Config {
            success_threshold: 0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn validate_valid_config() {
        assert!(Config::default().validate().is_ok());
    }

    #[test]
    fn config_error_display() {
        let err = ConfigError::InvalidParameter {
            name: "foo",
            reason: "bar",
        };
        assert_eq!(format!("{err}"), "invalid parameter 'foo': bar");
    }
}
