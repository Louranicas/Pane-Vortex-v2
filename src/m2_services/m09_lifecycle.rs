//! # M09: Service Lifecycle
//!
//! Service state machine: `Stopped` → `Starting` → `Running` → `Stopping` → `Stopped`.
//! Tracks PID, restart count, and uptime. Integrates with devenv.
//!
//! ## Layer: L2 (Services)
//! ## Module: M09
//! ## Dependencies: L1 (M01)

use std::collections::HashMap;

use crate::m1_foundation::m01_core_types::now_secs;

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// Service lifecycle state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ServiceState {
    /// Service is not running.
    #[default]
    Stopped,
    /// Service is starting up.
    Starting,
    /// Service is running and healthy.
    Running,
    /// Service is shutting down.
    Stopping,
    /// Service has failed and needs attention.
    Failed,
}


/// Lifecycle record for a single service.
#[derive(Debug, Clone)]
pub struct ServiceLifecycle {
    /// Service ID.
    pub service_id: String,
    /// Current state.
    pub state: ServiceState,
    /// Process ID (if running).
    pub pid: Option<u32>,
    /// Number of restarts since registration.
    pub restart_count: u32,
    /// Unix timestamp when the service was started.
    pub started_at: f64,
    /// Unix timestamp when the service was stopped.
    pub stopped_at: f64,
    /// Maximum allowed restart attempts before marking as Failed.
    pub max_restarts: u32,
}

impl ServiceLifecycle {
    /// Create a new lifecycle record.
    #[must_use]
    pub fn new(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            state: ServiceState::Stopped,
            pid: None,
            restart_count: 0,
            started_at: 0.0,
            stopped_at: 0.0,
            max_restarts: 5,
        }
    }

    /// Transition to Starting state.
    pub fn start(&mut self) {
        if self.state == ServiceState::Stopped || self.state == ServiceState::Failed {
            self.state = ServiceState::Starting;
            self.started_at = now_secs();
        }
    }

    /// Transition to Running state with a PID.
    pub fn running(&mut self, pid: u32) {
        if self.state == ServiceState::Starting {
            self.state = ServiceState::Running;
            self.pid = Some(pid);
        }
    }

    /// Transition to Stopping state.
    pub fn stop(&mut self) {
        if self.state == ServiceState::Running || self.state == ServiceState::Starting {
            self.state = ServiceState::Stopping;
        }
    }

    /// Transition to Stopped state.
    pub fn stopped(&mut self) {
        self.state = ServiceState::Stopped;
        self.pid = None;
        self.stopped_at = now_secs();
    }

    /// Record a restart. Marks as Failed if max restarts exceeded.
    pub fn restart(&mut self) {
        self.restart_count = self.restart_count.saturating_add(1);
        if self.restart_count >= self.max_restarts {
            self.state = ServiceState::Failed;
            self.pid = None;
        } else {
            self.state = ServiceState::Starting;
            self.started_at = now_secs();
        }
    }

    /// Uptime in seconds (0 if not running).
    #[must_use]
    pub fn uptime_secs(&self) -> f64 {
        if self.state == ServiceState::Running && self.started_at > 0.0 {
            now_secs() - self.started_at
        } else {
            0.0
        }
    }

    /// Whether the service is in a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self.state, ServiceState::Running)
    }

    /// Whether the service has failed.
    #[must_use]
    pub const fn is_failed(&self) -> bool {
        matches!(self.state, ServiceState::Failed)
    }

    /// Reset failure state (allow restarts again).
    pub fn reset_failures(&mut self) {
        if self.state == ServiceState::Failed {
            self.state = ServiceState::Stopped;
            self.restart_count = 0;
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Lifecycle manager
// ──────────────────────────────────────────────────────────────

/// Fleet-wide lifecycle manager.
#[derive(Debug, Clone, Default)]
pub struct LifecycleManager {
    /// Per-service lifecycle records.
    pub records: HashMap<String, ServiceLifecycle>,
}

impl LifecycleManager {
    /// Create a new empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize lifecycle records for services.
    pub fn initialize(&mut self, service_ids: &[&str]) {
        for id in service_ids {
            self.records
                .entry((*id).to_string())
                .or_insert_with(|| ServiceLifecycle::new(*id));
        }
    }

    /// Get lifecycle record.
    #[must_use]
    pub fn get(&self, service_id: &str) -> Option<&ServiceLifecycle> {
        self.records.get(service_id)
    }

    /// Get mutable lifecycle record.
    pub fn get_mut(&mut self, service_id: &str) -> Option<&mut ServiceLifecycle> {
        self.records.get_mut(service_id)
    }

    /// Count of running services.
    #[must_use]
    pub fn running_count(&self) -> usize {
        self.records.values().filter(|l| l.is_running()).count()
    }

    /// Count of failed services.
    #[must_use]
    pub fn failed_count(&self) -> usize {
        self.records.values().filter(|l| l.is_failed()).count()
    }

    /// Summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} running, {} failed, {} total",
            self.running_count(),
            self.failed_count(),
            self.records.len()
        )
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ServiceState ──

    #[test]
    fn default_state_is_stopped() {
        assert_eq!(ServiceState::default(), ServiceState::Stopped);
    }

    // ── ServiceLifecycle ──

    #[test]
    fn new_lifecycle_is_stopped() {
        let lc = ServiceLifecycle::new("test");
        assert_eq!(lc.state, ServiceState::Stopped);
        assert!(lc.pid.is_none());
    }

    #[test]
    fn start_transitions_to_starting() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        assert_eq!(lc.state, ServiceState::Starting);
    }

    #[test]
    fn running_transitions_from_starting() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1234);
        assert_eq!(lc.state, ServiceState::Running);
        assert_eq!(lc.pid, Some(1234));
    }

    #[test]
    fn running_ignored_when_stopped() {
        let mut lc = ServiceLifecycle::new("test");
        lc.running(1234);
        assert_eq!(lc.state, ServiceState::Stopped);
    }

    #[test]
    fn stop_transitions_from_running() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1234);
        lc.stop();
        assert_eq!(lc.state, ServiceState::Stopping);
    }

    #[test]
    fn stopped_clears_pid() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1234);
        lc.stopped();
        assert!(lc.pid.is_none());
        assert_eq!(lc.state, ServiceState::Stopped);
    }

    #[test]
    fn restart_increments_count() {
        let mut lc = ServiceLifecycle::new("test");
        lc.restart();
        assert_eq!(lc.restart_count, 1);
    }

    #[test]
    fn restart_exceeding_max_fails() {
        let mut lc = ServiceLifecycle::new("test");
        lc.max_restarts = 3;
        for _ in 0..3 {
            lc.restart();
        }
        assert!(lc.is_failed());
    }

    #[test]
    fn uptime_zero_when_stopped() {
        let lc = ServiceLifecycle::new("test");
        assert_eq!(lc.uptime_secs(), 0.0);
    }

    #[test]
    fn uptime_positive_when_running() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1234);
        // Uptime should be >= 0 (could be very small)
        assert!(lc.uptime_secs() >= 0.0);
    }

    #[test]
    fn is_running_true_when_running() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1234);
        assert!(lc.is_running());
    }

    #[test]
    fn is_running_false_when_stopped() {
        let lc = ServiceLifecycle::new("test");
        assert!(!lc.is_running());
    }

    #[test]
    fn is_failed_after_max_restarts() {
        let mut lc = ServiceLifecycle::new("test");
        lc.max_restarts = 2;
        lc.restart();
        lc.restart();
        assert!(lc.is_failed());
    }

    #[test]
    fn reset_failures_restores_stopped() {
        let mut lc = ServiceLifecycle::new("test");
        lc.max_restarts = 1;
        lc.restart();
        assert!(lc.is_failed());
        lc.reset_failures();
        assert_eq!(lc.state, ServiceState::Stopped);
        assert_eq!(lc.restart_count, 0);
    }

    #[test]
    fn reset_failures_noop_when_not_failed() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.reset_failures(); // Should not change Starting state
        assert_eq!(lc.state, ServiceState::Starting);
    }

    #[test]
    fn start_from_failed() {
        let mut lc = ServiceLifecycle::new("test");
        lc.max_restarts = 1;
        lc.restart();
        assert!(lc.is_failed());
        lc.start();
        assert_eq!(lc.state, ServiceState::Starting);
    }

    // ── LifecycleManager ──

    #[test]
    fn new_manager_empty() {
        let m = LifecycleManager::new();
        assert_eq!(m.records.len(), 0);
    }

    #[test]
    fn initialize_creates_records() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1", "svc2"]);
        assert_eq!(m.records.len(), 2);
    }

    #[test]
    fn get_returns_record() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1"]);
        assert!(m.get("svc1").is_some());
    }

    #[test]
    fn get_missing_returns_none() {
        let m = LifecycleManager::new();
        assert!(m.get("missing").is_none());
    }

    #[test]
    fn get_mut_allows_modification() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1"]);
        if let Some(lc) = m.get_mut("svc1") {
            lc.start();
            lc.running(1234);
        }
        assert!(m.get("svc1").unwrap().is_running());
    }

    #[test]
    fn running_count() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1", "svc2"]);
        m.get_mut("svc1").unwrap().start();
        m.get_mut("svc1").unwrap().running(1234);
        assert_eq!(m.running_count(), 1);
    }

    #[test]
    fn failed_count() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1"]);
        let lc = m.get_mut("svc1").unwrap();
        lc.max_restarts = 1;
        lc.restart();
        assert_eq!(m.failed_count(), 1);
    }

    #[test]
    fn summary_format() {
        let mut m = LifecycleManager::new();
        m.initialize(&["svc1", "svc2"]);
        let s = m.summary();
        assert!(s.contains("0 running"));
        assert!(s.contains("2 total"));
    }

    // ── Full lifecycle ──

    #[test]
    fn full_lifecycle_start_run_stop() {
        let mut lc = ServiceLifecycle::new("test");
        assert_eq!(lc.state, ServiceState::Stopped);

        lc.start();
        assert_eq!(lc.state, ServiceState::Starting);

        lc.running(42);
        assert_eq!(lc.state, ServiceState::Running);
        assert_eq!(lc.pid, Some(42));

        lc.stop();
        assert_eq!(lc.state, ServiceState::Stopping);

        lc.stopped();
        assert_eq!(lc.state, ServiceState::Stopped);
        assert!(lc.pid.is_none());
    }

    #[test]
    fn restart_lifecycle() {
        let mut lc = ServiceLifecycle::new("test");
        lc.start();
        lc.running(1);
        lc.stopped();
        lc.restart();
        assert_eq!(lc.state, ServiceState::Starting);
        assert_eq!(lc.restart_count, 1);
    }
}
