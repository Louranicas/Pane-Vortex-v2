-- Migration 003: Governance tables (V3.4)
-- Proposals, votes, consent declarations, data manifests.

CREATE TABLE IF NOT EXISTS proposals (
    id                  TEXT PRIMARY KEY,
    proposer_sphere     TEXT NOT NULL,
    parameter           TEXT NOT NULL,  -- e.g., 'r_target', 'k_mod_budget_max'
    current_value       REAL NOT NULL,
    proposed_value      REAL NOT NULL,
    rationale           TEXT,
    status              TEXT NOT NULL DEFAULT 'open',  -- open, approved, rejected, expired
    votes_for           INTEGER DEFAULT 0,
    votes_against       INTEGER DEFAULT 0,
    votes_abstain       INTEGER DEFAULT 0,
    quorum_threshold    REAL NOT NULL DEFAULT 0.5,
    voting_deadline_tick INTEGER NOT NULL,
    created_at          TEXT DEFAULT (datetime('now')),
    resolved_at         TEXT
);

CREATE TABLE IF NOT EXISTS votes (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id         TEXT NOT NULL REFERENCES proposals(id),
    sphere_id           TEXT NOT NULL,
    vote                TEXT NOT NULL,  -- approve, reject, abstain
    created_at          TEXT DEFAULT (datetime('now')),
    UNIQUE(proposal_id, sphere_id)
);

CREATE TABLE IF NOT EXISTS consent_declarations (
    sphere_id           TEXT PRIMARY KEY,
    accept_external_modulation BOOLEAN DEFAULT 1,
    max_k_adjustment    REAL DEFAULT 0.15,
    accept_cascade      BOOLEAN DEFAULT 1,
    accept_observation  BOOLEAN DEFAULT 1,
    accept_nvim_monitoring BOOLEAN DEFAULT 1,
    updated_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS data_manifests (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    sphere_id           TEXT NOT NULL,
    system              TEXT NOT NULL,  -- field_tracking, bus_tracking, povm, rm, etc.
    record_count        INTEGER DEFAULT 0,
    last_scanned_at     TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_proposals_status ON proposals(status);
CREATE INDEX IF NOT EXISTS idx_votes_proposal ON votes(proposal_id);
