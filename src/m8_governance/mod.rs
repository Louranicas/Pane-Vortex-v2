//! # Layer 8: Governance (Feature-gated: `governance`)
//!
//! Collective voting, proposals, consent declaration, data sovereignty.
//! NA-P-15: the field should have a voice.
//! Depends on L1, L3, L7.
//!
//! ## Design Constraints: C1 C8 C10
//! - C8: Consent gate applies to ALL governance actions
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m37_proposals` | ~300 | /field/propose, proposal lifecycle |
//! | `m38_voting` | ~250 | /sphere/{id}/vote/{proposal_id}, quorum |
//! | `m39_consent_declaration` | ~200 | /sphere/{id}/consent, explicit posture |
//! | `m40_data_sovereignty` | ~250 | /sphere/{id}/data-manifest, /forget |
//! | `m41_evolution_chamber` | ~400 | Patterns, anomalies, emergence (feature: evolution) |

pub mod m37_proposals;
pub mod m38_voting;
pub mod m39_consent_declaration;
pub mod m40_data_sovereignty;

#[cfg(feature = "evolution")]
pub mod m41_evolution_chamber;
