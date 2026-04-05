//! # M41: Evolution Chamber
//!
//! Pattern detection, anomaly scoring, and emergence events.
//! Feature-gated: `evolution`.
//!
//! ## Layer: L8 (Governance)
//! ## Module: M41
//! ## Dependencies: L1 (M01)

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::m01_core_types::PaneId;

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// An observed pattern in the field dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPattern {
    /// Pattern identifier.
    pub id: String,
    /// Pattern type.
    pub pattern_type: PatternType,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Tick at which the pattern was first observed.
    pub first_seen_tick: u64,
    /// Number of times this pattern has been observed.
    pub observation_count: u64,
    /// Description of the pattern.
    pub description: String,
}

/// Types of field patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Periodic oscillation in r.
    RhythmicR,
    /// Chimera formation/dissolution cycle.
    ChimeraCycle,
    /// Coordinated tool usage across spheres.
    ToolCorrelation,
    /// Phase locking between specific sphere pairs.
    PhaseLocking,
    /// Anomalous behavior (outlier detection).
    Anomaly,
}

/// An emergence event (significant field state transition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceEvent {
    /// Event type.
    pub event_type: EmergenceType,
    /// Tick at which the event occurred.
    pub tick: u64,
    /// Unix timestamp.
    pub timestamp: f64,
    /// Spheres involved.
    pub involved_spheres: Vec<PaneId>,
    /// Description.
    pub description: String,
}

/// Types of emergence events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergenceType {
    /// New chimera state detected.
    ChimeraDetected,
    /// Field transitioned from incoherent to coherent.
    CoherenceEmergence,
    /// Spontaneous synchronization without conductor intervention.
    SpontaneousSync,
    /// New tunnel formed between spheres.
    TunnelFormation,
    /// Phase regime change.
    RegimeChange,
}

/// A field observation submitted by a sphere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldObservation {
    /// Observing sphere.
    pub observer: PaneId,
    /// What was observed.
    pub observation: String,
    /// Tick of observation.
    pub tick: u64,
    /// Timestamp.
    pub timestamp: f64,
}

// ──────────────────────────────────────────────────────────────
// Evolution chamber
// ──────────────────────────────────────────────────────────────

/// The evolution chamber: observes, records, and detects field patterns.
///
/// Respects `opt_out_observation` — spheres can refuse to be observed.
#[derive(Debug, Clone, Default)]
pub struct EvolutionChamber {
    /// Detected patterns.
    pub patterns: Vec<FieldPattern>,
    /// Emergence events.
    pub events: VecDeque<EmergenceEvent>,
    /// Field observations from spheres.
    pub observations: VecDeque<FieldObservation>,
    /// Maximum events to retain.
    max_events: usize,
    /// Maximum observations to retain.
    max_observations: usize,
}

/// Maximum events and observations.
const DEFAULT_MAX_EVENTS: usize = 100;
const DEFAULT_MAX_OBSERVATIONS: usize = 500;

impl EvolutionChamber {
    /// Create a new evolution chamber.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            patterns: Vec::new(),
            events: VecDeque::new(),
            observations: VecDeque::new(),
            max_events: DEFAULT_MAX_EVENTS,
            max_observations: DEFAULT_MAX_OBSERVATIONS,
        }
    }

    /// Record an emergence event.
    pub fn record_event(&mut self, event: EmergenceEvent) {
        self.events.push_back(event);
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    /// Record a field observation from a sphere.
    pub fn record_observation(&mut self, observation: FieldObservation) {
        self.observations.push_back(observation);
        while self.observations.len() > self.max_observations {
            self.observations.pop_front();
        }
    }

    /// Add or update a pattern.
    ///
    /// Confidence is normalised to [0.0, 1.0] on every write to guard against
    /// NaN, infinity, and out-of-range values propagating into stored state.
    /// Non-finite values are replaced with 0.0 before clamping.
    ///
    /// The observation count is managed internally starting at 1 on first
    /// insert; the `observation_count` field on the incoming `pattern` is
    /// ignored for existing entries.
    pub fn observe_pattern(&mut self, pattern: FieldPattern) {
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.id == pattern.id) {
            existing.observation_count = existing.observation_count.saturating_add(1);
            // Sanitise incoming confidence: replace NaN/inf with 0.0, then clamp.
            let incoming = if pattern.confidence.is_finite() {
                pattern.confidence.clamp(0.0, 1.0)
            } else {
                0.0
            };
            // Running average, re-clamped for safety.
            existing.confidence = ((existing.confidence + incoming) / 2.0).clamp(0.0, 1.0);
        } else {
            let mut new_pattern = pattern;
            // Normalise on insert: non-finite → 0.0, then clamp; count starts at ≥ 1.
            new_pattern.confidence = if new_pattern.confidence.is_finite() {
                new_pattern.confidence.clamp(0.0, 1.0)
            } else {
                0.0
            };
            new_pattern.observation_count = new_pattern.observation_count.max(1);
            self.patterns.push(new_pattern);
        }
    }

    /// Get recent events.
    #[must_use]
    pub fn recent_events(&self, limit: usize) -> Vec<&EmergenceEvent> {
        self.events.iter().rev().take(limit).collect()
    }

    /// Get recent observations.
    #[must_use]
    pub fn recent_observations(&self, limit: usize) -> Vec<&FieldObservation> {
        self.observations.iter().rev().take(limit).collect()
    }

    /// Total pattern count.
    #[must_use]
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Total event count.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Summary for diagnostics.
    #[must_use]
    pub fn summary(&self) -> EvolutionSummary {
        EvolutionSummary {
            patterns: self.patterns.len(),
            events: self.events.len(),
            observations: self.observations.len(),
        }
    }
}

/// Evolution chamber summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvolutionSummary {
    /// Number of detected patterns.
    pub patterns: usize,
    /// Number of emergence events.
    pub events: usize,
    /// Number of field observations.
    pub observations: usize,
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_foundation::m01_core_types::now_secs;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    fn test_event() -> EmergenceEvent {
        EmergenceEvent {
            event_type: EmergenceType::ChimeraDetected,
            tick: 100,
            timestamp: now_secs(),
            involved_spheres: vec![pid("a"), pid("b")],
            description: "chimera detected".into(),
        }
    }

    fn test_observation() -> FieldObservation {
        FieldObservation {
            observer: pid("a"),
            observation: "r is oscillating".into(),
            tick: 50,
            timestamp: now_secs(),
        }
    }

    fn test_pattern() -> FieldPattern {
        FieldPattern {
            id: "rhythm-1".into(),
            pattern_type: PatternType::RhythmicR,
            confidence: 0.8,
            first_seen_tick: 10,
            observation_count: 1,
            description: "periodic r oscillation".into(),
        }
    }

    // ── Construction ──

    #[test]
    fn new_chamber_empty() {
        let c = EvolutionChamber::new();
        assert_eq!(c.pattern_count(), 0);
        assert_eq!(c.event_count(), 0);
    }

    #[test]
    fn default_matches_new() {
        let c = EvolutionChamber::default();
        assert_eq!(c.pattern_count(), 0);
    }

    // ── Events ──

    #[test]
    fn record_event_adds() {
        let mut c = EvolutionChamber::new();
        c.record_event(test_event());
        assert_eq!(c.event_count(), 1);
    }

    #[test]
    fn record_event_caps_at_max() {
        let mut c = EvolutionChamber::new();
        for _ in 0..150 {
            c.record_event(test_event());
        }
        assert!(c.event_count() <= DEFAULT_MAX_EVENTS);
    }

    #[test]
    fn recent_events_order() {
        let mut c = EvolutionChamber::new();
        for i in 0..5 {
            let mut e = test_event();
            e.tick = i;
            c.record_event(e);
        }
        let recent = c.recent_events(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].tick, 4); // Most recent first
    }

    // ── Observations ──

    #[test]
    fn record_observation_adds() {
        let mut c = EvolutionChamber::new();
        c.record_observation(test_observation());
        assert_eq!(c.observations.len(), 1);
    }

    #[test]
    fn record_observation_caps() {
        let mut c = EvolutionChamber::new();
        for _ in 0..600 {
            c.record_observation(test_observation());
        }
        assert!(c.observations.len() <= DEFAULT_MAX_OBSERVATIONS);
    }

    #[test]
    fn recent_observations_limit() {
        let mut c = EvolutionChamber::new();
        for _ in 0..10 {
            c.record_observation(test_observation());
        }
        let recent = c.recent_observations(3);
        assert_eq!(recent.len(), 3);
    }

    // ── Patterns ──

    #[test]
    fn observe_pattern_adds() {
        let mut c = EvolutionChamber::new();
        c.observe_pattern(test_pattern());
        assert_eq!(c.pattern_count(), 1);
    }

    #[test]
    fn observe_pattern_updates_existing() {
        let mut c = EvolutionChamber::new();
        c.observe_pattern(test_pattern());
        c.observe_pattern(test_pattern());
        assert_eq!(c.pattern_count(), 1);
        assert_eq!(c.patterns[0].observation_count, 2);
    }

    #[test]
    fn observe_pattern_different_ids() {
        let mut c = EvolutionChamber::new();
        let mut p1 = test_pattern();
        p1.id = "p1".into();
        let mut p2 = test_pattern();
        p2.id = "p2".into();
        c.observe_pattern(p1);
        c.observe_pattern(p2);
        assert_eq!(c.pattern_count(), 2);
    }

    #[test]
    fn observe_pattern_averages_confidence() {
        let mut c = EvolutionChamber::new();
        let mut p1 = test_pattern();
        p1.confidence = 0.6;
        c.observe_pattern(p1);
        let mut p2 = test_pattern();
        p2.confidence = 1.0;
        c.observe_pattern(p2);
        // Average of 0.6 and 1.0 = 0.8
        assert!((c.patterns[0].confidence - 0.8).abs() < 0.01);
    }

    // ── Summary ──

    #[test]
    fn summary_counts() {
        let mut c = EvolutionChamber::new();
        c.record_event(test_event());
        c.record_event(test_event());
        c.record_observation(test_observation());
        c.observe_pattern(test_pattern());
        let s = c.summary();
        assert_eq!(s.patterns, 1);
        assert_eq!(s.events, 2);
        assert_eq!(s.observations, 1);
    }

    #[test]
    fn summary_default() {
        let s = EvolutionSummary::default();
        assert_eq!(s.patterns, 0);
        assert_eq!(s.events, 0);
        assert_eq!(s.observations, 0);
    }

    #[test]
    fn summary_serde_roundtrip() {
        let s = EvolutionSummary {
            patterns: 5,
            events: 10,
            observations: 20,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: EvolutionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.patterns, 5);
    }

    // ── Serde ──

    #[test]
    fn emergence_event_serde_roundtrip() {
        let e = test_event();
        let json = serde_json::to_string(&e).unwrap();
        let back: EmergenceEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tick, 100);
        assert_eq!(back.event_type, EmergenceType::ChimeraDetected);
    }

    #[test]
    fn field_pattern_serde_roundtrip() {
        let p = test_pattern();
        let json = serde_json::to_string(&p).unwrap();
        let back: FieldPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "rhythm-1");
    }

    #[test]
    fn field_observation_serde_roundtrip() {
        let o = test_observation();
        let json = serde_json::to_string(&o).unwrap();
        let back: FieldObservation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.observer.as_str(), "a");
    }

    #[test]
    fn pattern_type_serde_roundtrip() {
        for pt in &[
            PatternType::RhythmicR,
            PatternType::ChimeraCycle,
            PatternType::ToolCorrelation,
            PatternType::PhaseLocking,
            PatternType::Anomaly,
        ] {
            let json = serde_json::to_string(pt).unwrap();
            let back: PatternType = serde_json::from_str(&json).unwrap();
            assert_eq!(*pt, back);
        }
    }

    #[test]
    fn emergence_type_serde_roundtrip() {
        for et in &[
            EmergenceType::ChimeraDetected,
            EmergenceType::CoherenceEmergence,
            EmergenceType::SpontaneousSync,
            EmergenceType::TunnelFormation,
            EmergenceType::RegimeChange,
        ] {
            let json = serde_json::to_string(et).unwrap();
            let back: EmergenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(*et, back);
        }
    }

    // ── observe_pattern: confidence clamping ──

    #[test]
    fn observe_pattern_confidence_clamped_on_insert() {
        let mut c = EvolutionChamber::new();
        let mut p = test_pattern();
        p.confidence = 1.5; // Out of range
        c.observe_pattern(p);
        assert!(c.patterns[0].confidence <= 1.0, "confidence must be clamped to [0.0, 1.0]");
        assert!(c.patterns[0].confidence >= 0.0);
    }

    #[test]
    fn observe_pattern_confidence_clamped_on_update() {
        let mut c = EvolutionChamber::new();
        c.observe_pattern(test_pattern()); // confidence 0.8
        let mut p2 = test_pattern();
        p2.confidence = -5.0; // Out of range
        c.observe_pattern(p2);
        assert!(c.patterns[0].confidence >= 0.0, "confidence must not go below 0.0");
        assert!(c.patterns[0].confidence <= 1.0);
    }

    #[test]
    fn observe_pattern_observation_count_starts_at_one() {
        let mut c = EvolutionChamber::new();
        let mut p = test_pattern();
        p.observation_count = 0; // Caller passes zero
        c.observe_pattern(p);
        // Chamber normalises to 1 on insert
        assert_eq!(c.patterns[0].observation_count, 1);
    }

    #[test]
    fn observe_pattern_confidence_nan_replaced_with_zero() {
        let mut c = EvolutionChamber::new();
        let mut p = test_pattern();
        p.confidence = f64::NAN;
        c.observe_pattern(p);
        // NaN is replaced with 0.0 before clamping.
        assert!(
            c.patterns[0].confidence.is_finite(),
            "NaN confidence must be replaced with a finite value"
        );
        assert_eq!(c.patterns[0].confidence, 0.0);
    }

    #[test]
    fn observe_pattern_confidence_inf_replaced_with_zero() {
        let mut c = EvolutionChamber::new();
        let mut p = test_pattern();
        p.confidence = f64::INFINITY;
        c.observe_pattern(p);
        // Non-finite values (including INFINITY) are replaced with 0.0.
        assert!(c.patterns[0].confidence.is_finite());
        assert_eq!(c.patterns[0].confidence, 0.0);
    }
}
