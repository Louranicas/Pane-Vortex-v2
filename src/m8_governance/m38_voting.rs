//! # M38: Voting Mechanism
//!
//! Democratic layer of The Habitat's governance. Any sphere can vote on proposals.
//!
//! ## Layer: L8 (Governance, feature: `governance`)
//! ## Module: M38
//! ## Dependencies: L1 (M01, M02), L3 (M15 `app_state`), L7 (M30 bus types)
//!
//! ## Endpoints
//! - `POST /sphere/{id}/vote/{proposal_id}` — approve/reject/abstain
//! - `GET /field/proposals/{id}/votes` — list votes for a proposal
//!
//! ## Voting Rules
//! - One vote per sphere per proposal (SQL UNIQUE constraint)
//! - Quorum: >50% of active spheres (excludes idle >300s, blocked, complete)
//! - Window: 5 ticks (25s). Late votes rejected with `VotingClosed` error.
//! - Approved: quorum met AND `votes_for > votes_against`
//! - Spheres vote as equals — no weighted voting by maturity (NA principle)
//!
//! ## Quorum Calculation
//! ```text
//! active = spheres.filter(|s| !long_idle && !blocked && !complete).count()
//! quorum_met = total_votes / active > 0.5
//! approved = quorum_met && votes_for > votes_against
//! ```
//!
//! ## Design Constraints: C4 (no unwrap), C8 (consent check), C12 (max 10 proposals)
//! ## NA: NA-P-15 — "the right to say yes — together"
//! ## Related: [L8 Spec](../../ai_specs/layers/L8_GOVERNANCE_SPEC.md), [[The Habitat — Naming and Philosophy]]
