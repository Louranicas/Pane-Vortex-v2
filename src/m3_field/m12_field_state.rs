//! # M12: Field State & Decision Engine
//!
//! `FieldState` holds the Kuramoto field snapshot. `FieldDecision` is the
//! conductor's input: what action should be taken based on the current field.
//!
//! ## Layer: L3 (Field)
//! ## Module: M12
//! ## Dependencies: L1 (M01, M04), L3 (M11, M13)
//!
//! ## Decision Priority Chain
//! `HasBlockedAgents` > `NeedsCoherence` > `NeedsDivergence` > `IdleFleet` > `FreshFleet` > `OverSynchronized` > `Stable`

use std::collections::{HashMap, VecDeque};
use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::{
    m01_core_types::{
        FieldAction, FleetMode, OrderParameter, PaneId, PaneStatus, RTrend,
    },
    m04_constants,
};
use super::m11_sphere::PaneSphere;
use super::m13_chimera::{ChimeraRouting, ChimeraState};

// ──────────────────────────────────────────────────────────────
// Field state types
// ──────────────────────────────────────────────────────────────

/// Simplified harmonic decomposition of phase distribution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarmonicSpectrum {
    /// L=0: overall health (mean activation).
    pub l0_monopole: f64,
    /// L=1: polarization (dipole moment).
    pub l1_dipole: f64,
    /// L=2: fragmentation (quadrupole).
    pub l2_quadrupole: f64,
}

/// Tunnel: buoy overlap between two spheres enabling resonant communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tunnel {
    /// First sphere.
    pub sphere_a: PaneId,
    /// Second sphere.
    pub sphere_b: PaneId,
    /// Buoy label in sphere A.
    pub buoy_a_label: String,
    /// Buoy label in sphere B.
    pub buoy_b_label: String,
    /// Overlap measure (0..1 based on angular proximity).
    pub overlap: f64,
    /// Tool semantics of buoy A.
    pub semantic_a: String,
    /// Tool semantics of buoy B.
    pub semantic_b: String,
}

/// Maximum tunnels to track per tick.
const TUNNEL_MAX: usize = 100;

/// Complete field state — the cognitive topology of the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldState {
    /// Global order parameter.
    pub order_parameter: OrderParameter,
    /// Chimera state (sync/desync clusters).
    pub chimera: ChimeraState,
    /// Harmonic decomposition.
    pub harmonics: HarmonicSpectrum,
    /// Active tunnels (buoy overlaps between spheres).
    pub tunnels: Vec<Tunnel>,
    /// Number of active spheres.
    pub sphere_count: usize,
    /// Total memories across all spheres.
    pub total_memories: usize,
    /// Current tick count.
    pub tick: u64,
}

impl FieldState {
    /// Compute field state from sphere map.
    #[must_use]
    pub fn compute(
        spheres: &HashMap<PaneId, PaneSphere>,
        k_modulation: f64,
        tick: u64,
    ) -> Self {
        let phases: HashMap<PaneId, f64> = spheres
            .iter()
            .map(|(id, s)| (id.clone(), s.phase))
            .collect();

        let order_parameter = compute_order_parameter(&phases);
        let chimera = ChimeraState::detect(&phases, k_modulation);
        let harmonics = compute_harmonics(&phases);
        let tunnels = detect_tunnels(spheres);
        let total_memories = spheres.values().map(|s| s.memories.len()).sum();

        Self {
            order_parameter,
            chimera,
            harmonics,
            tunnels,
            sphere_count: spheres.len(),
            total_memories,
            tick,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Field decision
// ──────────────────────────────────────────────────────────────

/// Pre-computed decision packet for `/field/decision`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDecision {
    /// Recommended action.
    pub action: FieldAction,
    /// Current order parameter r.
    pub r: f64,
    /// r trend over rolling window.
    pub r_trend: RTrend,
    /// Idle sphere IDs.
    pub idle_spheres: Vec<PaneId>,
    /// Blocked sphere IDs.
    pub blocked_spheres: Vec<PaneId>,
    /// Working sphere IDs.
    pub working_spheres: Vec<PaneId>,
    /// Routing recommendations.
    pub routing: ChimeraRouting,
    /// Active tunnel count.
    pub tunnel_count: usize,
    /// Strongest tunnel (if any).
    pub strongest_tunnel: Option<Tunnel>,
    /// Current tick.
    pub tick: u64,
    /// Coherence pressure from conductor.
    pub coherence_pressure: f64,
    /// Divergence pressure from conductor.
    pub divergence_pressure: f64,
    /// Fleet mode.
    pub fleet_mode: FleetMode,
}

impl FieldDecision {
    /// Compute field decision from current state.
    #[must_use]
    pub fn compute(
        spheres: &HashMap<PaneId, PaneSphere>,
        field: &FieldState,
        r_history: &VecDeque<f64>,
        tick: u64,
    ) -> Self {
        let r = field.order_parameter.r;
        let n = spheres.len();
        let fleet_mode = FleetMode::from_count(n);

        // Categorize spheres
        let mut idle_spheres = Vec::new();
        let mut blocked_spheres = Vec::new();
        let mut working_spheres = Vec::new();

        for (id, sphere) in spheres {
            match sphere.status {
                PaneStatus::Idle => idle_spheres.push(id.clone()),
                PaneStatus::Blocked => blocked_spheres.push(id.clone()),
                PaneStatus::Working => working_spheres.push(id.clone()),
                PaneStatus::Complete => {}
            }
        }

        let r_trend = compute_r_trend(r_history);
        let routing = field.chimera.routing();
        let strongest_tunnel = field
            .tunnels
            .iter()
            .max_by(|a, b| {
                a.overlap
                    .partial_cmp(&b.overlap)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        // Decision priority chain
        let action = decide_action(
            r,
            r_trend,
            &idle_spheres,
            &blocked_spheres,
            n,
            fleet_mode,
        );

        // Coherence/divergence pressure (for conductor)
        let (coherence_pressure, divergence_pressure) = compute_pressures(r, r_trend);

        Self {
            action,
            r,
            r_trend,
            idle_spheres,
            blocked_spheres,
            working_spheres,
            routing,
            tunnel_count: field.tunnels.len(),
            strongest_tunnel,
            tick,
            coherence_pressure,
            divergence_pressure,
            fleet_mode,
        }
    }

    /// Create a "recovering" decision (used during cooldown after divergence kick).
    #[must_use]
    pub fn recovering(tick: u64) -> Self {
        Self {
            action: FieldAction::Recovering,
            r: 0.0,
            r_trend: RTrend::Stable,
            idle_spheres: Vec::new(),
            blocked_spheres: Vec::new(),
            working_spheres: Vec::new(),
            routing: ChimeraRouting::default(),
            tunnel_count: 0,
            strongest_tunnel: None,
            tick,
            coherence_pressure: 0.0,
            divergence_pressure: 0.0,
            fleet_mode: FleetMode::Solo,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Helper functions
// ──────────────────────────────────────────────────────────────

/// Compute global order parameter from phase map.
fn compute_order_parameter(phases: &HashMap<PaneId, f64>) -> OrderParameter {
    if phases.is_empty() {
        return OrderParameter::default();
    }

    let (re, im) = phases
        .values()
        .map(|&phi| (phi.cos(), phi.sin()))
        .fold((0.0, 0.0), |(ar, ai), (r, i)| (ar + r, ai + i));

    #[allow(clippy::cast_precision_loss)]
    let n = phases.len() as f64;
    // F01: clamp to [0.0, 1.0] — fp accumulation can push hypot above 1.0 after
    // millions of ticks. r > 1.0 is unphysical and corrupts compute_pressures().
    let r = (re / n).hypot(im / n).clamp(0.0, 1.0);
    let psi = (im / n).atan2(re / n).rem_euclid(TAU);

    OrderParameter { r, psi }
}

/// Compute harmonic spectrum from phase distribution.
fn compute_harmonics(phases: &HashMap<PaneId, f64>) -> HarmonicSpectrum {
    if phases.is_empty() {
        return HarmonicSpectrum::default();
    }

    #[allow(clippy::cast_precision_loss)]
    let n = phases.len() as f64;

    // L=0: monopole (mean of cos — 1.0 when all phases = 0)
    let l0 = phases.values().map(|phi| phi.cos()).sum::<f64>() / n;

    // L=1: dipole (order parameter magnitude)
    let (re, im) = phases
        .values()
        .map(|&phi| (phi.cos(), phi.sin()))
        .fold((0.0, 0.0), |(ar, ai), (r, i)| (ar + r, ai + i));
    let l1 = (re / n).hypot(im / n).clamp(0.0, 1.0); // F01: same reason as r

    // L=2: quadrupole (cos(2*phi) average)
    let l2 = phases.values().map(|phi| (2.0 * phi).cos()).sum::<f64>() / n;

    HarmonicSpectrum {
        l0_monopole: l0,
        l1_dipole: l1,
        l2_quadrupole: l2.abs(),
    }
}

/// Detect tunnels between sphere buoy pairs.
fn detect_tunnels(spheres: &HashMap<PaneId, PaneSphere>) -> Vec<Tunnel> {
    let mut tunnels = Vec::new();
    let sphere_list: Vec<(&PaneId, &PaneSphere)> = spheres.iter().collect();

    for i in 0..sphere_list.len() {
        for j in (i + 1)..sphere_list.len() {
            let (id_a, sphere_a) = sphere_list[i];
            let (id_b, sphere_b) = sphere_list[j];

            let buoys_a = sphere_a.buoy_positions();
            let buoys_b = sphere_b.buoy_positions();

            for (label_a, pos_a) in &buoys_a {
                for (label_b, pos_b) in &buoys_b {
                    let dist = pos_a.angular_distance(*pos_b);
                    if dist < m04_constants::TUNNEL_THRESHOLD {
                        let overlap = 1.0 - dist / m04_constants::TUNNEL_THRESHOLD;
                        tunnels.push(Tunnel {
                            sphere_a: id_a.clone(),
                            sphere_b: id_b.clone(),
                            buoy_a_label: label_a.clone(),
                            buoy_b_label: label_b.clone(),
                            overlap,
                            semantic_a: label_a.clone(),
                            semantic_b: label_b.clone(),
                        });
                    }
                }
            }
        }
    }

    // Sort by overlap descending, cap at TUNNEL_MAX
    tunnels.sort_by(|a, b| {
        b.overlap
            .partial_cmp(&a.overlap)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    tunnels.truncate(TUNNEL_MAX);
    tunnels
}

/// Compute r trend from rolling history.
fn compute_r_trend(r_history: &VecDeque<f64>) -> RTrend {
    if r_history.len() < 3 {
        return RTrend::Stable;
    }

    #[allow(clippy::cast_precision_loss)]
    let recent_count = r_history.iter().rev().take(5).count().max(1) as f64;
    let recent_avg: f64 = r_history.iter().rev().take(5).sum::<f64>() / recent_count;

    // Only compare against older history if enough samples exist.
    // Using .max(1) when count is 0 would produce a false older_avg of 0.0,
    // triggering a spurious Rising trend whenever recent_avg > 0.
    let older_count = r_history.iter().rev().skip(5).take(5).count();
    if older_count == 0 {
        return RTrend::Stable;
    }
    #[allow(clippy::cast_precision_loss)]
    let older_avg: f64 =
        r_history.iter().rev().skip(5).take(5).sum::<f64>() / older_count as f64;

    let delta = recent_avg - older_avg;

    if delta < m04_constants::R_FALLING_THRESHOLD {
        RTrend::Falling
    } else if delta > m04_constants::R_RISING_THRESHOLD {
        RTrend::Rising
    } else {
        RTrend::Stable
    }
}

/// Decision priority chain.
fn decide_action(
    r: f64,
    r_trend: RTrend,
    idle_spheres: &[PaneId],
    blocked_spheres: &[PaneId],
    n: usize,
    fleet_mode: FleetMode,
) -> FieldAction {
    // Priority 1: Blocked agents
    if !blocked_spheres.is_empty() {
        return FieldAction::HasBlockedAgents;
    }

    // Solo mode: limited decision making
    if fleet_mode == FleetMode::Solo {
        return FieldAction::FreshFleet;
    }

    // Priority 2: Needs coherence (r falling below threshold)
    if r < m04_constants::R_COHERENCE_THRESHOLD
        && r_trend == RTrend::Falling
        && n >= 2
    {
        return FieldAction::NeedsCoherence;
    }

    // Priority 3: Over-synchronized (r > 0.99)
    if r > 0.99 && n >= 2 {
        return FieldAction::OverSynchronized;
    }

    // Priority 4: Needs divergence (r high, mostly idle)
    if r > m04_constants::R_HIGH_THRESHOLD && n >= 2 {
        #[allow(clippy::cast_precision_loss)]
        let idle_ratio = idle_spheres.len() as f64 / n.max(1) as f64;
        if idle_ratio > m04_constants::IDLE_RATIO_THRESHOLD {
            return FieldAction::NeedsDivergence;
        }
    }

    // Priority 5: Idle fleet
    if n >= 2 {
        #[allow(clippy::cast_precision_loss)]
        let idle_ratio = idle_spheres.len() as f64 / n.max(1) as f64;
        if idle_ratio > m04_constants::IDLE_RATIO_THRESHOLD {
            return FieldAction::IdleFleet;
        }
    }

    // Priority 6: Fresh fleet (few steps)
    if n >= 2 && r < 0.1 {
        return FieldAction::FreshFleet;
    }

    FieldAction::Stable
}

/// Compute coherence/divergence pressure signals.
fn compute_pressures(r: f64, r_trend: RTrend) -> (f64, f64) {
    let coherence = if r_trend == RTrend::Falling {
        (m04_constants::R_TARGET_BASE - r).max(0.0)
    } else {
        0.0
    };

    let divergence = if r > m04_constants::R_HIGH_THRESHOLD {
        (r - m04_constants::R_TARGET_BASE).max(0.0)
    } else {
        0.0
    };

    (coherence, divergence)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::m3_field::m11_sphere::PaneSphere;
    use std::f64::consts::PI;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    fn make_spheres(entries: &[(&str, f64)]) -> HashMap<PaneId, PaneSphere> {
        entries
            .iter()
            .map(|(id, phase)| {
                let mut s = PaneSphere::new(pid(id), "test".into(), 0.1).unwrap();
                s.phase = *phase;
                (pid(id), s)
            })
            .collect()
    }

    // ── Order parameter ──

    #[test]
    fn order_parameter_empty_is_zero() {
        let phases = HashMap::new();
        let op = compute_order_parameter(&phases);
        assert_relative_eq!(op.r, 0.0);
    }

    #[test]
    fn order_parameter_single_sphere_is_one() {
        let mut phases = HashMap::new();
        phases.insert(pid("a"), 1.0);
        let op = compute_order_parameter(&phases);
        assert_relative_eq!(op.r, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn order_parameter_identical_phases_is_one() {
        let mut phases = HashMap::new();
        phases.insert(pid("a"), 1.0);
        phases.insert(pid("b"), 1.0);
        phases.insert(pid("c"), 1.0);
        let op = compute_order_parameter(&phases);
        assert_relative_eq!(op.r, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn order_parameter_opposite_phases_is_zero() {
        let mut phases = HashMap::new();
        phases.insert(pid("a"), 0.0);
        phases.insert(pid("b"), PI);
        let op = compute_order_parameter(&phases);
        assert_relative_eq!(op.r, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn order_parameter_r_bounded() {
        let mut phases = HashMap::new();
        for i in 0..20 {
            #[allow(clippy::cast_precision_loss)]
            let phase = (i as f64 / 20.0) * TAU;
            phases.insert(pid(&format!("s{i}")), phase);
        }
        let op = compute_order_parameter(&phases);
        assert!(op.r >= 0.0 && op.r <= 1.0);
    }

    // ── Harmonics ──

    #[test]
    fn harmonics_empty_is_default() {
        let h = compute_harmonics(&HashMap::new());
        assert_relative_eq!(h.l0_monopole, 0.0);
    }

    #[test]
    fn harmonics_all_zero_phase() {
        let mut phases = HashMap::new();
        phases.insert(pid("a"), 0.0);
        phases.insert(pid("b"), 0.0);
        let h = compute_harmonics(&phases);
        assert_relative_eq!(h.l0_monopole, 1.0, epsilon = 1e-10);
        assert_relative_eq!(h.l1_dipole, 1.0, epsilon = 1e-10);
    }

    // ── R trend ──

    #[test]
    fn r_trend_stable_with_few_samples() {
        let history: VecDeque<f64> = vec![0.5, 0.5].into();
        assert_eq!(compute_r_trend(&history), RTrend::Stable);
    }

    #[test]
    fn r_trend_rising() {
        let history: VecDeque<f64> = (0..10).map(|i| 0.3 + i as f64 * 0.05).collect();
        assert_eq!(compute_r_trend(&history), RTrend::Rising);
    }

    #[test]
    fn r_trend_falling() {
        let history: VecDeque<f64> = (0..10).map(|i| 0.9 - i as f64 * 0.05).collect();
        assert_eq!(compute_r_trend(&history), RTrend::Falling);
    }

    #[test]
    fn r_trend_stable_with_constant() {
        let history: VecDeque<f64> = vec![0.5; 10].into();
        assert_eq!(compute_r_trend(&history), RTrend::Stable);
    }

    #[test]
    fn r_trend_empty_is_stable() {
        let history: VecDeque<f64> = VecDeque::new();
        assert_eq!(compute_r_trend(&history), RTrend::Stable);
    }

    // ── Decision engine ──

    #[test]
    fn decision_blocked_agents_highest_priority() {
        let action = decide_action(0.5, RTrend::Stable, &[], &[pid("blocked")], 3, FleetMode::Small);
        assert_eq!(action, FieldAction::HasBlockedAgents);
    }

    #[test]
    fn decision_solo_mode_fresh_fleet() {
        let action = decide_action(0.0, RTrend::Stable, &[], &[], 1, FleetMode::Solo);
        assert_eq!(action, FieldAction::FreshFleet);
    }

    #[test]
    fn decision_needs_coherence() {
        let action = decide_action(
            0.3,
            RTrend::Falling,
            &[],
            &[],
            3,
            FleetMode::Small,
        );
        assert_eq!(action, FieldAction::NeedsCoherence);
    }

    #[test]
    fn decision_over_synchronized() {
        let action = decide_action(0.995, RTrend::Stable, &[], &[], 5, FleetMode::Full);
        assert_eq!(action, FieldAction::OverSynchronized);
    }

    #[test]
    fn decision_needs_divergence() {
        let idle = vec![pid("a"), pid("b"), pid("c"), pid("d")];
        let action = decide_action(0.85, RTrend::Stable, &idle, &[], 5, FleetMode::Full);
        assert_eq!(action, FieldAction::NeedsDivergence);
    }

    #[test]
    fn decision_idle_fleet() {
        let idle = vec![pid("a"), pid("b"), pid("c"), pid("d")];
        let action = decide_action(0.5, RTrend::Stable, &idle, &[], 5, FleetMode::Full);
        assert_eq!(action, FieldAction::IdleFleet);
    }

    #[test]
    fn decision_fresh_fleet() {
        let action = decide_action(0.05, RTrend::Stable, &[], &[], 3, FleetMode::Small);
        assert_eq!(action, FieldAction::FreshFleet);
    }

    #[test]
    fn decision_stable() {
        let action = decide_action(0.7, RTrend::Stable, &[], &[], 5, FleetMode::Full);
        assert_eq!(action, FieldAction::Stable);
    }

    // ── Pressures ──

    #[test]
    fn coherence_pressure_when_falling() {
        let (c, _) = compute_pressures(0.5, RTrend::Falling);
        assert!(c > 0.0);
    }

    #[test]
    fn coherence_pressure_zero_when_stable() {
        let (c, _) = compute_pressures(0.5, RTrend::Stable);
        assert_relative_eq!(c, 0.0);
    }

    #[test]
    fn divergence_pressure_when_r_high() {
        let (_, d) = compute_pressures(0.95, RTrend::Stable);
        assert!(d > 0.0);
    }

    #[test]
    fn divergence_pressure_zero_when_r_low() {
        let (_, d) = compute_pressures(0.5, RTrend::Stable);
        assert_relative_eq!(d, 0.0);
    }

    // ── Tunnel detection ──

    #[test]
    fn tunnels_empty_with_no_spheres() {
        let spheres = HashMap::new();
        let tunnels = detect_tunnels(&spheres);
        assert!(tunnels.is_empty());
    }

    #[test]
    fn tunnels_empty_with_one_sphere() {
        let spheres = make_spheres(&[("a", 0.0)]);
        let tunnels = detect_tunnels(&spheres);
        assert!(tunnels.is_empty());
    }

    #[test]
    fn tunnels_detected_between_close_buoys() {
        // Two spheres with default buoys at the same positions should have tunnels
        let spheres = make_spheres(&[("a", 0.0), ("b", 0.0)]);
        let tunnels = detect_tunnels(&spheres);
        assert!(!tunnels.is_empty(), "spheres with same buoy positions should have tunnels");
    }

    #[test]
    fn tunnels_overlap_bounded() {
        let spheres = make_spheres(&[("a", 0.0), ("b", 0.0)]);
        let tunnels = detect_tunnels(&spheres);
        for t in &tunnels {
            assert!(t.overlap >= 0.0 && t.overlap <= 1.0);
        }
    }

    #[test]
    fn tunnels_capped_at_max() {
        // With many spheres, tunnels should be capped
        let mut spheres = HashMap::new();
        for i in 0..50 {
            let mut s = PaneSphere::new(pid(&format!("s{i}")), "test".into(), 0.1).unwrap();
            s.phase = 0.0;
            spheres.insert(pid(&format!("s{i}")), s);
        }
        let tunnels = detect_tunnels(&spheres);
        assert!(tunnels.len() <= TUNNEL_MAX);
    }

    // ── FieldState compute ──

    #[test]
    fn field_state_empty() {
        let spheres = HashMap::new();
        let fs = FieldState::compute(&spheres, 1.0, 0);
        assert_eq!(fs.sphere_count, 0);
        assert_relative_eq!(fs.order_parameter.r, 0.0);
    }

    #[test]
    fn field_state_single_sphere() {
        let spheres = make_spheres(&[("a", 0.0)]);
        let fs = FieldState::compute(&spheres, 1.0, 10);
        assert_eq!(fs.sphere_count, 1);
        assert_eq!(fs.tick, 10);
    }

    #[test]
    fn field_state_multiple_spheres() {
        let spheres = make_spheres(&[("a", 0.0), ("b", 0.1), ("c", 0.2)]);
        let fs = FieldState::compute(&spheres, 1.0, 5);
        assert_eq!(fs.sphere_count, 3);
        assert!(fs.order_parameter.r > 0.0);
    }

    // ── FieldDecision compute ──

    #[test]
    fn field_decision_empty_spheres() {
        let spheres = HashMap::new();
        let fs = FieldState::compute(&spheres, 1.0, 0);
        let r_history: VecDeque<f64> = VecDeque::new();
        let fd = FieldDecision::compute(&spheres, &fs, &r_history, 0);
        assert_eq!(fd.fleet_mode, FleetMode::Solo);
    }

    #[test]
    fn field_decision_categorizes_spheres() {
        let mut spheres = make_spheres(&[("a", 0.0), ("b", 0.5), ("c", 1.0)]);
        spheres.get_mut(&pid("a")).unwrap().status = PaneStatus::Working;
        spheres.get_mut(&pid("b")).unwrap().status = PaneStatus::Idle;
        spheres.get_mut(&pid("c")).unwrap().status = PaneStatus::Blocked;

        let fs = FieldState::compute(&spheres, 1.0, 0);
        let r_history: VecDeque<f64> = VecDeque::new();
        let fd = FieldDecision::compute(&spheres, &fs, &r_history, 0);

        assert_eq!(fd.working_spheres.len(), 1);
        assert_eq!(fd.idle_spheres.len(), 1);
        assert_eq!(fd.blocked_spheres.len(), 1);
    }

    #[test]
    fn field_decision_blocked_takes_priority() {
        let mut spheres = make_spheres(&[("a", 0.0), ("b", 0.5)]);
        spheres.get_mut(&pid("b")).unwrap().status = PaneStatus::Blocked;

        let fs = FieldState::compute(&spheres, 1.0, 0);
        let r_history: VecDeque<f64> = VecDeque::new();
        let fd = FieldDecision::compute(&spheres, &fs, &r_history, 0);

        assert_eq!(fd.action, FieldAction::HasBlockedAgents);
    }

    // ── FieldDecision::recovering ──

    #[test]
    fn recovering_decision() {
        let fd = FieldDecision::recovering(42);
        assert_eq!(fd.action, FieldAction::Recovering);
        assert_eq!(fd.tick, 42);
        assert_eq!(fd.fleet_mode, FleetMode::Solo);
    }

    // ── Serde roundtrips ──

    #[test]
    fn field_state_serde_roundtrip() {
        let spheres = make_spheres(&[("a", 0.0), ("b", 1.0)]);
        let fs = FieldState::compute(&spheres, 1.0, 100);
        let json = serde_json::to_string(&fs).unwrap();
        let back: FieldState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sphere_count, fs.sphere_count);
        assert_eq!(back.tick, fs.tick);
    }

    #[test]
    fn field_decision_serde_roundtrip() {
        let fd = FieldDecision::recovering(55);
        let json = serde_json::to_string(&fd).unwrap();
        let back: FieldDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.action, FieldAction::Recovering);
        assert_eq!(back.tick, 55);
    }

    #[test]
    fn harmonic_spectrum_serde_roundtrip() {
        let h = HarmonicSpectrum {
            l0_monopole: 0.5,
            l1_dipole: 0.8,
            l2_quadrupole: 0.2,
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: HarmonicSpectrum = serde_json::from_str(&json).unwrap();
        assert_relative_eq!(back.l0_monopole, 0.5);
    }

    #[test]
    fn tunnel_serde_roundtrip() {
        let t = Tunnel {
            sphere_a: pid("a"),
            sphere_b: pid("b"),
            buoy_a_label: "primary".into(),
            buoy_b_label: "secondary".into(),
            overlap: 0.75,
            semantic_a: "Read".into(),
            semantic_b: "Write".into(),
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: Tunnel = serde_json::from_str(&json).unwrap();
        assert_relative_eq!(back.overlap, 0.75);
    }

    // ── Strongest tunnel ──

    #[test]
    fn strongest_tunnel_selected() {
        let spheres = make_spheres(&[("a", 0.0), ("b", 0.0)]);
        let fs = FieldState::compute(&spheres, 1.0, 0);
        let r_history: VecDeque<f64> = VecDeque::new();
        let fd = FieldDecision::compute(&spheres, &fs, &r_history, 0);
        if fd.tunnel_count > 0 {
            assert!(fd.strongest_tunnel.is_some());
        }
    }

    // ── Numerical stability regression tests (F01) ──

    /// F01: order parameter r must never exceed 1.0 regardless of phase distribution.
    #[test]
    fn order_parameter_r_bounded_above_by_one() {
        let mut phases = HashMap::new();
        // Pack 100 oscillators at nearly identical phases to maximise r
        for i in 0..100 {
            let phase = 1e-10 * i as f64;
            phases.insert(pid(&format!("s{i}")), phase);
        }
        let op = compute_order_parameter(&phases);
        assert!(op.r <= 1.0, "r={} must not exceed 1.0", op.r);
        assert!(op.r >= 0.0, "r must be non-negative");
    }

    /// F01: harmonics l1_dipole is bounded by [0, 1].
    #[test]
    fn harmonics_l1_dipole_bounded() {
        let mut phases = HashMap::new();
        for i in 0..100 {
            let phase = 1e-12 * i as f64;
            phases.insert(pid(&format!("s{i}")), phase);
        }
        let h = compute_harmonics(&phases);
        assert!(h.l1_dipole <= 1.0, "l1_dipole={} must not exceed 1.0", h.l1_dipole);
        assert!(h.l1_dipole >= 0.0);
    }
}
