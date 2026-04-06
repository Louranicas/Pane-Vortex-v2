-- Migration 001: Field tracking tables
-- Applied on first startup. Idempotent (IF NOT EXISTS).

CREATE TABLE IF NOT EXISTS field_snapshots (
    tick                INTEGER PRIMARY KEY,
    r                   REAL NOT NULL,
    k                   REAL NOT NULL,
    k_mod               REAL NOT NULL,
    effective_k         REAL,
    sphere_count        INTEGER NOT NULL,
    idle_count          INTEGER DEFAULT 0,
    working_count       INTEGER DEFAULT 0,
    blocked_count       INTEGER DEFAULT 0,
    decision_action     TEXT,
    chimera_detected    INTEGER DEFAULT 0,
    chimera_cluster_count INTEGER DEFAULT 0,
    breathing_amplitude REAL DEFAULT 0.0,
    mean_phase          REAL,
    phase_spread        REAL,
    modulation_breakdown TEXT,  -- JSON: {synthex, nexus, me, conductor, consent_scale}
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sphere_history (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    sphere_id           TEXT NOT NULL,
    event_type          TEXT NOT NULL,  -- registered, deregistered, status_change
    tick                INTEGER NOT NULL,
    phase               REAL,
    frequency           REAL,
    status              TEXT,
    persona             TEXT,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS coupling_history (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tick                INTEGER NOT NULL,
    sphere_a            TEXT NOT NULL,
    sphere_b            TEXT NOT NULL,
    weight              REAL NOT NULL,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sphere_history_sphere ON sphere_history(sphere_id);
CREATE INDEX IF NOT EXISTS idx_sphere_history_tick ON sphere_history(tick);
CREATE INDEX IF NOT EXISTS idx_coupling_history_tick ON coupling_history(tick);
CREATE INDEX IF NOT EXISTS idx_coupling_history_pair ON coupling_history(sphere_a, sphere_b);
