//! # M08: Health Monitor
//!
//! Health status tracking for ULTRAPLATE services.
//! Tracks last check time, consecutive failures, and circuit breaker state.
//!
//! ## Layer: L2 (Services)
//! ## Module: M08
//! ## Dependencies: L1 (M01, M02), M07
//!
//! ## Audit Fixes (Agent-1, Session 089)
//! - BUG-06: `should_check` had a redundant `if elapsed >= X { true } else { false }`
//!   pattern — simplified to a direct boolean expression.

use std::collections::HashMap;

use crate::m1_foundation::m01_core_types::now_secs;

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// Health status of a single service.
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    /// Service ID.
    pub service_id: String,
    /// Whether the last health check succeeded.
    pub healthy: bool,
    /// HTTP status code from last check (0 if unreachable).
    pub last_status: u16,
    /// Unix timestamp of last successful check.
    pub last_success: f64,
    /// Unix timestamp of last check attempt.
    pub last_checked: f64,
    /// Consecutive failure count.
    pub consecutive_failures: u32,
    /// Circuit breaker state.
    pub circuit_state: CircuitState,
}

/// Circuit breaker state for health monitoring.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation — health checks proceeding.
    #[default]
    Closed,
    /// Too many failures — health checks temporarily suspended.
    Open,
    /// Testing recovery — one check allowed through.
    HalfOpen,
}


/// Threshold for opening the circuit breaker.
const CIRCUIT_OPEN_THRESHOLD: u32 = 5;
/// Seconds before trying half-open after circuit opens.
const CIRCUIT_RECOVERY_SECS: f64 = 60.0;

impl ServiceHealth {
    /// Create a new health record for a service.
    #[must_use]
    pub fn new(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            healthy: false,
            last_status: 0,
            last_success: 0.0,
            last_checked: 0.0,
            consecutive_failures: 0,
            circuit_state: CircuitState::Closed,
        }
    }

    /// Record a successful health check.
    pub fn record_success(&mut self, status: u16) {
        let now = now_secs();
        self.healthy = true;
        self.last_status = status;
        self.last_success = now;
        self.last_checked = now;
        self.consecutive_failures = 0;
        self.circuit_state = CircuitState::Closed;
    }

    /// Record a failed health check.
    ///
    /// If the circuit is in `HalfOpen` (recovery probe), a single failure
    /// immediately returns it to `Open` — the recovery test failed. Otherwise
    /// the circuit opens once `consecutive_failures` reaches the threshold.
    pub fn record_failure(&mut self, status: u16) {
        let now = now_secs();
        self.healthy = false;
        self.last_status = status;
        self.last_checked = now;
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);

        if self.circuit_state == CircuitState::HalfOpen {
            // Recovery probe failed — immediately re-open regardless of count.
            self.circuit_state = CircuitState::Open;
        } else if self.consecutive_failures >= CIRCUIT_OPEN_THRESHOLD {
            self.circuit_state = CircuitState::Open;
        }
    }

    /// Whether this service should be checked (respects circuit breaker).
    ///
    /// When the circuit is open, allows a single probe through after
    /// `CIRCUIT_RECOVERY_SECS` have elapsed (half-open recovery test).
    #[must_use]
    pub fn should_check(&self) -> bool {
        match self.circuit_state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let elapsed = now_secs() - self.last_checked;
                elapsed >= CIRCUIT_RECOVERY_SECS
            }
        }
    }

    /// Transition to half-open state (for recovery testing).
    pub fn try_half_open(&mut self) {
        if self.circuit_state == CircuitState::Open {
            self.circuit_state = CircuitState::HalfOpen;
        }
    }

    /// Seconds since last successful check.
    #[must_use]
    pub fn staleness_secs(&self) -> f64 {
        if self.last_success <= 0.0 {
            return f64::INFINITY;
        }
        now_secs() - self.last_success
    }

    /// Whether the service is considered stale (no success in given seconds).
    #[must_use]
    pub fn is_stale(&self, threshold_secs: f64) -> bool {
        self.staleness_secs() > threshold_secs
    }
}

// ──────────────────────────────────────────────────────────────
// Health monitor
// ──────────────────────────────────────────────────────────────

/// Fleet-wide health monitor.
#[derive(Debug, Clone, Default)]
pub struct HealthMonitor {
    /// Per-service health records.
    pub records: HashMap<String, ServiceHealth>,
}

impl HealthMonitor {
    /// Create a new empty monitor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize health records for a set of service IDs.
    pub fn initialize(&mut self, service_ids: &[&str]) {
        for id in service_ids {
            self.records
                .entry((*id).to_string())
                .or_insert_with(|| ServiceHealth::new(*id));
        }
    }

    /// Record a health check result.
    pub fn record(&mut self, service_id: &str, success: bool, status: u16) {
        let health = self
            .records
            .entry(service_id.to_string())
            .or_insert_with(|| ServiceHealth::new(service_id));

        if success {
            health.record_success(status);
        } else {
            health.record_failure(status);
        }
    }

    /// Get health status for a service.
    #[must_use]
    pub fn get(&self, service_id: &str) -> Option<&ServiceHealth> {
        self.records.get(service_id)
    }

    /// Count of healthy services.
    #[must_use]
    pub fn healthy_count(&self) -> usize {
        self.records.values().filter(|h| h.healthy).count()
    }

    /// Count of unhealthy services.
    #[must_use]
    pub fn unhealthy_count(&self) -> usize {
        self.records.values().filter(|h| !h.healthy).count()
    }

    /// Services with open circuit breakers.
    #[must_use]
    pub fn open_circuits(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, h)| h.circuit_state == CircuitState::Open)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Summary string for logging.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}/{} healthy, {} open circuits",
            self.healthy_count(),
            self.records.len(),
            self.open_circuits().len()
        )
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ServiceHealth ──

    #[test]
    fn new_service_health_not_healthy() {
        let h = ServiceHealth::new("test");
        assert!(!h.healthy);
    }

    #[test]
    fn record_success_makes_healthy() {
        let mut h = ServiceHealth::new("test");
        h.record_success(200);
        assert!(h.healthy);
        assert_eq!(h.last_status, 200);
    }

    #[test]
    fn record_failure_makes_unhealthy() {
        let mut h = ServiceHealth::new("test");
        h.record_success(200);
        h.record_failure(503);
        assert!(!h.healthy);
    }

    #[test]
    fn consecutive_failures_increment() {
        let mut h = ServiceHealth::new("test");
        h.record_failure(0);
        h.record_failure(0);
        h.record_failure(0);
        assert_eq!(h.consecutive_failures, 3);
    }

    #[test]
    fn success_resets_failures() {
        let mut h = ServiceHealth::new("test");
        h.record_failure(0);
        h.record_failure(0);
        h.record_success(200);
        assert_eq!(h.consecutive_failures, 0);
    }

    #[test]
    fn circuit_opens_after_threshold() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        assert_eq!(h.circuit_state, CircuitState::Open);
    }

    #[test]
    fn circuit_closes_on_success() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.record_success(200);
        assert_eq!(h.circuit_state, CircuitState::Closed);
    }

    #[test]
    fn half_open_from_open() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.try_half_open();
        assert_eq!(h.circuit_state, CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_from_closed_noop() {
        let mut h = ServiceHealth::new("test");
        h.try_half_open();
        assert_eq!(h.circuit_state, CircuitState::Closed);
    }

    #[test]
    fn should_check_when_closed() {
        let h = ServiceHealth::new("test");
        assert!(h.should_check());
    }

    #[test]
    fn should_check_when_half_open() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.try_half_open();
        assert!(h.should_check());
    }

    #[test]
    fn staleness_never_checked() {
        let h = ServiceHealth::new("test");
        assert!(h.staleness_secs().is_infinite());
    }

    #[test]
    fn is_stale_when_never_checked() {
        let h = ServiceHealth::new("test");
        assert!(h.is_stale(60.0));
    }

    #[test]
    fn not_stale_after_success() {
        let mut h = ServiceHealth::new("test");
        h.record_success(200);
        assert!(!h.is_stale(60.0));
    }

    // ── HealthMonitor ──

    #[test]
    fn new_monitor_empty() {
        let m = HealthMonitor::new();
        assert_eq!(m.records.len(), 0);
    }

    #[test]
    fn initialize_creates_records() {
        let mut m = HealthMonitor::new();
        m.initialize(&["svc1", "svc2"]);
        assert_eq!(m.records.len(), 2);
    }

    #[test]
    fn record_success() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        assert!(m.get("svc1").unwrap().healthy);
    }

    #[test]
    fn record_failure() {
        let mut m = HealthMonitor::new();
        m.record("svc1", false, 503);
        assert!(!m.get("svc1").unwrap().healthy);
    }

    #[test]
    fn healthy_count() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        m.record("svc2", false, 503);
        assert_eq!(m.healthy_count(), 1);
    }

    #[test]
    fn unhealthy_count() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        m.record("svc2", false, 503);
        assert_eq!(m.unhealthy_count(), 1);
    }

    #[test]
    fn open_circuits_empty_when_healthy() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        assert!(m.open_circuits().is_empty());
    }

    #[test]
    fn open_circuits_after_failures() {
        let mut m = HealthMonitor::new();
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            m.record("svc1", false, 0);
        }
        assert_eq!(m.open_circuits().len(), 1);
    }

    #[test]
    fn summary_format() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        m.record("svc2", false, 503);
        let s = m.summary();
        assert!(s.contains("1/2 healthy"));
    }

    #[test]
    fn get_missing_returns_none() {
        let m = HealthMonitor::new();
        assert!(m.get("missing").is_none());
    }

    // ── CircuitState ──

    #[test]
    fn circuit_state_default_is_closed() {
        assert_eq!(CircuitState::default(), CircuitState::Closed);
    }

    // ── Integration ──

    #[test]
    fn monitor_full_lifecycle() {
        let mut m = HealthMonitor::new();
        m.initialize(&["svc1"]);

        // Initially unhealthy (never checked)
        assert!(!m.get("svc1").unwrap().healthy);

        // Becomes healthy
        m.record("svc1", true, 200);
        assert!(m.get("svc1").unwrap().healthy);

        // Fails multiple times
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            m.record("svc1", false, 0);
        }
        assert_eq!(
            m.get("svc1").unwrap().circuit_state,
            CircuitState::Open
        );

        // Recovers
        m.record("svc1", true, 200);
        assert_eq!(
            m.get("svc1").unwrap().circuit_state,
            CircuitState::Closed
        );
    }

    // -- FINDING-4: HalfOpen -> Open on single failure --

    #[test]
    fn half_open_failure_immediately_reopens_circuit() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        assert_eq!(h.circuit_state, CircuitState::Open);
        h.try_half_open();
        assert_eq!(h.circuit_state, CircuitState::HalfOpen);
        // Recovery probe fails -- circuit must immediately re-open
        h.record_failure(503);
        assert_eq!(
            h.circuit_state,
            CircuitState::Open,
            "single failure in HalfOpen must re-open circuit immediately"
        );
    }

    #[test]
    fn half_open_success_closes_circuit() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.try_half_open();
        h.record_success(200);
        assert_eq!(
            h.circuit_state,
            CircuitState::Closed,
            "success in HalfOpen must close circuit"
        );
    }

    #[test]
    fn half_open_failure_does_not_require_threshold_to_reopen() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.try_half_open();
        // Simulate consecutive_failures being 0 as if a prior success reset it
        h.consecutive_failures = 0;
        h.circuit_state = CircuitState::HalfOpen;
        h.record_failure(503);
        assert_eq!(
            h.circuit_state,
            CircuitState::Open,
            "HalfOpen must not linger after probe failure regardless of count"
        );
    }

    // ── Additional coverage: open circuit within recovery window ──

    #[test]
    fn should_not_check_when_open_and_recently_failed() {
        let mut h = ServiceHealth::new("test");
        // Open the circuit
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        assert_eq!(h.circuit_state, CircuitState::Open);
        // last_checked was just set by record_failure — well within 60s window
        assert!(
            !h.should_check(),
            "open circuit with recent failure should not allow check within recovery window"
        );
    }

    #[test]
    fn staleness_secs_after_success_is_finite() {
        let mut h = ServiceHealth::new("test");
        h.record_success(200);
        let staleness = h.staleness_secs();
        assert!(
            staleness.is_finite(),
            "staleness must be finite after a successful check"
        );
        assert!(staleness >= 0.0, "staleness must be non-negative");
    }

    #[test]
    fn try_half_open_noop_when_already_half_open() {
        let mut h = ServiceHealth::new("test");
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            h.record_failure(0);
        }
        h.try_half_open();
        assert_eq!(h.circuit_state, CircuitState::HalfOpen);
        // Calling again should not change state (no-op on non-Open)
        h.try_half_open();
        assert_eq!(
            h.circuit_state,
            CircuitState::HalfOpen,
            "try_half_open must be a no-op when already HalfOpen"
        );
    }

    // ── Additional HealthMonitor coverage ──

    #[test]
    fn initialize_idempotent_does_not_duplicate() {
        let mut m = HealthMonitor::new();
        m.initialize(&["svc1"]);
        m.initialize(&["svc1"]); // second call must not reset existing state
        assert_eq!(m.records.len(), 1, "duplicate initialize must not create duplicate entries");
    }

    #[test]
    fn initialize_preserves_existing_state() {
        let mut m = HealthMonitor::new();
        m.initialize(&["svc1"]);
        m.record("svc1", true, 200); // mark healthy
        m.initialize(&["svc1"]); // re-initialize must not reset
        assert!(
            m.get("svc1").unwrap().healthy,
            "re-initializing must not overwrite existing record"
        );
    }

    #[test]
    fn record_creates_entry_for_unknown_service() {
        let mut m = HealthMonitor::new();
        // No prior initialize — record must create the entry
        m.record("new-svc", true, 200);
        assert!(m.get("new-svc").is_some(), "record must create entry for unknown service");
        assert!(m.get("new-svc").unwrap().healthy);
    }

    #[test]
    fn unhealthy_count_zero_when_all_healthy() {
        let mut m = HealthMonitor::new();
        m.record("svc1", true, 200);
        m.record("svc2", true, 200);
        assert_eq!(m.unhealthy_count(), 0);
    }

    #[test]
    fn healthy_count_zero_when_all_unhealthy() {
        let mut m = HealthMonitor::new();
        m.record("svc1", false, 503);
        m.record("svc2", false, 503);
        assert_eq!(m.healthy_count(), 0);
    }

    #[test]
    fn summary_contains_open_circuit_count() {
        let mut m = HealthMonitor::new();
        for _ in 0..CIRCUIT_OPEN_THRESHOLD {
            m.record("svc1", false, 0);
        }
        let s = m.summary();
        assert!(
            s.contains("1 open circuits"),
            "summary must report open circuit count, got: {s}"
        );
    }

    #[test]
    fn record_failure_status_code_stored() {
        let mut h = ServiceHealth::new("test");
        h.record_failure(503);
        assert_eq!(h.last_status, 503);
    }

    #[test]
    fn record_success_updates_last_checked() {
        let mut h = ServiceHealth::new("test");
        assert_eq!(h.last_checked, 0.0);
        h.record_success(200);
        assert!(h.last_checked > 0.0, "last_checked must be updated on success");
        assert!(h.last_success > 0.0, "last_success must be updated on success");
    }

    #[test]
    fn consecutive_failures_below_threshold_stays_closed() {
        let mut h = ServiceHealth::new("test");
        // One less than threshold
        for _ in 0..(CIRCUIT_OPEN_THRESHOLD - 1) {
            h.record_failure(0);
        }
        assert_eq!(
            h.circuit_state,
            CircuitState::Closed,
            "circuit must stay closed until threshold is reached"
        );
    }
}
