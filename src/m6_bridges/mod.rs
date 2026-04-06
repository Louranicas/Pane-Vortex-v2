//! # Layer 6: Bridges
//!
//! External service bridges using raw TCP HTTP (fire-and-forget pattern).
//! Depends on L1 (Foundation), L3 (Field).
//!
//! ## Design Constraints: C1 C8 C14
//! - C8: ALL bridges route through `consent_gated_k_adjustment()` (M28)
//! - C14: Fire-and-forget for writes (`tokio::spawn`, no blocking)
//!
//! ## Bridge Pattern
//! `TcpStream::connect(addr) → write HTTP → read response → parse JSON`
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m22_synthex_bridge` | ~300 | SYNTHEX :8090 thermal k_adjustment |
//! | `m23_nexus_bridge` | ~400 | SAN-K7 :8100 nested Kuramoto, strategy |
//! | `m24_me_bridge` | ~250 | ME :8080 observer fitness (BUG-008!) |
//! | `m25_povm_bridge` | ~200 | POVM :8125 snapshots + Hebbian weights |
//! | `m26_rm_bridge` | ~150 | RM :8130 TSV conductor decisions |
//! | `m27_vms_bridge` | ~150 | VMS :8120 field memory seeding |
//! | `m28_consent_gate` | ~200 | consent_gated_k_adjustment(), k_mod budget |

pub mod m22_synthex_bridge;
pub mod m23_nexus_bridge;
pub mod m24_me_bridge;
pub mod m25_povm_bridge;
pub mod m26_rm_bridge;
pub mod m27_vms_bridge;
pub mod m28_consent_gate;

use std::collections::HashMap;

use crate::m1_foundation::{
    m01_core_types::{BridgeAdjustments, BridgeStaleness, PaneId},
    m03_config::BridgesConfig,
    m04_constants,
};
use crate::m3_field::{m11_sphere::PaneSphere, m15_app_state::AppState};
use crate::m4_coupling::m16_coupling_network::CouplingNetwork;

use m22_synthex_bridge::SynthexBridge;
use m23_nexus_bridge::NexusBridge;
use m24_me_bridge::MeBridge;
use m25_povm_bridge::PovmBridge;
use m26_rm_bridge::RmBridge;
use m27_vms_bridge::VmsBridge;
use m28_consent_gate::{ConsentGate, SphereConsent};

// Import Bridgeable trait to access is_stale() on bridge structs
use crate::m1_foundation::m05_traits::Bridgeable;

// ──────────────────────────────────────────────────────────────
// BridgeSet
// ──────────────────────────────────────────────────────────────

/// Aggregates all 6 external service bridges + the consent gate.
///
/// Created once in `main()`, wrapped in `Arc`, and shared with the tick loop
/// and bridge polling tasks. Each bridge uses interior mutability for its
/// cached state, so `&self` suffices for all operations.
#[derive(Debug)]
pub struct BridgeSet {
    /// SYNTHEX thermal bridge (:8090).
    pub synthex: SynthexBridge,
    /// SAN-K7 nexus bridge (:8100).
    pub nexus: NexusBridge,
    /// Maintenance Engine bridge (:8080).
    pub me: MeBridge,
    /// POVM persistence bridge (:8125).
    pub povm: PovmBridge,
    /// Reasoning Memory bridge (:8130).
    pub rm: RmBridge,
    /// Vortex Memory System bridge (:8120).
    pub vms: VmsBridge,
    /// Consent gate — ALL adjustments route through here.
    pub consent_gate: ConsentGate,
}

impl BridgeSet {
    /// Create a `BridgeSet` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::from_config(&BridgesConfig::default())
    }

    /// Create a `BridgeSet` from explicit configuration.
    #[must_use]
    pub fn from_config(config: &BridgesConfig) -> Self {
        // BUG-033 fix: bridges use raw TCP (SocketAddr), not HTTP URLs.
        // Pass "host:port" not "http://host:port".
        Self {
            synthex: SynthexBridge::with_config(
                "127.0.0.1:8090",
                config.synthex_poll_interval,
            ),
            nexus: NexusBridge::with_config(
                "127.0.0.1:8100",
                config.nexus_poll_interval,
            ),
            me: MeBridge::with_config(
                "127.0.0.1:8080",
                config.me_poll_interval,
            ),
            povm: PovmBridge::with_config(
                "127.0.0.1:8125",
                config.povm_snapshot_interval,
                config.povm_weights_interval,
            ),
            rm: RmBridge::with_config(
                "127.0.0.1:8130",
                config.rm_post_interval,
            ),
            vms: VmsBridge::with_config(
                "127.0.0.1:8120",
                config.vms_post_interval,
                config.vms_post_interval,
            ),
            consent_gate: ConsentGate::new(),
        }
    }

    /// Build consent records from the current sphere map.
    #[must_use]
    pub fn build_consents(spheres: &HashMap<PaneId, PaneSphere>) -> Vec<SphereConsent> {
        spheres
            .iter()
            .map(|(id, s)| {
                SphereConsent::new(id.clone(), s.receptivity, s.total_steps)
                    .with_opt_out(s.opt_out_external_modulation)
                    .with_divergence(s.receptivity < 0.15)
            })
            .collect()
    }

    /// Read cached adjustments from all 6 bridges, gate through consent,
    /// and apply multiplicatively to `network.k_modulation`.
    ///
    /// Updates `state.last_bridge_adjustments` and `state.prev_bridge_staleness`.
    pub fn apply_k_mod(&self, state: &mut AppState, network: &mut CouplingNetwork) {
        let tick = state.tick;

        // Read cached adjustments (each bridge has interior-mutable RwLock)
        let sx = self.synthex.cached_adjustment();
        let nx = self.nexus.cached_adjustment();
        let me = self.me.cached_adjustment();
        let pv = self.povm.cached_adjustment();
        let rm = self.rm.cached_adjustment();
        let vm = self.vms.cached_adjustment();

        // Build consent vector from current spheres
        let consents = Self::build_consents(&state.spheres);

        // Apply combined consent-gated adjustment
        match self.consent_gate.apply_combined_all(
            sx, nx, me, pv, rm, vm, &consents, tick,
        ) {
            Ok(gated) => {
                // Multiplicative composition
                network.k_modulation *= gated;

                // Clamp to budget
                let budget_max = state
                    .k_mod_budget_max_override
                    .unwrap_or(m04_constants::K_MOD_BUDGET_MAX);
                network.k_modulation = network
                    .k_modulation
                    .clamp(m04_constants::K_MOD_BUDGET_MIN, budget_max);

                // Record for API visibility
                state.last_bridge_adjustments = BridgeAdjustments {
                    synthex_adj: sx,
                    nexus_adj: nx,
                    me_adj: me,
                    combined_effect: gated,
                    updated_at: tick,
                };
            }
            Err(e) => {
                tracing::warn!("bridge consent gate failed: {e}");
            }
        }

        // Update staleness flags
        state.prev_bridge_staleness = BridgeStaleness {
            synthex_stale: self.synthex.is_stale(tick),
            nexus_stale: self.nexus.is_stale(tick),
            me_stale: self.me.is_stale(tick),
            povm_stale: self.povm.is_stale(tick),
            rm_stale: self.rm.is_stale(tick),
            vms_stale: self.vms.is_stale(tick),
        };
    }
}

impl Default for BridgeSet {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_set_new_creates_all_bridges() {
        let bs = BridgeSet::new();
        assert!((bs.synthex.cached_adjustment() - 1.0).abs() < f64::EPSILON);
        assert!((bs.nexus.cached_adjustment() - 1.0).abs() < f64::EPSILON);
        assert!((bs.me.cached_adjustment() - 1.0).abs() < f64::EPSILON);
        assert!((bs.povm.cached_adjustment() - 1.0).abs() < f64::EPSILON);
        assert!((bs.rm.cached_adjustment() - 1.0).abs() < f64::EPSILON);
        assert!((bs.vms.cached_adjustment() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bridge_set_default_same_as_new() {
        let bs = BridgeSet::default();
        assert!((bs.synthex.cached_adjustment() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_consents_empty_spheres() {
        let consents = BridgeSet::build_consents(&HashMap::new());
        assert!(consents.is_empty());
    }

    #[test]
    fn build_consents_extracts_receptivity() {
        let mut spheres = HashMap::new();
        let mut s = PaneSphere::new(
            PaneId::new("test"),
            "analyst".to_owned(),
            0.1,
        )
        .unwrap();
        s.receptivity = 0.75;
        s.total_steps = 100;
        spheres.insert(PaneId::new("test"), s);

        let consents = BridgeSet::build_consents(&spheres);
        assert_eq!(consents.len(), 1);
        assert!((consents[0].receptivity - 0.75).abs() < f64::EPSILON);
        assert!(!consents[0].is_newcomer);
        assert!(!consents[0].divergence_requested);
    }

    #[test]
    fn build_consents_detects_divergence() {
        let mut spheres = HashMap::new();
        let mut s = PaneSphere::new(
            PaneId::new("div"),
            "analyst".to_owned(),
            0.1,
        )
        .unwrap();
        s.receptivity = 0.1; // Below 0.15 threshold
        spheres.insert(PaneId::new("div"), s);

        let consents = BridgeSet::build_consents(&spheres);
        assert!(consents[0].divergence_requested);
    }

    #[test]
    fn apply_k_mod_neutral_when_all_bridges_default() {
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        let mut network = CouplingNetwork::new();
        let k_before = network.k_modulation;

        bs.apply_k_mod(&mut state, &mut network);

        // All cached adjustments are 1.0, so k_mod should be unchanged
        assert!(
            (network.k_modulation - k_before).abs() < f64::EPSILON,
            "k_mod changed: {k_before} -> {}",
            network.k_modulation,
        );
    }

    #[test]
    fn apply_k_mod_updates_staleness() {
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        state.tick = 100;
        let mut network = CouplingNetwork::new();

        bs.apply_k_mod(&mut state, &mut network);

        // All bridges are stale since they've never been polled at tick 100
        assert!(state.prev_bridge_staleness.synthex_stale);
    }

    #[test]
    fn apply_k_mod_records_adjustments() {
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        state.tick = 5;
        let mut network = CouplingNetwork::new();

        bs.apply_k_mod(&mut state, &mut network);

        assert_eq!(state.last_bridge_adjustments.updated_at, 5);
    }

    // ── Edge cases: apply_k_mod with empty sphere map ──

    #[test]
    fn apply_k_mod_empty_spheres_returns_neutral() {
        // With no spheres the consent vector is empty → gate returns neutral 1.0.
        // k_modulation *= 1.0 → unchanged.
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        let mut network = CouplingNetwork::new();
        let k_before = network.k_modulation;

        bs.apply_k_mod(&mut state, &mut network);

        assert!(
            (network.k_modulation - k_before).abs() < f64::EPSILON,
            "empty fleet must not change k_modulation",
        );
    }

    // ── Edge cases: apply_k_mod keeps k_modulation within budget after many ticks ──

    #[test]
    fn apply_k_mod_repeated_calls_stay_in_budget() {
        // Verify the post-clamp in apply_k_mod keeps k_modulation bounded
        // even after 1000 successive calls (simulated tick drift).
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        let mut network = CouplingNetwork::new();

        for t in 0..1000u64 {
            state.tick = t;
            bs.apply_k_mod(&mut state, &mut network);
            assert!(
                network.k_modulation >= m04_constants::K_MOD_BUDGET_MIN,
                "k_modulation below min at tick {t}: {}",
                network.k_modulation,
            );
            assert!(
                network.k_modulation <= m04_constants::K_MOD_BUDGET_MAX,
                "k_modulation above max at tick {t}: {}",
                network.k_modulation,
            );
        }
    }

    // ── Edge cases: apply_k_mod with max tick counter ──

    #[test]
    fn apply_k_mod_at_max_tick_does_not_panic() {
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        state.tick = u64::MAX;
        let mut network = CouplingNetwork::new();
        bs.apply_k_mod(&mut state, &mut network);
        assert!(network.k_modulation.is_finite());
    }

    // ── Edge cases: build_consents with sphere at receptivity exactly 0.15 ──

    #[test]
    fn build_consents_receptivity_at_divergence_threshold_is_not_divergent() {
        // The threshold is `< 0.15`, so exactly 0.15 must NOT trigger divergence.
        let mut spheres = HashMap::new();
        let mut s = PaneSphere::new(PaneId::new("edge"), "analyst".to_owned(), 0.1).unwrap();
        s.receptivity = 0.15;
        s.total_steps = 100;
        spheres.insert(PaneId::new("edge"), s);

        let consents = BridgeSet::build_consents(&spheres);
        assert!(!consents[0].divergence_requested, "receptivity=0.15 is NOT < 0.15");
    }

    #[test]
    fn build_consents_receptivity_just_below_threshold_is_divergent() {
        let mut spheres = HashMap::new();
        let mut s = PaneSphere::new(PaneId::new("div"), "analyst".to_owned(), 0.1).unwrap();
        // 0.14999... is < 0.15
        s.receptivity = 0.149_999;
        s.total_steps = 100;
        spheres.insert(PaneId::new("div"), s);

        let consents = BridgeSet::build_consents(&spheres);
        assert!(consents[0].divergence_requested, "receptivity<0.15 must be divergent");
    }

    // ── Edge cases: all bridges stale at tick 0 ──

    #[test]
    fn all_bridges_stale_at_tick_zero_because_never_polled() {
        // All bridges have last_poll_tick=0 and are marked stale by default.
        // is_stale(0) checks stale flag (true) OR time condition.
        let bs = BridgeSet::new();
        let mut state = AppState::default();
        let mut network = CouplingNetwork::new();

        bs.apply_k_mod(&mut state, &mut network);

        assert!(state.prev_bridge_staleness.synthex_stale, "synthex must be stale initially");
        assert!(state.prev_bridge_staleness.nexus_stale, "nexus must be stale initially");
        assert!(state.prev_bridge_staleness.me_stale, "me must be stale initially");
    }

    // ── Edge cases: build_consents produces correct newcomer flag ──

    #[test]
    fn build_consents_newcomer_flag_based_on_total_steps() {
        let mut spheres = HashMap::new();

        // Sphere with steps=0 → newcomer
        let mut s_new = PaneSphere::new(PaneId::new("new"), "a".to_owned(), 0.1).unwrap();
        s_new.total_steps = 0;
        spheres.insert(PaneId::new("new"), s_new);

        let consents = BridgeSet::build_consents(&spheres);
        assert!(consents[0].is_newcomer, "sphere with 0 steps must be a newcomer");
    }

    #[test]
    fn build_consents_veteran_flag_at_fifty_steps() {
        let mut spheres = HashMap::new();
        let mut s = PaneSphere::new(PaneId::new("vet"), "a".to_owned(), 0.1).unwrap();
        s.total_steps = 50; // Exactly NEWCOMER_STEPS → NOT newcomer
        spheres.insert(PaneId::new("vet"), s);

        let consents = BridgeSet::build_consents(&spheres);
        assert!(!consents[0].is_newcomer, "sphere with 50 steps must NOT be a newcomer");
    }
}
