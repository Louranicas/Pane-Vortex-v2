//! # M28: Consent Gate
//!
//! The consent pattern that must propagate to every external control mechanism.
//! `consent_gated_k_adjustment()` scales external influence by fleet receptivity.
//!
//! ## Layer: L6 (Bridges)
//! ## Module: M28
//! ## Dependencies: L1 (M01, M02, M04), L3 (M11 sphere, M15 `app_state`)
//!
//! ## Core Function
//! ```text
//! consent_gated_k_adjustment(raw_adj, spheres) -> gated_adj
//!   1. Compute mean receptivity across all spheres
//!   2. Exclude opted-out spheres (opt_out_external_modulation)
//!   3. Scale raw_adj by mean receptivity
//!   4. Clamp combined effect to K_MOD_BUDGET [0.85, 1.15]
//!   5. Apply newcomer dampening (80% reduction for first 50 steps)
//!   6. Exempt spheres with active divergence requests
//! ```
//!
//! ## Design Constraints
//! - C8: ALL external bridges MUST route through this function
//! - PG-5: Budget captures ALL bridges (present and future)
//! - PG-12: ME bridge included (added Session 034e)
//!
//! ## Open NA-P Gaps (addressed in V3.3)
//! - NA-P-1: Consent is observed not declared → V3.3.1 adds /sphere/{id}/consent
//! - NA-P-4: Thermal influence global → V3.3.2 adds per-sphere `k_eff` isolation
//! - NA-P-2: Budget fixed → V3.4.7 makes budget fleet-adaptive
//!
//! ## Philosophy
//! > "The consent gate gave spheres the right to say no."
//! > — [[Session 034e — NA Gap Analysis of Master Plan V2]]
//!
//! ## Related Documentation
//! - [Consent Spec](../../ai_specs/CONSENT_SPEC.md)
//! - [[Session 034d — NA Consent Gate Implementation]]
//! - [[The Habitat — Naming and Philosophy]]
