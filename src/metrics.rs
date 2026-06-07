//! Metrics collection for circuit breaker events.
//!
//! [`Metrics`] tracks counts of successes, failures, rejected requests,
//! state transitions, and timing information useful for monitoring.


/// Collected metrics for a circuit breaker instance.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Total successful requests recorded.
    pub successes: u64,
    /// Total failed requests recorded.
    pub failures: u64,
    /// Total requests rejected because the circuit was open.
    pub rejected: u64,
    /// Number of times the circuit transitioned to Open.
    pub opened_count: u64,
    /// Number of times the circuit transitioned to Closed.
    pub closed_count: u64,
    /// Number of times the circuit transitioned to HalfOpen.
    pub half_opened_count: u64,
    /// Time of the last state transition (epoch seconds).
    pub last_transition_secs: Option<u64>,
    /// Duration spent in Open state total (seconds).
    pub total_open_duration_secs: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Create a new zeroed metrics instance.
    pub fn new() -> Self {
        Self {
            successes: 0,
            failures: 0,
            rejected: 0,
            opened_count: 0,
            closed_count: 0,
            half_opened_count: 0,
            last_transition_secs: None,
            total_open_duration_secs: 0,
        }
    }

    /// Record a successful request.
    pub fn record_success(&mut self) {
        self.successes += 1;
    }

    /// Record a failed request.
    pub fn record_failure(&mut self) {
        self.failures += 1;
    }

    /// Record a rejected request.
    pub fn record_rejected(&mut self) {
        self.rejected += 1;
    }

    /// Record a transition to a state at the given time (epoch seconds).
    pub fn record_transition(&mut self, time_secs: u64) {
        self.last_transition_secs = Some(time_secs);
    }

    /// Add open duration in seconds.
    pub fn add_open_duration(&mut self, secs: u64) {
        self.total_open_duration_secs += secs;
    }

    /// Increment the opened counter.
    pub fn increment_opened(&mut self) {
        self.opened_count += 1;
    }

    /// Increment the closed counter.
    pub fn increment_closed(&mut self) {
        self.closed_count += 1;
    }

    /// Increment the half-opened counter.
    pub fn increment_half_opened(&mut self) {
        self.half_opened_count += 1;
    }

    /// Total requests (successes + failures + rejected).
    pub fn total_requests(&self) -> u64 {
        self.successes + self.failures + self.rejected
    }

    /// Failure rate as a fraction (0.0 ..= 1.0). Returns 0.0 if no requests.
    pub fn failure_rate(&self) -> f64 {
        let total = self.successes + self.failures;
        if total == 0 {
            return 0.0;
        }
        self.failures as f64 / total as f64
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metrics_zeroed() {
        let m = Metrics::new();
        assert_eq!(m.successes, 0);
        assert_eq!(m.failures, 0);
        assert_eq!(m.rejected, 0);
        assert_eq!(m.opened_count, 0);
        assert_eq!(m.closed_count, 0);
        assert_eq!(m.half_opened_count, 0);
        assert_eq!(m.last_transition_secs, None);
        assert_eq!(m.total_open_duration_secs, 0);
    }

    #[test]
    fn record_success() {
        let mut m = Metrics::new();
        m.record_success();
        m.record_success();
        assert_eq!(m.successes, 2);
    }

    #[test]
    fn record_failure() {
        let mut m = Metrics::new();
        m.record_failure();
        assert_eq!(m.failures, 1);
    }

    #[test]
    fn record_rejected() {
        let mut m = Metrics::new();
        m.record_rejected();
        m.record_rejected();
        m.record_rejected();
        assert_eq!(m.rejected, 3);
    }

    #[test]
    fn total_requests() {
        let mut m = Metrics::new();
        m.record_success();
        m.record_failure();
        m.record_rejected();
        assert_eq!(m.total_requests(), 3);
    }

    #[test]
    fn failure_rate_no_requests() {
        let m = Metrics::new();
        assert_eq!(m.failure_rate(), 0.0);
    }

    #[test]
    fn failure_rate_calculation() {
        let mut m = Metrics::new();
        m.record_success();
        m.record_success();
        m.record_failure();
        m.record_failure();
        assert!((m.failure_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reset() {
        let mut m = Metrics::new();
        m.record_success();
        m.record_failure();
        m.reset();
        assert_eq!(m.successes, 0);
        assert_eq!(m.failures, 0);
    }

    #[test]
    fn transition_recording() {
        let mut m = Metrics::new();
        m.record_transition(100);
        assert_eq!(m.last_transition_secs, Some(100));
    }

    #[test]
    fn add_open_duration() {
        let mut m = Metrics::new();
        m.add_open_duration(30);
        m.add_open_duration(20);
        assert_eq!(m.total_open_duration_secs, 50);
    }

    #[test]
    fn increment_counters() {
        let mut m = Metrics::new();
        m.increment_opened();
        m.increment_closed();
        m.increment_half_opened();
        assert_eq!(m.opened_count, 1);
        assert_eq!(m.closed_count, 1);
        assert_eq!(m.half_opened_count, 1);
    }
}
