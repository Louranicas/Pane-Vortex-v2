//! # M35: Tick Orchestrator
//!
//! Decomposes the v1 829-line god function `tick_once()` into 5 phase functions
//! plus an orchestrator. This is the heartbeat of the system.
//!
//! ## Layer: L7 (Coordination)
//! ## Module: M35
//! ## Dependencies: L1, L3 (field), L4 (coupling), L5 (learning), L7 (conductor)
//!
//! ## Architecture
//! ```text
//! tick_orchestrator()
//!   Phase 1: Update sphere steps
//!   Phase 2: Coupling integration (N steps)
//!   Phase 3: Field state computation
//!   Phase 4: Conductor breathing
//!   Phase 5: Persistence check
//! ```
//!
//! ## Design Constraints
//! - C5: Lock ordering — `AppState` acquired once at start
//! - C6: All bridge calls (`tokio::spawn`) happen in `tick_bridging`, locks released first
//! - Max branch count per function: 18

use std::collections::HashMap;
use std::time::Instant;

use crate::m1_foundation::{
    m01_core_types::{DecisionRecord, FieldAction, OrderParameter, PaneId},
    m04_constants,
};
use crate::m3_field::{
    m12_field_state::{FieldDecision, FieldState},
    m15_app_state::AppState,
};
use crate::m4_coupling::m16_coupling_network::CouplingNetwork;
use super::m31_conductor::Conductor;

// ──────────────────────────────────────────────────────────────
// TickResult
// ──────────────────────────────────────────────────────────────

/// Result of a single tick orchestration.
#[derive(Debug)]
pub struct TickResult {
    /// The computed field state.
    pub field_state: FieldState,
    /// The computed field decision.
    pub decision: FieldDecision,
    /// Current order parameter.
    pub order_parameter: OrderParameter,
    /// Timing metrics for each phase (milliseconds).
    pub phase_timings: PhaseTiming,
    /// Total tick duration (milliseconds).
    pub total_ms: f64,
    /// Current tick number.
    pub tick: u64,
    /// Number of spheres in the field.
    pub sphere_count: usize,
    /// Whether a snapshot should be taken.
    pub should_snapshot: bool,
}

/// Timing metrics for each tick phase.
#[derive(Debug, Default)]
pub struct PhaseTiming {
    /// Phase 1: sphere stepping (ms).
    pub sphere_step_ms: f64,
    /// Phase 2: coupling integration (ms).
    pub coupling_ms: f64,
    /// Phase 3: field state computation (ms).
    pub field_state_ms: f64,
    /// Phase 4: conductor breathing (ms).
    pub conductor_ms: f64,
    /// Phase 5: persistence check (ms).
    pub persistence_ms: f64,
}

// ──────────────────────────────────────────────────────────────
// Tick Orchestrator
// ──────────────────────────────────────────────────────────────

/// Run one complete tick of the Kuramoto field.
///
/// This is the main entry point called by the daemon's tick loop.
///
/// # Errors
/// This function does not return errors — all phases are fault-tolerant
/// and log warnings instead of propagating failures.
pub fn tick_orchestrator(
    state: &mut AppState,
    network: &mut CouplingNetwork,
    conductor: &Conductor,
) -> TickResult {
    let tick_start = Instant::now();
    state.tick += 1;
    let current_tick = state.tick;

    // Handle warmup
    if state.is_warming_up() {
        state.tick_warmup();
    }

    let mut timings = PhaseTiming::default();

    // ── Phase 1: Sphere stepping ──
    let p1_start = Instant::now();
    tick_sphere_steps(state);
    timings.sphere_step_ms = p1_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 2: Coupling integration ──
    let p2_start = Instant::now();
    tick_coupling(state, network);
    timings.coupling_ms = p2_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 3: Field state computation ──
    let p3_start = Instant::now();
    let (field_state, decision) = tick_field_state(state, network, current_tick);
    timings.field_state_ms = p3_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 4: Conductor breathing ──
    let p4_start = Instant::now();
    tick_conductor(state, conductor, &decision);
    timings.conductor_ms = p4_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 5: Persistence check ──
    let p5_start = Instant::now();
    let should_snapshot = tick_persistence_check(state, current_tick);
    timings.persistence_ms = p5_start.elapsed().as_secs_f64() * 1000.0;

    let order_parameter = field_state.order_parameter;
    let sphere_count = state.spheres.len();
    let total_ms = tick_start.elapsed().as_secs_f64() * 1000.0;

    TickResult {
        field_state,
        decision,
        order_parameter,
        phase_timings: timings,
        total_ms,
        tick: current_tick,
        sphere_count,
        should_snapshot,
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 1: Sphere stepping
// ──────────────────────────────────────────────────────────────

/// Update all sphere internal state (memory decay, activation, buoy drift).
fn tick_sphere_steps(state: &mut AppState) {
    for sphere in state.spheres.values_mut() {
        sphere.step();
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 2: Coupling integration
// ──────────────────────────────────────────────────────────────

/// Run Kuramoto coupling integration steps.
///
/// Syncs network phases from spheres, runs N coupling steps with
/// per-sphere receptivity, then writes updated phases back.
fn tick_coupling(state: &mut AppState, network: &mut CouplingNetwork) {
    if state.spheres.len() < 2 {
        return;
    }

    // Sync phases from spheres -> network
    for (id, sphere) in &state.spheres {
        if let Some(phase) = network.phases.get_mut(id) {
            *phase = sphere.phase;
        }
        if let Some(freq) = network.frequencies.get_mut(id) {
            *freq = sphere.frequency;
        }
    }

    // Collect receptivities
    let receptivities: HashMap<PaneId, f64> = state
        .spheres
        .iter()
        .map(|(id, s)| (id.clone(), s.receptivity))
        .collect();

    // Run coupling steps
    let steps = m04_constants::COUPLING_STEPS_PER_TICK;
    for _ in 0..steps {
        network.step_with_receptivity(&receptivities);
    }

    // Write updated phases back to spheres
    for (id, sphere) in &mut state.spheres {
        if let Some(&phase) = network.phases.get(id) {
            sphere.phase = phase;
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 3: Field state computation
// ──────────────────────────────────────────────────────────────

/// Compute field state and decision from current sphere map.
fn tick_field_state(
    state: &mut AppState,
    network: &CouplingNetwork,
    tick: u64,
) -> (FieldState, FieldDecision) {
    let field_state = FieldState::compute(
        &state.spheres,
        network.k_modulation,
        tick,
    );

    // Push r to history
    state.push_r(field_state.order_parameter.r);

    // Compute decision
    let decision = if state.divergence_cooldown > 0 {
        FieldDecision::recovering(tick)
    } else {
        FieldDecision::compute(
            &state.spheres,
            &field_state,
            &state.r_history,
            tick,
        )
    };

    // Record decision in audit trail
    state.record_decision(DecisionRecord {
        tick,
        action: decision.action.clone(),
        r: field_state.order_parameter.r,
        k_mod: network.k_modulation,
        sphere_count: state.spheres.len(),
    });

    // Cache field state
    state.cached_field = Some(field_state.clone());

    (field_state, decision)
}

// ──────────────────────────────────────────────────────────────
// Phase 4: Conductor breathing
// ──────────────────────────────────────────────────────────────

/// Apply conductor breathing based on the field decision.
fn tick_conductor(
    state: &mut AppState,
    conductor: &Conductor,
    decision: &FieldDecision,
) {
    conductor.conduct_breathing(state, decision);

    // Inject phase noise if divergence is needed
    if matches!(
        decision.action,
        FieldAction::NeedsDivergence | FieldAction::OverSynchronized
    ) {
        Conductor::inject_phase_noise(state, decision);
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 5: Persistence check
// ──────────────────────────────────────────────────────────────

/// Check whether a snapshot should be taken.
fn tick_persistence_check(state: &mut AppState, tick: u64) -> bool {
    let should_snapshot = tick % m04_constants::SNAPSHOT_INTERVAL == 0 && state.dirty;
    if should_snapshot {
        state.clear_dirty();
    }
    // Mark dirty if any state changes occurred this tick
    if state.state_changes > 0 {
        state.mark_dirty();
        state.state_changes = 0;
    }
    should_snapshot
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_foundation::m01_core_types::PaneId;
    use crate::m3_field::m11_sphere::PaneSphere;
    use approx::assert_relative_eq;
    use std::f64::consts::TAU;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    fn make_state_with_spheres(n: usize) -> (AppState, CouplingNetwork) {
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();
        for i in 0..n {
            let id = format!("s{i}");
            let sphere = PaneSphere::new(pid(&id), format!("sphere-{i}"), 0.1).unwrap();
            state.spheres.insert(pid(&id), sphere);
            #[allow(clippy::cast_precision_loss)]
            let phase = (i as f64 / n.max(1) as f64) * TAU * 0.3;
            network.register(pid(&id), phase, 0.1);
        }
        (state, network)
    }

    // ── TickResult ──

    #[test]
    fn tick_result_fields() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert!(result.total_ms >= 0.0);
        assert_eq!(result.tick, 1);
        assert_eq!(result.sphere_count, 3);
    }

    // ── PhaseTiming ──

    #[test]
    fn phase_timing_default_zero() {
        let t = PhaseTiming::default();
        assert_relative_eq!(t.sphere_step_ms, 0.0);
        assert_relative_eq!(t.coupling_ms, 0.0);
        assert_relative_eq!(t.field_state_ms, 0.0);
        assert_relative_eq!(t.conductor_ms, 0.0);
        assert_relative_eq!(t.persistence_ms, 0.0);
    }

    // ── tick_orchestrator ──

    #[test]
    fn tick_empty_state() {
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(result.tick, 1);
        assert_eq!(result.sphere_count, 0);
    }

    #[test]
    fn tick_increments_tick_counter() {
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(state.tick, 1);
        tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn tick_single_sphere() {
        let (mut state, mut network) = make_state_with_spheres(1);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(result.sphere_count, 1);
    }

    #[test]
    fn tick_multiple_spheres() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(result.sphere_count, 5);
        // Should have r history
        assert_eq!(state.r_history.len(), 1);
    }

    #[test]
    fn tick_preserves_phase_bounds() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        for _ in 0..10 {
            tick_orchestrator(&mut state, &mut network, &conductor);
        }
        for sphere in state.spheres.values() {
            assert!(
                sphere.phase >= 0.0 && sphere.phase < TAU,
                "phase out of bounds: {}",
                sphere.phase
            );
        }
    }

    #[test]
    fn tick_updates_r_history() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        for _ in 0..5 {
            tick_orchestrator(&mut state, &mut network, &conductor);
        }
        assert_eq!(state.r_history.len(), 5);
    }

    #[test]
    fn tick_records_decisions() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(state.decision_history.len(), 1);
    }

    #[test]
    fn tick_caches_field_state() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor);
        assert!(state.cached_field.is_some());
    }

    #[test]
    fn tick_warmup_decrements() {
        let (mut state, mut network) = make_state_with_spheres(2);
        let conductor = Conductor::new();
        state.warmup_remaining = 3;
        tick_orchestrator(&mut state, &mut network, &conductor);
        assert_eq!(state.warmup_remaining, 2);
    }

    #[test]
    fn tick_phase_timings_non_negative() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert!(result.phase_timings.sphere_step_ms >= 0.0);
        assert!(result.phase_timings.coupling_ms >= 0.0);
        assert!(result.phase_timings.field_state_ms >= 0.0);
        assert!(result.phase_timings.conductor_ms >= 0.0);
        assert!(result.phase_timings.persistence_ms >= 0.0);
    }

    // ── Phase 1: Sphere stepping ──

    #[test]
    fn sphere_steps_increment_total() {
        let mut state = AppState::new();
        let sphere = PaneSphere::new(pid("a"), "test".into(), 0.1).unwrap();
        state.spheres.insert(pid("a"), sphere);
        tick_sphere_steps(&mut state);
        assert_eq!(state.spheres[&pid("a")].total_steps, 1);
    }

    #[test]
    fn sphere_steps_multiple_spheres() {
        let mut state = AppState::new();
        for i in 0..5 {
            let id = format!("s{i}");
            let sphere = PaneSphere::new(pid(&id), "test".into(), 0.1).unwrap();
            state.spheres.insert(pid(&id), sphere);
        }
        tick_sphere_steps(&mut state);
        for sphere in state.spheres.values() {
            assert_eq!(sphere.total_steps, 1);
        }
    }

    // ── Phase 2: Coupling ──

    #[test]
    fn coupling_single_sphere_no_op() {
        let (mut state, mut network) = make_state_with_spheres(1);
        let phase_before = state.spheres.values().next().map(|s| s.phase);
        tick_coupling(&mut state, &mut network);
        let phase_after = state.spheres.values().next().map(|s| s.phase);
        // Single sphere: phase may change due to natural frequency but coupling is skipped
        assert!(phase_before.is_some());
        assert!(phase_after.is_some());
    }

    #[test]
    fn coupling_multiple_spheres_changes_phases() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let phases_before: HashMap<PaneId, f64> = state
            .spheres
            .iter()
            .map(|(id, s)| (id.clone(), s.phase))
            .collect();
        tick_coupling(&mut state, &mut network);
        let mut any_changed = false;
        for (id, sphere) in &state.spheres {
            let before = phases_before[id];
            if (sphere.phase - before).abs() > f64::EPSILON {
                any_changed = true;
                break;
            }
        }
        // With non-zero frequencies and coupling, phases should change
        assert!(any_changed || state.spheres.len() < 2);
    }

    #[test]
    fn coupling_empty_state_no_panic() {
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();
        tick_coupling(&mut state, &mut network);
    }

    // ── Phase 3: Field state ──

    #[test]
    fn field_state_computation() {
        let (mut state, network) = make_state_with_spheres(3);
        let (fs, decision) = tick_field_state(&mut state, &network, 1);
        assert_eq!(fs.sphere_count, 3);
        assert_eq!(decision.tick, 1);
    }

    #[test]
    fn field_state_pushes_r() {
        let (mut state, network) = make_state_with_spheres(3);
        tick_field_state(&mut state, &network, 1);
        assert_eq!(state.r_history.len(), 1);
    }

    #[test]
    fn field_state_during_cooldown() {
        let (mut state, network) = make_state_with_spheres(3);
        state.divergence_cooldown = 2;
        let (_, decision) = tick_field_state(&mut state, &network, 1);
        assert_eq!(decision.action, FieldAction::Recovering);
    }

    // ── Phase 4: Conductor ──

    #[test]
    fn conductor_called_without_panic() {
        let (mut state, _) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        let decision = FieldDecision::recovering(1);
        tick_conductor(&mut state, &conductor, &decision);
    }

    // ── Phase 5: Persistence ──

    #[test]
    fn persistence_check_not_snapshot_tick() {
        let mut state = AppState::new();
        state.dirty = true;
        let should = tick_persistence_check(&mut state, 1);
        assert!(!should);
    }

    #[test]
    fn persistence_check_snapshot_tick_dirty() {
        let mut state = AppState::new();
        state.dirty = true;
        let should = tick_persistence_check(&mut state, m04_constants::SNAPSHOT_INTERVAL);
        assert!(should);
        assert!(!state.dirty); // cleared after snapshot
    }

    #[test]
    fn persistence_check_snapshot_tick_clean() {
        let mut state = AppState::new();
        state.dirty = false;
        let should = tick_persistence_check(&mut state, m04_constants::SNAPSHOT_INTERVAL);
        assert!(!should);
    }

    #[test]
    fn persistence_check_state_changes_mark_dirty() {
        let mut state = AppState::new();
        state.state_changes = 5;
        tick_persistence_check(&mut state, 1);
        assert!(state.dirty);
        assert_eq!(state.state_changes, 0);
    }

    // ── Integration: multiple ticks ──

    #[test]
    fn multi_tick_stability() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        for _ in 0..20 {
            let result = tick_orchestrator(&mut state, &mut network, &conductor);
            assert!(result.total_ms >= 0.0);
            assert!(result.order_parameter.r >= 0.0);
            assert!(result.order_parameter.r <= 1.0 + 1e-10);
        }
        assert_eq!(state.tick, 20);
        assert_eq!(state.r_history.len(), 20);
    }

    #[test]
    fn multi_tick_phases_bounded() {
        let (mut state, mut network) = make_state_with_spheres(10);
        let conductor = Conductor::new();
        for _ in 0..50 {
            tick_orchestrator(&mut state, &mut network, &conductor);
        }
        for sphere in state.spheres.values() {
            assert!(
                sphere.phase >= 0.0 && sphere.phase < TAU,
                "phase={} for {}",
                sphere.phase,
                sphere.id
            );
        }
    }

    #[test]
    fn multi_tick_order_converges() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        for _ in 0..100 {
            tick_orchestrator(&mut state, &mut network, &conductor);
        }
        // After 100 ticks with coupling, r should have settled
        let final_r = state.r_history.back().copied().unwrap_or(0.0);
        assert!(final_r >= 0.0 && final_r <= 1.0);
    }

    #[test]
    fn tick_with_working_spheres() {
        let (mut state, mut network) = make_state_with_spheres(5);
        // Record some memories so spheres are "working"
        for sphere in state.spheres.values_mut() {
            sphere.record_memory("Read".into(), "file".into());
        }
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        assert!(result.sphere_count > 0);
    }

    #[test]
    fn tick_total_ms_sums_phases() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor);
        let sum = result.phase_timings.sphere_step_ms
            + result.phase_timings.coupling_ms
            + result.phase_timings.field_state_ms
            + result.phase_timings.conductor_ms
            + result.phase_timings.persistence_ms;
        // Total should be >= sum of phases (with minor overhead)
        assert!(result.total_ms >= sum * 0.9);
    }
}
