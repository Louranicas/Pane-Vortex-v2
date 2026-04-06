//! # M32: Executor
//!
//! Thin Zellij dispatch. 5-step: identify target -> navigate tab -> verify pane ->
//! send command -> return. Pane mapping from fleet inventory.
//!
//! ## Layer: L7 | Module: M32 | Dependencies: L1, L3 (M11, M12)

use std::collections::HashMap;

use crate::m1_foundation::{
    m01_core_types::{now_secs, PaneId, PaneStatus},
    m02_error_handling::{PvError, PvResult},
};
use crate::m3_field::m11_sphere::PaneSphere;
use super::m30_bus_types::{BusTask, TaskTarget};

// ──────────────────────────────────────────────────────────────
// ExecutorResult
// ──────────────────────────────────────────────────────────────

/// Result of a task dispatch attempt.
#[derive(Debug, Clone)]
pub struct ExecutorResult {
    /// The sphere that was selected for execution.
    pub target_sphere: PaneId,
    /// Whether the dispatch was successful.
    pub success: bool,
    /// Execution/dispatch time in milliseconds.
    pub execution_ms: f64,
    /// Human-readable reason for success or failure.
    pub reason: String,
}

impl ExecutorResult {
    /// Create a successful result.
    #[must_use]
    #[allow(dead_code)] // Result constructors for dispatch completions; wired in future L7 pass
    pub(crate) fn success(target: PaneId, execution_ms: f64) -> Self {
        Self {
            target_sphere: target,
            success: true,
            execution_ms,
            reason: "dispatched".into(),
        }
    }

    /// Create a failed result.
    #[must_use]
    #[allow(dead_code)] // Paired with success(); both used in dispatch result handling
    pub(crate) const fn failure(target: PaneId, reason: String) -> Self {
        Self {
            target_sphere: target,
            success: false,
            execution_ms: 0.0,
            reason,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Executor
// ──────────────────────────────────────────────────────────────

/// Task executor — selects target spheres and dispatches tasks.
///
/// The executor does not directly interact with Zellij; it selects the best
/// target sphere based on the task's `TaskTarget` and the current fleet state.
/// Actual Zellij dispatch (tab navigation, pane focus) is done by the binary.
#[derive(Debug)]
pub struct Executor {
    /// Dispatch queue (task IDs waiting for dispatch).
    dispatch_queue: Vec<PaneId>,
    /// Maximum queue depth.
    max_queue: usize,
}

impl Executor {
    /// Create a new executor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dispatch_queue: Vec::new(),
            max_queue: 100,
        }
    }

    /// Create an executor with a custom queue depth.
    #[must_use]
    #[allow(dead_code)] // Test helper + configuration variant
    pub(crate) fn with_max_queue(max_queue: usize) -> Self {
        Self {
            dispatch_queue: Vec::new(),
            max_queue: max_queue.max(1),
        }
    }

    /// Select a target sphere for a task based on `TaskTarget`.
    ///
    /// # Errors
    /// Returns `PvError::SphereNotFound` if no suitable target is found.
    #[allow(clippy::implicit_hasher)]
    pub fn dispatch(
        &mut self,
        task: &BusTask,
        spheres: &HashMap<PaneId, PaneSphere>,
    ) -> PvResult<PaneId> {
        let start = now_secs();

        let target = match &task.target {
            TaskTarget::Specific { pane_id } => {
                if spheres.contains_key(pane_id) {
                    Ok(pane_id.clone())
                } else {
                    Err(PvError::SphereNotFound(pane_id.as_str().to_owned()))
                }
            }
            TaskTarget::AnyIdle => select_idle_sphere(spheres),
            TaskTarget::FieldDriven => select_field_driven(spheres),
            TaskTarget::Willing => select_willing_sphere(spheres),
        }?;

        // Track in dispatch queue (bounded)
        if self.dispatch_queue.len() < self.max_queue {
            self.dispatch_queue.push(target.clone());
        }

        let elapsed = (now_secs() - start) * 1000.0; // Convert to ms
        tracing::debug!(
            target = %target,
            task_id = %task.id,
            elapsed_ms = elapsed,
            "executor dispatched task"
        );

        Ok(target)
    }

    /// Execute a dispatch and return a detailed result.
    ///
    /// # Errors
    /// Returns `PvError::SphereNotFound` if no suitable target is found.
    #[allow(clippy::implicit_hasher)]
    pub fn execute(
        &mut self,
        task: &BusTask,
        spheres: &HashMap<PaneId, PaneSphere>,
    ) -> PvResult<ExecutorResult> {
        let start = now_secs();
        match self.dispatch(task, spheres) {
            Ok(target) => {
                let elapsed_ms = (now_secs() - start) * 1000.0;
                Ok(ExecutorResult::success(target, elapsed_ms))
            }
            Err(e) => {
                let target = match &task.target {
                    TaskTarget::Specific { pane_id } => pane_id.clone(),
                    _ => PaneId::new("unknown"),
                };
                Ok(ExecutorResult::failure(target, e.to_string()))
            }
        }
    }

    /// Current dispatch queue length.
    #[must_use]
    #[allow(dead_code)] // Queue depth monitoring; used in future dispatch rate-limiting
    pub(crate) fn queue_len(&self) -> usize {
        self.dispatch_queue.len()
    }

    /// Clear the dispatch queue.
    #[allow(dead_code)] // Queue management API; called on executor reset
    pub(crate) fn clear_queue(&mut self) {
        self.dispatch_queue.clear();
    }

    /// Recent dispatch targets (from the queue).
    #[must_use]
    #[allow(dead_code)] // Dispatch history for debugging and API inspection
    pub(crate) fn recent_dispatches(&self, n: usize) -> Vec<&PaneId> {
        self.dispatch_queue.iter().rev().take(n).collect()
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────
// Selection strategies
// ──────────────────────────────────────────────────────────────

/// Select an idle sphere (prefer least recently active).
#[allow(clippy::implicit_hasher)]
fn select_idle_sphere(spheres: &HashMap<PaneId, PaneSphere>) -> PvResult<PaneId> {
    spheres
        .iter()
        .filter(|(_, s)| s.status == PaneStatus::Idle)
        .min_by(|(_, a), (_, b)| {
            a.last_heartbeat
                .partial_cmp(&b.last_heartbeat)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(id, _)| id.clone())
        .ok_or_else(|| PvError::SphereNotFound("no idle spheres available".into()))
}

/// Select a sphere using field-driven heuristics (prefer high receptivity + idle).
#[allow(clippy::implicit_hasher)]
fn select_field_driven(spheres: &HashMap<PaneId, PaneSphere>) -> PvResult<PaneId> {
    spheres
        .iter()
        .filter(|(_, s)| s.status == PaneStatus::Idle || s.status == PaneStatus::Working)
        .max_by(|(_, a), (_, b)| {
            let score_a = field_score(a);
            let score_b = field_score(b);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(id, _)| id.clone())
        .ok_or_else(|| PvError::SphereNotFound("no suitable spheres for field-driven dispatch".into()))
}

/// Select a willing sphere (prefer high receptivity).
#[allow(clippy::implicit_hasher)]
fn select_willing_sphere(spheres: &HashMap<PaneId, PaneSphere>) -> PvResult<PaneId> {
    spheres
        .iter()
        .filter(|(_, s)| {
            s.receptivity > 0.3
                && !s.opt_out_cross_activation
                && (s.status == PaneStatus::Idle || s.status == PaneStatus::Working)
        })
        .max_by(|(_, a), (_, b)| {
            a.receptivity
                .partial_cmp(&b.receptivity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(id, _)| id.clone())
        .ok_or_else(|| PvError::SphereNotFound("no willing spheres available".into()))
}

/// Field-driven score: receptivity * (1 if idle, 0.5 if working) * (1 - `opt_out_penalty`).
fn field_score(sphere: &PaneSphere) -> f64 {
    let status_weight = if sphere.status == PaneStatus::Idle {
        1.0
    } else {
        0.5
    };
    let opt_out_penalty = if sphere.opt_out_cross_activation {
        0.3
    } else {
        0.0
    };
    sphere.receptivity * status_weight * (1.0 - opt_out_penalty)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_foundation::m01_core_types::PaneId;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    fn make_sphere(id: &str, status: PaneStatus) -> (PaneId, PaneSphere) {
        let mut s = PaneSphere::new(pid(id), "test".into(), 0.1).unwrap();
        s.status = status;
        (pid(id), s)
    }

    fn make_spheres_with_status(entries: &[(&str, PaneStatus)]) -> HashMap<PaneId, PaneSphere> {
        entries
            .iter()
            .map(|(id, status)| make_sphere(id, *status))
            .collect()
    }

    fn make_task(target: TaskTarget) -> BusTask {
        BusTask::new("test task".into(), target, pid("submitter"))
    }

    // ── ExecutorResult ──

    #[test]
    fn executor_result_success() {
        let r = ExecutorResult::success(pid("target"), 5.0);
        assert!(r.success);
        assert_eq!(r.target_sphere.as_str(), "target");
        assert!((r.execution_ms - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn executor_result_failure() {
        let r = ExecutorResult::failure(pid("target"), "error".into());
        assert!(!r.success);
        assert_eq!(r.reason, "error");
    }

    // ── Executor construction ──

    #[test]
    fn executor_default() {
        let e = Executor::default();
        assert_eq!(e.queue_len(), 0);
    }

    #[test]
    fn executor_new() {
        let e = Executor::new();
        assert_eq!(e.queue_len(), 0);
    }

    #[test]
    fn executor_with_max_queue() {
        let e = Executor::with_max_queue(50);
        assert_eq!(e.max_queue, 50);
    }

    #[test]
    fn executor_with_max_queue_min_one() {
        let e = Executor::with_max_queue(0);
        assert_eq!(e.max_queue, 1);
    }

    // ── Dispatch: Specific target ──

    #[test]
    fn dispatch_specific_exists() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::Specific {
            pane_id: pid("alpha"),
        });
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "alpha");
    }

    #[test]
    fn dispatch_specific_not_found() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::Specific {
            pane_id: pid("nonexistent"),
        });
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    // ── Dispatch: AnyIdle ──

    #[test]
    fn dispatch_any_idle_finds_idle() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[
            ("busy", PaneStatus::Working),
            ("idle", PaneStatus::Idle),
        ]);
        let task = make_task(TaskTarget::AnyIdle);
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "idle");
    }

    #[test]
    fn dispatch_any_idle_none_available() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[
            ("busy1", PaneStatus::Working),
            ("busy2", PaneStatus::Working),
        ]);
        let task = make_task(TaskTarget::AnyIdle);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    #[test]
    fn dispatch_any_idle_empty_spheres() {
        let mut executor = Executor::new();
        let spheres = HashMap::new();
        let task = make_task(TaskTarget::AnyIdle);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    // ── Dispatch: FieldDriven ──

    #[test]
    fn dispatch_field_driven_prefers_high_receptivity() {
        let mut executor = Executor::new();
        let mut spheres = make_spheres_with_status(&[
            ("low", PaneStatus::Idle),
            ("high", PaneStatus::Idle),
        ]);
        spheres.get_mut(&pid("low")).unwrap().receptivity = 0.1;
        spheres.get_mut(&pid("high")).unwrap().receptivity = 0.9;
        let task = make_task(TaskTarget::FieldDriven);
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "high");
    }

    #[test]
    fn dispatch_field_driven_empty_fails() {
        let mut executor = Executor::new();
        let spheres = HashMap::new();
        let task = make_task(TaskTarget::FieldDriven);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    // ── Dispatch: Willing ──

    #[test]
    fn dispatch_willing_finds_receptive() {
        let mut executor = Executor::new();
        let mut spheres = make_spheres_with_status(&[
            ("receptive", PaneStatus::Idle),
            ("closed", PaneStatus::Idle),
        ]);
        spheres.get_mut(&pid("receptive")).unwrap().receptivity = 0.9;
        spheres.get_mut(&pid("closed")).unwrap().receptivity = 0.1;
        let task = make_task(TaskTarget::Willing);
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "receptive");
    }

    #[test]
    fn dispatch_willing_excludes_opted_out() {
        let mut executor = Executor::new();
        let mut spheres = make_spheres_with_status(&[
            ("opted_out", PaneStatus::Idle),
            ("willing", PaneStatus::Idle),
        ]);
        spheres.get_mut(&pid("opted_out")).unwrap().opt_out_cross_activation = true;
        spheres.get_mut(&pid("opted_out")).unwrap().receptivity = 0.9;
        spheres.get_mut(&pid("willing")).unwrap().receptivity = 0.5;
        let task = make_task(TaskTarget::Willing);
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "willing");
    }

    #[test]
    fn dispatch_willing_none_available() {
        let mut executor = Executor::new();
        let mut spheres = make_spheres_with_status(&[("closed", PaneStatus::Idle)]);
        spheres.get_mut(&pid("closed")).unwrap().receptivity = 0.1;
        let task = make_task(TaskTarget::Willing);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    // ── Execute ──

    #[test]
    fn execute_success() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::AnyIdle);
        let result = executor.execute(&task, &spheres).unwrap();
        assert!(result.success);
    }

    #[test]
    fn execute_failure() {
        let mut executor = Executor::new();
        let spheres = HashMap::new();
        let task = make_task(TaskTarget::AnyIdle);
        let result = executor.execute(&task, &spheres).unwrap();
        assert!(!result.success);
    }

    // ── Queue management ──

    #[test]
    fn dispatch_adds_to_queue() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::AnyIdle);
        executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(executor.queue_len(), 1);
    }

    #[test]
    fn clear_queue() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::AnyIdle);
        executor.dispatch(&task, &spheres).unwrap();
        executor.clear_queue();
        assert_eq!(executor.queue_len(), 0);
    }

    #[test]
    fn recent_dispatches() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::AnyIdle);
        executor.dispatch(&task, &spheres).unwrap();
        executor.dispatch(&task, &spheres).unwrap();
        let recent = executor.recent_dispatches(1);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn queue_bounded_by_max() {
        let mut executor = Executor::with_max_queue(2);
        let spheres = make_spheres_with_status(&[("alpha", PaneStatus::Idle)]);
        let task = make_task(TaskTarget::AnyIdle);
        for _ in 0..5 {
            executor.dispatch(&task, &spheres).unwrap();
        }
        assert!(executor.queue_len() <= 2);
    }

    // ── Selection strategies ──

    #[test]
    fn field_score_idle_higher_than_working() {
        let mut idle = PaneSphere::new(pid("idle"), "t".into(), 0.1).unwrap();
        idle.status = PaneStatus::Idle;
        idle.receptivity = 0.8;

        let mut working = PaneSphere::new(pid("work"), "t".into(), 0.1).unwrap();
        working.status = PaneStatus::Working;
        working.receptivity = 0.8;

        assert!(field_score(&idle) > field_score(&working));
    }

    #[test]
    fn field_score_opt_out_penalized() {
        let mut normal = PaneSphere::new(pid("normal"), "t".into(), 0.1).unwrap();
        normal.status = PaneStatus::Idle;
        normal.receptivity = 0.8;

        let mut opted = PaneSphere::new(pid("opted"), "t".into(), 0.1).unwrap();
        opted.status = PaneStatus::Idle;
        opted.receptivity = 0.8;
        opted.opt_out_cross_activation = true;

        assert!(field_score(&normal) > field_score(&opted));
    }

    #[test]
    fn field_score_high_receptivity_wins() {
        let mut low = PaneSphere::new(pid("low"), "t".into(), 0.1).unwrap();
        low.status = PaneStatus::Idle;
        low.receptivity = 0.2;

        let mut high = PaneSphere::new(pid("high"), "t".into(), 0.1).unwrap();
        high.status = PaneStatus::Idle;
        high.receptivity = 0.9;

        assert!(field_score(&high) > field_score(&low));
    }

    // ── Edge cases ──

    #[test]
    fn dispatch_blocked_sphere_not_idle() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("blocked", PaneStatus::Blocked)]);
        let task = make_task(TaskTarget::AnyIdle);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    #[test]
    fn dispatch_complete_sphere_not_idle() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("complete", PaneStatus::Complete)]);
        let task = make_task(TaskTarget::AnyIdle);
        assert!(executor.dispatch(&task, &spheres).is_err());
    }

    #[test]
    fn dispatch_specific_working_sphere_ok() {
        let mut executor = Executor::new();
        let spheres = make_spheres_with_status(&[("worker", PaneStatus::Working)]);
        let task = make_task(TaskTarget::Specific {
            pane_id: pid("worker"),
        });
        // Specific target does not check status
        let target = executor.dispatch(&task, &spheres).unwrap();
        assert_eq!(target.as_str(), "worker");
    }
}
