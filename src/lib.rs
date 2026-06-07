//! # circuit-breaker-rs
//!
//! A pure-Rust implementation of the **Circuit Breaker** pattern for fault tolerance.
//!
//! The circuit breaker protects downstream services from cascading failures by tracking
//! consecutive failures and "tripping" (opening) the circuit when a threshold is reached.
//! After a recovery timeout, the circuit enters a **half-open** state where a limited
//! number of probe requests are allowed. Successful probes close the circuit; failures
//! re-open it.
//!
//! ## State Machine
//!
//! ```text
//! Closed ──(failure threshold reached)──▶ Open
//!    ▲                                       │
//!    │                                       │ (recovery timeout expires)
//!    │                                       ▼
//!    └───(probe succeeds)────────────── HalfOpen
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use circuit_breaker_rs::{CircuitBreaker, Config};
//!
//! let config = Config::builder()
//!     .failure_threshold(3)
//!     .recovery_timeout_secs(30)
//!     .half_open_max_probes(2)
//!     .build()
//!     .unwrap();
//!
//! let mut breaker = CircuitBreaker::new(config);
//!
//! // Allow requests in closed state
//! assert!(breaker.allow_request());
//!
//! // Record failures to trip the breaker
//! for _ in 0..3 {
//!     breaker.record_failure();
//! }
//!
//! // Circuit is now open — requests are rejected
//! assert!(!breaker.allow_request());
//! ```

// ── Modules ──────────────────────────────────────────────────────────────────

pub mod breaker;
pub mod config;
pub mod health;
pub mod metrics;
pub mod state;

// ── Re-exports ───────────────────────────────────────────────────────────────

pub use breaker::CircuitBreaker;
pub use config::Config;
pub use health::{HealthCheck, HealthStatus};
pub use metrics::Metrics;
pub use state::CircuitState;
