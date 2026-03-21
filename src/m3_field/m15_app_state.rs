//! # M15: Application State
//!
//! `AppState` wrapped in `Arc<parking_lot::RwLock<T>>` as `SharedState`.
//! Contains spheres, coupling state, field history, ghost traces, and conductor state.
//!
//! ## Layer: L3 (Field)
//! ## Module: M15
//! ## Dependencies: L1 (M01, M04), M11, M12

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::m1_foundation::{
    m01_core_types::{
        BridgeAdjustments, BridgeStaleness, DecisionRecord, FieldAction, FleetMode,
        GhostTrace, OrderParameter, PaneId,
    },
    m04_constants,
};
use super::m11_sphere::PaneSphere;
use super::m12_field_state::FieldState;

// ──────────────────────────────────────────────────────────────
// Shared state type alias
// ──────────────────────────────────────────────────────────────

/// Thread-safe shared application state.
///
/// Lock ordering: acquire `AppState` BEFORE any `BusState` lock. Always.
pub type SharedState = Arc<RwLock<AppState>>;

/// Create a new shared state with default `AppState`.
#[must_use]
pub fn new_shared_state() -> SharedState {
    Arc::new(RwLock::new(AppState::new()))
}

// ──────────────────────────────────────────────────────────────
// AppState
// ──────────────────────────────────────────────────────────────

/// Shared application state for the Pane-Vortex daemon.
///
/// Serialized for snapshot persistence; ephemeral fields are `#[serde(skip)]`.
#[derive(Serialize, Deserialize)]
pub struct AppState {
    // ── Field state ──
    /// Active spheres indexed by ID.
    pub spheres: HashMap<PaneId, PaneSphere>,
    /// Current tick count.
    pub tick: u64,
    /// Rolling window of order parameter r values.
    pub r_history: VecDeque<f64>,
    /// Recent log messages (bounded).
    pub message_log: VecDeque<String>,
    /// API calls since last tick (dirty tracking).
    pub state_changes: u32,

    // ── Conductor state ──
    /// EMA of divergence signal.
    pub divergence_ema: f64,
    /// EMA of coherence signal.
    pub coherence_ema: f64,
    /// PI controller integral term.
    pub divergence_integral: f64,
    /// Stability counter for thrashing guard.
    pub prev_decision_ticks: u32,
    /// Previous tick's decision action.
    pub prev_decision_action: FieldAction,
    /// Decision audit trail (bounded).
    pub decision_history: VecDeque<DecisionRecord>,

    // ── Ghost traces ──
    /// Ghost traces of deregistered spheres (FIFO, capped).
    pub ghosts: VecDeque<GhostTrace>,

    // ── Ephemeral (not serialized) ──
    /// Whether state has changed since last snapshot.
    #[serde(skip)]
    pub dirty: bool,
    /// Warmup ticks remaining after snapshot restore.
    #[serde(skip)]
    pub warmup_remaining: u32,
    /// Divergence cooldown ticks remaining.
    #[serde(skip)]
    pub divergence_cooldown: u32,
    /// Cached field state from previous tick.
    #[serde(skip)]
    pub cached_field: Option<FieldState>,
    /// Cascade events in current rate-limit window.
    #[serde(skip)]
    pub cascade_events_this_window: u32,
    /// Tick at which the current cascade window started.
    #[serde(skip)]
    pub cascade_window_start_tick: u64,
    /// r history max size (from config).
    #[serde(skip)]
    pub r_history_max: usize,
    /// EMA of sphere count for fleet mode transitions.
    #[serde(skip)]
    pub sphere_count_ema: f64,
    /// Last bridge adjustment values.
    #[serde(skip)]
    pub last_bridge_adjustments: BridgeAdjustments,
    /// Bridge staleness flags.
    #[serde(skip)]
    pub prev_bridge_staleness: BridgeStaleness,
    /// Current cascade depth.
    #[serde(skip)]
    pub cascade_depth: u32,
    /// Governance: proposal manager (V3.4, feature-gated).
    /// Uses `serde(default)` so snapshots without governance data still deserialize.
    #[cfg(feature = "governance")]
    #[serde(default)]
    pub proposal_manager: crate::m8_governance::m37_proposals::ProposalManager,
    /// Dynamic `r_target` override from governance proposals (GAP-2).
    #[serde(default)]
    pub r_target_override: Option<f64>,
    /// Dynamic `k_mod_budget_max` override from governance proposals (GAP-2).
    #[serde(default)]
    pub k_mod_budget_max_override: Option<f64>,
}

impl AppState {
    /// Create a new empty state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            spheres: HashMap::new(),
            tick: 0,
            r_history: VecDeque::with_capacity(m04_constants::R_HISTORY_MAX),
            message_log: VecDeque::new(),
            state_changes: 0,
            divergence_ema: 0.0,
            coherence_ema: 0.0,
            divergence_integral: 0.0,
            prev_decision_ticks: 0,
            prev_decision_action: FieldAction::Stable,
            decision_history: VecDeque::with_capacity(m04_constants::DECISION_HISTORY_MAX),
            ghosts: VecDeque::new(),
            dirty: false,
            warmup_remaining: 0,
            divergence_cooldown: 0,
            cached_field: None,
            cascade_events_this_window: 0,
            cascade_window_start_tick: 0,
            r_history_max: m04_constants::R_HISTORY_MAX,
            sphere_count_ema: 0.0,
            last_bridge_adjustments: BridgeAdjustments::default(),
            prev_bridge_staleness: BridgeStaleness::default(),
            cascade_depth: 0,
            #[cfg(feature = "governance")]
            proposal_manager: crate::m8_governance::m37_proposals::ProposalManager::new(),
            r_target_override: None,
            k_mod_budget_max_override: None,
        }
    }

    /// Current order parameter.
    #[must_use]
    pub fn order_parameter(&self) -> OrderParameter {
        let r = self.r_history.back().copied().unwrap_or(0.0);
        OrderParameter { r, psi: 0.0 }
    }

    /// Current fleet mode.
    #[must_use]
    pub fn fleet_mode(&self) -> FleetMode {
        FleetMode::from_count(self.spheres.len())
    }

    /// Push r value to history, maintaining max size.
    pub fn push_r(&mut self, r: f64) {
        self.r_history.push_back(r);
        while self.r_history.len() > self.r_history_max {
            self.r_history.pop_front();
        }
    }

    /// Log a message (bounded to `LOG_MAX`).
    pub fn log(&mut self, msg: impl Into<String>) {
        self.message_log.push_back(msg.into());
        while self.message_log.len() > m04_constants::LOG_MAX {
            self.message_log.pop_front();
        }
    }

    /// Record a decision in the audit trail.
    pub fn record_decision(&mut self, record: DecisionRecord) {
        self.decision_history.push_back(record);
        while self.decision_history.len() > m04_constants::DECISION_HISTORY_MAX {
            self.decision_history.pop_front();
        }
    }

    /// Add a ghost trace (FIFO, capped at `GHOST_MAX`).
    pub fn add_ghost(&mut self, ghost: GhostTrace) {
        self.ghosts.push_back(ghost);
        while self.ghosts.len() > m04_constants::GHOST_MAX {
            self.ghosts.pop_front();
        }
    }

    /// Accept (consume) a ghost trace by exact ID match.
    ///
    /// Returns the ghost if found and consumed, `None` if no matching ghost exists.
    /// This prevents multiple claimants (G4 gap fix).
    pub fn accept_ghost(&mut self, id: &PaneId) -> Option<GhostTrace> {
        let pos = self.ghosts.iter().position(|g| g.id == *id)?;
        self.ghosts.remove(pos)
    }

    /// Whether the field is in warmup mode.
    #[must_use]
    pub const fn is_warming_up(&self) -> bool {
        self.warmup_remaining > 0
    }

    /// Decrement warmup counter.
    pub fn tick_warmup(&mut self) {
        self.warmup_remaining = self.warmup_remaining.saturating_sub(1);
    }

    /// Mark state as dirty (needs snapshot).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear dirty flag after snapshot.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Reconcile all spheres after snapshot restore.
    pub fn reconcile_after_restore(&mut self) {
        for sphere in self.spheres.values_mut() {
            sphere.reconcile_after_restore();
        }
        self.warmup_remaining = m04_constants::WARMUP_TICKS;

        // BUG-032 fix: ensure ProposalManager has valid config after restore.
        self.proposal_manager.reconcile();
    }

    /// Sphere count.
    #[must_use]
    pub fn sphere_count(&self) -> usize {
        self.spheres.len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("tick", &self.tick)
            .field("spheres", &self.spheres.len())
            .field("ghosts", &self.ghosts.len())
            .field("r_latest", &self.r_history.back())
            .field("dirty", &self.dirty)
            .field("warmup", &self.warmup_remaining)
            .finish_non_exhaustive()
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_foundation::m01_core_types::WorkSignature;
    use approx::assert_relative_eq;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    // ── Construction ──

    #[test]
    fn new_state_is_empty() {
        let s = AppState::new();
        assert!(s.spheres.is_empty());
        assert_eq!(s.tick, 0);
    }

    #[test]
    fn new_state_r_history_empty() {
        let s = AppState::new();
        assert!(s.r_history.is_empty());
    }

    #[test]
    fn new_state_not_dirty() {
        let s = AppState::new();
        assert!(!s.dirty);
    }

    #[test]
    fn new_state_no_warmup() {
        let s = AppState::new();
        assert!(!s.is_warming_up());
    }

    #[test]
    fn new_state_fleet_mode_solo() {
        let s = AppState::new();
        assert_eq!(s.fleet_mode(), FleetMode::Solo);
    }

    #[test]
    fn default_matches_new() {
        let a = AppState::new();
        let b = AppState::default();
        assert_eq!(a.tick, b.tick);
        assert_eq!(a.spheres.len(), b.spheres.len());
    }

    // ── SharedState ──

    #[test]
    fn shared_state_creation() {
        let ss = new_shared_state();
        let guard = ss.read();
        assert_eq!(guard.tick, 0);
    }

    #[test]
    fn shared_state_write() {
        let ss = new_shared_state();
        {
            let mut guard = ss.write();
            guard.tick = 42;
        }
        let guard = ss.read();
        assert_eq!(guard.tick, 42);
    }

    // ── Order parameter ──

    #[test]
    fn order_parameter_empty_history() {
        let s = AppState::new();
        let op = s.order_parameter();
        assert_relative_eq!(op.r, 0.0);
    }

    #[test]
    fn order_parameter_from_history() {
        let mut s = AppState::new();
        s.push_r(0.95);
        let op = s.order_parameter();
        assert_relative_eq!(op.r, 0.95);
    }

    // ── push_r ──

    #[test]
    fn push_r_adds_value() {
        let mut s = AppState::new();
        s.push_r(0.5);
        assert_eq!(s.r_history.len(), 1);
    }

    #[test]
    fn push_r_caps_at_max() {
        let mut s = AppState::new();
        for i in 0..100 {
            #[allow(clippy::cast_precision_loss)]
            s.push_r(i as f64 / 100.0);
        }
        assert!(s.r_history.len() <= m04_constants::R_HISTORY_MAX);
    }

    #[test]
    fn push_r_preserves_most_recent() {
        let mut s = AppState::new();
        for i in 0..100 {
            #[allow(clippy::cast_precision_loss)]
            s.push_r(i as f64 / 100.0);
        }
        assert_relative_eq!(*s.r_history.back().unwrap(), 0.99);
    }

    // ── log ──

    #[test]
    fn log_adds_message() {
        let mut s = AppState::new();
        s.log("test message");
        assert_eq!(s.message_log.len(), 1);
    }

    #[test]
    fn log_caps_at_max() {
        let mut s = AppState::new();
        for i in 0..1500 {
            s.log(format!("msg {i}"));
        }
        assert!(s.message_log.len() <= m04_constants::LOG_MAX);
    }

    // ── record_decision ──

    #[test]
    fn record_decision_adds() {
        let mut s = AppState::new();
        s.record_decision(DecisionRecord {
            tick: 1,
            action: FieldAction::Stable,
            r: 0.5,
            k_mod: 1.0,
            sphere_count: 3,
        });
        assert_eq!(s.decision_history.len(), 1);
    }

    #[test]
    fn record_decision_caps_at_max() {
        let mut s = AppState::new();
        for i in 0..200 {
            s.record_decision(DecisionRecord {
                tick: i,
                action: FieldAction::Stable,
                r: 0.5,
                k_mod: 1.0,
                sphere_count: 3,
            });
        }
        assert!(s.decision_history.len() <= m04_constants::DECISION_HISTORY_MAX);
    }

    // ── Ghost traces ──

    #[test]
    fn add_ghost_adds() {
        let mut s = AppState::new();
        s.add_ghost(GhostTrace {
            id: pid("departed"),
            persona: "explorer".into(),
            deregistered_at: 100,
            total_steps_lived: 50,
            memory_count: 10,
            top_tools: vec!["Read".into()],
            phase_at_departure: 1.5,
            receptivity: 0.8,
            work_signature: WorkSignature::default(),
            strongest_neighbors: vec![],
        });
        assert_eq!(s.ghosts.len(), 1);
    }

    #[test]
    fn accept_ghost_consumes() {
        let mut s = AppState::new();
        s.add_ghost(GhostTrace {
            id: pid("reborn"),
            persona: "returner".into(),
            deregistered_at: 50,
            total_steps_lived: 25,
            memory_count: 5,
            top_tools: vec![],
            phase_at_departure: 2.0,
            receptivity: 0.9,
            work_signature: WorkSignature::default(),
            strongest_neighbors: vec![],
        });
        let ghost = s.accept_ghost(&pid("reborn"));
        assert!(ghost.is_some());
        assert_eq!(ghost.expect("ghost").id, pid("reborn"));
        // Ghost should be consumed — not findable again
        assert!(s.accept_ghost(&pid("reborn")).is_none());
        assert!(s.ghosts.is_empty());
    }

    #[test]
    fn accept_ghost_missing_returns_none() {
        let mut s = AppState::new();
        assert!(s.accept_ghost(&pid("nonexistent")).is_none());
    }

    #[test]
    fn accept_ghost_exact_id_match() {
        let mut s = AppState::new();
        s.add_ghost(GhostTrace {
            id: pid("specific-id"),
            persona: "test".into(),
            deregistered_at: 10,
            total_steps_lived: 5,
            memory_count: 0,
            top_tools: vec![],
            phase_at_departure: 1.0,
            receptivity: 1.0,
            work_signature: WorkSignature::default(),
            strongest_neighbors: vec![],
        });
        // Different ID should not match
        assert!(s.accept_ghost(&pid("different-id")).is_none());
        // Exact ID should match
        assert!(s.accept_ghost(&pid("specific-id")).is_some());
    }

    #[test]
    fn add_ghost_caps_at_max() {
        let mut s = AppState::new();
        for i in 0..30 {
            s.add_ghost(GhostTrace {
                id: pid(&format!("g{i}")),
                persona: "test".into(),
                deregistered_at: i,
                total_steps_lived: 10,
                memory_count: 0,
                top_tools: vec![],
                phase_at_departure: 0.0,
                receptivity: 1.0,
                work_signature: WorkSignature::default(),
                strongest_neighbors: vec![],
            });
        }
        assert!(s.ghosts.len() <= m04_constants::GHOST_MAX);
    }

    // ── Warmup ──

    #[test]
    fn is_warming_up_when_remaining() {
        let mut s = AppState::new();
        s.warmup_remaining = 3;
        assert!(s.is_warming_up());
    }

    #[test]
    fn tick_warmup_decrements() {
        let mut s = AppState::new();
        s.warmup_remaining = 3;
        s.tick_warmup();
        assert_eq!(s.warmup_remaining, 2);
    }

    #[test]
    fn tick_warmup_stops_at_zero() {
        let mut s = AppState::new();
        s.warmup_remaining = 0;
        s.tick_warmup();
        assert_eq!(s.warmup_remaining, 0);
    }

    // ── Dirty flag ──

    #[test]
    fn mark_dirty() {
        let mut s = AppState::new();
        s.mark_dirty();
        assert!(s.dirty);
    }

    #[test]
    fn clear_dirty() {
        let mut s = AppState::new();
        s.mark_dirty();
        s.clear_dirty();
        assert!(!s.dirty);
    }

    // ── Reconcile ──

    #[test]
    fn reconcile_sets_warmup() {
        let mut s = AppState::new();
        s.reconcile_after_restore();
        assert_eq!(s.warmup_remaining, m04_constants::WARMUP_TICKS);
    }

    #[test]
    fn reconcile_fixes_sphere_ids() {
        let mut s = AppState::new();
        let mut sphere = PaneSphere::new(pid("test"), "tester".into(), 0.1).unwrap();
        sphere.record_memory("Read".into(), "file".into());
        s.spheres.insert(pid("test"), sphere);
        s.reconcile_after_restore();
        // Memory IDs should be reconciled
        assert!(s.spheres.get(&pid("test")).is_some());
    }

    // ── Fleet mode ──

    #[test]
    fn fleet_mode_changes_with_spheres() {
        let mut s = AppState::new();
        assert_eq!(s.fleet_mode(), FleetMode::Solo);

        s.spheres.insert(
            pid("a"),
            PaneSphere::new(pid("a"), "a".into(), 0.1).unwrap(),
        );
        assert_eq!(s.fleet_mode(), FleetMode::Solo);

        s.spheres.insert(
            pid("b"),
            PaneSphere::new(pid("b"), "b".into(), 0.1).unwrap(),
        );
        assert_eq!(s.fleet_mode(), FleetMode::Pair);

        for i in 0..3 {
            let id = format!("s{i}");
            s.spheres.insert(
                pid(&id),
                PaneSphere::new(pid(&id), "test".into(), 0.1).unwrap(),
            );
        }
        assert_eq!(s.fleet_mode(), FleetMode::Full);
    }

    // ── Sphere count ──

    #[test]
    fn sphere_count_empty() {
        let s = AppState::new();
        assert_eq!(s.sphere_count(), 0);
    }

    #[test]
    fn sphere_count_matches() {
        let mut s = AppState::new();
        s.spheres.insert(
            pid("a"),
            PaneSphere::new(pid("a"), "a".into(), 0.1).unwrap(),
        );
        assert_eq!(s.sphere_count(), 1);
    }

    // ── Debug ──

    #[test]
    fn debug_format() {
        let s = AppState::new();
        let debug = format!("{s:?}");
        assert!(debug.contains("AppState"));
        assert!(debug.contains("tick"));
    }

    // ── Serde roundtrip ──

    #[test]
    fn app_state_serde_roundtrip() {
        let mut s = AppState::new();
        s.tick = 42;
        s.push_r(0.75);
        s.spheres.insert(
            pid("test"),
            PaneSphere::new(pid("test"), "tester".into(), 0.1).unwrap(),
        );
        let json = serde_json::to_string(&s).unwrap();
        let back: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tick, 42);
        assert_eq!(back.spheres.len(), 1);
        assert_eq!(back.r_history.len(), 1);
    }

    #[test]
    fn app_state_serde_skips_ephemeral() {
        let mut s = AppState::new();
        s.dirty = true;
        s.warmup_remaining = 5;
        s.cached_field = None;
        let json = serde_json::to_string(&s).unwrap();
        let back: AppState = serde_json::from_str(&json).unwrap();
        // Ephemeral fields should be defaults after deserialization
        assert!(!back.dirty);
        assert_eq!(back.warmup_remaining, 0);
    }

    #[test]
    fn app_state_serde_preserves_ghosts() {
        let mut s = AppState::new();
        s.add_ghost(GhostTrace {
            id: pid("ghost"),
            persona: "departed".into(),
            deregistered_at: 100,
            total_steps_lived: 50,
            memory_count: 5,
            top_tools: vec!["Read".into()],
            phase_at_departure: 1.0,
            receptivity: 0.9,
            work_signature: WorkSignature::default(),
            strongest_neighbors: vec![],
        });
        let json = serde_json::to_string(&s).unwrap();
        let back: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.ghosts.len(), 1);
        assert_eq!(back.ghosts[0].id.as_str(), "ghost");
    }

    #[test]
    fn app_state_serde_preserves_decision_history() {
        let mut s = AppState::new();
        s.record_decision(DecisionRecord {
            tick: 10,
            action: FieldAction::NeedsCoherence,
            r: 0.3,
            k_mod: 1.1,
            sphere_count: 5,
        });
        let json = serde_json::to_string(&s).unwrap();
        let back: AppState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.decision_history.len(), 1);
        assert_eq!(back.decision_history[0].action, FieldAction::NeedsCoherence);
    }
}
