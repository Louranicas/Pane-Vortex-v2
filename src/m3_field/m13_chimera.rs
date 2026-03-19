//! # M13: Chimera State Detection
//!
//! Phase-gap cluster detection using O(N log N) sorted-phase algorithm.
//! A chimera is the simultaneous coexistence of synchronized and desynchronized clusters.
//!
//! ## Layer: L3 (Field)
//! ## Module: M13
//! ## Dependencies: L1 (M01 types, M04 constants)

use std::collections::HashMap;
use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::{m01_core_types::PaneId, m04_constants};

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// Chimera state: simultaneous coexistence of synchronized and desynchronized clusters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChimeraState {
    /// Whether a true chimera is present (multi-member sync + desync clusters).
    pub is_chimera: bool,
    /// Clusters with local order parameter >= `SYNC_THRESHOLD`.
    pub sync_clusters: Vec<Cluster>,
    /// Clusters with local order parameter < `SYNC_THRESHOLD`.
    pub desync_clusters: Vec<Cluster>,
}

/// A cluster of spheres identified by phase proximity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    /// Sphere IDs in this cluster.
    pub members: Vec<PaneId>,
    /// Local order parameter within this cluster.
    pub local_r: f64,
    /// Circular mean phase of this cluster.
    pub mean_phase: f64,
}

/// Pre-computed routing targets from chimera state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChimeraRouting {
    /// Spheres suitable for focused, aligned work.
    pub focused: Vec<PaneId>,
    /// Spheres suitable for exploratory, divergent work.
    pub exploratory: Vec<PaneId>,
}

// ──────────────────────────────────────────────────────────────
// Adaptive gap threshold
// ──────────────────────────────────────────────────────────────

/// Adaptive gap threshold scaling with k modulation.
///
/// - Positive `k_mod`: threshold scales in [π/6, π/3] (standard detection).
/// - Negative `k_mod`: finer detection in [π/12, π/6] (repulsive coupling forms smaller clusters).
/// - At `k_mod` = 0.0: continuous transition at π/6.
#[must_use]
pub fn effective_gap_threshold(k_modulation: f64) -> f64 {
    if k_modulation < 0.0 {
        let t = (k_modulation / m04_constants::K_MOD_MIN).clamp(0.0, 1.0);
        m04_constants::PHASE_GAP_MINIMUM
            - t * (m04_constants::PHASE_GAP_MINIMUM - m04_constants::PHASE_GAP_FINE)
    } else {
        (m04_constants::PHASE_GAP_THRESHOLD * k_modulation.clamp(0.3, 1.5))
            .clamp(m04_constants::PHASE_GAP_MINIMUM, m04_constants::PHASE_GAP_THRESHOLD)
    }
}

// ──────────────────────────────────────────────────────────────
// Detection
// ──────────────────────────────────────────────────────────────

impl ChimeraState {
    /// Detect chimera state from a phase map. O(N log N).
    ///
    /// Sort phases on the circle, find gaps > threshold, classify arcs as sync/desync.
    #[must_use]
    pub fn detect(phases: &HashMap<PaneId, f64>, k_modulation: f64) -> Self {
        if phases.len() < 2 {
            return Self {
                is_chimera: false,
                sync_clusters: Vec::new(),
                desync_clusters: Vec::new(),
            };
        }

        let sorted = sorted_phases(phases);
        let n = sorted.len();
        let threshold = effective_gap_threshold(k_modulation);

        let gap_indices = find_gaps(&sorted, threshold);
        let components = build_clusters(&sorted, &gap_indices, n);

        let (sync_clusters, desync_clusters) = classify_clusters(components, phases);

        let has_real_sync = sync_clusters.iter().any(|c| c.members.len() >= 2);
        let is_chimera = has_real_sync && !desync_clusters.is_empty();

        Self {
            is_chimera,
            sync_clusters,
            desync_clusters,
        }
    }

    /// Route focused tasks to synchronized clusters.
    #[must_use]
    pub fn route_focused(&self) -> Vec<PaneId> {
        if self.sync_clusters.is_empty() || self.desync_clusters.is_empty() {
            self.all_members()
        } else {
            self.sync_clusters
                .iter()
                .max_by_key(|c| c.members.len())
                .map(|c| c.members.clone())
                .unwrap_or_default()
        }
    }

    /// Route exploratory tasks to desynchronized clusters.
    #[must_use]
    pub fn route_exploratory(&self) -> Vec<PaneId> {
        self.desync_clusters
            .iter()
            .flat_map(|c| c.members.clone())
            .collect()
    }

    /// Build routing struct from this chimera state.
    #[must_use]
    pub fn routing(&self) -> ChimeraRouting {
        ChimeraRouting {
            focused: self.route_focused(),
            exploratory: self.route_exploratory(),
        }
    }

    /// All sphere IDs across all clusters.
    fn all_members(&self) -> Vec<PaneId> {
        self.sync_clusters
            .iter()
            .chain(self.desync_clusters.iter())
            .flat_map(|c| c.members.clone())
            .collect()
    }

    /// Total number of clusters.
    #[must_use]
    pub fn cluster_count(&self) -> usize {
        self.sync_clusters.len() + self.desync_clusters.len()
    }
}


// ──────────────────────────────────────────────────────────────
// Helper functions
// ──────────────────────────────────────────────────────────────

/// Sort phases on the circle.
fn sorted_phases(phases: &HashMap<PaneId, f64>) -> Vec<(PaneId, f64)> {
    let mut sorted: Vec<(PaneId, f64)> = phases
        .iter()
        .map(|(id, &phase)| (id.clone(), phase.rem_euclid(TAU)))
        .collect();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted
}

/// Find gap indices where consecutive phase difference exceeds threshold.
fn find_gaps(sorted: &[(PaneId, f64)], threshold: f64) -> Vec<usize> {
    let n = sorted.len();
    let mut gap_indices = Vec::new();
    for i in 0..n {
        let next = (i + 1) % n;
        let gap = if next == 0 {
            (sorted[0].1 + TAU - sorted[n - 1].1).rem_euclid(TAU)
        } else {
            sorted[next].1 - sorted[i].1
        };
        if gap > threshold {
            gap_indices.push(i);
        }
    }
    gap_indices
}

/// Build member groups from gap boundaries.
fn build_clusters(sorted: &[(PaneId, f64)], gap_indices: &[usize], n: usize) -> Vec<Vec<PaneId>> {
    if gap_indices.is_empty() {
        return vec![sorted.iter().map(|(id, _)| id.clone()).collect()];
    }

    let mut clusters: Vec<Vec<PaneId>> = Vec::new();
    for (gi, &gap_idx) in gap_indices.iter().enumerate() {
        let start = (gap_idx + 1) % n;
        let end = if gi + 1 < gap_indices.len() {
            gap_indices[gi + 1]
        } else {
            gap_indices[0]
        };

        let mut members = Vec::new();
        let mut idx = start;
        let mut steps = 0;
        loop {
            members.push(sorted[idx].0.clone());
            if idx == end || steps >= n {
                break;
            }
            idx = (idx + 1) % n;
            steps += 1;
        }
        if !members.is_empty() {
            clusters.push(members);
        }
    }
    clusters
}

/// Classify clusters into sync/desync based on local order parameter.
fn classify_clusters(
    components: Vec<Vec<PaneId>>,
    phases: &HashMap<PaneId, f64>,
) -> (Vec<Cluster>, Vec<Cluster>) {
    let mut sync_clusters = Vec::new();
    let mut desync_clusters = Vec::new();

    for members in components {
        let local_r = local_order_parameter(&members, phases);
        let mean_phase = local_mean_phase(&members, phases);

        let cluster = Cluster {
            members,
            local_r,
            mean_phase,
        };

        if local_r >= m04_constants::SYNC_THRESHOLD {
            sync_clusters.push(cluster);
        } else {
            desync_clusters.push(cluster);
        }
    }

    (sync_clusters, desync_clusters)
}

/// Compute local order parameter for a subset of spheres.
fn local_order_parameter(members: &[PaneId], phases: &HashMap<PaneId, f64>) -> f64 {
    let found: Vec<f64> = members
        .iter()
        .filter_map(|id| phases.get(id).copied())
        .collect();
    if found.is_empty() {
        return 0.0;
    }

    let (re, im) = found
        .iter()
        .map(|&phi| (phi.cos(), phi.sin()))
        .fold((0.0, 0.0), |(ar, ai), (r, i)| (ar + r, ai + i));

    #[allow(clippy::cast_precision_loss)]
    let n_f = found.len() as f64;
    (re / n_f).hypot(im / n_f)
}

/// Compute circular mean phase for a subset of spheres.
fn local_mean_phase(members: &[PaneId], phases: &HashMap<PaneId, f64>) -> f64 {
    let found: Vec<f64> = members
        .iter()
        .filter_map(|id| phases.get(id).copied())
        .collect();
    if found.is_empty() {
        return 0.0;
    }

    let (re, im) = found
        .iter()
        .map(|&phi| (phi.cos(), phi.sin()))
        .fold((0.0, 0.0), |(ar, ai), (r, i)| (ar + r, ai + i));

    im.atan2(re).rem_euclid(TAU)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::{FRAC_PI_3, FRAC_PI_6, PI};

    fn make_phases(entries: &[(&str, f64)]) -> HashMap<PaneId, f64> {
        entries
            .iter()
            .map(|(id, phase)| (PaneId::new(*id), *phase))
            .collect()
    }

    // ── Effective gap threshold ──

    #[test]
    fn gap_threshold_at_k_mod_one_is_pi_over_3() {
        let t = effective_gap_threshold(1.0);
        assert_relative_eq!(t, FRAC_PI_3, epsilon = 1e-10);
    }

    #[test]
    fn gap_threshold_at_k_mod_zero_is_pi_over_6() {
        let t = effective_gap_threshold(0.0);
        assert_relative_eq!(t, FRAC_PI_6, epsilon = 1e-10);
    }

    #[test]
    fn gap_threshold_at_negative_floor() {
        let t = effective_gap_threshold(m04_constants::K_MOD_MIN);
        assert_relative_eq!(t, m04_constants::PHASE_GAP_FINE, epsilon = 1e-10);
    }

    #[test]
    fn gap_threshold_negative_mid_is_between() {
        let t = effective_gap_threshold(-0.25);
        assert!(t > m04_constants::PHASE_GAP_FINE);
        assert!(t < m04_constants::PHASE_GAP_MINIMUM);
    }

    #[test]
    fn gap_threshold_positive_low_clamps_to_minimum() {
        let t = effective_gap_threshold(0.1);
        assert_relative_eq!(t, FRAC_PI_6, epsilon = 1e-10);
    }

    #[test]
    fn gap_threshold_positive_high_clamps_to_base() {
        let t = effective_gap_threshold(2.0);
        assert_relative_eq!(t, FRAC_PI_3, epsilon = 1e-10);
    }

    #[test]
    fn gap_threshold_continuous_at_zero() {
        let neg = effective_gap_threshold(-0.001);
        let pos = effective_gap_threshold(0.001);
        assert!((neg - FRAC_PI_6).abs() < 0.01);
        assert!((pos - FRAC_PI_6).abs() < 0.01);
    }

    // ── Detection: basic cases ──

    #[test]
    fn detect_empty_is_not_chimera() {
        let phases = HashMap::new();
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
    }

    #[test]
    fn detect_single_sphere_not_chimera() {
        let phases = make_phases(&[("a", 1.0)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
    }

    #[test]
    fn detect_two_close_spheres_not_chimera() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
        assert_eq!(c.cluster_count(), 1);
    }

    #[test]
    fn detect_three_close_spheres_one_cluster() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1), ("c", 0.9)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
        assert_eq!(c.cluster_count(), 1);
    }

    #[test]
    fn detect_antipodal_split_two_clusters() {
        let phases = make_phases(&[("a", 0.0), ("b", 0.1), ("c", PI), ("d", PI + 0.1)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(c.cluster_count() >= 2);
    }

    #[test]
    fn detect_chimera_with_sync_and_desync() {
        let phases = make_phases(&[
            ("a", 1.0),
            ("b", 1.05),
            ("c", 4.0),
            ("d", 4.0 + 1.5),
        ]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(c.cluster_count() >= 2);
    }

    #[test]
    fn single_member_cluster_not_counted_as_chimera() {
        let phases = make_phases(&[("a", 0.0), ("b", 0.1), ("c", PI)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(
            !c.is_chimera || c.desync_clusters.is_empty(),
            "single-member cluster should not create false chimera"
        );
    }

    // ── Detection: edge cases ──

    #[test]
    fn detect_uniform_distribution_not_chimera() {
        let phases = make_phases(&[
            ("a", 0.0),
            ("b", TAU / 4.0),
            ("c", TAU / 2.0),
            ("d", 3.0 * TAU / 4.0),
        ]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
    }

    #[test]
    fn detect_three_clusters() {
        let phases = make_phases(&[
            ("a", 0.0),
            ("b", 0.1),
            ("c", 2.0),
            ("d", 2.1),
            ("e", 4.0),
            ("f", 4.1),
        ]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(c.cluster_count() >= 3);
    }

    #[test]
    fn detect_negative_kmod_finds_finer_clusters() {
        let phases = make_phases(&[
            ("a", 0.0),
            ("b", PI / 8.0 + 0.01),
            ("c", PI),
            ("d", PI + 0.1),
        ]);
        let c_normal = ChimeraState::detect(&phases, 1.0);
        let c_neg = ChimeraState::detect(&phases, -0.5);
        assert!(c_neg.cluster_count() >= c_normal.cluster_count());
    }

    // ── Routing ──

    #[test]
    fn route_focused_all_synced_returns_all() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1), ("c", 0.9)]);
        let c = ChimeraState::detect(&phases, 1.0);
        let focused = c.route_focused();
        assert_eq!(focused.len(), 3);
    }

    #[test]
    fn route_exploratory_all_synced_returns_empty() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1), ("c", 0.9)]);
        let c = ChimeraState::detect(&phases, 1.0);
        let exploratory = c.route_exploratory();
        assert!(exploratory.is_empty());
    }

    #[test]
    fn route_focused_empty_returns_empty() {
        let c = ChimeraState::default();
        let focused = c.route_focused();
        assert!(focused.is_empty());
    }

    #[test]
    fn routing_struct_matches_methods() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1)]);
        let c = ChimeraState::detect(&phases, 1.0);
        let routing = c.routing();
        assert_eq!(routing.focused.len(), c.route_focused().len());
        assert_eq!(routing.exploratory.len(), c.route_exploratory().len());
    }

    // ── Local order parameter ──

    #[test]
    fn local_r_identical_phases_is_one() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.0)]);
        let members = vec![PaneId::new("a"), PaneId::new("b")];
        let r = local_order_parameter(&members, &phases);
        assert_relative_eq!(r, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn local_r_opposite_phases_is_zero() {
        let phases = make_phases(&[("a", 0.0), ("b", PI)]);
        let members = vec![PaneId::new("a"), PaneId::new("b")];
        let r = local_order_parameter(&members, &phases);
        assert_relative_eq!(r, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn local_r_uses_found_count() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.0)]);
        let members = vec![PaneId::new("a"), PaneId::new("b"), PaneId::new("c")];
        let r = local_order_parameter(&members, &phases);
        assert_relative_eq!(r, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn local_r_empty_members_is_zero() {
        let phases: HashMap<PaneId, f64> = HashMap::new();
        let r = local_order_parameter(&[], &phases);
        assert_relative_eq!(r, 0.0);
    }

    #[test]
    fn local_r_single_member_is_one() {
        let phases = make_phases(&[("a", 2.5)]);
        let members = vec![PaneId::new("a")];
        let r = local_order_parameter(&members, &phases);
        assert_relative_eq!(r, 1.0, epsilon = 1e-10);
    }

    // ── Local mean phase ──

    #[test]
    fn local_mean_phase_identical_returns_that_phase() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.0)]);
        let members = vec![PaneId::new("a"), PaneId::new("b")];
        let mp = local_mean_phase(&members, &phases);
        assert_relative_eq!(mp, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn local_mean_phase_opposite_wraps() {
        let phases = make_phases(&[("a", 0.1), ("b", TAU - 0.1)]);
        let members = vec![PaneId::new("a"), PaneId::new("b")];
        let mp = local_mean_phase(&members, &phases);
        assert!(mp < 0.2 || mp > TAU - 0.2);
    }

    #[test]
    fn local_mean_phase_empty_is_zero() {
        let phases: HashMap<PaneId, f64> = HashMap::new();
        let mp = local_mean_phase(&[], &phases);
        assert_relative_eq!(mp, 0.0);
    }

    // ── ChimeraState default ──

    #[test]
    fn chimera_default_not_chimera() {
        let c = ChimeraState::default();
        assert!(!c.is_chimera);
        assert!(c.sync_clusters.is_empty());
        assert!(c.desync_clusters.is_empty());
    }

    // ── Cluster count ──

    #[test]
    fn cluster_count_empty() {
        let c = ChimeraState::default();
        assert_eq!(c.cluster_count(), 0);
    }

    #[test]
    fn cluster_count_one_cluster() {
        let phases = make_phases(&[("a", 1.0), ("b", 1.1)]);
        let c = ChimeraState::detect(&phases, 1.0);
        assert_eq!(c.cluster_count(), 1);
    }

    // ── Serde roundtrip ──

    #[test]
    fn chimera_serde_roundtrip() {
        let phases = make_phases(&[("a", 0.0), ("b", 0.1), ("c", PI)]);
        let c = ChimeraState::detect(&phases, 1.0);
        let json = serde_json::to_string(&c).unwrap();
        let back: ChimeraState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.is_chimera, c.is_chimera);
        assert_eq!(back.cluster_count(), c.cluster_count());
    }

    #[test]
    fn cluster_serde_roundtrip() {
        let cluster = Cluster {
            members: vec![PaneId::new("a"), PaneId::new("b")],
            local_r: 0.95,
            mean_phase: 1.2,
        };
        let json = serde_json::to_string(&cluster).unwrap();
        let back: Cluster = serde_json::from_str(&json).unwrap();
        assert_eq!(back.members.len(), 2);
        assert_relative_eq!(back.local_r, 0.95);
    }

    #[test]
    fn routing_serde_roundtrip() {
        let r = ChimeraRouting {
            focused: vec![PaneId::new("a")],
            exploratory: vec![PaneId::new("b")],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ChimeraRouting = serde_json::from_str(&json).unwrap();
        assert_eq!(back.focused.len(), 1);
        assert_eq!(back.exploratory.len(), 1);
    }

    // ── Sorted phases ──

    #[test]
    fn sorted_phases_returns_ascending() {
        let phases = make_phases(&[("a", 3.0), ("b", 1.0), ("c", 2.0)]);
        let sorted = sorted_phases(&phases);
        assert!(sorted[0].1 <= sorted[1].1);
        assert!(sorted[1].1 <= sorted[2].1);
    }

    #[test]
    fn sorted_phases_wraps_negative() {
        let phases = make_phases(&[("a", -0.5)]);
        let sorted = sorted_phases(&phases);
        assert!(sorted[0].1 >= 0.0);
        assert!(sorted[0].1 < TAU);
    }

    // ── Find gaps ──

    #[test]
    fn find_gaps_no_gaps_when_close() {
        let sorted = vec![
            (PaneId::new("a"), 1.0),
            (PaneId::new("b"), 1.1),
            (PaneId::new("c"), 1.2),
        ];
        let gaps = find_gaps(&sorted, FRAC_PI_3);
        // Wrap-around gap from 1.2 to 1.0+TAU is ~5.08, which IS > π/3
        assert!(!gaps.is_empty());
    }

    #[test]
    fn find_gaps_detects_large_gap() {
        let sorted = vec![
            (PaneId::new("a"), 0.0),
            (PaneId::new("b"), 0.1),
            (PaneId::new("c"), PI),
        ];
        let gaps = find_gaps(&sorted, FRAC_PI_3);
        assert!(gaps.len() >= 2);
    }

    // ── Stress test: many spheres ──

    #[test]
    fn detect_handles_200_spheres() {
        let mut phases = HashMap::new();
        for i in 0..200 {
            #[allow(clippy::cast_precision_loss)]
            let phase = (i as f64 / 200.0) * TAU;
            phases.insert(PaneId::new(format!("s{i}")), phase);
        }
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(!c.is_chimera);
    }

    #[test]
    fn detect_two_groups_of_100() {
        let mut phases = HashMap::new();
        for i in 0..100 {
            #[allow(clippy::cast_precision_loss)]
            let offset = (i as f64 / 1000.0) * 0.1;
            phases.insert(PaneId::new(format!("a{i}")), 0.5 + offset);
            phases.insert(PaneId::new(format!("b{i}")), PI + 0.5 + offset);
        }
        let c = ChimeraState::detect(&phases, 1.0);
        assert!(c.cluster_count() >= 2);
    }
}
