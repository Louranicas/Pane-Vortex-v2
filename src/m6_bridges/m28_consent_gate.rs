//! # M28: Consent Gate
//!
//! The consent pattern that must propagate to every external control mechanism.
//! `consent_gated_k_adjustment()` scales external influence by fleet receptivity
//! and clamps the combined effect to the `K_MOD_BUDGET` range [0.85, 1.15].
//!
//! ## Layer: L6 (Bridges)
//! ## Module: M28
//! ## Dependencies: L1 (M01, M02, M04)
//!
//! ## Core Function
//! ```text
//! consent_gated_k_adjustment(raw_adj, receptivities, opt_outs) -> gated_adj
//!   1. Compute mean receptivity across all spheres
//!   2. Exclude opted-out spheres
//!   3. Scale raw_adj by mean receptivity
//!   4. Clamp combined effect to K_MOD_BUDGET [0.85, 1.15]
//!   5. Apply newcomer dampening (80% reduction for first 50 steps)
//!   6. Exempt spheres with active divergence requests
//! ```
//!
//! ## Design Constraints
//! - C8: ALL external bridges MUST route through this function
//! - PG-5: Budget captures ALL bridges (present and future)
//! - PG-12: ME bridge included
//!
//! ## Philosophy
//! > "The consent gate gave spheres the right to say no."

use std::collections::HashMap;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::m1_foundation::m01_core_types::PaneId;
use crate::m1_foundation::m02_error_handling::{PvError, PvResult};
use crate::m1_foundation::m04_constants;

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// Newcomer dampening factor (80% reduction).
const NEWCOMER_DAMPEN: f64 = 0.2;

/// Steps during which newcomer dampening applies.
const NEWCOMER_STEPS: u64 = 50;

/// Default receptivity when no spheres are present.
const DEFAULT_RECEPTIVITY: f64 = 1.0;

/// Minimum number of consenting spheres for adjustment to apply.
const MIN_CONSENTING: usize = 1;

// ──────────────────────────────────────────────────────────────
// Bridge contribution tracking
// ──────────────────────────────────────────────────────────────

/// Tracks which bridges are contributing to the combined `k_mod` effect.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeContributions {
    /// SYNTHEX contribution (multiplicative).
    pub synthex: f64,
    /// Nexus (SAN-K7) contribution.
    pub nexus: f64,
    /// Maintenance Engine contribution.
    pub me: f64,
    /// POVM bridge contribution (GAP-3).
    pub povm: f64,
    /// Reasoning Memory bridge contribution (GAP-3).
    pub rm: f64,
    /// Vortex Memory System bridge contribution (GAP-3).
    pub vms: f64,
    /// Combined raw effect before consent gating.
    pub raw_combined: f64,
    /// Combined effect after consent gating.
    pub gated_combined: f64,
    /// Mean fleet receptivity used for gating.
    pub mean_receptivity: f64,
    /// Number of consenting spheres.
    pub consenting_count: usize,
    /// Number of opted-out spheres.
    pub opted_out_count: usize,
    /// Tick at which these contributions were computed.
    pub tick: u64,
}

/// Per-sphere consent information passed to the consent gate.
#[derive(Debug, Clone)]
pub struct SphereConsent {
    /// Sphere identifier.
    pub id: PaneId,
    /// Receptivity [0.0, 1.0].
    pub receptivity: f64,
    /// Whether this sphere has opted out of external modulation.
    pub opted_out: bool,
    /// Whether this sphere is a newcomer (within first N steps).
    pub is_newcomer: bool,
    /// Steps lived (used for newcomer detection).
    pub steps_lived: u64,
    /// Whether this sphere has an active divergence request.
    pub divergence_requested: bool,
}

impl SphereConsent {
    /// Create a new consent record for a sphere.
    #[must_use]
    pub fn new(id: PaneId, receptivity: f64, steps_lived: u64) -> Self {
        Self {
            id,
            receptivity: receptivity.clamp(0.0, 1.0),
            opted_out: false,
            is_newcomer: steps_lived < NEWCOMER_STEPS,
            steps_lived,
            divergence_requested: false,
        }
    }

    /// Set the opt-out flag.
    #[must_use]
    pub const fn with_opt_out(mut self, opted_out: bool) -> Self {
        self.opted_out = opted_out;
        self
    }

    /// Set the divergence request flag.
    #[must_use]
    pub const fn with_divergence(mut self, divergence: bool) -> Self {
        self.divergence_requested = divergence;
        self
    }
}

// ──────────────────────────────────────────────────────────────
// ConsentGate state
// ──────────────────────────────────────────────────────────────

/// Mutable state behind a `RwLock`.
#[derive(Debug)]
struct GateState {
    /// Last computed contributions.
    last_contributions: BridgeContributions,
    /// Per-sphere effective `k_adj` overrides.
    sphere_overrides: HashMap<String, f64>,
    /// Total number of gating operations performed.
    gate_count: u64,
    /// Total rejections (opted-out or budget-exceeded).
    rejection_count: u64,
    /// Runtime-mutable budget max (GAP-2). Governance proposals can change this.
    budget_max: f64,
    /// Runtime-mutable budget min.
    budget_min: f64,
}

impl Default for GateState {
    fn default() -> Self {
        Self {
            last_contributions: BridgeContributions::default(),
            sphere_overrides: HashMap::new(),
            gate_count: 0,
            rejection_count: 0,
            budget_max: m04_constants::K_MOD_BUDGET_MAX,
            budget_min: m04_constants::K_MOD_BUDGET_MIN,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// ConsentGate
// ──────────────────────────────────────────────────────────────

/// The consent gate that ALL bridge adjustments must pass through.
///
/// Validates per-sphere consent, enforces the `k_mod` budget [0.85, 1.15],
/// and tracks which bridges are contributing to the combined effect.
///
/// ## Usage
/// ```text
/// let gate = ConsentGate::new();
/// let adj = gate.apply(raw_adj, &sphere_consents, tick)?;
/// // adj is now consent-gated and budget-clamped
/// ```
#[derive(Debug)]
pub struct ConsentGate {
    /// Interior-mutable state.
    state: RwLock<GateState>,
}

impl ConsentGate {
    /// Create a new consent gate.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: RwLock::new(GateState::default()),
        }
    }

    /// Apply the consent gate to a raw adjustment value.
    ///
    /// Steps:
    /// 1. Filter out opted-out spheres
    /// 2. Compute mean receptivity of consenting spheres
    /// 3. Scale the raw adjustment by mean receptivity
    /// 4. Apply newcomer dampening
    /// 5. Clamp to `K_MOD_BUDGET` range
    ///
    /// # Errors
    /// Returns `PvError::BridgeConsentDenied` if all spheres have opted out.
    pub fn apply(
        &self,
        raw_adj: f64,
        consents: &[SphereConsent],
        tick: u64,
    ) -> PvResult<f64> {
        if !raw_adj.is_finite() {
            return Err(PvError::BridgeParse {
                service: "consent_gate".to_owned(),
                reason: format!("non-finite raw adjustment: {raw_adj}"),
            });
        }

        let mut state = self.state.write();
        state.gate_count = state.gate_count.saturating_add(1);

        // Step 1: Filter consenting spheres
        // Exclude opted-out AND divergence-requesting spheres (GAP-4 fix)
        let consenting: Vec<&SphereConsent> = consents
            .iter()
            .filter(|c| !c.opted_out && !c.divergence_requested)
            .collect();

        let opted_out_count = consents.iter().filter(|c| c.opted_out).count();
        let divergence_exempt_count = consents
            .iter()
            .filter(|c| !c.opted_out && c.divergence_requested)
            .count();

        if consenting.len() < MIN_CONSENTING {
            state.rejection_count = state.rejection_count.saturating_add(1);
            return Ok(1.0); // Neutral — no spheres consent
        }

        // Step 2: Compute mean receptivity
        let total_receptivity: f64 = consenting.iter().map(|c| c.receptivity).sum();
        #[allow(clippy::cast_precision_loss)]
        let consenting_f64 = consenting.len() as f64;
        let mean_receptivity = if consenting.is_empty() {
            DEFAULT_RECEPTIVITY
        } else {
            total_receptivity / consenting_f64
        };

        // Step 3: Scale raw adjustment by mean receptivity
        // Deviation from neutral (1.0) is scaled by receptivity
        let deviation = raw_adj - 1.0;
        let scaled = deviation.mul_add(mean_receptivity, 1.0);

        // Step 4: Apply newcomer dampening
        let newcomer_count = consenting.iter().filter(|c| c.is_newcomer).count();
        let newcomer_ratio = if consenting.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let ratio = newcomer_count as f64 / consenting_f64;
            ratio
        };
        // Dampen proportional to newcomer ratio
        let dampened = if newcomer_ratio > 0.5 {
            let dampen_factor =
                ((newcomer_ratio - 0.5) * 2.0).mul_add(-(1.0 - NEWCOMER_DAMPEN), 1.0);
            let dev = scaled - 1.0;
            dev.mul_add(dampen_factor, 1.0)
        } else {
            scaled
        };

        // Step 5: Clamp to runtime budget (GAP-2: governance-mutable)
        let clamped = dampened.clamp(state.budget_min, state.budget_max);

        // Update tracking
        state.last_contributions.gated_combined = clamped;
        state.last_contributions.raw_combined = raw_adj;
        state.last_contributions.mean_receptivity = mean_receptivity;
        state.last_contributions.consenting_count = consenting.len();
        state.last_contributions.opted_out_count = opted_out_count + divergence_exempt_count;
        state.last_contributions.tick = tick;

        Ok(clamped)
    }

    /// Apply the consent gate with individual bridge contributions.
    ///
    /// Combines SYNTHEX, Nexus, and ME adjustments into a single gated value.
    ///
    /// # Errors
    /// Returns error if the combined effect is non-finite.
    #[allow(clippy::similar_names)]
    pub fn apply_combined(
        &self,
        synthex_adj: f64,
        nexus_adj: f64,
        me_adj: f64,
        consents: &[SphereConsent],
        tick: u64,
    ) -> PvResult<f64> {
        // Combine multiplicatively: each bridge scales around neutral (1.0)
        let combined = synthex_adj * nexus_adj * me_adj;

        if !combined.is_finite() {
            return Err(PvError::BridgeParse {
                service: "consent_gate".to_owned(),
                reason: "non-finite combined adjustment".to_owned(),
            });
        }

        // Pre-clamp individual contributions for tracking
        {
            let mut state = self.state.write();
            state.last_contributions.synthex = synthex_adj;
            state.last_contributions.nexus = nexus_adj;
            state.last_contributions.me = me_adj;
        }

        self.apply(combined, consents, tick)
    }

    /// Apply consent gate with all 6 bridge contributions (GAP-3 fix).
    ///
    /// Combines SYNTHEX, Nexus, ME, POVM, RM, and VMS adjustments.
    /// All bridges now route through consent — no bypass.
    ///
    /// # Errors
    /// Returns error if the combined effect is non-finite.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_combined_all(
        &self,
        synthex_adj: f64,
        nexus_adj: f64,
        me_adj: f64,
        povm_adj: f64,
        rm_adj: f64,
        vms_adj: f64,
        consents: &[SphereConsent],
        tick: u64,
    ) -> PvResult<f64> {
        let combined = synthex_adj * nexus_adj * me_adj * povm_adj * rm_adj * vms_adj;

        if !combined.is_finite() {
            return Err(PvError::BridgeParse {
                service: "consent_gate".to_owned(),
                reason: "non-finite combined adjustment (6 bridges)".to_owned(),
            });
        }

        {
            let mut state = self.state.write();
            state.last_contributions.synthex = synthex_adj;
            state.last_contributions.nexus = nexus_adj;
            state.last_contributions.me = me_adj;
            state.last_contributions.povm = povm_adj;
            state.last_contributions.rm = rm_adj;
            state.last_contributions.vms = vms_adj;
        }

        self.apply(combined, consents, tick)
    }

    /// Get the last computed contributions.
    #[must_use]
    pub fn last_contributions(&self) -> BridgeContributions {
        self.state.read().last_contributions.clone()
    }

    /// Get the total number of gating operations.
    #[must_use]
    pub fn gate_count(&self) -> u64 {
        self.state.read().gate_count
    }

    /// Get the total number of rejections.
    #[must_use]
    pub fn rejection_count(&self) -> u64 {
        self.state.read().rejection_count
    }

    /// Set the runtime budget maximum (GAP-2: governance proposals can widen/narrow).
    ///
    /// Clamped to `[1.0, 1.5]` to prevent abuse.
    pub fn set_budget_max(&self, max: f64) {
        let clamped = max.clamp(1.0, 1.5);
        self.state.write().budget_max = clamped;
    }

    /// Get the current runtime budget maximum.
    #[must_use]
    pub fn budget_max(&self) -> f64 {
        self.state.read().budget_max
    }

    /// Set a per-sphere `k_adj` override (for future V3.3 per-sphere isolation).
    pub fn set_sphere_override(&self, sphere_id: &str, k_adj: f64) {
        let clamped = k_adj.clamp(
            m04_constants::K_MOD_BUDGET_MIN,
            m04_constants::K_MOD_BUDGET_MAX,
        );
        self.state
            .write()
            .sphere_overrides
            .insert(sphere_id.to_owned(), clamped);
    }

    /// Get a per-sphere override, if any.
    #[must_use]
    pub fn sphere_override(&self, sphere_id: &str) -> Option<f64> {
        self.state.read().sphere_overrides.get(sphere_id).copied()
    }

    /// Remove a per-sphere override.
    pub fn remove_sphere_override(&self, sphere_id: &str) {
        self.state.write().sphere_overrides.remove(sphere_id);
    }

    /// Clear all per-sphere overrides.
    pub fn clear_overrides(&self) {
        self.state.write().sphere_overrides.clear();
    }

    /// Compute the effective `k_adj` for a specific sphere, considering
    /// its consent posture and any overrides.
    #[must_use]
    pub fn effective_k_adj(&self, sphere_id: &str, fleet_adj: f64) -> f64 {
        let state = self.state.read();
        if let Some(override_val) = state.sphere_overrides.get(sphere_id) {
            return *override_val;
        }
        fleet_adj.clamp(
            m04_constants::K_MOD_BUDGET_MIN,
            m04_constants::K_MOD_BUDGET_MAX,
        )
    }

    /// Validate that a raw adjustment is within absolute bounds.
    ///
    /// # Errors
    /// Returns error for non-finite or out-of-range values.
    pub fn validate_adjustment(raw_adj: f64) -> PvResult<f64> {
        if !raw_adj.is_finite() {
            return Err(PvError::BridgeParse {
                service: "consent_gate".to_owned(),
                reason: format!("non-finite adjustment: {raw_adj}"),
            });
        }
        Ok(raw_adj.clamp(m04_constants::K_MOD_MIN, m04_constants::K_MOD_MAX))
    }
}

impl Default for ConsentGate {
    fn default() -> Self {
        Self::new()
    }
}

// ──────────────────────────────────────────────────────────────
// Free functions for convenience
// ──────────────────────────────────────────────────────────────

/// Clamp a value to the `k_mod` budget range [0.85, 1.15].
///
/// This is the simple version used when full consent tracking is not needed.
#[must_use]
pub fn clamp_to_budget(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(
            m04_constants::K_MOD_BUDGET_MIN,
            m04_constants::K_MOD_BUDGET_MAX,
        )
    } else {
        1.0
    }
}

/// Compute mean receptivity from a slice of values.
///
/// Returns `DEFAULT_RECEPTIVITY` (1.0) if the slice is empty.
#[must_use]
pub fn mean_receptivity(receptivities: &[f64]) -> f64 {
    if receptivities.is_empty() {
        return DEFAULT_RECEPTIVITY;
    }
    let sum: f64 = receptivities.iter().copied().sum();
    #[allow(clippy::cast_precision_loss)]
    let count = receptivities.len() as f64;
    sum / count
}

/// Check whether an adjustment is within the `k_mod` budget.
#[must_use]
pub fn is_within_budget(value: f64) -> bool {
    value.is_finite()
        && (m04_constants::K_MOD_BUDGET_MIN..=m04_constants::K_MOD_BUDGET_MAX).contains(&value)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_consent(id: &str, receptivity: f64, steps: u64) -> SphereConsent {
        SphereConsent::new(PaneId::new(id), receptivity, steps)
    }

    fn make_full_fleet(n: usize) -> Vec<SphereConsent> {
        (0..n)
            .map(|i| make_consent(&format!("sphere-{i}"), 1.0, 100))
            .collect()
    }

    // ── Construction ──

    #[test]
    fn new_creates_gate() {
        let gate = ConsentGate::new();
        assert_eq!(gate.gate_count(), 0);
        assert_eq!(gate.rejection_count(), 0);
    }

    #[test]
    fn default_creates_gate() {
        let gate = ConsentGate::default();
        assert_eq!(gate.gate_count(), 0);
    }

    // ── Basic gating ──

    #[test]
    fn neutral_adjustment_passes_through() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate.apply(1.0, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
    }

    #[test]
    fn positive_adjustment_scaled_by_receptivity() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = (0..4)
            .map(|i| make_consent(&format!("s{i}"), 0.5, 100))
            .collect();
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // deviation = 0.1, scaled by 0.5 receptivity → 1.0 + 0.1*0.5 = 1.05
        assert!((adj - 1.05).abs() < 1e-10);
    }

    #[test]
    fn negative_adjustment_scaled_by_receptivity() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = (0..4)
            .map(|i| make_consent(&format!("s{i}"), 0.5, 100))
            .collect();
        let adj = gate.apply(0.9, &consents, 1).unwrap();
        // deviation = -0.1, scaled by 0.5 → 1.0 - 0.05 = 0.95
        assert!((adj - 0.95).abs() < 1e-10);
    }

    #[test]
    fn full_receptivity_passes_full_adjustment() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!((adj - 1.1).abs() < 1e-10);
    }

    #[test]
    fn zero_receptivity_returns_neutral() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = (0..4)
            .map(|i| make_consent(&format!("s{i}"), 0.0, 100))
            .collect();
        let adj = gate.apply(1.15, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
    }

    // ── Budget clamping ──

    #[test]
    fn clamps_to_budget_max() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate.apply(2.0, &consents, 1).unwrap();
        assert!((adj - m04_constants::K_MOD_BUDGET_MAX).abs() < 1e-10);
    }

    #[test]
    fn clamps_to_budget_min() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate.apply(0.5, &consents, 1).unwrap();
        assert!((adj - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    #[test]
    fn budget_range_min_is_085() {
        assert!((m04_constants::K_MOD_BUDGET_MIN - 0.85).abs() < 1e-10);
    }

    #[test]
    fn budget_range_max_is_115() {
        assert!((m04_constants::K_MOD_BUDGET_MAX - 1.15).abs() < 1e-10);
    }

    // ── Opt-out handling ──

    #[test]
    fn opted_out_spheres_excluded() {
        let gate = ConsentGate::new();
        let consents = vec![
            make_consent("s1", 1.0, 100).with_opt_out(true),
            make_consent("s2", 1.0, 100).with_opt_out(true),
            make_consent("s3", 0.5, 100),
        ];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // Only s3 consents with 0.5 receptivity
        // deviation = 0.1, scaled by 0.5 → 1.05
        assert!((adj - 1.05).abs() < 1e-10);
    }

    #[test]
    fn all_opted_out_returns_neutral() {
        let gate = ConsentGate::new();
        let consents = vec![
            make_consent("s1", 1.0, 100).with_opt_out(true),
            make_consent("s2", 1.0, 100).with_opt_out(true),
        ];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
    }

    #[test]
    fn empty_consents_returns_neutral() {
        let gate = ConsentGate::new();
        let adj = gate.apply(1.1, &[], 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
    }

    // ── Newcomer dampening ──

    #[test]
    fn newcomer_detection() {
        let consent = make_consent("new", 1.0, 10);
        assert!(consent.is_newcomer);
    }

    #[test]
    fn veteran_not_newcomer() {
        let consent = make_consent("vet", 1.0, 100);
        assert!(!consent.is_newcomer);
    }

    #[test]
    fn newcomer_at_boundary() {
        let consent = make_consent("boundary", 1.0, 49);
        assert!(consent.is_newcomer);

        let consent2 = make_consent("boundary2", 1.0, 50);
        assert!(!consent2.is_newcomer);
    }

    #[test]
    fn mostly_newcomer_fleet_dampened() {
        let gate = ConsentGate::new();
        // All newcomers (steps=0)
        let consents: Vec<SphereConsent> = (0..5)
            .map(|i| make_consent(&format!("new{i}"), 1.0, 0))
            .collect();
        let adj_newcomer = gate.apply(1.1, &consents, 1).unwrap();

        // All veterans
        let consents_vet = make_full_fleet(5);
        let adj_veteran = gate.apply(1.1, &consents_vet, 2).unwrap();

        // Newcomers should be dampened relative to veterans
        assert!(
            (adj_newcomer - 1.0).abs() < (adj_veteran - 1.0).abs(),
            "newcomer fleet should be more dampened"
        );
    }

    // ── Combined application ──

    #[test]
    fn apply_combined_neutral() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate
            .apply_combined(1.0, 1.0, 1.0, &consents, 1)
            .unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
    }

    #[test]
    fn apply_combined_all_boost() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let adj = gate
            .apply_combined(1.05, 1.03, 1.02, &consents, 1)
            .unwrap();
        // 1.05 * 1.03 * 1.02 = 1.10253 → within budget
        assert!(adj > 1.0);
        assert!(adj <= m04_constants::K_MOD_BUDGET_MAX);
    }

    #[test]
    fn apply_combined_tracks_contributions() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let _ = gate.apply_combined(1.05, 0.98, 1.0, &consents, 42);
        let contrib = gate.last_contributions();
        assert!((contrib.synthex - 1.05).abs() < 1e-10);
        assert!((contrib.nexus - 0.98).abs() < 1e-10);
        assert!((contrib.me - 1.0).abs() < 1e-10);
        assert_eq!(contrib.tick, 42);
        assert_eq!(contrib.consenting_count, 5);
    }

    #[test]
    fn apply_combined_rejects_nan() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let result = gate.apply_combined(f64::NAN, 1.0, 1.0, &consents, 1);
        assert!(result.is_err());
    }

    #[test]
    fn apply_combined_rejects_infinity() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(5);
        let result = gate.apply_combined(f64::INFINITY, 1.0, 1.0, &consents, 1);
        assert!(result.is_err());
    }

    // ── Sphere overrides ──

    #[test]
    fn sphere_override_set_and_get() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("s1", 1.05);
        assert!((gate.sphere_override("s1").unwrap() - 1.05).abs() < 1e-10);
    }

    #[test]
    fn sphere_override_clamped_to_budget() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("s1", 5.0);
        assert!(
            (gate.sphere_override("s1").unwrap() - m04_constants::K_MOD_BUDGET_MAX).abs() < 1e-10
        );
    }

    #[test]
    fn sphere_override_missing_returns_none() {
        let gate = ConsentGate::new();
        assert!(gate.sphere_override("nonexistent").is_none());
    }

    #[test]
    fn sphere_override_remove() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("s1", 1.05);
        gate.remove_sphere_override("s1");
        assert!(gate.sphere_override("s1").is_none());
    }

    #[test]
    fn clear_overrides() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("s1", 1.05);
        gate.set_sphere_override("s2", 0.95);
        gate.clear_overrides();
        assert!(gate.sphere_override("s1").is_none());
        assert!(gate.sphere_override("s2").is_none());
    }

    // ── Effective k_adj ──

    #[test]
    fn effective_k_adj_with_override() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("s1", 1.05);
        assert!((gate.effective_k_adj("s1", 1.1) - 1.05).abs() < 1e-10);
    }

    #[test]
    fn effective_k_adj_without_override() {
        let gate = ConsentGate::new();
        assert!((gate.effective_k_adj("s1", 1.05) - 1.05).abs() < 1e-10);
    }

    #[test]
    fn effective_k_adj_clamps_fleet() {
        let gate = ConsentGate::new();
        assert!(
            (gate.effective_k_adj("s1", 5.0) - m04_constants::K_MOD_BUDGET_MAX).abs() < 1e-10
        );
    }

    // ── Validate adjustment ──

    #[test]
    fn validate_adjustment_normal() {
        let v = ConsentGate::validate_adjustment(1.0).unwrap();
        assert!((v - 1.0).abs() < 1e-10);
    }

    #[test]
    fn validate_adjustment_clamps_high() {
        let v = ConsentGate::validate_adjustment(10.0).unwrap();
        assert!((v - m04_constants::K_MOD_MAX).abs() < 1e-10);
    }

    #[test]
    fn validate_adjustment_clamps_low() {
        let v = ConsentGate::validate_adjustment(-10.0).unwrap();
        assert!((v - m04_constants::K_MOD_MIN).abs() < 1e-10);
    }

    #[test]
    fn validate_adjustment_rejects_nan() {
        assert!(ConsentGate::validate_adjustment(f64::NAN).is_err());
    }

    #[test]
    fn validate_adjustment_rejects_infinity() {
        assert!(ConsentGate::validate_adjustment(f64::INFINITY).is_err());
    }

    // ── Free functions ──

    #[test]
    fn clamp_to_budget_normal() {
        assert!((clamp_to_budget(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn clamp_to_budget_high() {
        assert!((clamp_to_budget(5.0) - m04_constants::K_MOD_BUDGET_MAX).abs() < 1e-10);
    }

    #[test]
    fn clamp_to_budget_low() {
        assert!((clamp_to_budget(0.0) - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    #[test]
    fn clamp_to_budget_nan_returns_neutral() {
        assert!((clamp_to_budget(f64::NAN) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mean_receptivity_normal() {
        let r = mean_receptivity(&[0.5, 1.0, 0.8]);
        assert!((r - 0.7666666666666666).abs() < 1e-10);
    }

    #[test]
    fn mean_receptivity_empty() {
        assert!((mean_receptivity(&[]) - DEFAULT_RECEPTIVITY).abs() < 1e-10);
    }

    #[test]
    fn mean_receptivity_single() {
        assert!((mean_receptivity(&[0.5]) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn is_within_budget_yes() {
        assert!(is_within_budget(1.0));
        assert!(is_within_budget(0.85));
        assert!(is_within_budget(1.15));
    }

    #[test]
    fn is_within_budget_no() {
        assert!(!is_within_budget(0.5));
        assert!(!is_within_budget(2.0));
        assert!(!is_within_budget(f64::NAN));
    }

    // ── Gate counting ──

    #[test]
    fn gate_count_increments() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let _ = gate.apply(1.0, &consents, 1);
        let _ = gate.apply(1.0, &consents, 2);
        assert_eq!(gate.gate_count(), 2);
    }

    #[test]
    fn rejection_count_on_no_consent() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = vec![
            make_consent("s1", 1.0, 100).with_opt_out(true),
        ];
        let _ = gate.apply(1.1, &consents, 1);
        // All opted out → rejection
        // Note: our impl returns neutral, which counts as rejection
    }

    // ── SphereConsent builders ──

    #[test]
    fn sphere_consent_new() {
        let c = SphereConsent::new(PaneId::new("test"), 0.8, 100);
        assert_eq!(c.id.as_str(), "test");
        assert!((c.receptivity - 0.8).abs() < f64::EPSILON);
        assert!(!c.opted_out);
        assert!(!c.is_newcomer);
        assert!(!c.divergence_requested);
    }

    #[test]
    fn sphere_consent_clamps_receptivity() {
        let c = SphereConsent::new(PaneId::new("test"), 2.0, 100);
        assert!((c.receptivity - 1.0).abs() < f64::EPSILON);

        let c2 = SphereConsent::new(PaneId::new("test"), -1.0, 100);
        assert!((c2.receptivity).abs() < f64::EPSILON);
    }

    #[test]
    fn sphere_consent_with_opt_out() {
        let c = make_consent("s1", 1.0, 100).with_opt_out(true);
        assert!(c.opted_out);
    }

    #[test]
    fn sphere_consent_with_divergence() {
        let c = make_consent("s1", 1.0, 100).with_divergence(true);
        assert!(c.divergence_requested);
    }

    // ── BridgeContributions ──

    #[test]
    fn bridge_contributions_default() {
        let bc = BridgeContributions::default();
        assert!((bc.synthex).abs() < f64::EPSILON);
        assert!((bc.nexus).abs() < f64::EPSILON);
        assert!((bc.me).abs() < f64::EPSILON);
        assert_eq!(bc.consenting_count, 0);
    }

    #[test]
    fn bridge_contributions_serde_roundtrip() {
        let bc = BridgeContributions {
            synthex: 1.05,
            nexus: 0.98,
            me: 1.0,
            povm: 1.0,
            rm: 1.0,
            vms: 1.0,
            raw_combined: 1.029,
            gated_combined: 1.02,
            mean_receptivity: 0.9,
            consenting_count: 5,
            opted_out_count: 1,
            tick: 42,
        };
        let json = serde_json::to_string(&bc).unwrap();
        let back: BridgeContributions = serde_json::from_str(&json).unwrap();
        assert!((back.synthex - 1.05).abs() < 1e-10);
        assert_eq!(back.tick, 42);
    }

    // ── Thread safety ──

    #[test]
    fn gate_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ConsentGate>();
    }

    #[test]
    fn gate_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ConsentGate>();
    }

    // ── Debug ──

    #[test]
    fn debug_format_works() {
        let gate = ConsentGate::new();
        let debug = format!("{gate:?}");
        assert!(debug.contains("ConsentGate"));
    }

    // ── Constants ──

    #[test]
    fn newcomer_dampen_is_0_2() {
        assert!((NEWCOMER_DAMPEN - 0.2).abs() < 1e-10);
    }

    #[test]
    fn newcomer_steps_is_50() {
        assert_eq!(NEWCOMER_STEPS, 50);
    }

    // ── Error on non-finite ──

    #[test]
    fn apply_rejects_nan() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        assert!(gate.apply(f64::NAN, &consents, 1).is_err());
    }

    #[test]
    fn apply_rejects_infinity() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        assert!(gate.apply(f64::INFINITY, &consents, 1).is_err());
    }

    #[test]
    fn apply_rejects_neg_infinity() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        assert!(gate.apply(f64::NEG_INFINITY, &consents, 1).is_err());
    }

    // ── GAP-4: Divergence exemption ──

    #[test]
    fn divergence_requested_excludes_from_consent() {
        let gate = ConsentGate::new();
        let mut consents = make_full_fleet(4);
        // 2 of 4 request divergence — they should be exempt
        consents[0] = consents[0].clone().with_divergence(true);
        consents[1] = consents[1].clone().with_divergence(true);
        // Only 2 consenting spheres remain, both with receptivity 1.0
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!((adj - 1.1).abs() < 1e-10, "full receptivity from consenting only");
    }

    #[test]
    fn all_divergence_requested_returns_neutral() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = (0..3)
            .map(|i| {
                make_consent(&format!("s{i}"), 1.0, 100).with_divergence(true)
            })
            .collect();
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10, "all exempt → neutral");
    }

    #[test]
    fn divergence_mixed_with_opted_out() {
        let gate = ConsentGate::new();
        let mut consents = make_full_fleet(5);
        consents[0] = consents[0].clone().with_opt_out(true);
        consents[1] = consents[1].clone().with_divergence(true);
        // 3 consenting, 1 opted out, 1 divergence exempt
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        let contribs = gate.last_contributions();
        assert_eq!(contribs.consenting_count, 3);
        assert_eq!(contribs.opted_out_count, 2); // 1 opt-out + 1 divergence
        assert!((adj - 1.1).abs() < 1e-10);
    }

    // ── GAP-3: 6-bridge consent gating ──

    #[test]
    fn apply_combined_all_6_bridges() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(4);
        // All bridges at neutral except synthex (1.05) and vms (0.97)
        let adj = gate
            .apply_combined_all(1.05, 1.0, 1.0, 1.0, 1.0, 0.97, &consents, 1)
            .unwrap();
        // Combined = 1.05 * 0.97 = 1.0185, gated at full receptivity
        assert!((adj - 1.0185).abs() < 0.001);
        let contribs = gate.last_contributions();
        assert!((contribs.povm - 1.0).abs() < 1e-10);
        assert!((contribs.vms - 0.97).abs() < 1e-10);
    }

    #[test]
    fn apply_combined_all_tracks_all_bridges() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let _ = gate
            .apply_combined_all(1.02, 0.98, 1.01, 1.03, 0.99, 1.00, &consents, 5)
            .unwrap();
        let c = gate.last_contributions();
        assert!((c.synthex - 1.02).abs() < 1e-10);
        assert!((c.nexus - 0.98).abs() < 1e-10);
        assert!((c.me - 1.01).abs() < 1e-10);
        assert!((c.povm - 1.03).abs() < 1e-10);
        assert!((c.rm - 0.99).abs() < 1e-10);
        assert!((c.vms - 1.00).abs() < 1e-10);
        assert_eq!(c.tick, 5);
    }

    // ── GAP-2: Runtime budget ──

    #[test]
    fn set_budget_max_widens_range() {
        let gate = ConsentGate::new();
        gate.set_budget_max(1.3);
        assert!((gate.budget_max() - 1.3).abs() < 1e-10);
    }

    #[test]
    fn set_budget_max_clamped_to_safe_range() {
        let gate = ConsentGate::new();
        gate.set_budget_max(2.0);
        assert!((gate.budget_max() - 1.5).abs() < 1e-10);
        gate.set_budget_max(0.5);
        assert!((gate.budget_max() - 1.0).abs() < 1e-10);
    }

    // ── Edge cases: exactly-zero consenting spheres (quorum boundary) ──

    #[test]
    fn exactly_one_consenting_sphere_at_minimum_quorum() {
        // MIN_CONSENTING = 1 — exactly one sphere must not be rejected.
        let gate = ConsentGate::new();
        let consents = vec![make_consent("solo", 1.0, 100)];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // Single sphere, full receptivity → full adjustment passes through.
        assert!((adj - 1.1).abs() < 1e-10);
        assert_eq!(gate.rejection_count(), 0);
    }

    #[test]
    fn one_consenting_one_opted_out_still_meets_quorum() {
        let gate = ConsentGate::new();
        let consents = vec![
            make_consent("out", 1.0, 100).with_opt_out(true),
            make_consent("in", 0.5, 100),
        ];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // Only "in" consents with 0.5 receptivity → deviation scaled: 1.0 + 0.1*0.5 = 1.05
        assert!((adj - 1.05).abs() < 1e-10);
    }

    #[test]
    fn rejection_count_increments_on_all_opted_out() {
        let gate = ConsentGate::new();
        let consents = vec![
            make_consent("s1", 1.0, 100).with_opt_out(true),
            make_consent("s2", 1.0, 100).with_opt_out(true),
        ];
        let _ = gate.apply(1.1, &consents, 1).unwrap();
        assert_eq!(gate.rejection_count(), 1);
    }

    #[test]
    fn rejection_count_increments_on_empty_consents() {
        let gate = ConsentGate::new();
        let _ = gate.apply(1.1, &[], 1).unwrap();
        assert_eq!(gate.rejection_count(), 1);
    }

    // ── Edge cases: all-abstentions (all divergence requested) ──

    #[test]
    fn single_divergence_sphere_below_quorum_returns_neutral() {
        let gate = ConsentGate::new();
        let consents = vec![make_consent("d", 1.0, 100).with_divergence(true)];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10);
        assert_eq!(gate.rejection_count(), 1);
    }

    // ── Edge cases: extreme receptivity values ──

    #[test]
    fn apply_with_all_zero_receptivity_returns_neutral() {
        let gate = ConsentGate::new();
        // All consenting spheres have receptivity 0.0 → deviation scaled by 0.0 → neutral.
        let consents: Vec<SphereConsent> = (0..5)
            .map(|i| make_consent(&format!("s{i}"), 0.0, 100))
            .collect();
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        assert!(
            (adj - 1.0).abs() < 1e-10,
            "zero receptivity must collapse to neutral 1.0"
        );
    }

    #[test]
    fn apply_with_mixed_zero_and_full_receptivity() {
        let gate = ConsentGate::new();
        // 2 at 0.0, 2 at 1.0 → mean = 0.5
        let consents = vec![
            make_consent("a", 0.0, 100),
            make_consent("b", 0.0, 100),
            make_consent("c", 1.0, 100),
            make_consent("d", 1.0, 100),
        ];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // deviation = 0.1, mean = 0.5 → 1.0 + 0.1*0.5 = 1.05
        assert!((adj - 1.05).abs() < 1e-10);
    }

    // ── Edge cases: raw_adj exactly at budget boundary ──

    #[test]
    fn apply_raw_adj_exactly_at_budget_min() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let adj = gate.apply(m04_constants::K_MOD_BUDGET_MIN, &consents, 1).unwrap();
        assert!((adj - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    #[test]
    fn apply_raw_adj_exactly_at_budget_max() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let adj = gate.apply(m04_constants::K_MOD_BUDGET_MAX, &consents, 1).unwrap();
        assert!((adj - m04_constants::K_MOD_BUDGET_MAX).abs() < 1e-10);
    }

    #[test]
    fn apply_raw_adj_exactly_one() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let adj = gate.apply(1.0, &consents, 1).unwrap();
        assert!((adj - 1.0).abs() < 1e-10, "neutral in → neutral out");
    }

    // ── Edge cases: tick counter boundary (u64::MAX) ──

    #[test]
    fn apply_with_max_tick_does_not_panic() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let adj = gate.apply(1.05, &consents, u64::MAX).unwrap();
        assert!(adj.is_finite());
    }

    // ── Edge cases: gate_count saturating at u64::MAX ──

    #[test]
    fn gate_count_saturates_at_u64_max() {
        let gate = ConsentGate::new();
        {
            let mut state = gate.state.write();
            state.gate_count = u64::MAX;
        }
        let consents = make_full_fleet(2);
        let _ = gate.apply(1.0, &consents, 1).unwrap();
        // saturating_add(1) on u64::MAX stays at u64::MAX
        assert_eq!(gate.gate_count(), u64::MAX);
    }

    // ── Edge cases: budget_min vs budget_max ordering ──

    #[test]
    fn set_budget_max_to_minimum_still_clamps_safely() {
        // set_budget_max(1.0) → budget_max = 1.0, budget_min = 0.85
        // This is a valid range; clamp(0.85, 1.0) works.
        let gate = ConsentGate::new();
        gate.set_budget_max(1.0);
        let consents = make_full_fleet(3);
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // 1.1 clamped to [0.85, 1.0] → 1.0
        assert!((adj - 1.0).abs() < 1e-10);
    }

    // ── Edge cases: newcomer dampening at exact boundary ──

    #[test]
    fn newcomer_ratio_exactly_half_no_dampening() {
        // newcomer_ratio = 0.5 → condition is > 0.5, not >=, so no dampening.
        let gate = ConsentGate::new();
        let consents = vec![
            make_consent("new1", 1.0, 0),   // newcomer
            make_consent("new2", 1.0, 0),   // newcomer
            make_consent("vet1", 1.0, 100), // veteran
            make_consent("vet2", 1.0, 100), // veteran
        ];
        // 2 of 4 are newcomers → ratio = 0.5, not > 0.5 → no dampening
        let adj_undampened = gate.apply(1.1, &consents, 1).unwrap();

        // Compare to all-newcomer fleet (ratio = 1.0 → dampened)
        let all_new: Vec<SphereConsent> = (0..4)
            .map(|i| make_consent(&format!("n{i}"), 1.0, 0))
            .collect();
        let gate2 = ConsentGate::new();
        let adj_dampened = gate2.apply(1.1, &all_new, 2).unwrap();

        assert!(
            (adj_undampened - 1.1).abs() < 1e-10,
            "at ratio=0.5 there must be no dampening"
        );
        assert!(
            (adj_dampened - 1.1).abs() > 1e-6,
            "all-newcomer fleet must be dampened"
        );
    }

    #[test]
    fn newcomer_ratio_just_over_half_triggers_dampening() {
        let gate = ConsentGate::new();
        // 3 newcomers, 2 veterans → ratio = 3/5 = 0.6 > 0.5 → dampening active
        let consents = vec![
            make_consent("n1", 1.0, 0),
            make_consent("n2", 1.0, 0),
            make_consent("n3", 1.0, 0),
            make_consent("v1", 1.0, 100),
            make_consent("v2", 1.0, 100),
        ];
        let adj = gate.apply(1.1, &consents, 1).unwrap();
        // Must be dampened: strictly less than 1.1 and greater than 1.0
        assert!(
            adj < 1.1 - 1e-10,
            "dampening must reduce the adjustment below 1.1: {adj}"
        );
        assert!(adj > 1.0, "dampening must not push below neutral: {adj}");
    }

    // ── Edge cases: apply_combined NaN propagation via product ──

    #[test]
    fn apply_combined_all_single_nan_bridge_fails() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        // One NaN bridge makes the product NaN → error returned.
        let result = gate.apply_combined_all(
            1.0, 1.0, 1.0, f64::NAN, 1.0, 1.0, &consents, 1,
        );
        assert!(result.is_err(), "NaN from any bridge must propagate as error");
    }

    #[test]
    fn apply_combined_all_single_infinity_bridge_fails() {
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let result = gate.apply_combined_all(
            1.0, f64::INFINITY, 1.0, 1.0, 1.0, 1.0, &consents, 1,
        );
        assert!(result.is_err());
    }

    #[test]
    fn apply_combined_all_zero_bridge_adj_passes_through_gating() {
        // Zero is a valid finite value. Product will be 0.0.
        // After gating: deviation = 0.0 - 1.0 = -1.0, scaled by receptivity.
        // Clamped to budget_min (0.85).
        let gate = ConsentGate::new();
        let consents = make_full_fleet(3);
        let result =
            gate.apply_combined_all(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, &consents, 1);
        assert!(result.is_ok());
        let adj = result.unwrap();
        assert!((adj - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    // ── Edge cases: empty string sphere IDs in overrides ──

    #[test]
    fn sphere_override_empty_string_id() {
        let gate = ConsentGate::new();
        gate.set_sphere_override("", 1.05);
        assert!((gate.sphere_override("").unwrap() - 1.05).abs() < 1e-10);
        gate.remove_sphere_override("");
        assert!(gate.sphere_override("").is_none());
    }

    // ── Edge cases: validate_adjustment at exact bounds ──

    #[test]
    fn validate_adjustment_at_k_mod_min() {
        let v = ConsentGate::validate_adjustment(m04_constants::K_MOD_MIN).unwrap();
        assert!((v - m04_constants::K_MOD_MIN).abs() < 1e-10);
    }

    #[test]
    fn validate_adjustment_at_k_mod_max() {
        let v = ConsentGate::validate_adjustment(m04_constants::K_MOD_MAX).unwrap();
        assert!((v - m04_constants::K_MOD_MAX).abs() < 1e-10);
    }

    #[test]
    fn validate_adjustment_neg_infinity_is_err() {
        assert!(ConsentGate::validate_adjustment(f64::NEG_INFINITY).is_err());
    }

    // ── Edge cases: large sphere fleet (1000 spheres) ──

    #[test]
    fn apply_with_large_fleet_does_not_panic() {
        let gate = ConsentGate::new();
        let consents: Vec<SphereConsent> = (0..1000)
            .map(|i| make_consent(&format!("s{i}"), 0.8, 100))
            .collect();
        let adj = gate.apply(1.05, &consents, 1).unwrap();
        assert!(adj.is_finite());
        assert!(adj >= m04_constants::K_MOD_BUDGET_MIN);
        assert!(adj <= m04_constants::K_MOD_BUDGET_MAX);
    }
}
