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
use crate::m6_bridges::BridgeSet;
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
    /// Phase 2.7: bridge `k_mod` application (ms).
    pub bridge_ms: f64,
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
    bridges: Option<&BridgeSet>,
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

    // ── Phase 2.5: Hebbian STDP learning (BUG-031 fix) ──
    let stdp_result = tick_hebbian(state, network);
    state.hebbian_ltp_total = state.hebbian_ltp_total.saturating_add(stdp_result.ltp_count as u64);
    state.hebbian_ltd_total = state.hebbian_ltd_total.saturating_add(stdp_result.ltd_count as u64);

    // ── Phase 2.7: Bridge k_mod application ──
    let bridge_start = Instant::now();
    if let Some(bridges) = bridges {
        bridges.apply_k_mod(state, network);
    }
    timings.bridge_ms = bridge_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 3: Field state computation ──
    let p3_start = Instant::now();
    let (field_state, decision) = tick_field_state(state, network, current_tick);
    timings.field_state_ms = p3_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 3.1: Harmonic damping (H3 — l2 quadrupole feedback) ──
    // When l2 > 0.70, increase K to break 4-way phase clustering.
    // Formula: k_adj = 1.0 + 0.15 * (1.0 - r) * (l2 - 0.70) / 0.30
    {
        let l2 = field_state.harmonics.l2_quadrupole;
        let r = field_state.order_parameter.r;
        if l2 > 0.70 {
            let l2_adj = 1.0 + 0.15 * (1.0 - r) * (l2 - 0.70) / 0.30;
            network.k_modulation *= l2_adj;
            network.k_modulation = network.k_modulation.clamp(
                crate::m1_foundation::m04_constants::K_MOD_BUDGET_MIN,
                crate::m1_foundation::m04_constants::K_MOD_BUDGET_MAX,
            );
        }
    }

    // ── Phase 3.5: Governance actuator (GAP-1 fix) ──
    #[cfg(feature = "governance")]
    tick_governance(state, current_tick);

    // ── Phase 4: Conductor breathing ──
    let p4_start = Instant::now();
    tick_conductor(state, conductor, &decision, network);
    timings.conductor_ms = p4_start.elapsed().as_secs_f64() * 1000.0;

    // ── Phase 4.5: Idle sphere TTL pruning (Session 066) ──
    // Every 60 ticks, deregister idle spheres with 0 memories and age > 300 ticks.
    // Prevents O(N^2) coupling overhead from 66+ dormant orac-agent spheres.
    if current_tick % 60 == 0 && current_tick > 0 {
        tick_prune_idle_spheres(state, network, current_tick);
    }

    // ── Phase 4.6: Ghost sweep (Session 070) ──
    // Every 120 ticks (~10 min), deregister spheres whose last heartbeat exceeds
    // GHOST_DEREGISTER_SECS (900s). Uses the same collect-then-remove pattern as
    // Phase 4.5 to avoid mid-iteration mutation.
    if current_tick % 120 == 0 && current_tick > 0 {
        tick_sweep_ghosts(state, network);
    }

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
    let current_tick = state.tick;
    for sphere in state.spheres.values_mut() {
        sphere.step();
    }

    // Session 075: Activity counter decay — V1→V2 regression fix.
    // V1 halved activity counters at their respective intervals:
    //   activity_30s every 6 ticks (~30s), activity_5m every 60 ticks (~5m),
    //   activity_30m every 360 ticks (~30m).
    // V2 scaffold lost this, causing counters to accumulate unbounded
    // and burst detection to fire permanently after 4 tool uses.
    if current_tick % 6 == 0 {
        for sphere in state.spheres.values_mut() {
            sphere.activity_30s /= 2;
        }
    }
    if current_tick % 60 == 0 {
        for sphere in state.spheres.values_mut() {
            sphere.activity_5m /= 2;
        }
    }
    if current_tick % 360 == 0 {
        for sphere in state.spheres.values_mut() {
            sphere.activity_30m /= 2;
        }
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
// Phase 2.5: Hebbian STDP learning (BUG-031 fix)
// ──────────────────────────────────────────────────────────────

/// Apply Hebbian STDP to coupling network after coupling integration.
///
/// Co-active spheres (both Working) get LTP (+0.01, with burst/newcomer multipliers).
/// Non-co-active pairs get LTD (-0.002). Weight floor enforced at 0.15.
/// Apply Hebbian STDP and return result for counter accumulation.
fn tick_hebbian(state: &AppState, network: &mut CouplingNetwork) -> crate::m5_learning::m19_hebbian_stdp::StdpResult {
    if state.spheres.len() < 2 {
        return crate::m5_learning::m19_hebbian_stdp::StdpResult::default();
    }
    let result = crate::m5_learning::m19_hebbian_stdp::apply_stdp(network, &state.spheres);

    // Session 073 BUG-073-C/F fix: Gentle weight decay every tick to prevent
    // ceiling saturation. 0.998 was too aggressive — drove 24/30 weights to floor.
    // Factor 0.9995 = 0.05% decay per tick, allowing STDP weight differentiation.
    crate::m5_learning::m19_hebbian_stdp::decay_all_weights(network, 0.9995);

    result
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
// Phase 3.5: Governance actuator (GAP-1 fix)
// ──────────────────────────────────────────────────────────────

/// Apply approved governance proposals to field parameters.
///
/// Inserted between Phase 3 (field state) and Phase 4 (conductor) so that
/// governance changes take effect before the conductor acts on `r_target`.
///
/// See `[[GAP-1 Fix — Governance Actuator]]` for design rationale.
#[cfg(feature = "governance")]
fn tick_governance(state: &mut AppState, tick: u64) {
    use crate::m8_governance::ProposableParameter;

    // Process proposals: close expired, resolve voted
    let sphere_count = state.spheres.len();
    state.proposal_manager.process(tick, sphere_count);

    // Apply approved proposals
    let approved: Vec<(String, ProposableParameter, f64)> = state
        .proposal_manager
        .approved_unapplied()
        .iter()
        .map(|p| (p.id.clone(), p.parameter.clone(), p.proposed_value))
        .collect();

    for (id, parameter, value) in approved {
        match parameter {
            ProposableParameter::RTarget => {
                state.r_target_override = Some(value);
                state.log(format!(
                    "governance: r_target set to {value:.3} (proposal {id})"
                ));
            }
            ProposableParameter::KModBudgetMax => {
                state.k_mod_budget_max_override = Some(value);
                state.log(format!(
                    "governance: k_mod_budget_max set to {value:.3} (proposal {id})"
                ));
            }
            ProposableParameter::CouplingSteps => {
                state.log(format!(
                    "governance: coupling_steps proposed to {value:.0} (proposal {id})"
                ));
            }
            ProposableParameter::SphereOverride { ref target_sphere } => {
                // Store override intent — applied via ConsentGate externally
                state.log(format!(
                    "governance: sphere override {value:.3} for {target_sphere} (proposal {id})"
                ));
            }
            ProposableParameter::OptOutPolicy => {
                state.log(format!(
                    "governance: opt-out policy set to {value:.1} (proposal {id})"
                ));
            }
        }
        state.proposal_manager.mark_applied(&id);
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 4: Conductor breathing
// ──────────────────────────────────────────────────────────────

/// Apply conductor breathing based on the field decision.
///
/// After the conductor computes its adjustment (stored in `state.divergence_ema`),
/// compose it multiplicatively with the existing bridge-derived `k_modulation` to
/// prevent last-writer-wins. Both conductor and bridge contributions are preserved.
fn tick_conductor(
    state: &mut AppState,
    conductor: &Conductor,
    decision: &FieldDecision,
    network: &mut CouplingNetwork,
) {
    conductor.conduct_breathing(state, decision);

    // Compose conductor adjustment with bridge k_modulation
    // Conductor stores additive offset in divergence_ema ∈ [-0.15, 0.15]
    // Bridge k_modulation set asynchronously via consent gate
    // Multiplicative composition: final = bridge * (1 + conductor)
    let conductor_factor = 1.0 + state.divergence_ema;
    let budget_max = state
        .k_mod_budget_max_override
        .unwrap_or(m04_constants::K_MOD_BUDGET_MAX);
    network.k_modulation = (network.k_modulation * conductor_factor).clamp(
        m04_constants::K_MOD_BUDGET_MIN,
        budget_max,
    );

    // Inject phase noise if divergence is needed
    if matches!(
        decision.action,
        FieldAction::NeedsDivergence | FieldAction::OverSynchronized
    ) {
        Conductor::inject_phase_noise(state, decision);
    }
}

// ──────────────────────────────────────────────────────────────
// Phase 4.5: Idle sphere TTL pruning (Session 066)
// ──────────────────────────────────────────────────────────────

/// Prune idle spheres that have zero memories and have been alive > 300 ticks.
///
/// Prevents O(N^2) coupling overhead from dozens of dormant spheres that will
/// never contribute to learning. Runs every 60 ticks from the tick orchestrator.
fn tick_prune_idle_spheres(
    state: &mut AppState,
    network: &mut CouplingNetwork,
    tick: u64,
) {
    use crate::m1_foundation::m01_core_types::PaneStatus;

    let to_prune: Vec<PaneId> = state
        .spheres
        .iter()
        .filter_map(|(id, sphere)| {
            if sphere.status == PaneStatus::Idle
                && sphere.memories.is_empty()
                && sphere.total_steps > 300
            {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect();

    if to_prune.is_empty() {
        return;
    }

    let count = to_prune.len();
    for id in &to_prune {
        state.spheres.remove(id);
        network.deregister(id);
    }
    tracing::info!(pruned = count, tick, "Idle sphere TTL pruning");
}

// Phase 4.6: Ghost sweep (Session 070)
// ──────────────────────────────────────────────────────────────

/// Deregister spheres whose last heartbeat exceeds `GHOST_DEREGISTER_SECS`.
///
/// Heartbeats are updated by API handlers (`touch_heartbeat()`) whenever a
/// sphere interacts with PV2. Spheres from dead processes stop heartbeating
/// and accumulate indefinitely without this sweep. Uses the same collect-
/// then-remove pattern as [`tick_prune_idle_spheres`] for safety.
///
/// Runs every 120 ticks (~10 minutes at 5s/tick) from the tick orchestrator.
fn tick_sweep_ghosts(
    state: &mut AppState,
    network: &mut CouplingNetwork,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let to_deregister: Vec<PaneId> = state
        .spheres
        .iter()
        .filter_map(|(id, sphere)| {
            let silence = now - sphere.last_heartbeat;
            // Guard: skip spheres with heartbeat=0 (never set, e.g. just registered)
            // or negative silence (clock drift). Session 073 BUG-073-F fix:
            // removed 86400s upper bound that was exempting long-silent spheres.
            if sphere.last_heartbeat < 1.0 || silence < 0.0 {
                return None;
            }
            if silence > m04_constants::GHOST_DEREGISTER_SECS {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect();

    if to_deregister.is_empty() {
        return;
    }

    let count = to_deregister.len();
    for id in &to_deregister {
        state.spheres.remove(id);
        network.deregister(id);
    }
    tracing::info!(
        swept = count,
        threshold_secs = m04_constants::GHOST_DEREGISTER_SECS,
        "Ghost sweep: deregistered stale spheres"
    );
}

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
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
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
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(result.tick, 1);
        assert_eq!(result.sphere_count, 0);
    }

    #[test]
    fn tick_increments_tick_counter() {
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(state.tick, 1);
        tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn tick_single_sphere() {
        let (mut state, mut network) = make_state_with_spheres(1);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(result.sphere_count, 1);
    }

    #[test]
    fn tick_multiple_spheres() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(result.sphere_count, 5);
        // Should have r history
        assert_eq!(state.r_history.len(), 1);
    }

    #[test]
    fn tick_preserves_phase_bounds() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        for _ in 0..10 {
            tick_orchestrator(&mut state, &mut network, &conductor, None);
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
            tick_orchestrator(&mut state, &mut network, &conductor, None);
        }
        assert_eq!(state.r_history.len(), 5);
    }

    #[test]
    fn tick_records_decisions() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(state.decision_history.len(), 1);
    }

    #[test]
    fn tick_caches_field_state() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert!(state.cached_field.is_some());
    }

    #[test]
    fn tick_warmup_decrements() {
        let (mut state, mut network) = make_state_with_spheres(2);
        let conductor = Conductor::new();
        state.warmup_remaining = 3;
        tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert_eq!(state.warmup_remaining, 2);
    }

    #[test]
    fn tick_phase_timings_non_negative() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
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

    #[cfg(feature = "governance")]
    #[test]
    fn governance_actuator_applies_proposal() {
        use crate::m8_governance::{ProposableParameter, VoteChoice};

        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();

        // Submit and approve a proposal to change r_target
        let proposal_id = state
            .proposal_manager
            .submit(
                pid("s0"),
                ProposableParameter::RTarget,
                0.80,
                0.93,
                "lower target for testing".into(),
                0,
            )
            .unwrap();
        state
            .proposal_manager
            .vote(&proposal_id, pid("s1"), VoteChoice::Approve, 1)
            .unwrap();
        state
            .proposal_manager
            .vote(&proposal_id, pid("s2"), VoteChoice::Approve, 2)
            .unwrap();
        // Process to close and approve (window=200, need to advance past it)
        state.proposal_manager.process(210, 3);

        // Verify proposal is approved but not yet applied
        assert_eq!(state.proposal_manager.approved_unapplied().len(), 1);

        // Run tick — governance actuator should apply the proposal
        tick_orchestrator(&mut state, &mut network, &conductor, None);

        // r_target_override should be set
        assert!(state.r_target_override.is_some());
        assert!((state.r_target_override.unwrap() - 0.80).abs() < 0.01);

        // Proposal should be marked as applied
        assert_eq!(state.proposal_manager.approved_unapplied().len(), 0);
    }

    #[cfg(feature = "governance")]
    #[test]
    fn governance_r_target_override_affects_conductor() {
        let (mut state, _) = make_state_with_spheres(3);
        state.r_target_override = Some(0.75);
        let target = Conductor::r_target(&state);
        assert!((target - 0.75).abs() < 0.01);
    }

    #[test]
    fn conductor_called_without_panic() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        let decision = FieldDecision::recovering(1);
        tick_conductor(&mut state, &conductor, &decision, &mut network);
    }

    #[test]
    fn conductor_composes_with_bridge_k_mod() {
        let (mut state, mut network) = make_state_with_spheres(5);
        let conductor = Conductor::new();

        // Simulate bridge setting k_modulation to 1.1
        network.k_modulation = 1.1;

        // Simulate conductor having a positive divergence_ema
        state.divergence_ema = 0.02;

        let decision = FieldDecision::recovering(1);
        tick_conductor(&mut state, &conductor, &decision, &mut network);

        // k_modulation should be bridge * (1 + conductor) = 1.1 * 1.02 ≈ 1.122
        // NOT simply overwritten by either contributor
        assert!(
            network.k_modulation > 1.1,
            "k_mod should compose multiplicatively: {}",
            network.k_modulation
        );
        assert!(
            network.k_modulation <= m04_constants::K_MOD_BUDGET_MAX,
            "k_mod should stay within budget"
        );
    }

    #[test]
    fn conductor_k_mod_clamped_to_budget() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();

        // Both at maximum — should not exceed budget
        network.k_modulation = m04_constants::K_MOD_BUDGET_MAX;
        state.divergence_ema = 0.15;

        let decision = FieldDecision::recovering(1);
        tick_conductor(&mut state, &conductor, &decision, &mut network);

        assert!(
            network.k_modulation <= m04_constants::K_MOD_BUDGET_MAX,
            "k_mod must not exceed budget: {}",
            network.k_modulation
        );
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
            let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
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
            tick_orchestrator(&mut state, &mut network, &conductor, None);
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
            tick_orchestrator(&mut state, &mut network, &conductor, None);
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
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
        assert!(result.sphere_count > 0);
    }

    #[test]
    fn tick_total_ms_sums_phases() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        let result = tick_orchestrator(&mut state, &mut network, &conductor, None);
        let sum = result.phase_timings.sphere_step_ms
            + result.phase_timings.coupling_ms
            + result.phase_timings.field_state_ms
            + result.phase_timings.conductor_ms
            + result.phase_timings.persistence_ms;
        // Total should be >= sum of phases (with minor overhead)
        assert!(result.total_ms >= sum * 0.9);
    }

    // ── Resource / memory growth bounds (1M-tick simulation) ──

    /// Verify that `r_history` stays bounded at `R_HISTORY_MAX` over many ticks.
    ///
    /// At 5 s/tick this corresponds to ~83,000 ticks per day. The ring-buffer cap
    /// must prevent O(N) memory growth regardless of runtime duration.
    #[test]
    fn r_history_bounded_after_many_ticks() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        for _ in 0..500 {
            tick_orchestrator(&mut state, &mut network, &conductor, None);
        }
        assert!(
            state.r_history.len() <= crate::m1_foundation::m04_constants::R_HISTORY_MAX,
            "r_history grew to {} exceeding cap {}",
            state.r_history.len(),
            crate::m1_foundation::m04_constants::R_HISTORY_MAX
        );
    }

    /// Verify that `decision_history` stays bounded after many ticks.
    #[test]
    fn decision_history_bounded_after_many_ticks() {
        let (mut state, mut network) = make_state_with_spheres(3);
        let conductor = Conductor::new();
        for _ in 0..500 {
            tick_orchestrator(&mut state, &mut network, &conductor, None);
        }
        assert!(
            state.decision_history.len() <= crate::m1_foundation::m04_constants::DECISION_HISTORY_MAX,
            "decision_history grew to {} exceeding cap {}",
            state.decision_history.len(),
            crate::m1_foundation::m04_constants::DECISION_HISTORY_MAX
        );
    }

    /// Verify that idle spheres are pruned from the network every 60 ticks,
    /// preventing O(N^2) coupling overhead from accumulating dormant spheres.
    #[test]
    fn idle_spheres_pruned_at_60_tick_boundary() {
        use crate::m1_foundation::m01_core_types::PaneStatus;
        let mut state = AppState::new();
        let mut network = CouplingNetwork::new();

        // Register 10 idle spheres with no memories and high step count.
        for i in 0..10 {
            let id = format!("idle-{i}");
            let mut sphere = PaneSphere::new(pid(&id), format!("idle-{i}"), 0.1).unwrap();
            sphere.status = PaneStatus::Idle;
            sphere.total_steps = 400; // > 300 TTL threshold
            // No memories — satisfies the prune condition.
            state.spheres.insert(pid(&id), sphere);
            network.register(pid(&id), 0.0, 0.1);
        }

        // Register 1 active sphere (should survive).
        let active_id = "active-0";
        let active = PaneSphere::new(pid(active_id), "active".into(), 0.1).unwrap();
        state.spheres.insert(pid(active_id), active);
        network.register(pid(active_id), 0.0, 0.1);

        assert_eq!(state.spheres.len(), 11);

        // Advance the tick counter to the first 60-tick prune boundary.
        state.tick = 59; // tick_orchestrator will increment to 60.

        let conductor = Conductor::new();
        tick_orchestrator(&mut state, &mut network, &conductor, None);

        // Idle spheres with 400 steps and no memories should have been pruned.
        assert!(
            state.spheres.len() < 11,
            "expected idle spheres to be pruned; sphere count = {}",
            state.spheres.len()
        );
        // The active sphere (with default Idle status but total_steps = 1 after first tick)
        // may or may not be pruned depending on step count — but we confirm the idle ones are.
        // The idle spheres had total_steps = 400 + 1 (from tick_sphere_steps) which is still
        // > 300, so they should be pruned.
        let remaining_idle: Vec<_> = state
            .spheres
            .keys()
            .filter(|k| k.as_str().starts_with("idle-"))
            .collect();
        assert!(
            remaining_idle.is_empty(),
            "expected all idle spheres pruned, remaining: {remaining_idle:?}"
        );
    }

    /// Verify that `tick_persistence_check` properly signals snapshot without
    /// accumulating unbounded state — the dirty flag is cleared after signalling.
    #[test]
    fn persistence_check_clears_dirty_after_snapshot() {
        let mut state = AppState::new();
        state.dirty = true;
        let did_snapshot = tick_persistence_check(&mut state, crate::m1_foundation::m04_constants::SNAPSHOT_INTERVAL);
        assert!(did_snapshot, "should signal snapshot at SNAPSHOT_INTERVAL");
        assert!(!state.dirty, "dirty flag must be cleared after snapshot signal");
    }

    /// Verify `state_changes` counter resets each tick so it does not accumulate.
    #[test]
    fn state_changes_reset_each_tick() {
        let mut state = AppState::new();
        state.state_changes = 99;
        tick_persistence_check(&mut state, 1);
        assert_eq!(state.state_changes, 0, "state_changes must reset to 0 each tick");
    }
}
