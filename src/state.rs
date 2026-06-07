//! Circuit state definitions and transitions.
//!
//! The circuit breaker operates as a three-state finite state machine:
//! - **Closed** — normal operation; requests flow through.
//! - **Open** — tripped; all requests are rejected immediately.
//! - **HalfOpen** — probing; a limited number of requests are allowed to test recovery.

use core::fmt;

/// The three states of a circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CircuitState {
    /// Normal operation. Requests are allowed.
    Closed,
    /// Tripped. Requests are rejected.
    Open,
    /// Probing recovery. Limited requests allowed.
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

impl CircuitState {
    /// Returns `true` if requests should be allowed in this state.
    pub fn allows_requests(&self) -> bool {
        matches!(self, CircuitState::Closed | CircuitState::HalfOpen)
    }

    /// Returns `true` if the circuit is tripped (open or half-open).
    pub fn is_tripped(&self) -> bool {
        !matches!(self, CircuitState::Closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_closed() {
        assert_eq!(format!("{}", CircuitState::Closed), "Closed");
    }

    #[test]
    fn display_open() {
        assert_eq!(format!("{}", CircuitState::Open), "Open");
    }

    #[test]
    fn display_half_open() {
        assert_eq!(format!("{}", CircuitState::HalfOpen), "HalfOpen");
    }

    #[test]
    fn closed_allows_requests() {
        assert!(CircuitState::Closed.allows_requests());
    }

    #[test]
    fn open_rejects_requests() {
        assert!(!CircuitState::Open.allows_requests());
    }

    #[test]
    fn half_open_allows_requests() {
        assert!(CircuitState::HalfOpen.allows_requests());
    }

    #[test]
    fn closed_not_tripped() {
        assert!(!CircuitState::Closed.is_tripped());
    }

    #[test]
    fn open_is_tripped() {
        assert!(CircuitState::Open.is_tripped());
    }

    #[test]
    fn half_open_is_tripped() {
        assert!(CircuitState::HalfOpen.is_tripped());
    }

    #[test]
    fn state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_ne!(CircuitState::Closed, CircuitState::Open);
    }

    #[test]
    fn state_copy() {
        let s = CircuitState::Closed;
        let s2 = s;
        assert_eq!(s, s2);
    }
}
