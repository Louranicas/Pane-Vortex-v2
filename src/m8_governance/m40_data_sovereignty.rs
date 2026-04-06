//! # M40: Data Sovereignty
//!
//! Spheres can enumerate, inspect, and request deletion of all data stored about them.
//! NA-P-13: "sphere can enumerate, correct, and delete its own data."
//!
//! ## Layer: L8 (Governance) — feature-gated: `governance`
//! ## Module: M40
//! ## Dependencies: L1 (M01)

use serde::{Deserialize, Serialize};

use crate::m1_foundation::m01_core_types::{now_secs, PaneId};

// ──────────────────────────────────────────────────────────────
// Data manifest
// ──────────────────────────────────────────────────────────────

/// Manifest of all data stored about a sphere.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataManifest {
    /// Sphere this manifest describes.
    pub sphere_id: PaneId,
    /// Number of memories on the sphere surface.
    pub memory_count: usize,
    /// Number of coupling connections involving this sphere.
    pub connection_count: usize,
    /// Whether this sphere has entries in RM (reasoning memory).
    pub has_rm_entries: bool,
    /// Whether this sphere has entries in POVM.
    pub has_povm_entries: bool,
    /// Number of bus events involving this sphere.
    pub bus_event_count: usize,
    /// Number of bus tasks submitted by or assigned to this sphere.
    pub bus_task_count: usize,
    /// Whether a ghost trace exists for this sphere.
    pub has_ghost_trace: bool,
    /// Total data points across all systems.
    pub total_data_points: usize,
}

impl DataManifest {
    /// Create an empty manifest for a sphere.
    #[must_use]
    pub fn new(sphere_id: PaneId) -> Self {
        Self {
            sphere_id,
            ..Self::default()
        }
    }

    /// Compute total data points.
    pub fn compute_total(&mut self) {
        self.total_data_points = self.memory_count
            + self.connection_count
            + usize::from(self.has_rm_entries)
            + usize::from(self.has_povm_entries)
            + self.bus_event_count
            + self.bus_task_count
            + usize::from(self.has_ghost_trace);
    }
}

// ──────────────────────────────────────────────────────────────
// Forget request
// ──────────────────────────────────────────────────────────────

/// Request to delete all data about a sphere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgetRequest {
    /// Sphere requesting data deletion.
    pub sphere_id: PaneId,
    /// Tick at which the request was made.
    pub requested_at_tick: u64,
    /// Unix timestamp.
    pub requested_at: f64,
    /// Whether to keep the ghost trace (memory of existence).
    pub preserve_ghost: bool,
    /// Processing status.
    pub status: ForgetStatus,
    /// Number of data points deleted so far.
    pub deleted_count: usize,
}

/// Status of a forget request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForgetStatus {
    /// Request submitted, awaiting processing.
    #[default]
    Pending,
    /// Processing in progress.
    InProgress,
    /// All data deleted.
    Completed,
    /// Request was rejected (e.g. sphere still active).
    Rejected,
}

impl ForgetRequest {
    /// Create a new forget request.
    #[must_use]
    pub fn new(sphere_id: PaneId, tick: u64, preserve_ghost: bool) -> Self {
        Self {
            sphere_id,
            requested_at_tick: tick,
            requested_at: now_secs(),
            preserve_ghost,
            status: ForgetStatus::Pending,
            deleted_count: 0,
        }
    }

    /// Mark the request as in progress.
    pub fn start_processing(&mut self) {
        if self.status == ForgetStatus::Pending {
            self.status = ForgetStatus::InProgress;
        }
    }

    /// Mark the request as completed.
    ///
    /// Only valid from `InProgress`. Calling `complete` on an already-`Completed`
    /// or `Rejected` request is a no-op.
    pub fn complete(&mut self, deleted: usize) {
        if self.status == ForgetStatus::InProgress {
            self.status = ForgetStatus::Completed;
            self.deleted_count = deleted;
        }
    }

    /// Reject the request.
    ///
    /// Valid from `Pending` or `InProgress` only. Calling `reject` on a `Completed`
    /// request is a no-op.
    pub fn reject(&mut self) {
        if matches!(self.status, ForgetStatus::Pending | ForgetStatus::InProgress) {
            self.status = ForgetStatus::Rejected;
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    // ── DataManifest ──

    #[test]
    fn manifest_new_empty() {
        let m = DataManifest::new(pid("test"));
        assert_eq!(m.sphere_id.as_str(), "test");
        assert_eq!(m.total_data_points, 0);
    }

    #[test]
    fn manifest_compute_total() {
        let mut m = DataManifest::new(pid("test"));
        m.memory_count = 10;
        m.connection_count = 5;
        m.has_rm_entries = true;
        m.bus_event_count = 3;
        m.compute_total();
        assert_eq!(m.total_data_points, 19); // 10+5+1+0+3+0+0
    }

    #[test]
    fn manifest_compute_total_all() {
        let mut m = DataManifest::new(pid("test"));
        m.memory_count = 10;
        m.connection_count = 5;
        m.has_rm_entries = true;
        m.has_povm_entries = true;
        m.bus_event_count = 3;
        m.bus_task_count = 2;
        m.has_ghost_trace = true;
        m.compute_total();
        assert_eq!(m.total_data_points, 23);
    }

    #[test]
    fn manifest_default_zero() {
        let m = DataManifest::default();
        assert_eq!(m.memory_count, 0);
        assert_eq!(m.total_data_points, 0);
    }

    #[test]
    fn manifest_serde_roundtrip() {
        let mut m = DataManifest::new(pid("test"));
        m.memory_count = 42;
        m.compute_total();
        let json = serde_json::to_string(&m).unwrap();
        let back: DataManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.memory_count, 42);
    }

    // ── ForgetRequest ──

    #[test]
    fn forget_request_new() {
        let r = ForgetRequest::new(pid("test"), 100, false);
        assert_eq!(r.sphere_id.as_str(), "test");
        assert_eq!(r.status, ForgetStatus::Pending);
        assert!(!r.preserve_ghost);
    }

    #[test]
    fn forget_request_preserve_ghost() {
        let r = ForgetRequest::new(pid("test"), 100, true);
        assert!(r.preserve_ghost);
    }

    #[test]
    fn forget_start_processing() {
        let mut r = ForgetRequest::new(pid("test"), 100, false);
        r.start_processing();
        assert_eq!(r.status, ForgetStatus::InProgress);
    }

    #[test]
    fn forget_complete() {
        let mut r = ForgetRequest::new(pid("test"), 100, false);
        r.start_processing();
        r.complete(42);
        assert_eq!(r.status, ForgetStatus::Completed);
        assert_eq!(r.deleted_count, 42);
    }

    #[test]
    fn forget_reject() {
        let mut r = ForgetRequest::new(pid("test"), 100, false);
        r.reject();
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    #[test]
    fn forget_start_only_from_pending() {
        let mut r = ForgetRequest::new(pid("test"), 100, false);
        r.reject();
        r.start_processing(); // Should not change Rejected → InProgress
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    #[test]
    fn forget_status_default() {
        assert_eq!(ForgetStatus::default(), ForgetStatus::Pending);
    }

    #[test]
    fn forget_serde_roundtrip() {
        let r = ForgetRequest::new(pid("test"), 50, true);
        let json = serde_json::to_string(&r).unwrap();
        let back: ForgetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sphere_id.as_str(), "test");
        assert!(back.preserve_ghost);
        assert_eq!(back.requested_at_tick, 50);
    }

    // ── Full lifecycle ──

    #[test]
    fn full_forget_lifecycle() {
        let mut r = ForgetRequest::new(pid("sphere-1"), 200, false);
        assert_eq!(r.status, ForgetStatus::Pending);

        r.start_processing();
        assert_eq!(r.status, ForgetStatus::InProgress);

        r.complete(15);
        assert_eq!(r.status, ForgetStatus::Completed);
        assert_eq!(r.deleted_count, 15);
    }

    // ── Edge: reject then try to process ──

    #[test]
    fn rejected_cannot_start() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.reject();
        r.start_processing();
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    // ── Manifest with no data ──

    #[test]
    fn manifest_no_data_zero_total() {
        let mut m = DataManifest::new(pid("empty"));
        m.compute_total();
        assert_eq!(m.total_data_points, 0);
    }

    // -- Manifest sphere_id --

    #[test]
    fn manifest_sphere_id_preserved() {
        let m = DataManifest::new(pid("my-sphere"));
        assert_eq!(m.sphere_id.as_str(), "my-sphere");
    }

    // -- FINDING-14: complete() is guarded --

    #[test]
    fn complete_noop_when_already_completed() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        r.complete(10);
        assert_eq!(r.deleted_count, 10);
        r.complete(999);
        assert_eq!(r.deleted_count, 10, "complete() must be idempotent after Completed");
        assert_eq!(r.status, ForgetStatus::Completed);
    }

    #[test]
    fn complete_noop_when_pending() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.complete(42);
        assert_eq!(r.status, ForgetStatus::Pending, "complete() must not apply to Pending");
        assert_eq!(r.deleted_count, 0);
    }

    // -- FINDING-15: reject() is guarded --

    #[test]
    fn reject_noop_when_completed() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        r.complete(10);
        assert_eq!(r.status, ForgetStatus::Completed);
        r.reject();
        assert_eq!(
            r.status,
            ForgetStatus::Completed,
            "reject() must not downgrade Completed -> Rejected"
        );
    }

    #[test]
    fn reject_allowed_from_pending() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.reject();
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    #[test]
    fn reject_allowed_from_in_progress() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        r.reject();
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    // ── Additional coverage ──

    #[test]
    fn start_processing_idempotent_from_in_progress() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        assert_eq!(r.status, ForgetStatus::InProgress);
        // Second call must be a no-op
        r.start_processing();
        assert_eq!(
            r.status,
            ForgetStatus::InProgress,
            "start_processing must be idempotent when already InProgress"
        );
    }

    #[test]
    fn reject_idempotent_when_already_rejected() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.reject();
        assert_eq!(r.status, ForgetStatus::Rejected);
        // Second call must be a no-op (Rejected is not Pending or InProgress)
        r.reject();
        assert_eq!(r.status, ForgetStatus::Rejected);
    }

    #[test]
    fn complete_noop_when_rejected() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.reject();
        r.complete(42);
        assert_eq!(
            r.status,
            ForgetStatus::Rejected,
            "complete() must not apply to Rejected status"
        );
        assert_eq!(r.deleted_count, 0);
    }

    #[test]
    fn manifest_only_ghost_trace_counts_one() {
        let mut m = DataManifest::new(pid("test"));
        m.has_ghost_trace = true;
        m.compute_total();
        assert_eq!(m.total_data_points, 1, "ghost trace alone counts as 1 data point");
    }

    #[test]
    fn manifest_only_rm_entries_counts_one() {
        let mut m = DataManifest::new(pid("test"));
        m.has_rm_entries = true;
        m.compute_total();
        assert_eq!(m.total_data_points, 1, "rm entries alone count as 1 data point");
    }

    #[test]
    fn manifest_only_povm_entries_counts_one() {
        let mut m = DataManifest::new(pid("test"));
        m.has_povm_entries = true;
        m.compute_total();
        assert_eq!(m.total_data_points, 1, "povm entries alone count as 1 data point");
    }

    #[test]
    fn forget_request_tick_preserved() {
        let r = ForgetRequest::new(pid("a"), 999, false);
        assert_eq!(r.requested_at_tick, 999);
    }

    #[test]
    fn forget_request_requested_at_is_positive() {
        let r = ForgetRequest::new(pid("a"), 1, false);
        assert!(
            r.requested_at > 0.0,
            "requested_at must be a positive unix timestamp"
        );
    }

    #[test]
    fn forget_status_in_progress_variant_distinct() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        assert_eq!(r.status, ForgetStatus::InProgress);
        assert_ne!(r.status, ForgetStatus::Pending);
        assert_ne!(r.status, ForgetStatus::Completed);
        assert_ne!(r.status, ForgetStatus::Rejected);
    }

    #[test]
    fn manifest_bus_event_and_task_counts() {
        let mut m = DataManifest::new(pid("test"));
        m.bus_event_count = 7;
        m.bus_task_count = 3;
        m.compute_total();
        assert_eq!(m.total_data_points, 10);
    }

    #[test]
    fn forget_serde_roundtrip_status_rejected() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.reject();
        let json = serde_json::to_string(&r).unwrap();
        let back: ForgetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, ForgetStatus::Rejected);
    }

    #[test]
    fn forget_serde_roundtrip_status_completed() {
        let mut r = ForgetRequest::new(pid("a"), 1, false);
        r.start_processing();
        r.complete(100);
        let json = serde_json::to_string(&r).unwrap();
        let back: ForgetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, ForgetStatus::Completed);
        assert_eq!(back.deleted_count, 100);
    }

    #[test]
    fn manifest_compute_total_is_additive_and_repeatable() {
        let mut m = DataManifest::new(pid("test"));
        m.memory_count = 5;
        m.compute_total();
        let first = m.total_data_points;
        m.compute_total(); // calling twice must produce the same result
        assert_eq!(m.total_data_points, first, "compute_total must be idempotent");
    }

    #[test]
    fn forget_status_default_is_pending() {
        // ForgetStatus derives Default which is Pending
        let s = ForgetStatus::default();
        assert_eq!(s, ForgetStatus::Pending);
    }
}
