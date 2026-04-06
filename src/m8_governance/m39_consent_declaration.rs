//! # M39: Consent Declaration
//!
//! Explicit consent posture for spheres. Replaces observed-not-declared consent (NA-P-1).
//! Each sphere can declare what modulation it accepts and its limits.
//!
//! ## Layer: L8 (Governance) — feature-gated: `governance`
//! ## Module: M39
//! ## Dependencies: L1 (M01)

use serde::{Deserialize, Serialize};

use crate::m1_foundation::m01_core_types::PaneId;

// ──────────────────────────────────────────────────────────────
// Consent declaration
// ──────────────────────────────────────────────────────────────

/// Explicit consent declaration from a sphere.
///
/// Replaces the implicit opt-out flags with an active, declared posture.
/// The consent gate checks these declarations before applying modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConsentDeclaration {
    /// Sphere making the declaration.
    pub sphere_id: PaneId,
    /// Accept coupling modulation from bridges.
    pub accept_modulation: bool,
    /// Maximum k adjustment this sphere consents to.
    pub max_k_adj: f64,
    /// Accept cascade work dispatch.
    pub accept_cascade: bool,
    /// Accept observation by evolution chamber / analytics.
    pub accept_observation: bool,
    /// Accept nvim autocmd monitoring.
    pub accept_nvim_monitoring: bool,
    /// Accept RM logging of tool calls.
    pub accept_rm_logging: bool,
    /// Tick at which this declaration was made.
    pub declared_at_tick: u64,
    /// Tick at which all consent was formally revoked, if any.
    ///
    /// A `Some` value means the sphere has withdrawn consent. The tick
    /// provides an auditable timestamp of the revocation event.
    #[serde(default)]
    pub revoked_at_tick: Option<u64>,
}

impl ConsentDeclaration {
    /// Create a fully-open consent declaration (default state).
    #[must_use]
    pub const fn fully_open(sphere_id: PaneId, tick: u64) -> Self {
        Self {
            sphere_id,
            accept_modulation: true,
            max_k_adj: 0.15,
            accept_cascade: true,
            accept_observation: true,
            accept_nvim_monitoring: true,
            accept_rm_logging: true,
            declared_at_tick: tick,
            revoked_at_tick: None,
        }
    }

    /// Create a fully-closed consent declaration.
    #[must_use]
    pub const fn fully_closed(sphere_id: PaneId, tick: u64) -> Self {
        Self {
            sphere_id,
            accept_modulation: false,
            max_k_adj: 0.0,
            accept_cascade: false,
            accept_observation: false,
            accept_nvim_monitoring: false,
            accept_rm_logging: false,
            declared_at_tick: tick,
            revoked_at_tick: None,
        }
    }

    /// Formally revoke all consent at the given tick.
    ///
    /// Records the revocation tick for audit purposes. Once revoked,
    /// `is_revoked()` returns `true`. Calling `revoke` again when already
    /// revoked is a no-op (the original revocation tick is preserved).
    pub fn revoke(&mut self, tick: u64) {
        if self.revoked_at_tick.is_none() {
            self.revoked_at_tick = Some(tick);
        }
    }

    /// Whether this declaration has been formally revoked.
    ///
    /// Callers must check `is_revoked()` as the definitive gate before
    /// relying on individual `accept_*` fields.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked_at_tick.is_some()
    }

    /// Whether this sphere accepts a specific kind of modulation.
    ///
    /// Returns `false` if the declaration has been revoked. Unknown modulation
    /// type strings default to `self.accept_modulation`.
    #[must_use]
    pub fn accepts(&self, modulation_type: &str) -> bool {
        if self.is_revoked() {
            return false;
        }
        match modulation_type {
            "cascade" | "dispatch" => self.accept_cascade,
            "observation" | "analytics" | "evolution" => self.accept_observation,
            "nvim" | "nvim_monitoring" => self.accept_nvim_monitoring,
            "rm" | "rm_logging" | "reasoning_memory" => self.accept_rm_logging,
            // "modulation", "coupling", and any unknown type default to modulation consent
            _ => self.accept_modulation,
        }
    }

    /// How many modulation types are accepted.
    #[must_use]
    pub fn acceptance_count(&self) -> usize {
        let flags = [
            self.accept_modulation,
            self.accept_cascade,
            self.accept_observation,
            self.accept_nvim_monitoring,
            self.accept_rm_logging,
        ];
        flags.iter().filter(|&&f| f).count()
    }

    /// Whether the sphere has restricted any consent.
    #[must_use]
    pub fn has_restrictions(&self) -> bool {
        self.acceptance_count() < 5
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn pid(s: &str) -> PaneId {
        PaneId::new(s)
    }

    // ── Construction ──

    #[test]
    fn fully_open_all_true() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accept_modulation);
        assert!(c.accept_cascade);
        assert!(c.accept_observation);
        assert!(c.accept_nvim_monitoring);
        assert!(c.accept_rm_logging);
    }

    #[test]
    fn fully_closed_all_false() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert!(!c.accept_modulation);
        assert!(!c.accept_cascade);
        assert!(!c.accept_observation);
        assert!(!c.accept_nvim_monitoring);
        assert!(!c.accept_rm_logging);
    }

    #[test]
    fn fully_open_max_k_adj() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert_relative_eq!(c.max_k_adj, 0.15);
    }

    #[test]
    fn fully_closed_max_k_adj_zero() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert_relative_eq!(c.max_k_adj, 0.0);
    }

    // ── accepts ──

    #[test]
    fn accepts_modulation() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("modulation"));
        assert!(c.accepts("coupling"));
    }

    #[test]
    fn accepts_cascade() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("cascade"));
        assert!(c.accepts("dispatch"));
    }

    #[test]
    fn accepts_observation() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("observation"));
        assert!(c.accepts("analytics"));
        assert!(c.accepts("evolution"));
    }

    #[test]
    fn accepts_nvim() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("nvim"));
        assert!(c.accepts("nvim_monitoring"));
    }

    #[test]
    fn accepts_rm() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("rm"));
        assert!(c.accepts("rm_logging"));
        assert!(c.accepts("reasoning_memory"));
    }

    #[test]
    fn closed_rejects_all() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert!(!c.accepts("modulation"));
        assert!(!c.accepts("cascade"));
        assert!(!c.accepts("observation"));
        assert!(!c.accepts("nvim"));
        assert!(!c.accepts("rm"));
    }

    #[test]
    fn accepts_unknown_defaults_to_modulation() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(c.accepts("unknown_type"));
    }

    // ── acceptance_count ──

    #[test]
    fn acceptance_count_fully_open() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert_eq!(c.acceptance_count(), 5);
    }

    #[test]
    fn acceptance_count_fully_closed() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert_eq!(c.acceptance_count(), 0);
    }

    #[test]
    fn acceptance_count_partial() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.accept_nvim_monitoring = false;
        c.accept_rm_logging = false;
        assert_eq!(c.acceptance_count(), 3);
    }

    // ── has_restrictions ──

    #[test]
    fn has_restrictions_when_partially_closed() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.accept_observation = false;
        assert!(c.has_restrictions());
    }

    #[test]
    fn no_restrictions_when_fully_open() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(!c.has_restrictions());
    }

    #[test]
    fn has_restrictions_when_fully_closed() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert!(c.has_restrictions());
    }

    // ── Serde ──

    #[test]
    fn consent_serde_roundtrip() {
        let c = ConsentDeclaration::fully_open(pid("test"), 42);
        let json = serde_json::to_string(&c).unwrap();
        let back: ConsentDeclaration = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sphere_id.as_str(), "test");
        assert!(back.accept_modulation);
        assert_eq!(back.declared_at_tick, 42);
    }

    #[test]
    fn consent_partial_serde_roundtrip() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 1);
        c.accept_observation = false;
        c.max_k_adj = 0.05;
        let json = serde_json::to_string(&c).unwrap();
        let back: ConsentDeclaration = serde_json::from_str(&json).unwrap();
        assert!(!back.accept_observation);
        assert_relative_eq!(back.max_k_adj, 0.05);
    }

    // ── Tick tracking ──

    #[test]
    fn declared_at_tick_preserved() {
        let c = ConsentDeclaration::fully_open(pid("a"), 999);
        assert_eq!(c.declared_at_tick, 999);
    }

    // -- Sphere ID --

    #[test]
    fn sphere_id_preserved() {
        let c = ConsentDeclaration::fully_open(pid("my-sphere"), 10);
        assert_eq!(c.sphere_id.as_str(), "my-sphere");
    }

    // -- FINDING-12: Revocation mechanism --

    #[test]
    fn not_revoked_by_default() {
        let c = ConsentDeclaration::fully_open(pid("a"), 10);
        assert!(!c.is_revoked());
        assert!(c.revoked_at_tick.is_none());
    }

    #[test]
    fn revoke_sets_flag() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.revoke(50);
        assert!(c.is_revoked());
        assert_eq!(c.revoked_at_tick, Some(50));
    }

    #[test]
    fn revoke_idempotent_preserves_first_tick() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.revoke(50);
        c.revoke(99);
        assert_eq!(c.revoked_at_tick, Some(50), "revocation tick must not be overwritten");
    }

    #[test]
    fn accepts_returns_false_when_revoked() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.revoke(50);
        assert!(!c.accepts("modulation"));
        assert!(!c.accepts("cascade"));
        assert!(!c.accepts("observation"));
        assert!(!c.accepts("nvim"));
        assert!(!c.accepts("rm"));
        assert!(!c.accepts("unknown_type"));
    }

    #[test]
    fn closed_declaration_not_revoked_initially() {
        let c = ConsentDeclaration::fully_closed(pid("a"), 10);
        assert!(!c.is_revoked());
    }

    #[test]
    fn revoke_serde_roundtrip() {
        let mut c = ConsentDeclaration::fully_open(pid("a"), 10);
        c.revoke(77);
        let json = serde_json::to_string(&c).unwrap();
        let back: ConsentDeclaration = serde_json::from_str(&json).unwrap();
        assert!(back.is_revoked());
        assert_eq!(back.revoked_at_tick, Some(77));
    }

    #[test]
    fn revoked_at_tick_none_serde_roundtrip() {
        // Older snapshots without revoked_at_tick must deserialize via #[serde(default)]
        let json = r#"{"sphere_id":"test","accept_modulation":true,"max_k_adj":0.15,"accept_cascade":true,"accept_observation":true,"accept_nvim_monitoring":true,"accept_rm_logging":true,"declared_at_tick":10}"#;
        let c: ConsentDeclaration = serde_json::from_str(json).unwrap();
        assert!(!c.is_revoked(), "missing revoked_at_tick must default to None");
    }
}
