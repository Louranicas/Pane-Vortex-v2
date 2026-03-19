# Database Specification

> SQLite persistence layer for field tracking, bus tasks, and governance.
> 3 migration files, 15 tables, WAL mode, cross-DB query patterns.
> Module: m36_persistence.rs | Plan: `MASTERPLAN.md` V3.1
> Obsidian: `[[Session 034f — Database Architecture Schematics]]`

## Overview

Pane-vortex v2 uses two SQLite databases for persistence:
- `data/field_tracking.db` — field snapshots, sphere history, coupling history
- `data/bus_tracking.db` — tasks, events, subscriptions, cascades, governance

Both databases use WAL mode for concurrent reader/writer access. Migrations are
idempotent (CREATE TABLE IF NOT EXISTS). Applied on first startup.

## 1. Migration 001: Field Tables

Source: `migrations/001_field_tables.sql`

### 1.1 field_snapshots

Periodic snapshots of field state, one per SNAPSHOT_INTERVAL (60) ticks.

```sql
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
```

**Write frequency:** Every 60 ticks (5 minutes at 5s/tick).
**Retention:** No auto-delete. Manual archival recommended after 10K rows.
**Key queries:**
```sql
-- Latest snapshot
SELECT * FROM field_snapshots ORDER BY tick DESC LIMIT 1;

-- r trend over last hour
SELECT tick, r FROM field_snapshots WHERE tick > (SELECT MAX(tick) - 720 FROM field_snapshots);

-- Chimera events
SELECT tick, chimera_cluster_count FROM field_snapshots WHERE chimera_detected = 1;

-- Decision distribution
SELECT decision_action, COUNT(*) FROM field_snapshots GROUP BY decision_action;
```

### 1.2 sphere_history

Lifecycle events for spheres (registration, deregistration, status changes).

```sql
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

CREATE INDEX IF NOT EXISTS idx_sphere_history_sphere ON sphere_history(sphere_id);
CREATE INDEX IF NOT EXISTS idx_sphere_history_tick ON sphere_history(tick);
```

**Write frequency:** On sphere lifecycle events (registration, status changes).
**Key queries:**
```sql
-- Sphere lifecycle
SELECT event_type, tick, status FROM sphere_history WHERE sphere_id = ? ORDER BY tick;

-- Active spheres at a given tick
SELECT DISTINCT sphere_id FROM sphere_history
WHERE event_type = 'registered' AND tick <= ?
AND sphere_id NOT IN (SELECT sphere_id FROM sphere_history WHERE event_type = 'deregistered' AND tick <= ?);

-- Status transitions
SELECT sphere_id, status, COUNT(*) FROM sphere_history WHERE event_type = 'status_change' GROUP BY sphere_id, status;
```

### 1.3 coupling_history

Periodic snapshots of coupling weights between spheres.

```sql
CREATE TABLE IF NOT EXISTS coupling_history (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tick                INTEGER NOT NULL,
    sphere_a            TEXT NOT NULL,
    sphere_b            TEXT NOT NULL,
    weight              REAL NOT NULL,
    created_at          TEXT DEFAULT (datetime('now'))
);
```

**Write frequency:** Every POVM_WEIGHTS_INTERVAL (60 ticks = 5 min), alongside POVM bridge writes.
**Key queries:**
```sql
-- Weight evolution between two spheres
SELECT tick, weight FROM coupling_history WHERE sphere_a = ? AND sphere_b = ? ORDER BY tick;

-- Strongest connections at latest tick
SELECT sphere_a, sphere_b, weight FROM coupling_history
WHERE tick = (SELECT MAX(tick) FROM coupling_history) ORDER BY weight DESC LIMIT 10;
```

## 2. Migration 002: Bus Tables

Source: `migrations/002_bus_tables.sql`

### 2.1 bus_tasks

Task queue with full lifecycle tracking.

```sql
CREATE TABLE IF NOT EXISTS bus_tasks (
    id                  TEXT PRIMARY KEY,
    status              TEXT NOT NULL DEFAULT 'submitted',
    source_sphere       TEXT NOT NULL,
    target              TEXT,
    target_type         TEXT NOT NULL,  -- specific, any_idle, field_driven, willing
    description         TEXT NOT NULL,
    payload             TEXT,           -- JSON payload
    claimed_by          TEXT,
    submitted_at        TEXT DEFAULT (datetime('now')),
    claimed_at          TEXT,
    completed_at        TEXT,
    ttl_secs            INTEGER DEFAULT 3600
);

CREATE INDEX IF NOT EXISTS idx_bus_tasks_status ON bus_tasks(status);
```

**Task status FSM:**
```
submitted --> claimed --> completed
    |            |
    |            +--> failed
    |
    +--> expired (checked on read, updated lazily)
```

**Key queries:**
```sql
-- Pending tasks
SELECT * FROM bus_tasks WHERE status = 'submitted' ORDER BY submitted_at;

-- Tasks by sphere
SELECT * FROM bus_tasks WHERE source_sphere = ? OR claimed_by = ?;

-- Completion rate
SELECT COUNT(*) as total,
       SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed,
       SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
FROM bus_tasks;
```

### 2.2 bus_events

Event log for audit and replay.

```sql
CREATE TABLE IF NOT EXISTS bus_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type          TEXT NOT NULL,
    source_sphere       TEXT,
    data                TEXT,           -- JSON event data
    tick                INTEGER,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bus_events_type ON bus_events(event_type);
CREATE INDEX IF NOT EXISTS idx_bus_events_tick ON bus_events(tick);
```

**Write frequency:** Batched — every 10 events or 30 seconds.
**Retention:** 10K rows max; oldest pruned on batch write.

### 2.3 event_subscriptions

Persistent subscription records (survives restarts).

```sql
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    sphere_id           TEXT NOT NULL,
    pattern             TEXT NOT NULL,  -- glob pattern (e.g., 'field.*')
    created_at          TEXT DEFAULT (datetime('now'))
);
```

### 2.4 cascade_events

Cascade handoff tracking (audit trail).

```sql
CREATE TABLE IF NOT EXISTS cascade_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    source_sphere       TEXT NOT NULL,
    target_sphere       TEXT,
    brief               TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'dispatched',
    depth               INTEGER DEFAULT 0,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_cascade_status ON cascade_events(status);
```

### 2.5 task_tags

Tag associations for tasks.

```sql
CREATE TABLE IF NOT EXISTS task_tags (
    task_id             TEXT NOT NULL REFERENCES bus_tasks(id),
    tag                 TEXT NOT NULL,
    PRIMARY KEY (task_id, tag)
);
```

### 2.6 task_dependencies

DAG of task dependencies.

```sql
CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id             TEXT NOT NULL REFERENCES bus_tasks(id),
    depends_on          TEXT NOT NULL REFERENCES bus_tasks(id),
    PRIMARY KEY (task_id, depends_on)
);
```

## 3. Migration 003: Governance Tables

Source: `migrations/003_governance_tables.sql`

### 3.1 proposals

Parameter change proposals with voting.

```sql
CREATE TABLE IF NOT EXISTS proposals (
    id                  TEXT PRIMARY KEY,
    proposer_sphere     TEXT NOT NULL,
    parameter           TEXT NOT NULL,
    current_value       REAL NOT NULL,
    proposed_value      REAL NOT NULL,
    rationale           TEXT,
    status              TEXT NOT NULL DEFAULT 'open',
    votes_for           INTEGER DEFAULT 0,
    votes_against       INTEGER DEFAULT 0,
    votes_abstain       INTEGER DEFAULT 0,
    quorum_threshold    REAL NOT NULL DEFAULT 0.5,
    voting_deadline_tick INTEGER NOT NULL,
    created_at          TEXT DEFAULT (datetime('now')),
    resolved_at         TEXT
);

CREATE INDEX IF NOT EXISTS idx_proposals_status ON proposals(status);
```

**Proposal status FSM:**
```
open --> approved (quorum met, votes_for > votes_against)
  |
  +--> rejected (quorum met, votes_against >= votes_for)
  |
  +--> expired (voting_deadline_tick reached without quorum)
```

### 3.2 votes

Individual vote records.

```sql
CREATE TABLE IF NOT EXISTS votes (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id         TEXT NOT NULL REFERENCES proposals(id),
    sphere_id           TEXT NOT NULL,
    vote                TEXT NOT NULL,  -- approve, reject, abstain
    created_at          TEXT DEFAULT (datetime('now')),
    UNIQUE(proposal_id, sphere_id)
);

CREATE INDEX IF NOT EXISTS idx_votes_proposal ON votes(proposal_id);
```

### 3.3 consent_declarations

Per-sphere consent preferences (persistent across restarts).

```sql
CREATE TABLE IF NOT EXISTS consent_declarations (
    sphere_id           TEXT PRIMARY KEY,
    accept_external_modulation BOOLEAN DEFAULT 1,
    max_k_adjustment    REAL DEFAULT 0.15,
    accept_cascade      BOOLEAN DEFAULT 1,
    accept_observation  BOOLEAN DEFAULT 1,
    accept_nvim_monitoring BOOLEAN DEFAULT 1,
    updated_at          TEXT DEFAULT (datetime('now'))
);
```

### 3.4 data_manifests

Data sovereignty tracking — what data exists about each sphere in each system.

```sql
CREATE TABLE IF NOT EXISTS data_manifests (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    sphere_id           TEXT NOT NULL,
    system              TEXT NOT NULL,
    record_count        INTEGER DEFAULT 0,
    last_scanned_at     TEXT DEFAULT (datetime('now'))
);
```

## 4. WAL Configuration

Both databases use Write-Ahead Logging for concurrent access:

```rust
/// m36_persistence.rs
fn configure_db(conn: &Connection) -> PvResult<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", WAL_BUSY_TIMEOUT_MS)?; // 5000ms
    conn.pragma_update(None, "synchronous", "NORMAL")?;  // WAL-safe
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}
```

| Pragma | Value | Rationale |
|--------|-------|-----------|
| journal_mode | WAL | Concurrent readers + single writer |
| busy_timeout | 5000 | Wait 5s before SQLITE_BUSY |
| synchronous | NORMAL | WAL-safe; checkpoint handles durability |
| foreign_keys | ON | Enforce task_tags/task_dependencies FK |

## 5. Cross-DB Query Patterns

### 5.1 Correlating Field State with Tasks

```sql
-- Attach both databases
ATTACH 'data/field_tracking.db' AS field;
ATTACH 'data/bus_tracking.db' AS bus;

-- Tasks completed during high-r periods
SELECT bt.id, bt.description, fs.r
FROM bus.bus_tasks bt
JOIN field.field_snapshots fs ON fs.tick = CAST(bt.completed_at AS INTEGER)
WHERE bt.status = 'completed' AND fs.r > 0.8;
```

### 5.2 Sphere Activity Timeline

```sql
-- Combine sphere history with task activity
SELECT 'lifecycle' as type, sh.tick, sh.event_type, sh.sphere_id
FROM sphere_history sh WHERE sh.sphere_id = ?
UNION ALL
SELECT 'task' as type, CAST(bt.submitted_at AS INTEGER), bt.status, bt.source_sphere
FROM bus_tasks bt WHERE bt.source_sphere = ?
ORDER BY tick;
```

### 5.3 Governance Impact Analysis

```sql
-- r values before and after approved proposals
SELECT p.parameter, p.proposed_value,
       fs_before.r as r_before, fs_after.r as r_after
FROM proposals p
JOIN field_snapshots fs_before ON fs_before.tick = p.voting_deadline_tick - 5
JOIN field_snapshots fs_after ON fs_after.tick = p.voting_deadline_tick + 5
WHERE p.status = 'approved';
```

## 6. Index Strategy

| Index | Table | Columns | Query Pattern |
|-------|-------|---------|---------------|
| PK | field_snapshots | tick | Latest snapshot, range queries |
| idx_sphere_history_sphere | sphere_history | sphere_id | Per-sphere lifecycle |
| idx_sphere_history_tick | sphere_history | tick | Temporal queries |
| idx_bus_tasks_status | bus_tasks | status | Pending task lookup |
| idx_bus_events_type | bus_events | event_type | Event type filtering |
| idx_bus_events_tick | bus_events | tick | Temporal event queries |
| idx_cascade_status | cascade_events | status | Active cascade tracking |
| idx_proposals_status | proposals | status | Open proposal lookup |
| idx_votes_proposal | votes | proposal_id | Vote tallying |

## 7. Data Retention

| Table | Retention | Strategy |
|-------|-----------|----------|
| field_snapshots | Unlimited | Manual archival recommended at 10K rows |
| sphere_history | Unlimited | Audit trail |
| coupling_history | 1000 rows | Oldest pruned on write |
| bus_tasks | 30 days | Expired tasks deleted on startup |
| bus_events | 10K rows | Batch prune on write |
| cascade_events | Unlimited | Audit trail |
| proposals | Unlimited | Governance record |
| votes | Unlimited | Governance record |
| consent_declarations | Current only | One row per sphere |
| data_manifests | Current only | Refreshed on scan |

## 8. Testing Strategy

| Test | Property |
|------|----------|
| Migration idempotency | Run all 3 migrations twice, no errors |
| WAL mode verification | PRAGMA journal_mode returns 'wal' |
| Concurrent read/write | Writer does not block readers |
| Field snapshot roundtrip | Write + read snapshot matches |
| Task lifecycle persistence | Submit, claim, complete — all persisted |
| Cross-DB attach | ATTACH + JOIN works correctly |
| Foreign key enforcement | task_tags with invalid task_id rejected |
| Busy timeout | Concurrent writers wait, not fail |

## 9. File Locations

| Database | Dev Path | Production Path |
|----------|----------|-----------------|
| field_tracking.db | `data/field_tracking.db` | `~/.local/share/pane-vortex/field_tracking.db` |
| bus_tracking.db | `data/bus_tracking.db` | `~/.local/share/pane-vortex/bus_tracking.db` |
| Migrations | `migrations/` | Embedded in binary (via include_str!) |
