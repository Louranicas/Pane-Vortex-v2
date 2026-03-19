//! # M37: Proposal System
//!
//! The field finds its voice. Any sphere can propose changes to field parameters.
//! Proposals are voted on by active spheres with quorum rules.
//!
//! ## Layer: L8 (Governance) — feature-gated: `governance`
//! ## Module: M37
//! ## Dependencies: L1 (M01, M02, M04), L3 (M15 app_state), L7 (M30 bus_types)
//!
//! ## Endpoints
//! - `POST /field/propose` — submit proposal (any sphere)
//! - `GET /field/proposals` — list all proposals with outcomes
//! - `GET /field/proposals/{id}` — single proposal detail
//!
//! ## Proposal Lifecycle
//! ```text
//! Open → (voting_window ticks) → Approved (>50% support) → Applied
//!                                → Rejected (≤50% or quorum not met) → Archived
//!                                → Expired (no quorum within window) → Archived
//! ```
//!
//! ## Proposable Parameters (start with r_target only, expand later)
//! - r_target: desired order parameter [0.5, 0.99]
//! - k_mod_budget_max: maximum external influence [1.0, 1.5]
//! - More parameters added via governance evolution
//!
//! ## Design Constraints
//! - NA-P-15: "The field has no voice in its own governance" — this closes it
//! - Quorum: >50% of active (non-idle, non-blocked) spheres
//! - Voting window: 5 ticks (25 seconds) — long enough for bus propagation
//!
//! ## Philosophy
//! > "The 35 NA features gave spheres hands. The consent gate gave them the
//! > right to say no. What's missing is the right to say yes — together."
//!
//! ## Related Documentation
//! - [Governance Layer Spec](../../ai_specs/layers/L8_GOVERNANCE_SPEC.md)
//! - [migrations/003_governance_tables.sql](../../migrations/003_governance_tables.sql)
//! - [[Session 034e — NA Gap Analysis of Master Plan V2]] — NA-P-15
//! - [[The Habitat — Naming and Philosophy]]
