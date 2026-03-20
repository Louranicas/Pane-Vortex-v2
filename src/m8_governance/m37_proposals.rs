//! # M37: Proposal System
//!
//! The field finds its voice. Any sphere can propose changes to field parameters.
//! Proposals are voted on by active spheres with quorum rules.
//!
//! ## Layer: L8 (Governance) — feature-gated: `governance`
//! ## Module: M37
//! ## Dependencies: L1 (M01, M02, M04)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::{
    m01_core_types::{now_secs, PaneId, TaskId},
    m02_error_handling::{PvError, PvResult},
    m04_constants,
};

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// A governance proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID.
    pub id: String,
    /// Sphere that submitted the proposal.
    pub proposer: PaneId,
    /// What parameter to change.
    pub parameter: ProposableParameter,
    /// Proposed new value.
    pub proposed_value: f64,
    /// Current value at time of proposal.
    pub current_value: f64,
    /// Human-readable reason.
    pub reason: String,
    /// Tick at which the proposal was submitted.
    pub submitted_at_tick: u64,
    /// Unix timestamp.
    pub submitted_at: f64,
    /// Current lifecycle status.
    pub status: ProposalStatus,
    /// Votes received.
    pub votes: Vec<Vote>,
}

/// Parameters that can be proposed for change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposableParameter {
    /// Target order parameter.
    RTarget,
    /// Maximum external influence budget.
    KModBudgetMax,
    /// Coupling steps per tick.
    CouplingSteps,
    /// Per-sphere `k_mod` override (GAP-5). Value is the override, target sphere in `reason`.
    SphereOverride {
        /// Target sphere ID for the override.
        target_sphere: String,
    },
    /// Fleet-wide opt-out policy (GAP-6). Value: 0.0=allow opt-out, 1.0=require participation.
    OptOutPolicy,
}

/// Proposal lifecycle status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Voting is open.
    #[default]
    Open,
    /// Quorum met, `votes_for` > `votes_against`.
    Approved,
    /// Quorum met, `votes_against` >= `votes_for`.
    Rejected,
    /// Voting window expired without quorum.
    Expired,
    /// Approved proposal has been applied.
    Applied,
}

/// A single vote on a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Voting sphere.
    pub voter: PaneId,
    /// Vote choice.
    pub choice: VoteChoice,
    /// Tick at which the vote was cast.
    pub tick: u64,
}

/// Vote choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    /// Support the proposal.
    Approve,
    /// Oppose the proposal.
    Reject,
    /// Participate but don't choose a side.
    Abstain,
}

// ──────────────────────────────────────────────────────────────
// Proposal manager
// ──────────────────────────────────────────────────────────────

/// Manages the lifecycle of governance proposals.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProposalManager {
    /// Active and archived proposals.
    proposals: HashMap<String, Proposal>,
    /// Maximum active proposals.
    max_active: usize,
    /// Voting window in ticks.
    voting_window: u64,
    /// Quorum threshold (fraction of active spheres).
    quorum_threshold: f64,
}

impl ProposalManager {
    /// Create a new proposal manager with default config.
    ///
    /// Voting window is 24 ticks (120s at 5s/tick) to accommodate
    /// Nexus bridge 60-tick poll interval (GAP-7 fix).
    #[must_use]
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            max_active: m04_constants::DECISION_HISTORY_MAX, // 100
            voting_window: 24,
            quorum_threshold: 0.5,
        }
    }

    /// Create with custom config.
    #[must_use]
    pub fn with_config(max_active: usize, voting_window: u64, quorum_threshold: f64) -> Self {
        Self {
            proposals: HashMap::new(),
            max_active,
            voting_window,
            quorum_threshold,
        }
    }

    /// Submit a new proposal.
    ///
    /// # Errors
    /// Returns error if max active proposals reached or parameter value out of range.
    pub fn submit(
        &mut self,
        proposer: PaneId,
        parameter: ProposableParameter,
        proposed_value: f64,
        current_value: f64,
        reason: String,
        current_tick: u64,
    ) -> PvResult<String> {
        let active_count = self
            .proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Open)
            .count();

        if active_count >= self.max_active {
            return Err(PvError::Internal("max active proposals reached".into()));
        }

        // Validate proposed value
        validate_proposed_value(&parameter, proposed_value)?;

        let id = TaskId::new().to_string();
        let proposal = Proposal {
            id: id.clone(),
            proposer,
            parameter,
            proposed_value,
            current_value,
            reason,
            submitted_at_tick: current_tick,
            submitted_at: now_secs(),
            status: ProposalStatus::Open,
            votes: Vec::new(),
        };

        self.proposals.insert(id.clone(), proposal);
        Ok(id)
    }

    /// Cast a vote on a proposal.
    ///
    /// # Errors
    /// Returns error if proposal not found, voting closed, or duplicate vote.
    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter: PaneId,
        choice: VoteChoice,
        current_tick: u64,
    ) -> PvResult<()> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| PvError::ProposalNotFound(proposal_id.to_owned()))?;

        if proposal.status != ProposalStatus::Open {
            return Err(PvError::VotingClosed(proposal_id.to_owned()));
        }

        if current_tick > proposal.submitted_at_tick + self.voting_window {
            return Err(PvError::VotingClosed(proposal_id.to_owned()));
        }

        // Check for duplicate vote
        if proposal.votes.iter().any(|v| v.voter == voter) {
            return Err(PvError::Internal("duplicate vote".into()));
        }

        proposal.votes.push(Vote {
            voter,
            choice,
            tick: current_tick,
        });

        Ok(())
    }

    /// Process proposals: close expired, resolve voted.
    pub fn process(&mut self, current_tick: u64, active_sphere_count: usize) {
        let proposal_ids: Vec<String> = self.proposals.keys().cloned().collect();

        for id in proposal_ids {
            if let Some(proposal) = self.proposals.get_mut(&id) {
                if proposal.status != ProposalStatus::Open {
                    continue;
                }

                let window_expired =
                    current_tick > proposal.submitted_at_tick + self.voting_window;

                if window_expired {
                    let result =
                        evaluate_proposal(proposal, active_sphere_count, self.quorum_threshold);
                    proposal.status = result;
                }
            }
        }
    }

    /// Get a proposal by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Proposal> {
        self.proposals.get(id)
    }

    /// List all proposals.
    #[must_use]
    pub fn all(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    /// List open proposals.
    #[must_use]
    pub fn open_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Open)
            .collect()
    }

    /// List approved proposals that haven't been applied yet.
    #[must_use]
    pub fn approved_unapplied(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Approved)
            .collect()
    }

    /// Mark a proposal as applied.
    pub fn mark_applied(&mut self, id: &str) {
        if let Some(p) = self.proposals.get_mut(id) {
            if p.status == ProposalStatus::Approved {
                p.status = ProposalStatus::Applied;
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

/// Validate a proposed parameter value.
fn validate_proposed_value(parameter: &ProposableParameter, value: f64) -> PvResult<()> {
    if !value.is_finite() {
        return Err(PvError::NonFinite {
            field: "proposed_value",
            value,
        });
    }

    match parameter {
        ProposableParameter::RTarget => {
            if !(0.5..=0.99).contains(&value) {
                return Err(PvError::OutOfRange {
                    field: "r_target",
                    value,
                    min: 0.5,
                    max: 0.99,
                });
            }
        }
        ProposableParameter::KModBudgetMax => {
            if !(1.0..=1.5).contains(&value) {
                return Err(PvError::OutOfRange {
                    field: "k_mod_budget_max",
                    value,
                    min: 1.0,
                    max: 1.5,
                });
            }
        }
        ProposableParameter::CouplingSteps => {
            if !(1.0..=50.0).contains(&value) {
                return Err(PvError::OutOfRange {
                    field: "coupling_steps",
                    value,
                    min: 1.0,
                    max: 50.0,
                });
            }
        }
        ProposableParameter::SphereOverride { .. } => {
            // Per-sphere k_mod override must be within budget range
            if !(0.5..=1.5).contains(&value) {
                return Err(PvError::OutOfRange {
                    field: "sphere_override",
                    value,
                    min: 0.5,
                    max: 1.5,
                });
            }
        }
        ProposableParameter::OptOutPolicy => {
            // 0.0 = opt-out allowed, 1.0 = participation required
            if !(0.0..=1.0).contains(&value) {
                return Err(PvError::OutOfRange {
                    field: "opt_out_policy",
                    value,
                    min: 0.0,
                    max: 1.0,
                });
            }
        }
    }
    Ok(())
}

/// Evaluate a proposal after voting window closes.
fn evaluate_proposal(
    proposal: &Proposal,
    active_sphere_count: usize,
    quorum_threshold: f64,
) -> ProposalStatus {
    if active_sphere_count == 0 {
        return ProposalStatus::Expired;
    }

    let total_votes = proposal.votes.len();
    #[allow(clippy::cast_precision_loss)]
    let participation = total_votes as f64 / active_sphere_count as f64;

    if participation < quorum_threshold {
        return ProposalStatus::Expired;
    }

    let votes_for = proposal
        .votes
        .iter()
        .filter(|v| v.choice == VoteChoice::Approve)
        .count();
    let votes_against = proposal
        .votes
        .iter()
        .filter(|v| v.choice == VoteChoice::Reject)
        .count();

    if votes_for > votes_against {
        ProposalStatus::Approved
    } else {
        ProposalStatus::Rejected
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

    fn test_manager() -> ProposalManager {
        ProposalManager::with_config(10, 5, 0.5)
    }

    // ── Submit ──

    #[test]
    fn submit_creates_proposal() {
        let mut mgr = test_manager();
        let id = mgr
            .submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "lower target".into(), 10)
            .unwrap();
        assert!(mgr.get(&id).is_some());
    }

    #[test]
    fn submit_sets_open_status() {
        let mut mgr = test_manager();
        let id = mgr
            .submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10)
            .unwrap();
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Open);
    }

    #[test]
    fn submit_rejects_nan() {
        let mut mgr = test_manager();
        let result = mgr.submit(pid("a"), ProposableParameter::RTarget, f64::NAN, 0.93, "test".into(), 10);
        assert!(result.is_err());
    }

    #[test]
    fn submit_rejects_out_of_range_r_target() {
        let mut mgr = test_manager();
        let result = mgr.submit(pid("a"), ProposableParameter::RTarget, 1.5, 0.93, "test".into(), 10);
        assert!(result.is_err());
    }

    #[test]
    fn submit_rejects_out_of_range_k_budget() {
        let mut mgr = test_manager();
        let result = mgr.submit(pid("a"), ProposableParameter::KModBudgetMax, 2.0, 1.15, "test".into(), 10);
        assert!(result.is_err());
    }

    #[test]
    fn submit_max_active_reached() {
        let mut mgr = ProposalManager::with_config(2, 5, 0.5);
        mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "1".into(), 10).unwrap();
        mgr.submit(pid("b"), ProposableParameter::RTarget, 0.80, 0.93, "2".into(), 10).unwrap();
        let result = mgr.submit(pid("c"), ProposableParameter::RTarget, 0.75, 0.93, "3".into(), 10);
        assert!(result.is_err());
    }

    // ── Vote ──

    #[test]
    fn vote_records_choice() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        assert_eq!(mgr.get(&id).unwrap().votes.len(), 1);
    }

    #[test]
    fn vote_reject_valid() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Reject, 11).unwrap();
        assert_eq!(mgr.get(&id).unwrap().votes[0].choice, VoteChoice::Reject);
    }

    #[test]
    fn vote_duplicate_fails() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        let result = mgr.vote(&id, pid("b"), VoteChoice::Reject, 12);
        assert!(result.is_err());
    }

    #[test]
    fn vote_after_window_fails() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        let result = mgr.vote(&id, pid("b"), VoteChoice::Approve, 20); // 10 ticks later > 5 window
        assert!(result.is_err());
    }

    #[test]
    fn vote_nonexistent_proposal_fails() {
        let mut mgr = test_manager();
        let result = mgr.vote("nonexistent", pid("a"), VoteChoice::Approve, 10);
        assert!(result.is_err());
    }

    // ── Process ──

    #[test]
    fn process_approves_with_quorum_and_majority() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        mgr.vote(&id, pid("c"), VoteChoice::Approve, 12).unwrap();
        mgr.process(20, 3); // 3 active, 2 voted > 50%
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Approved);
    }

    #[test]
    fn process_rejects_with_quorum_and_majority_against() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Reject, 11).unwrap();
        mgr.vote(&id, pid("c"), VoteChoice::Reject, 12).unwrap();
        mgr.process(20, 3);
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Rejected);
    }

    #[test]
    fn process_expires_without_quorum() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        mgr.process(20, 10); // 10 active, only 1 voted < 50%
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Expired);
    }

    #[test]
    fn process_does_not_change_already_resolved() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        mgr.vote(&id, pid("c"), VoteChoice::Approve, 12).unwrap();
        mgr.process(20, 3);
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Approved);
        mgr.process(30, 3); // Process again
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Approved);
    }

    // ── Mark applied ──

    #[test]
    fn mark_applied_changes_status() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        mgr.vote(&id, pid("c"), VoteChoice::Approve, 12).unwrap();
        mgr.process(20, 3);
        mgr.mark_applied(&id);
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Applied);
    }

    #[test]
    fn mark_applied_noop_on_open() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.mark_applied(&id);
        assert_eq!(mgr.get(&id).unwrap().status, ProposalStatus::Open);
    }

    // ── Queries ──

    #[test]
    fn open_proposals_lists_open() {
        let mut mgr = test_manager();
        mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "1".into(), 10).unwrap();
        mgr.submit(pid("b"), ProposableParameter::RTarget, 0.80, 0.93, "2".into(), 10).unwrap();
        assert_eq!(mgr.open_proposals().len(), 2);
    }

    #[test]
    fn all_proposals() {
        let mut mgr = test_manager();
        mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "1".into(), 10).unwrap();
        assert_eq!(mgr.all().len(), 1);
    }

    #[test]
    fn approved_unapplied() {
        let mut mgr = test_manager();
        let id = mgr.submit(pid("a"), ProposableParameter::RTarget, 0.85, 0.93, "test".into(), 10).unwrap();
        mgr.vote(&id, pid("b"), VoteChoice::Approve, 11).unwrap();
        mgr.vote(&id, pid("c"), VoteChoice::Approve, 12).unwrap();
        mgr.process(20, 3);
        assert_eq!(mgr.approved_unapplied().len(), 1);
        mgr.mark_applied(&id);
        assert_eq!(mgr.approved_unapplied().len(), 0);
    }

    // ── Validation ──

    #[test]
    fn validate_r_target_valid() {
        assert!(validate_proposed_value(&ProposableParameter::RTarget, 0.85).is_ok());
    }

    #[test]
    fn validate_r_target_too_low() {
        assert!(validate_proposed_value(&ProposableParameter::RTarget, 0.1).is_err());
    }

    #[test]
    fn validate_r_target_too_high() {
        assert!(validate_proposed_value(&ProposableParameter::RTarget, 1.5).is_err());
    }

    #[test]
    fn validate_k_budget_valid() {
        assert!(validate_proposed_value(&ProposableParameter::KModBudgetMax, 1.2).is_ok());
    }

    #[test]
    fn validate_coupling_steps_valid() {
        assert!(validate_proposed_value(&ProposableParameter::CouplingSteps, 15.0).is_ok());
    }

    // ── Evaluate ──

    #[test]
    fn evaluate_no_active_spheres_expires() {
        let p = Proposal {
            id: "test".into(),
            proposer: pid("a"),
            parameter: ProposableParameter::RTarget,
            proposed_value: 0.85,
            current_value: 0.93,
            reason: "test".into(),
            submitted_at_tick: 10,
            submitted_at: 0.0,
            status: ProposalStatus::Open,
            votes: vec![],
        };
        assert_eq!(evaluate_proposal(&p, 0, 0.5), ProposalStatus::Expired);
    }

    #[test]
    fn evaluate_abstain_counted_for_quorum() {
        let p = Proposal {
            id: "test".into(),
            proposer: pid("a"),
            parameter: ProposableParameter::RTarget,
            proposed_value: 0.85,
            current_value: 0.93,
            reason: "test".into(),
            submitted_at_tick: 10,
            submitted_at: 0.0,
            status: ProposalStatus::Open,
            votes: vec![
                Vote { voter: pid("a"), choice: VoteChoice::Approve, tick: 11 },
                Vote { voter: pid("b"), choice: VoteChoice::Abstain, tick: 12 },
            ],
        };
        // 2/3 = 66% > 50% quorum, 1 approve > 0 reject
        assert_eq!(evaluate_proposal(&p, 3, 0.5), ProposalStatus::Approved);
    }

    // ── Serde ──

    #[test]
    fn proposal_serde_roundtrip() {
        let p = Proposal {
            id: "test".into(),
            proposer: pid("a"),
            parameter: ProposableParameter::RTarget,
            proposed_value: 0.85,
            current_value: 0.93,
            reason: "test".into(),
            submitted_at_tick: 10,
            submitted_at: 0.0,
            status: ProposalStatus::Open,
            votes: vec![],
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Proposal = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "test");
        assert_relative_eq!(back.proposed_value, 0.85);
    }

    #[test]
    fn vote_choice_serde_roundtrip() {
        let json = serde_json::to_string(&VoteChoice::Approve).unwrap();
        let back: VoteChoice = serde_json::from_str(&json).unwrap();
        assert_eq!(back, VoteChoice::Approve);
    }

    #[test]
    fn proposal_status_default() {
        assert_eq!(ProposalStatus::default(), ProposalStatus::Open);
    }
}
