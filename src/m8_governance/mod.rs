//! # Layer 8: Governance (Feature-gated: `governance`)
//!
//! Collective voting, proposals, consent declaration, data sovereignty.
//! NA-P-15: the field should have a voice.
//! Depends on L1, L3, L7.

pub mod m37_proposals;
pub mod m38_voting;
pub mod m39_consent_declaration;
pub mod m40_data_sovereignty;

#[cfg(feature = "evolution")]
pub mod m41_evolution_chamber;

// ── Ergonomic re-exports ──

pub use m37_proposals::{
    ProposableParameter, Proposal, ProposalManager, ProposalStatus, Vote, VoteChoice,
};
pub use m38_voting::{VoteSummary, VotingHistory};
pub use m39_consent_declaration::ConsentDeclaration;
pub use m40_data_sovereignty::{DataManifest, ForgetRequest, ForgetStatus};

// Evolution chamber (feature-gated)
#[cfg(feature = "evolution")]
pub use m41_evolution_chamber::{
    EmergenceEvent, EmergenceType, EvolutionChamber, EvolutionSummary, FieldObservation,
    FieldPattern, PatternType,
};
