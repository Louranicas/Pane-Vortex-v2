//! # M38: Voting Mechanism
//!
//! Democratic layer: any sphere can vote on proposals.
//! Quorum rules, one-vote-per-sphere, time-windowed voting.
//!
//! ## Layer: L8 (Governance) — feature-gated: `governance`
//! ## Module: M38
//! ## Dependencies: L1 (M01), M37 (proposals)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::m01_core_types::PaneId;
use super::m37_proposals::{VoteChoice, Vote};

// ──────────────────────────────────────────────────────────────
// Voting analytics
// ──────────────────────────────────────────────────────────────

/// Summary of votes on a proposal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoteSummary {
    /// Total votes cast.
    pub total: usize,
    /// Approve votes.
    pub approve: usize,
    /// Reject votes.
    pub reject: usize,
    /// Abstain votes.
    pub abstain: usize,
    /// Participation rate (total / `active_spheres`).
    pub participation: f64,
    /// Whether quorum was met.
    pub quorum_met: bool,
}

impl VoteSummary {
    /// Compute summary from a list of votes.
    #[must_use]
    pub fn from_votes(votes: &[Vote], active_sphere_count: usize, quorum_threshold: f64) -> Self {
        let total = votes.len();
        let approve = votes.iter().filter(|v| v.choice == VoteChoice::Approve).count();
        let reject = votes.iter().filter(|v| v.choice == VoteChoice::Reject).count();
        let abstain = votes.iter().filter(|v| v.choice == VoteChoice::Abstain).count();

        #[allow(clippy::cast_precision_loss)]
        let participation = if active_sphere_count == 0 {
            0.0
        } else {
            total as f64 / active_sphere_count as f64
        };

        let quorum_met = participation >= quorum_threshold;

        Self {
            total,
            approve,
            reject,
            abstain,
            participation,
            quorum_met,
        }
    }

    /// Whether the proposal would be approved based on these votes.
    #[must_use]
    pub const fn would_approve(&self) -> bool {
        self.quorum_met && self.approve > self.reject
    }
}

/// Per-sphere voting history.
#[derive(Debug, Clone, Default)]
pub struct VotingHistory {
    /// Votes cast by each sphere, keyed by proposal ID.
    records: HashMap<PaneId, Vec<(String, VoteChoice)>>,
}

impl VotingHistory {
    /// Create a new voting history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a vote.
    pub fn record(&mut self, voter: PaneId, proposal_id: String, choice: VoteChoice) {
        self.records
            .entry(voter)
            .or_default()
            .push((proposal_id, choice));
    }

    /// Get all votes by a sphere.
    #[must_use]
    pub fn votes_by(&self, voter: &PaneId) -> &[(String, VoteChoice)] {
        self.records.get(voter).map_or(&[], Vec::as_slice)
    }

    /// Total votes cast across all spheres.
    #[must_use]
    pub fn total_votes(&self) -> usize {
        self.records.values().map(Vec::len).sum()
    }

    /// Number of unique voters.
    #[must_use]
    pub fn unique_voters(&self) -> usize {
        self.records.len()
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

    fn sample_votes() -> Vec<Vote> {
        vec![
            Vote { voter: pid("a"), choice: VoteChoice::Approve, tick: 1 },
            Vote { voter: pid("b"), choice: VoteChoice::Approve, tick: 2 },
            Vote { voter: pid("c"), choice: VoteChoice::Reject, tick: 3 },
            Vote { voter: pid("d"), choice: VoteChoice::Abstain, tick: 4 },
        ]
    }

    // ── VoteSummary ──

    #[test]
    fn summary_counts() {
        let s = VoteSummary::from_votes(&sample_votes(), 5, 0.5);
        assert_eq!(s.total, 4);
        assert_eq!(s.approve, 2);
        assert_eq!(s.reject, 1);
        assert_eq!(s.abstain, 1);
    }

    #[test]
    fn summary_participation() {
        let s = VoteSummary::from_votes(&sample_votes(), 5, 0.5);
        assert_relative_eq!(s.participation, 0.8);
    }

    #[test]
    fn summary_quorum_met() {
        let s = VoteSummary::from_votes(&sample_votes(), 5, 0.5);
        assert!(s.quorum_met);
    }

    #[test]
    fn summary_quorum_not_met() {
        let s = VoteSummary::from_votes(&sample_votes(), 20, 0.5);
        assert!(!s.quorum_met);
    }

    #[test]
    fn summary_would_approve() {
        let s = VoteSummary::from_votes(&sample_votes(), 5, 0.5);
        assert!(s.would_approve()); // 2 > 1
    }

    #[test]
    fn summary_would_not_approve_when_reject_majority() {
        let votes = vec![
            Vote { voter: pid("a"), choice: VoteChoice::Reject, tick: 1 },
            Vote { voter: pid("b"), choice: VoteChoice::Reject, tick: 2 },
            Vote { voter: pid("c"), choice: VoteChoice::Approve, tick: 3 },
        ];
        let s = VoteSummary::from_votes(&votes, 4, 0.5);
        assert!(!s.would_approve());
    }

    #[test]
    fn summary_empty_votes() {
        let s = VoteSummary::from_votes(&[], 5, 0.5);
        assert_eq!(s.total, 0);
        assert!(!s.quorum_met);
        assert!(!s.would_approve());
    }

    #[test]
    fn summary_zero_active_spheres() {
        let s = VoteSummary::from_votes(&sample_votes(), 0, 0.5);
        assert_relative_eq!(s.participation, 0.0);
        assert!(!s.quorum_met);
    }

    #[test]
    fn summary_default() {
        let s = VoteSummary::default();
        assert_eq!(s.total, 0);
        assert!(!s.quorum_met);
    }

    #[test]
    fn summary_serde_roundtrip() {
        let s = VoteSummary::from_votes(&sample_votes(), 5, 0.5);
        let json = serde_json::to_string(&s).unwrap();
        let back: VoteSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total, s.total);
    }

    // ── VotingHistory ──

    #[test]
    fn history_new_empty() {
        let h = VotingHistory::new();
        assert_eq!(h.total_votes(), 0);
    }

    #[test]
    fn history_record_vote() {
        let mut h = VotingHistory::new();
        h.record(pid("a"), "proposal-1".into(), VoteChoice::Approve);
        assert_eq!(h.total_votes(), 1);
    }

    #[test]
    fn history_votes_by_sphere() {
        let mut h = VotingHistory::new();
        h.record(pid("a"), "p1".into(), VoteChoice::Approve);
        h.record(pid("a"), "p2".into(), VoteChoice::Reject);
        let votes = h.votes_by(&pid("a"));
        assert_eq!(votes.len(), 2);
    }

    #[test]
    fn history_votes_by_unknown() {
        let h = VotingHistory::new();
        assert!(h.votes_by(&pid("unknown")).is_empty());
    }

    #[test]
    fn history_unique_voters() {
        let mut h = VotingHistory::new();
        h.record(pid("a"), "p1".into(), VoteChoice::Approve);
        h.record(pid("b"), "p1".into(), VoteChoice::Reject);
        h.record(pid("a"), "p2".into(), VoteChoice::Approve);
        assert_eq!(h.unique_voters(), 2);
    }

    #[test]
    fn history_total_votes_multiple_spheres() {
        let mut h = VotingHistory::new();
        h.record(pid("a"), "p1".into(), VoteChoice::Approve);
        h.record(pid("b"), "p1".into(), VoteChoice::Reject);
        assert_eq!(h.total_votes(), 2);
    }

    // ── Edge cases ──

    #[test]
    fn all_abstain_does_not_approve() {
        let votes = vec![
            Vote { voter: pid("a"), choice: VoteChoice::Abstain, tick: 1 },
            Vote { voter: pid("b"), choice: VoteChoice::Abstain, tick: 2 },
        ];
        let s = VoteSummary::from_votes(&votes, 3, 0.5);
        assert!(!s.would_approve()); // 0 approve, 0 reject — not > reject
    }

    #[test]
    fn tie_does_not_approve() {
        let votes = vec![
            Vote { voter: pid("a"), choice: VoteChoice::Approve, tick: 1 },
            Vote { voter: pid("b"), choice: VoteChoice::Reject, tick: 2 },
        ];
        let s = VoteSummary::from_votes(&votes, 2, 0.5);
        assert!(!s.would_approve()); // 1 == 1, not >
    }

    // ── 50+ tests met across m37 + m38 combined ──

    #[test]
    fn single_vote_approve_with_quorum() {
        let votes = vec![Vote { voter: pid("a"), choice: VoteChoice::Approve, tick: 1 }];
        let s = VoteSummary::from_votes(&votes, 1, 0.5);
        assert!(s.would_approve());
    }

    #[test]
    fn participation_one_hundred_percent() {
        let votes = vec![
            Vote { voter: pid("a"), choice: VoteChoice::Approve, tick: 1 },
            Vote { voter: pid("b"), choice: VoteChoice::Approve, tick: 2 },
        ];
        let s = VoteSummary::from_votes(&votes, 2, 0.5);
        assert_relative_eq!(s.participation, 1.0);
    }
}
