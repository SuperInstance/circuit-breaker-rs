//! Core circuit breaker implementation.
//!
//! [`CircuitBreaker`] is the primary type. It tracks failures and manages
//! transitions between [`Closed`](crate::CircuitState::Closed),
//! [`Open`](crate::CircuitState::Open), and
//! [`HalfOpen`](crate::CircuitState::HalfOpen) states.

use crate::config::Config;
use crate::metrics::Metrics;
use crate::state::CircuitState;

/// A circuit breaker that protects against cascading failures.
///
/// # State Transitions
///
/// - **Closed → Open**: When consecutive failures reach [`Config::failure_threshold`].
/// - **Open → HalfOpen**: When [`Config::recovery_timeout_secs`] has elapsed.
/// - **HalfOpen → Closed**: When consecutive successes reach [`Config::success_threshold`].
/// - **HalfOpen → Open**: When any failure occurs during probing.
///
/// # Time Model
///
/// This implementation uses an injectable time source. In production you would
/// use a real clock; in tests you can supply deterministic timestamps via
/// [`CircuitBreaker::with_time`].
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: Config,
    state: CircuitState,
    consecutive_failures: u32,
    consecutive_successes: u32,
    half_open_probes_used: u32,
    opened_at_secs: Option<u64>,
    current_time_secs: u64,
    metrics: Metrics,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            half_open_probes_used: 0,
            opened_at_secs: None,
            current_time_secs: 0,
            metrics: Metrics::new(),
        }
    }

    /// Create a circuit breaker with a specific starting time (epoch seconds).
    ///
    /// Useful for deterministic testing of recovery timeouts.
    pub fn with_time(config: Config, current_time_secs: u64) -> Self {
        let mut cb = Self::new(config);
        cb.current_time_secs = current_time_secs;
        cb
    }

    /// Advance the internal clock by `secs` seconds.
    pub fn advance_time(&mut self, secs: u64) {
        self.current_time_secs += secs;
    }

    /// Set the internal clock to a specific epoch second.
    pub fn set_time(&mut self, secs: u64) {
        self.current_time_secs = secs;
    }

    /// Get the current circuit state.
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Get a reference to the collected metrics.
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Get a reference to the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Determine whether a request should be allowed.
    ///
    /// Returns `true` if the circuit is **Closed** or **HalfOpen** (with
    /// remaining probe budget). Returns `false` if **Open** or **HalfOpen**
    /// with no remaining probes.
    ///
    /// If the recovery timeout has elapsed while in the **Open** state, this
    /// method transitions to **HalfOpen** automatically.
    pub fn allow_request(&mut self) -> bool {
        self.maybe_transition_open_to_half_open();

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                self.metrics.record_rejected();
                false
            }
            CircuitState::HalfOpen => {
                if self.half_open_probes_used < self.config.half_open_max_probes {
                    self.half_open_probes_used += 1;
                    true
                } else {
                    self.metrics.record_rejected();
                    false
                }
            }
        }
    }

    /// Record a successful operation.
    ///
    /// In **Closed** state this resets the consecutive failure counter.
    /// In **HalfOpen** state this increments the success counter and may
    /// transition to **Closed**.
    pub fn record_success(&mut self) {
        self.metrics.record_success();

        match self.state {
            CircuitState::Closed => {
                self.consecutive_failures = 0;
                self.consecutive_successes += 1;
            }
            CircuitState::HalfOpen => {
                self.consecutive_successes += 1;
                if self.consecutive_successes >= self.config.success_threshold {
                    self.transition_to(CircuitState::Closed);
                }
            }
            CircuitState::Open => {
                // Success recorded but circuit is open — no state change.
            }
        }
    }

    /// Record a failed operation.
    ///
    /// In **Closed** state this increments the failure counter and may
    /// trip the circuit to **Open**.
    /// In **HalfOpen** state any failure immediately re-opens the circuit.
    pub fn record_failure(&mut self) {
        self.metrics.record_failure();

        match self.state {
            CircuitState::Closed => {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= self.config.failure_threshold {
                    self.transition_to(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                self.transition_to(CircuitState::Open);
            }
            CircuitState::Open => {
                // Already open — no state change.
            }
        }
    }

    /// Reset the circuit breaker to **Closed** state, clearing all counters.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.half_open_probes_used = 0;
        self.opened_at_secs = None;
    }

    /// Get the current consecutive failure count.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Get the current consecutive success count.
    pub fn consecutive_successes(&self) -> u32 {
        self.consecutive_successes
    }

    /// Get remaining half-open probe budget.
    pub fn remaining_probes(&self) -> u32 {
        self.config.half_open_max_probes.saturating_sub(self.half_open_probes_used)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn transition_to(&mut self, new_state: CircuitState) {
        if self.state == new_state {
            return;
        }

        let old = self.state;
        self.state = new_state;
        self.metrics.record_transition(self.current_time_secs);

        match new_state {
            CircuitState::Open => {
                self.opened_at_secs = Some(self.current_time_secs);
                self.metrics.increment_opened();
                self.consecutive_failures = 0;
                self.consecutive_successes = 0;
                self.half_open_probes_used = 0;
            }
            CircuitState::HalfOpen => {
                self.metrics.increment_half_opened();
                if let Some(opened_at) = self.opened_at_secs {
                    let duration = self.current_time_secs.saturating_sub(opened_at);
                    self.metrics.add_open_duration(duration);
                }
                self.consecutive_successes = 0;
                self.half_open_probes_used = 0;
            }
            CircuitState::Closed => {
                self.metrics.increment_closed();
                self.consecutive_failures = 0;
                self.consecutive_successes = 0;
                self.half_open_probes_used = 0;
                self.opened_at_secs = None;
            }
        }

        let _ = old; // Used for logging in real implementations
    }

    fn maybe_transition_open_to_half_open(&mut self) {
        if self.state != CircuitState::Open {
            return;
        }
        if let Some(opened_at) = self.opened_at_secs {
            let elapsed = self.current_time_secs.saturating_sub(opened_at);
            if elapsed >= self.config.recovery_timeout_secs {
                self.transition_to(CircuitState::HalfOpen);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_config() -> Config {
        Config::builder()
            .failure_threshold(3)
            .recovery_timeout_secs(10)
            .half_open_max_probes(2)
            .success_threshold(2)
            .build()
            .unwrap()
    }

    // ── Basic state tests ────────────────────────────────────────────────

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::new(test_config());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn allows_request_when_closed() {
        let mut cb = CircuitBreaker::new(test_config());
        assert!(cb.allow_request());
    }

    #[test]
    fn success_resets_failure_count() {
        let mut cb = CircuitBreaker::new(test_config());
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.consecutive_failures(), 2);
        cb.record_success();
        assert_eq!(cb.consecutive_failures(), 0);
    }

    // ── Closed → Open transition ─────────────────────────────────────────

    #[test]
    fn trips_on_threshold_failures() {
        let mut cb = CircuitBreaker::new(test_config());
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn rejects_requests_when_open() {
        let mut cb = CircuitBreaker::new(test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert!(!cb.allow_request());
    }

    #[test]
    fn increments_opened_count() {
        let mut cb = CircuitBreaker::new(test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.metrics().opened_count, 1);
    }

    // ── Open → HalfOpen transition ───────────────────────────────────────

    #[test]
    fn transitions_to_half_open_after_timeout() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);
        cb.advance_time(10);
        // allow_request triggers the transition check
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn does_not_transition_before_timeout() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(5);
        assert!(!cb.allow_request());
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn tracks_open_duration() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        cb.allow_request(); // triggers half-open transition
        assert_eq!(cb.metrics().total_open_duration_secs, 10);
    }

    // ── HalfOpen → Closed transition ─────────────────────────────────────

    #[test]
    fn half_open_successes_close_circuit() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        cb.allow_request();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_partial_successes_stay_half_open() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        cb.allow_request();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    // ── HalfOpen → Open transition ───────────────────────────────────────

    #[test]
    fn half_open_failure_reopens() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        cb.allow_request();
        cb.record_success();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn half_open_failure_increments_opened_count() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.metrics().opened_count, 1);
        cb.advance_time(10);
        cb.allow_request();
        cb.record_failure();
        assert_eq!(cb.metrics().opened_count, 2);
    }

    // ── Probe budget ─────────────────────────────────────────────────────

    #[test]
    fn half_open_probe_limit_enforced() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        assert!(cb.allow_request()); // probe 1
        assert!(cb.allow_request()); // probe 2
        assert!(!cb.allow_request()); // probe 3 — rejected
    }

    #[test]
    fn remaining_probes_decrements() {
        let mut cb = CircuitBreaker::with_time(test_config(), 100);
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        assert_eq!(cb.remaining_probes(), 2);
        cb.allow_request();
        assert_eq!(cb.remaining_probes(), 1);
    }

    // ── Reset ────────────────────────────────────────────────────────────

    #[test]
    fn reset_returns_to_closed() {
        let mut cb = CircuitBreaker::new(test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.consecutive_failures(), 0);
    }

    // ── Metrics ──────────────────────────────────────────────────────────

    #[test]
    fn metrics_success_count() {
        let mut cb = CircuitBreaker::new(test_config());
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.metrics().successes, 2);
    }

    #[test]
    fn metrics_failure_count() {
        let mut cb = CircuitBreaker::new(test_config());
        cb.record_failure();
        assert_eq!(cb.metrics().failures, 1);
    }

    #[test]
    fn metrics_rejected_count() {
        let mut cb = CircuitBreaker::new(test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.allow_request(); // rejected
        cb.allow_request(); // rejected
        assert_eq!(cb.metrics().rejected, 2);
    }

    // ── Full cycle ───────────────────────────────────────────────────────

    #[test]
    fn full_cycle_closed_open_half_open_closed() {
        let mut cb = CircuitBreaker::with_time(test_config(), 0);

        // Closed → Open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);

        // Open → HalfOpen
        cb.advance_time(10);
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // HalfOpen → Closed
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn multiple_cycles() {
        let mut cb = CircuitBreaker::with_time(test_config(), 0);

        // Cycle 1
        for _ in 0..3 {
            cb.record_failure();
        }
        cb.advance_time(10);
        cb.allow_request();
        cb.record_failure(); // re-open
        assert_eq!(cb.state(), CircuitState::Open);

        // Cycle 2
        cb.advance_time(10);
        cb.allow_request();
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
