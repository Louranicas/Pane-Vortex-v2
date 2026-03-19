-- Migration 002: IPC bus tables
-- Task queue, events, subscriptions, cascade tracking.

CREATE TABLE IF NOT EXISTS bus_tasks (
    id                  TEXT PRIMARY KEY,
    status              TEXT NOT NULL DEFAULT 'submitted',  -- submitted, claimed, completed, failed, expired
    source_sphere       TEXT NOT NULL,
    target              TEXT,           -- specific sphere ID or routing hint
    target_type         TEXT NOT NULL,  -- specific, any_idle, field_driven, willing
    description         TEXT NOT NULL,
    payload             TEXT,           -- JSON payload
    claimed_by          TEXT,
    submitted_at        TEXT DEFAULT (datetime('now')),
    claimed_at          TEXT,
    completed_at        TEXT,
    ttl_secs            INTEGER DEFAULT 3600
);

CREATE TABLE IF NOT EXISTS bus_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type          TEXT NOT NULL,
    source_sphere       TEXT,
    data                TEXT,           -- JSON event data
    tick                INTEGER,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS event_subscriptions (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    sphere_id           TEXT NOT NULL,
    pattern             TEXT NOT NULL,  -- glob pattern (e.g., 'field.*')
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS cascade_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    source_sphere       TEXT NOT NULL,
    target_sphere       TEXT,
    brief               TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'dispatched',  -- dispatched, acked, rejected
    depth               INTEGER DEFAULT 0,
    created_at          TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS task_tags (
    task_id             TEXT NOT NULL REFERENCES bus_tasks(id),
    tag                 TEXT NOT NULL,
    PRIMARY KEY (task_id, tag)
);

CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id             TEXT NOT NULL REFERENCES bus_tasks(id),
    depends_on          TEXT NOT NULL REFERENCES bus_tasks(id),
    PRIMARY KEY (task_id, depends_on)
);

CREATE INDEX IF NOT EXISTS idx_bus_events_type ON bus_events(event_type);
CREATE INDEX IF NOT EXISTS idx_bus_events_tick ON bus_events(tick);
CREATE INDEX IF NOT EXISTS idx_bus_tasks_status ON bus_tasks(status);
CREATE INDEX IF NOT EXISTS idx_cascade_status ON cascade_events(status);
