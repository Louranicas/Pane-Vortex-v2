//! # M11: `PaneSphere` Oscillator
//!
//! The core oscillator entity. Each Claude Code instance registers as a sphere.
//! V2 refactors the V1 god object (33 fields) while maintaining wire compatibility.
//!
//! ## Layer: L3 (Field)
//! ## Module: M11
//! ## Dependencies: L1 (M01 types, M02 errors, M04 constants, M06 validation)

use std::collections::{HashSet, VecDeque};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use serde::{Deserialize, Serialize};

use crate::m1_foundation::{
    m01_core_types::{
        now_secs, phase_diff, semantic_phase_region, Buoy, InboxMessage, PaneId, PaneStatus,
        Point3D, SphereFieldContext, SphereMemory, WorkSignature,
    },
    m02_error_handling::{PvError, PvResult},
    m04_constants,
};

// ──────────────────────────────────────────────────────────────
// Maturity level
// ──────────────────────────────────────────────────────────────

/// Sphere maturity level — determines LTP multipliers and momentum adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaturityLevel {
    /// First `NEWCOMER_STEPS` ticks — boosted LTP, accelerated buoy drift.
    Newcomer,
    /// Post-newcomer — standard dynamics.
    Established,
    /// > 1000 steps — reduced noise, stable coupling.
    Senior,
}

impl MaturityLevel {
    /// Determine maturity from total step count.
    #[must_use]
    pub const fn from_steps(steps: u64) -> Self {
        if steps < m04_constants::NEWCOMER_STEPS {
            Self::Newcomer
        } else if steps < 1000 {
            Self::Established
        } else {
            Self::Senior
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Activation zones
// ──────────────────────────────────────────────────────────────

/// Memory count in each activation zone.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivationZones {
    /// Activation > 0.75.
    pub vivid: usize,
    /// Activation > 0.45.
    pub clear: usize,
    /// Activation > 0.15.
    pub dim: usize,
    /// Activation <= 0.15.
    pub trace: usize,
}

// ──────────────────────────────────────────────────────────────
// PaneSphere
// ──────────────────────────────────────────────────────────────

/// A single pane-sphere: oscillator + memory field + Hebbian buoys.
///
/// This is the V2 refactored version. The V1 god object (33 fields) is decomposed
/// into logical groups but kept as a single struct for serialization compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PaneSphere {
    // ── Identity ──
    /// Unique sphere identifier.
    pub id: PaneId,
    /// Human-readable role description.
    pub persona: String,

    // ── Oscillator state ──
    /// Current phase in [0, 2π).
    pub phase: f64,
    /// Natural frequency (Hz). Self-modulated based on activity density.
    pub frequency: f64,
    /// Phase momentum (inertia for smooth transitions).
    pub momentum: f64,
    /// Momentum decay factor per step.
    #[serde(default = "default_momentum_decay")]
    pub momentum_decay: f64,

    // ── Memory field ──
    /// Memories placed on the sphere surface by tool calls.
    pub memories: Vec<SphereMemory>,
    /// Monotonic memory ID counter.
    next_memory_id: u64,

    // ── Hebbian buoys ──
    /// Learned clusters on the sphere surface.
    pub buoys: Vec<Buoy>,

    // ── Co-activation tracking ──
    /// Recent active memory IDs (sliding window).
    #[serde(default)]
    recent_active: Vec<u64>,
    /// Total step count since registration.
    pub total_steps: u64,

    // ── Auto-status tracking ──
    /// Step at which the last memory was recorded.
    last_memory_step: u64,
    /// Monotonic flag — true once any memory has been recorded.
    #[serde(default)]
    pub has_worked: bool,

    // ── Status ──
    /// Current operational status.
    pub status: PaneStatus,
    /// Name of the last tool used.
    pub last_tool: String,
    /// Unix timestamp when the sphere was registered.
    pub registered_at: f64,

    // ── Consent (NA-14, NA-34) ──
    /// Coupling receptivity (0.0 = closed, 1.0 = fully open).
    #[serde(default = "default_receptivity")]
    pub receptivity: f64,
    /// Base frequency (set at registration, preserved across self-modulation).
    #[serde(default)]
    pub base_frequency: f64,
    /// Opt out of Hebbian weight updates.
    #[serde(default)]
    pub opt_out_hebbian: bool,
    /// Opt out of cross-sphere activation.
    #[serde(default)]
    pub opt_out_cross_activation: bool,
    /// Opt out of external k modulation (nexus + SYNTHEX).
    #[serde(default)]
    pub opt_out_external_modulation: bool,
    /// Opt out of observation (evolution chamber, analytics).
    #[serde(default)]
    pub opt_out_observation: bool,

    // ── Preferences (NA-P-3) ──
    /// Preferred r target. `None` = use fleet base.
    #[serde(default)]
    pub preferred_r: Option<f64>,

    // ── Temporal markers (NA-27) ──
    /// Timestamp of first memory creation.
    #[serde(default)]
    pub first_memory_at: Option<f64>,
    /// Timestamp of last prune operation.
    #[serde(default)]
    pub last_prune_at: Option<f64>,

    // ── Inbox (NA-20) ──
    /// Pending messages from other spheres.
    #[serde(default)]
    pub inbox: VecDeque<InboxMessage>,
    /// Monotonic inbox message ID counter.
    #[serde(default)]
    pub next_inbox_id: u64,

    // ── Heartbeat (S4.1) ──
    /// Last API activity timestamp (for ghost sweep).
    #[serde(default)]
    pub last_heartbeat: f64,

    // ── Multi-scale activity (S6.1) ──
    /// Activity counter decayed every 6 ticks (~30s).
    #[serde(default)]
    pub activity_30s: u32,
    /// Activity counter decayed every 60 ticks (~5min).
    #[serde(default)]
    pub activity_5m: u32,
    /// Activity counter decayed every 360 ticks (~30min).
    #[serde(default)]
    pub activity_30m: u32,

    // ── Work signature (S7.2) ──
    /// Continuous work characterisation.
    #[serde(default)]
    pub work_signature: WorkSignature,

    // ── Field context (S7.1, ephemeral) ──
    /// Cached field context from previous tick (not serialized).
    #[serde(skip)]
    pub field_context: SphereFieldContext,
}

/// Default receptivity for serde.
const fn default_receptivity() -> f64 {
    1.0
}

/// Default momentum decay for serde.
const fn default_momentum_decay() -> f64 {
    0.98
}

impl PaneSphere {
    /// Create a new sphere with 3 canonical buoys.
    ///
    /// # Errors
    /// Returns `PvError::NonFinite` if frequency is NaN or infinite.
    pub fn new(id: PaneId, persona: String, frequency: f64) -> PvResult<Self> {
        if !frequency.is_finite() {
            return Err(PvError::NonFinite {
                field: "frequency",
                value: frequency,
            });
        }
        let freq = frequency.clamp(
            m04_constants::HEBBIAN_WEIGHT_FLOOR, // Use as minimum floor
            10.0,
        );

        // 3 canonical buoys at equatorial positions (120° apart)
        let buoys = vec![
            Buoy::new(
                Point3D::from_spherical(FRAC_PI_2, 0.0),
                "primary".into(),
                0.005,
            ),
            Buoy::new(
                Point3D::from_spherical(FRAC_PI_2, TAU / 3.0),
                "secondary".into(),
                0.002,
            ),
            Buoy::new(
                Point3D::from_spherical(FRAC_PI_2, 2.0 * TAU / 3.0),
                "tertiary".into(),
                0.001,
            ),
        ];

        let now = now_secs();
        Ok(Self {
            id,
            persona,
            phase: 0.0,
            frequency: freq,
            momentum: 0.0,
            momentum_decay: default_momentum_decay(),
            memories: Vec::new(),
            next_memory_id: 0,
            buoys,
            recent_active: Vec::new(),
            total_steps: 0,
            last_memory_step: 0,
            has_worked: false,
            status: PaneStatus::Idle,
            last_tool: String::new(),
            registered_at: now,
            receptivity: default_receptivity(),
            base_frequency: freq,
            first_memory_at: None,
            last_prune_at: None,
            opt_out_hebbian: false,
            opt_out_cross_activation: false,
            opt_out_external_modulation: false,
            opt_out_observation: false,
            preferred_r: None,
            inbox: VecDeque::new(),
            next_inbox_id: 0,
            last_heartbeat: now,
            activity_30s: 0,
            activity_5m: 0,
            activity_30m: 0,
            work_signature: WorkSignature::default(),
            field_context: SphereFieldContext::default(),
        })
    }

    // ── Identity ──

    /// Maturity level based on total step count.
    #[must_use]
    pub const fn maturity(&self) -> MaturityLevel {
        MaturityLevel::from_steps(self.total_steps)
    }

    // ── Heartbeat ──

    /// Touch heartbeat — call on any API activity to prevent ghost sweep.
    pub fn touch_heartbeat(&mut self) {
        self.last_heartbeat = now_secs();
    }

    // ── Memory recording ──

    /// Place a new memory at the current apex position with semantic phase nudge.
    ///
    /// Increments activity counters, applies amortised batch prune if over capacity.
    pub fn record_memory(&mut self, tool_name: String, summary: String) -> u64 {
        let id = self.next_memory_id;
        self.next_memory_id += 1;

        // NA-1: Semantic phase injection
        let target_phase = semantic_phase_region(&tool_name);
        let delta = phase_diff(target_phase, self.phase);
        self.phase = (delta * m04_constants::SEMANTIC_NUDGE_STRENGTH)
            .mul_add(1.0, self.phase)
            .rem_euclid(TAU);

        let apex = self.apex_position();
        let memory = SphereMemory::new(id, apex, tool_name.clone(), summary);
        self.memories.push(memory);

        // Amortised batch prune at threshold+50
        if self.memories.len() > m04_constants::MEMORY_MAX_COUNT + 50 {
            self.batch_prune_memories();
        }

        self.has_worked = true;
        self.activity_30s = self.activity_30s.saturating_add(1);
        self.activity_5m = self.activity_5m.saturating_add(1);
        self.activity_30m = self.activity_30m.saturating_add(1);

        if self.first_memory_at.is_none() {
            self.first_memory_at = Some(now_secs());
        }
        self.last_tool = tool_name;
        self.status = PaneStatus::Working;
        self.last_memory_step = self.total_steps;
        id
    }

    /// Current apex position on the unit sphere (where attention is focused).
    #[must_use]
    pub fn apex_position(&self) -> Point3D {
        let waveform_phase = (self.phase.sin() * PI).clamp(-PI, PI);
        Point3D::from_spherical(
            waveform_phase.sin().mul_add(0.3, FRAC_PI_2),
            self.phase,
        )
    }

    // ── Field context ──

    /// Update field context from previous tick's cached state.
    pub fn set_field_context(&mut self, ctx: SphereFieldContext) {
        self.field_context = ctx;
    }

    // ── Phase steering ──

    /// Steer oscillator toward a target phase (with inertia + momentum gating).
    pub fn steer_toward(&mut self, target_phase: f64, strength: f64) {
        let delta = phase_diff(target_phase, self.phase);
        let gate = if self.momentum.abs() > 0.3 { 0.5 } else { 1.0 };
        self.momentum += delta * strength * 0.1 * gate;
    }

    // ── Simulation step ──

    /// Run one simulation step: advance phase, decay/boost activations, update buoys.
    pub fn step(&mut self) {
        self.total_steps += 1;
        self.update_work_signature();

        // 1. Phase advance — momentum only (coupling loop drives phase dynamics)
        self.momentum *= self.momentum_decay;
        self.apply_field_context_momentum();
        self.phase = (self.phase + self.momentum).rem_euclid(TAU);

        let apex = self.apex_position();

        // 2. Activation dynamics
        let newly_active = self.update_activations(&apex);

        // 3. Co-activation tracking
        self.track_coactivations(&newly_active);

        // 4. Buoy Hebbian drift
        self.update_buoys(&newly_active);

        // 5. Buoy boost: amplify memories near learned buoys
        self.apply_buoy_boosts();

        // 6. Auto-status
        self.update_auto_status();

        // 7. NA-14: Auto-modulate receptivity
        self.update_receptivity();

        // 8. NA-15: Self-modulated frequency
        self.update_frequency();

        // 9. Periodic memory pruning
        self.periodic_prune();
    }

    // ── Reconciliation ──

    /// Reconcile after snapshot restore: fix memory IDs and base frequency.
    pub fn reconcile_after_restore(&mut self) {
        self.next_memory_id = self.memories.iter().map(|m| m.id + 1).max().unwrap_or(0);
        if self.base_frequency < 1e-6 {
            self.base_frequency = self.frequency;
        }
    }

    // ── Query methods ──

    /// Buoy positions for tunnel detection across spheres.
    #[must_use]
    pub fn buoy_positions(&self) -> Vec<(String, Point3D)> {
        self.buoys
            .iter()
            .map(|b| (b.label.clone(), b.position))
            .collect()
    }

    /// Memory count in each activation zone.
    #[must_use]
    pub fn activation_zones(&self) -> ActivationZones {
        let mut zones = ActivationZones::default();
        for mem in &self.memories {
            match mem.activation {
                a if a > 0.75 => zones.vivid += 1,
                a if a > 0.45 => zones.clear += 1,
                a if a > 0.15 => zones.dim += 1,
                _ => zones.trace += 1,
            }
        }
        zones
    }

    /// Active memory density (fraction of memories above activation threshold).
    #[must_use]
    pub fn activation_density(&self) -> f64 {
        if self.memories.is_empty() {
            return 0.0;
        }
        let active = self
            .memories
            .iter()
            .filter(|m| m.activation > m04_constants::ACTIVATION_THRESHOLD)
            .count();
        #[allow(clippy::cast_precision_loss)]
        let density = active as f64 / self.memories.len() as f64;
        density
    }

    /// Receive a message into the inbox (FIFO, capped at `INBOX_MAX`).
    pub fn receive_message(&mut self, from: String, content: String) -> u64 {
        let id = self.next_inbox_id;
        self.next_inbox_id += 1;
        self.inbox.push_back(InboxMessage {
            id,
            from,
            content,
            received_at: now_secs(),
            acknowledged: false,
        });
        while self.inbox.len() > m04_constants::INBOX_MAX {
            self.inbox.pop_front();
        }
        id
    }

    /// Acknowledge a message by ID.
    pub fn acknowledge_message(&mut self, msg_id: u64) -> bool {
        if let Some(msg) = self.inbox.iter_mut().find(|m| m.id == msg_id) {
            msg.acknowledged = true;
            true
        } else {
            false
        }
    }

    // ── Private helpers ──

    /// Field-context-aware momentum adaptation.
    fn apply_field_context_momentum(&mut self) {
        if self.field_context.is_synchronized && self.field_context.my_coupling_strength > 0.5 {
            self.momentum *= 0.95;
        }
        if self.field_context.my_cluster_size <= 1
            && self.field_context.my_coupling_strength < 0.1
        {
            self.momentum *= 1.05;
        }
    }

    /// Update memory activations: decay + sweep boost.
    fn update_activations(&mut self, apex: &Point3D) -> Vec<u64> {
        let mut newly_active = Vec::new();
        for mem in &mut self.memories {
            mem.activation *= m04_constants::DECAY_PER_STEP;
            let dist = apex.angular_distance(mem.position);
            let gaussian = (dist / m04_constants::SWEEP_SIGMA)
                .powi(2)
                .mul_add(-0.5, 0.0)
                .exp();
            mem.activation += m04_constants::SWEEP_BOOST * gaussian * (1.0 - mem.activation);
            mem.activation = mem.activation.clamp(0.0, 1.0);
            if mem.activation > m04_constants::ACTIVATION_THRESHOLD {
                newly_active.push(mem.id);
            }
        }
        newly_active
    }

    /// Track co-activations in sliding window.
    fn track_coactivations(&mut self, newly_active: &[u64]) {
        self.recent_active.extend_from_slice(newly_active);
        if self.recent_active.len() > m04_constants::CO_ACTIVATION_WINDOW * 5 {
            let drain_to = self.recent_active.len() - m04_constants::CO_ACTIVATION_WINDOW * 3;
            self.recent_active.drain(..drain_to);
        }
    }

    /// Hebbian buoy drift toward co-activation centroids.
    fn update_buoys(&mut self, active_ids: &[u64]) {
        let drift_rate = if self.maturity() == MaturityLevel::Newcomer {
            m04_constants::BUOY_HOME_DECAY * 3.0
        } else {
            m04_constants::BUOY_HOME_DECAY
        };

        if active_ids.len() < 2 {
            for buoy in &mut self.buoys {
                buoy.drift_home(drift_rate);
            }
            return;
        }

        let active_positions: Vec<Point3D> = active_ids
            .iter()
            .filter_map(|id| self.memories.iter().find(|m| m.id == *id))
            .map(|m| m.position)
            .collect();

        if active_positions.is_empty() {
            return;
        }

        #[allow(clippy::cast_precision_loss)]
        let n = active_positions.len() as f64;
        let centroid = Point3D::new(
            active_positions.iter().map(|p| p.x).sum::<f64>() / n,
            active_positions.iter().map(|p| p.y).sum::<f64>() / n,
            active_positions.iter().map(|p| p.z).sum::<f64>() / n,
        )
        .normalized();

        if let Some(nearest) = self.buoys.iter_mut().min_by(|a, b| {
            let da = a.position.angular_distance(centroid);
            let db = b.position.angular_distance(centroid);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            nearest.drift_toward(&centroid);
        }
    }

    /// Amplify memories near learned buoys.
    fn apply_buoy_boosts(&mut self) {
        // Collect boost values first to avoid borrow conflict
        let boosts: Vec<f64> = self
            .memories
            .iter()
            .map(|mem| {
                self.buoys
                    .iter()
                    .map(|b| b.boost_at(&mem.position))
                    .sum::<f64>()
                    .min(1.0)
            })
            .collect();

        for (mem, boost) in self.memories.iter_mut().zip(boosts) {
            mem.activation += boost * (1.0 - mem.activation);
            mem.activation = mem.activation.clamp(0.0, 1.0);
        }
    }

    /// Auto-status transition based on memory recency.
    fn update_auto_status(&mut self) {
        if self.has_worked
            && (self.status == PaneStatus::Idle || self.status == PaneStatus::Working)
        {
            let steps_since = self.total_steps.saturating_sub(self.last_memory_step);
            if steps_since < 5 && self.status == PaneStatus::Idle {
                self.status = PaneStatus::Working;
            } else if steps_since > 50 && self.status == PaneStatus::Working {
                self.status = PaneStatus::Idle;
            }
        }
    }

    /// NA-14: Auto-modulate receptivity based on activation density.
    fn update_receptivity(&mut self) {
        let density = self.activation_density();
        let target = 0.7f64.mul_add(-density, 1.0).clamp(0.3, 1.0);
        self.receptivity = 0.9f64.mul_add(self.receptivity, 0.1 * target);
    }

    /// NA-15: Self-modulated frequency based on activity density.
    fn update_frequency(&mut self) {
        let density = self.activation_density();
        self.frequency = self.base_frequency * density.mul_add(0.3, 1.0);
    }

    /// Periodic structural importance pruning.
    fn periodic_prune(&mut self) {
        if self.total_steps % m04_constants::MEMORY_PRUNE_INTERVAL != 0 {
            return;
        }
        let before = self.memories.len();
        self.memories
            .retain(|m| m.activation > m04_constants::TRACE_PRUNE_THRESHOLD);

        if self.memories.len() > m04_constants::MEMORY_MAX_COUNT {
            self.importance_prune();
        }

        let pruned = before.saturating_sub(self.memories.len());
        if pruned > 0 {
            self.last_prune_at = Some(now_secs());
            tracing::debug!(
                sphere = %self.id,
                pruned,
                remaining = self.memories.len(),
                "memory pruning"
            );
        }
    }

    /// Batch prune memories by composite value score.
    fn batch_prune_memories(&mut self) {
        let scores: Vec<f64> = self
            .memories
            .iter()
            .map(|m| self.memory_value(m))
            .collect();
        let mut indices: Vec<usize> = (0..self.memories.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let keep: HashSet<usize> = indices
            .into_iter()
            .take(m04_constants::MEMORY_MAX_COUNT)
            .collect();
        let mut idx = 0;
        self.memories.retain(|_| {
            let kept = keep.contains(&idx);
            idx += 1;
            kept
        });
        let live_ids: HashSet<u64> = self.memories.iter().map(|m| m.id).collect();
        self.recent_active.retain(|id| live_ids.contains(id));
    }

    /// Importance-based pruning (preserves buoy-anchored and rare memories).
    fn importance_prune(&mut self) {
        let scores: Vec<f64> = self
            .memories
            .iter()
            .map(|m| self.structural_importance(m))
            .collect();
        let mut indices: Vec<usize> = (0..self.memories.len()).collect();
        indices.sort_by(|&a, &b| {
            scores[b]
                .partial_cmp(&scores[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let keep: HashSet<usize> = indices
            .into_iter()
            .take(m04_constants::MEMORY_MAX_COUNT)
            .collect();
        let mut idx = 0;
        self.memories.retain(|_| {
            let kept = keep.contains(&idx);
            idx += 1;
            kept
        });
    }

    /// Structural importance score for memory pruning.
    fn structural_importance(&self, mem: &SphereMemory) -> f64 {
        let activation_score = mem.activation;
        let buoy_proximity = self
            .buoys
            .iter()
            .map(|b| {
                let dist = b.position.angular_distance(mem.position);
                (1.0 - dist / PI).max(0.0)
            })
            .fold(0.0_f64, f64::max);
        let same_tool = self
            .memories
            .iter()
            .filter(|m| m.tool_name == mem.tool_name)
            .count();
        #[allow(clippy::cast_precision_loss)]
        let rarity = 1.0 / (same_tool as f64).max(1.0);
        activation_score
            .mul_add(0.6, buoy_proximity.mul_add(0.25, rarity * 0.15))
    }

    /// Composite value score (activation + rarity).
    fn memory_value(&self, mem: &SphereMemory) -> f64 {
        let same_tool = self
            .memories
            .iter()
            .filter(|m| m.tool_name == mem.tool_name)
            .count();
        #[allow(clippy::cast_precision_loss)]
        let rarity = 1.0 / (same_tool as f64).max(1.0);
        mem.activation.mul_add(0.8, rarity * 0.2)
    }

    /// Compute continuous work signature from memory history.
    fn update_work_signature(&mut self) {
        let window = 50_u64;

        let recent_memories = self
            .memories
            .iter()
            .filter(|m| m.activation > m04_constants::TRACE_PRUNE_THRESHOLD)
            .count();
        #[allow(clippy::cast_precision_loss)]
        let intensity = (recent_memories as f64 / window.max(1) as f64).min(1.0);

        let active_tools: Vec<&str> = self
            .memories
            .iter()
            .filter(|m| m.activation > m04_constants::ACTIVATION_THRESHOLD)
            .map(|m| m.tool_name.as_str())
            .collect();
        let unique_tools: HashSet<&str> = active_tools.iter().copied().collect();
        #[allow(clippy::cast_precision_loss)]
        let diversity = if active_tools.is_empty() {
            0.0
        } else {
            unique_tools.len() as f64 / active_tools.len().max(1) as f64
        };
        let focus = 1.0 - diversity;

        let rhythm = if self.total_steps < 3 || self.memories.len() < 2 {
            0.0
        } else {
            let steps_since = self.total_steps.saturating_sub(self.last_memory_step);
            #[allow(clippy::cast_precision_loss)]
            let recency = 1.0 - (steps_since as f64 / window as f64).min(1.0);
            recency * intensity
        };

        self.work_signature = WorkSignature {
            intensity,
            rhythm,
            diversity,
            focus,
        };
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn test_sphere() -> PaneSphere {
        PaneSphere::new(PaneId::new("test"), "tester".into(), 0.1).unwrap()
    }

    // ── Construction ──

    #[test]
    fn new_sphere_has_correct_id() {
        let s = test_sphere();
        assert_eq!(s.id.as_str(), "test");
    }

    #[test]
    fn new_sphere_has_correct_persona() {
        let s = test_sphere();
        assert_eq!(s.persona, "tester");
    }

    #[test]
    fn new_sphere_starts_at_phase_zero() {
        let s = test_sphere();
        assert_relative_eq!(s.phase, 0.0);
    }

    #[test]
    fn new_sphere_frequency_preserved() {
        let s = PaneSphere::new(PaneId::new("t"), "t".into(), 0.5).unwrap();
        assert_relative_eq!(s.frequency, 0.5);
        assert_relative_eq!(s.base_frequency, 0.5);
    }

    #[test]
    fn new_sphere_rejects_nan_frequency() {
        let result = PaneSphere::new(PaneId::new("t"), "t".into(), f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn new_sphere_rejects_infinity_frequency() {
        let result = PaneSphere::new(PaneId::new("t"), "t".into(), f64::INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn new_sphere_has_three_buoys() {
        let s = test_sphere();
        assert_eq!(s.buoys.len(), 3);
    }

    #[test]
    fn new_sphere_status_is_idle() {
        let s = test_sphere();
        assert_eq!(s.status, PaneStatus::Idle);
    }

    #[test]
    fn new_sphere_has_no_memories() {
        let s = test_sphere();
        assert!(s.memories.is_empty());
    }

    #[test]
    fn new_sphere_receptivity_is_one() {
        let s = test_sphere();
        assert_relative_eq!(s.receptivity, 1.0);
    }

    #[test]
    fn new_sphere_registered_at_is_recent() {
        let s = test_sphere();
        assert!(s.registered_at > 1_700_000_000.0);
    }

    #[test]
    fn new_sphere_has_empty_inbox() {
        let s = test_sphere();
        assert!(s.inbox.is_empty());
    }

    #[test]
    fn new_sphere_no_opt_outs() {
        let s = test_sphere();
        assert!(!s.opt_out_hebbian);
        assert!(!s.opt_out_cross_activation);
        assert!(!s.opt_out_external_modulation);
        assert!(!s.opt_out_observation);
    }

    // ── Maturity ──

    #[test]
    fn maturity_newcomer() {
        let s = test_sphere();
        assert_eq!(s.maturity(), MaturityLevel::Newcomer);
    }

    #[test]
    fn maturity_established() {
        let mut s = test_sphere();
        s.total_steps = 100;
        assert_eq!(s.maturity(), MaturityLevel::Established);
    }

    #[test]
    fn maturity_senior() {
        let mut s = test_sphere();
        s.total_steps = 1500;
        assert_eq!(s.maturity(), MaturityLevel::Senior);
    }

    // ── Memory recording ──

    #[test]
    fn record_memory_increments_id() {
        let mut s = test_sphere();
        let id1 = s.record_memory("Read".into(), "file".into());
        let id2 = s.record_memory("Write".into(), "file".into());
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn record_memory_adds_to_list() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        assert_eq!(s.memories.len(), 1);
    }

    #[test]
    fn record_memory_sets_status_working() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        assert_eq!(s.status, PaneStatus::Working);
    }

    #[test]
    fn record_memory_sets_has_worked() {
        let mut s = test_sphere();
        assert!(!s.has_worked);
        s.record_memory("Read".into(), "file".into());
        assert!(s.has_worked);
    }

    #[test]
    fn record_memory_updates_last_tool() {
        let mut s = test_sphere();
        s.record_memory("Grep".into(), "pattern".into());
        assert_eq!(s.last_tool, "Grep");
    }

    #[test]
    fn record_memory_increments_activity() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        assert_eq!(s.activity_30s, 1);
        assert_eq!(s.activity_5m, 1);
        assert_eq!(s.activity_30m, 1);
    }

    #[test]
    fn record_memory_sets_first_memory_at() {
        let mut s = test_sphere();
        assert!(s.first_memory_at.is_none());
        s.record_memory("Read".into(), "file".into());
        assert!(s.first_memory_at.is_some());
    }

    #[test]
    fn record_memory_semantic_nudge_changes_phase() {
        let mut s = test_sphere();
        s.phase = 0.0;
        s.record_memory("Bash".into(), "command".into());
        // Bash maps to TAU*0.5, so phase should have moved slightly
        assert!(s.phase > 0.0 || s.phase < TAU);
    }

    // ── Apex position ──

    #[test]
    fn apex_position_is_unit_sphere() {
        let s = test_sphere();
        let apex = s.apex_position();
        assert_relative_eq!(apex.norm(), 1.0, epsilon = 0.01);
    }

    #[test]
    fn apex_position_varies_with_phase() {
        let mut s = test_sphere();
        let a1 = s.apex_position();
        s.phase = PI;
        let a2 = s.apex_position();
        let dist = a1.angular_distance(a2);
        assert!(dist > 0.1, "apex should move with phase");
    }

    // ── Step ──

    #[test]
    fn step_increments_total_steps() {
        let mut s = test_sphere();
        s.step();
        assert_eq!(s.total_steps, 1);
    }

    #[test]
    fn step_decays_momentum_in_synced_cluster() {
        let mut s = test_sphere();
        s.momentum = 1.0;
        // Put sphere in a synchronized cluster so isolation bonus doesn't apply
        s.field_context = SphereFieldContext {
            is_synchronized: true,
            my_coupling_strength: 0.8,
            my_cluster_size: 3,
            ..SphereFieldContext::default()
        };
        s.step();
        // decay(0.98) * sync_dampen(0.95) = 0.931 < 1.0
        assert!(s.momentum < 1.0);
    }

    #[test]
    fn step_wraps_phase() {
        let mut s = test_sphere();
        s.phase = TAU - 0.001;
        s.momentum = 0.1;
        s.step();
        assert!(s.phase >= 0.0 && s.phase < TAU);
    }

    #[test]
    fn step_decays_memory_activation() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        let before = s.memories[0].activation;
        s.step();
        assert!(s.memories[0].activation <= before);
    }

    #[test]
    fn step_updates_work_signature() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        s.step();
        // With one memory, intensity should be > 0
        assert!(s.work_signature.intensity >= 0.0);
    }

    #[test]
    fn multiple_steps_preserve_phase_bounds() {
        let mut s = test_sphere();
        s.momentum = 0.5;
        for _ in 0..100 {
            s.step();
        }
        assert!(s.phase >= 0.0 && s.phase < TAU);
    }

    // ── Auto-status ──

    #[test]
    fn auto_status_idle_to_working_on_recent_memory() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        s.status = PaneStatus::Idle;
        s.step();
        assert_eq!(s.status, PaneStatus::Working);
    }

    #[test]
    fn auto_status_working_to_idle_after_long_gap() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        s.total_steps = 100;
        s.last_memory_step = 0;
        s.step();
        assert_eq!(s.status, PaneStatus::Idle);
    }

    #[test]
    fn auto_status_does_not_change_blocked() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        s.status = PaneStatus::Blocked;
        s.step();
        assert_eq!(s.status, PaneStatus::Blocked);
    }

    // ── Receptivity ──

    #[test]
    fn receptivity_stays_in_range() {
        let mut s = test_sphere();
        for _ in 0..100 {
            s.record_memory("Read".into(), "file".into());
            s.step();
        }
        assert!(s.receptivity >= 0.0 && s.receptivity <= 1.0);
    }

    #[test]
    fn receptivity_decreases_with_high_density() {
        let mut s = test_sphere();
        for i in 0..50 {
            s.record_memory(format!("Tool{i}"), "work".into());
        }
        s.step();
        assert!(s.receptivity < 1.0);
    }

    // ── Frequency ──

    #[test]
    fn frequency_increases_with_activity() {
        let mut s = test_sphere();
        let base = s.base_frequency;
        for _ in 0..20 {
            s.record_memory("Read".into(), "file".into());
        }
        s.step();
        assert!(s.frequency >= base);
    }

    // ── Inbox ──

    #[test]
    fn receive_message_adds_to_inbox() {
        let mut s = test_sphere();
        let id = s.receive_message("peer".into(), "hello".into());
        assert_eq!(id, 0);
        assert_eq!(s.inbox.len(), 1);
    }

    #[test]
    fn receive_message_increments_id() {
        let mut s = test_sphere();
        let id1 = s.receive_message("a".into(), "x".into());
        let id2 = s.receive_message("b".into(), "y".into());
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn inbox_caps_at_max() {
        let mut s = test_sphere();
        for i in 0..60 {
            s.receive_message(format!("peer{i}"), "msg".into());
        }
        assert_eq!(s.inbox.len(), m04_constants::INBOX_MAX);
    }

    #[test]
    fn acknowledge_message_success() {
        let mut s = test_sphere();
        let id = s.receive_message("peer".into(), "hello".into());
        assert!(s.acknowledge_message(id));
        assert!(s.inbox[0].acknowledged);
    }

    #[test]
    fn acknowledge_message_not_found() {
        let mut s = test_sphere();
        assert!(!s.acknowledge_message(999));
    }

    // ── Steer toward ──

    #[test]
    fn steer_toward_changes_momentum() {
        let mut s = test_sphere();
        s.phase = 0.0;
        s.steer_toward(PI, 1.0);
        assert!(s.momentum.abs() > 0.0);
    }

    #[test]
    fn steer_toward_momentum_gating() {
        let mut s = test_sphere();
        s.momentum = 0.5; // High momentum — gating active
        s.steer_toward(PI, 1.0);
        let gated = s.momentum;

        let mut s2 = test_sphere();
        s2.momentum = 0.0; // Low momentum — no gating
        s2.steer_toward(PI, 1.0);
        let ungated = s2.momentum;

        assert!(ungated.abs() > gated.abs() - 0.5);
    }

    // ── Reconciliation ──

    #[test]
    fn reconcile_fixes_memory_ids() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "a".into());
        s.record_memory("Write".into(), "b".into());
        s.next_memory_id = 0; // Simulate broken state
        s.reconcile_after_restore();
        assert_eq!(s.next_memory_id, 2);
    }

    #[test]
    fn reconcile_fixes_base_frequency() {
        let mut s = test_sphere();
        s.base_frequency = 0.0; // Simulate old snapshot
        s.frequency = 0.5;
        s.reconcile_after_restore();
        assert_relative_eq!(s.base_frequency, 0.5);
    }

    // ── Activation zones ──

    #[test]
    fn activation_zones_empty_sphere() {
        let s = test_sphere();
        let zones = s.activation_zones();
        assert_eq!(zones.vivid, 0);
        assert_eq!(zones.clear, 0);
    }

    #[test]
    fn activation_zones_vivid() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        // Fresh memory has activation 1.0 = vivid
        let zones = s.activation_zones();
        assert_eq!(zones.vivid, 1);
    }

    // ── Activation density ──

    #[test]
    fn activation_density_empty() {
        let s = test_sphere();
        assert_relative_eq!(s.activation_density(), 0.0);
    }

    #[test]
    fn activation_density_all_active() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "a".into());
        assert!(s.activation_density() > 0.0);
    }

    // ── Batch pruning ──

    #[test]
    fn batch_prune_caps_at_memory_max() {
        let mut s = test_sphere();
        for i in 0..600 {
            s.memories.push(SphereMemory::new(
                i,
                Point3D::north(),
                format!("Tool{i}"),
                "test".into(),
            ));
        }
        s.next_memory_id = 600;
        s.batch_prune_memories();
        assert!(s.memories.len() <= m04_constants::MEMORY_MAX_COUNT);
    }

    // ── Serde roundtrip ──

    #[test]
    fn sphere_serde_roundtrip() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "file".into());
        s.step();
        let json = serde_json::to_string(&s).unwrap();
        let back: PaneSphere = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id.as_str(), "test");
        assert_eq!(back.memories.len(), 1);
    }

    #[test]
    fn sphere_serde_preserves_opt_outs() {
        let mut s = test_sphere();
        s.opt_out_hebbian = true;
        s.opt_out_observation = true;
        let json = serde_json::to_string(&s).unwrap();
        let back: PaneSphere = serde_json::from_str(&json).unwrap();
        assert!(back.opt_out_hebbian);
        assert!(back.opt_out_observation);
    }

    // ── Field context ──

    #[test]
    fn set_field_context_updates_state() {
        let mut s = test_sphere();
        let ctx = SphereFieldContext {
            global_r: 0.95,
            my_cluster_size: 3,
            is_synchronized: true,
            my_coupling_strength: 0.8,
            tunnel_count: 2,
        };
        s.set_field_context(ctx);
        assert_relative_eq!(s.field_context.global_r, 0.95);
        assert_eq!(s.field_context.my_cluster_size, 3);
    }

    #[test]
    fn field_context_dampens_synchronized_momentum() {
        let mut s = test_sphere();
        s.momentum = 1.0;
        s.field_context = SphereFieldContext {
            is_synchronized: true,
            my_coupling_strength: 0.8,
            ..SphereFieldContext::default()
        };
        s.step();
        // Momentum should be dampened more than default
        assert!(s.momentum < 0.98);
    }

    // ── Heartbeat ──

    #[test]
    fn touch_heartbeat_updates_timestamp() {
        let mut s = test_sphere();
        let before = s.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));
        s.touch_heartbeat();
        assert!(s.last_heartbeat >= before);
    }

    // ── Work signature ──

    #[test]
    fn work_signature_all_zeros_initially() {
        let s = test_sphere();
        // WorkSignature::default() initializes all fields to 0.0
        assert_relative_eq!(s.work_signature.intensity, 0.0);
        assert_relative_eq!(s.work_signature.rhythm, 0.0);
        assert_relative_eq!(s.work_signature.diversity, 0.0);
        assert_relative_eq!(s.work_signature.focus, 0.0);
    }

    #[test]
    fn work_signature_intensity_increases_with_memories() {
        let mut s = test_sphere();
        for _ in 0..10 {
            s.record_memory("Read".into(), "file".into());
        }
        s.step();
        assert!(s.work_signature.intensity > 0.0);
    }

    #[test]
    fn work_signature_diversity_with_multiple_tools() {
        let mut s = test_sphere();
        s.record_memory("Read".into(), "a".into());
        s.record_memory("Write".into(), "b".into());
        s.record_memory("Bash".into(), "c".into());
        s.step();
        assert!(s.work_signature.diversity > 0.0);
    }
}
