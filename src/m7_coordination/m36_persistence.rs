//! # M36: Persistence
//!
//! `SQLite` WAL mode. `field_tracking.db` + `bus_tracking.db`. Snapshots every 60 ticks.
//! `busy_timeout` 5000ms. Migrations applied idempotently on startup.
//!
//! ## Layer: L7 | Module: M36 | Dependencies: L1, L3 (M15 `app_state`)
//! ## Feature Gate: `#[cfg(feature = "persistence")]`

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags};

use crate::m1_foundation::{
    m01_core_types::now_secs,
    m02_error_handling::{PvError, PvResult},
};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// `SQLite` busy timeout (milliseconds).
const BUSY_TIMEOUT_MS: i32 = 5000;

/// Default data directory for persistence files.
const DEFAULT_DATA_DIR: &str = "/tmp/pane-vortex-v2";

// ──────────────────────────────────────────────────────────────
// PersistenceManager
// ──────────────────────────────────────────────────────────────

/// `SQLite` WAL persistence manager for field snapshots and bus events.
///
/// Manages two databases:
/// - `field_tracking.db`: field snapshots, sphere history, coupling history
/// - `bus_tracking.db`: bus tasks, bus events
#[derive(Debug)]
pub struct PersistenceManager {
    /// Path to the field tracking database.
    field_db_path: PathBuf,
    /// Path to the bus tracking database.
    bus_db_path: PathBuf,
}

impl PersistenceManager {
    /// Create a new persistence manager with default paths.
    ///
    /// # Errors
    /// Returns `PvError::Database` if the data directory cannot be created.
    pub fn new() -> PvResult<Self> {
        Self::with_data_dir(DEFAULT_DATA_DIR)
    }

    /// Create a new persistence manager with a custom data directory.
    ///
    /// # Errors
    /// Returns `PvError::Database` if the directory cannot be created.
    pub fn with_data_dir(data_dir: &str) -> PvResult<Self> {
        let path = Path::new(data_dir);
        std::fs::create_dir_all(path).map_err(|e| {
            PvError::Database(format!("failed to create data dir {data_dir}: {e}"))
        })?;

        let field_db_path = path.join("field_tracking.db");
        let bus_db_path = path.join("bus_tracking.db");

        let manager = Self {
            field_db_path,
            bus_db_path,
        };

        // Initialize schemas
        manager.init_field_schema()?;
        manager.init_bus_schema()?;

        Ok(manager)
    }

    /// Create a persistence manager with a temporary directory.
    ///
    /// Uses a unique subdirectory under the system temp dir. Suitable for testing
    /// or ephemeral instances.
    ///
    /// # Errors
    /// Returns `PvError::Database` if the temporary directory cannot be created.
    pub fn temporary() -> PvResult<Self> {
        let unique = format!(
            "pane-vortex-v2-tmp-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        );
        let tmp_dir = std::env::temp_dir().join(unique);
        Self::with_data_dir(
            tmp_dir
                .to_str()
                .ok_or_else(|| PvError::Database("invalid temp path".into()))?,
        )
    }

    // ── Schema initialization ──

    /// Initialize field tracking database schema.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    fn init_field_schema(&self) -> PvResult<()> {
        let conn = self.open_field_db()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS field_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tick INTEGER NOT NULL,
                timestamp REAL NOT NULL,
                r REAL NOT NULL,
                psi REAL NOT NULL,
                sphere_count INTEGER NOT NULL,
                total_memories INTEGER NOT NULL,
                k_modulation REAL NOT NULL,
                snapshot_json TEXT NOT NULL,
                created_at REAL NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS sphere_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tick INTEGER NOT NULL,
                sphere_id TEXT NOT NULL,
                phase REAL NOT NULL,
                frequency REAL NOT NULL,
                memory_count INTEGER NOT NULL,
                activation_density REAL NOT NULL,
                receptivity REAL NOT NULL,
                status TEXT NOT NULL,
                created_at REAL NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS coupling_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tick INTEGER NOT NULL,
                from_sphere TEXT NOT NULL,
                to_sphere TEXT NOT NULL,
                weight REAL NOT NULL,
                created_at REAL NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_field_snapshots_tick ON field_snapshots(tick);
            CREATE INDEX IF NOT EXISTS idx_sphere_history_tick ON sphere_history(tick);
            CREATE INDEX IF NOT EXISTS idx_sphere_history_sphere ON sphere_history(sphere_id);
            CREATE INDEX IF NOT EXISTS idx_coupling_history_tick ON coupling_history(tick);
            ",
        )
        .map_err(|e| PvError::Database(format!("field schema init failed: {e}")))?;
        Ok(())
    }

    /// Initialize bus tracking database schema.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    fn init_bus_schema(&self) -> PvResult<()> {
        let conn = self.open_bus_db()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS bus_tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_pane_id TEXT,
                status TEXT NOT NULL,
                submitted_by TEXT NOT NULL,
                claimed_by TEXT,
                submitted_at REAL NOT NULL,
                completed_at REAL,
                created_at REAL NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE TABLE IF NOT EXISTS bus_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                data_json TEXT NOT NULL,
                tick INTEGER NOT NULL,
                timestamp REAL NOT NULL,
                created_at REAL NOT NULL DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_bus_tasks_status ON bus_tasks(status);
            CREATE INDEX IF NOT EXISTS idx_bus_events_tick ON bus_events(tick);
            CREATE INDEX IF NOT EXISTS idx_bus_events_type ON bus_events(event_type);
            ",
        )
        .map_err(|e| PvError::Database(format!("bus schema init failed: {e}")))?;
        Ok(())
    }

    // ── Database connections ──

    /// Open the field tracking database.
    ///
    /// # Errors
    /// Returns `PvError::Database` on connection failure.
    fn open_field_db(&self) -> PvResult<Connection> {
        open_wal_connection(&self.field_db_path)
    }

    /// Open the bus tracking database.
    ///
    /// # Errors
    /// Returns `PvError::Database` on connection failure.
    fn open_bus_db(&self) -> PvResult<Connection> {
        open_wal_connection(&self.bus_db_path)
    }

    // ── Snapshot operations ──

    /// Save a field snapshot.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    /// Returns `PvError::Snapshot` on serialization errors.
    #[allow(clippy::too_many_arguments)]
    pub fn save_snapshot(
        &self,
        tick: u64,
        r: f64,
        psi: f64,
        sphere_count: usize,
        total_memories: usize,
        k_modulation: f64,
        snapshot_json: &str,
    ) -> PvResult<i64> {
        let conn = self.open_field_db()?;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
        conn.execute(
            "INSERT INTO field_snapshots (tick, timestamp, r, psi, sphere_count, total_memories, k_modulation, snapshot_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                tick as i64,
                now_secs(),
                r,
                psi,
                sphere_count as i64,
                total_memories as i64,
                k_modulation,
                snapshot_json,
            ],
        )
        .map_err(|e| PvError::Database(format!("save_snapshot failed: {e}")))?;

        Ok(conn.last_insert_rowid())
    }

    /// Load the most recent snapshot.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    /// Returns `PvError::Snapshot` if no snapshot exists.
    pub fn load_snapshot(&self) -> PvResult<(u64, String)> {
        let conn = self.open_field_db()?;
        let result = conn
            .query_row(
                "SELECT tick, snapshot_json FROM field_snapshots ORDER BY tick DESC LIMIT 1",
                [],
                |row| {
                    let tick: i64 = row.get(0)?;
                    let json: String = row.get(1)?;
                    Ok((tick, json))
                },
            )
            .map_err(|e| PvError::Snapshot(format!("load_snapshot failed: {e}")))?;
        #[allow(clippy::cast_sign_loss)]
        Ok((result.0 as u64, result.1))
    }

    /// Load a snapshot at a specific tick.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    /// Returns `PvError::Snapshot` if the tick is not found.
    pub fn load_snapshot_at_tick(&self, tick: u64) -> PvResult<String> {
        let conn = self.open_field_db()?;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
        conn.query_row(
            "SELECT snapshot_json FROM field_snapshots WHERE tick = ?1",
            params![tick as i64],
            |row| row.get(0),
        )
        .map_err(|e| PvError::Snapshot(format!("snapshot at tick {tick} not found: {e}")))
    }

    // ── Event operations ──

    /// Save a bus event.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    pub fn save_event(
        &self,
        event_type: &str,
        data_json: &str,
        tick: u64,
    ) -> PvResult<i64> {
        let conn = self.open_bus_db()?;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
        conn.execute(
            "INSERT INTO bus_events (event_type, data_json, tick, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![event_type, data_json, tick as i64, now_secs()],
        )
        .map_err(|e| PvError::Database(format!("save_event failed: {e}")))?;

        Ok(conn.last_insert_rowid())
    }

    /// Retrieve recent events (most recent `limit`).
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    #[allow(clippy::cast_possible_wrap)]
    pub fn recent_events(&self, limit: usize) -> PvResult<Vec<(String, String, u64)>> {
        let conn = self.open_bus_db()?;
        let mut stmt = conn
            .prepare(
                "SELECT event_type, data_json, tick FROM bus_events
                 ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|e| PvError::Database(format!("prepare recent_events failed: {e}")))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let event_type: String = row.get(0)?;
                let data_json: String = row.get(1)?;
                let tick: i64 = row.get(2)?;
                #[allow(clippy::cast_sign_loss)]
                Ok((event_type, data_json, tick as u64))
            })
            .map_err(|e| PvError::Database(format!("query recent_events failed: {e}")))?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| PvError::Database(format!("row error: {e}")))?);
        }
        Ok(events)
    }

    /// Count events by type.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    pub fn event_count_by_type(&self, event_type: &str) -> PvResult<u64> {
        let conn = self.open_bus_db()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bus_events WHERE event_type = ?1",
                params![event_type],
                |row| row.get(0),
            )
            .map_err(|e| PvError::Database(format!("event_count failed: {e}")))?;
        #[allow(clippy::cast_sign_loss)]
        Ok(count as u64)
    }

    // ── Sphere history ──

    /// Save sphere state for historical tracking.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    #[allow(clippy::too_many_arguments)]
    pub fn save_sphere_history(
        &self,
        tick: u64,
        sphere_id: &str,
        phase: f64,
        frequency: f64,
        memory_count: usize,
        activation_density: f64,
        receptivity: f64,
        status: &str,
    ) -> PvResult<()> {
        let conn = self.open_field_db()?;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
        conn.execute(
            "INSERT INTO sphere_history (tick, sphere_id, phase, frequency, memory_count, activation_density, receptivity, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                tick as i64,
                sphere_id,
                phase,
                frequency,
                memory_count as i64,
                activation_density,
                receptivity,
                status,
            ],
        )
        .map_err(|e| PvError::Database(format!("save_sphere_history failed: {e}")))?;
        Ok(())
    }

    // ── Cleanup ──

    /// Prune old snapshots, keeping only the most recent `keep` entries.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    pub fn prune_snapshots(&self, keep: usize) -> PvResult<usize> {
        let conn = self.open_field_db()?;
        #[allow(clippy::cast_possible_wrap)]
        let deleted = conn
            .execute(
                "DELETE FROM field_snapshots WHERE id NOT IN (
                    SELECT id FROM field_snapshots ORDER BY tick DESC LIMIT ?1
                )",
                params![keep as i64],
            )
            .map_err(|e| PvError::Database(format!("prune_snapshots failed: {e}")))?;
        Ok(deleted)
    }

    /// Prune old events, keeping only the most recent `keep` entries.
    ///
    /// # Errors
    /// Returns `PvError::Database` on `SQLite` errors.
    pub fn prune_events(&self, keep: usize) -> PvResult<usize> {
        let conn = self.open_bus_db()?;
        #[allow(clippy::cast_possible_wrap)]
        let deleted = conn
            .execute(
                "DELETE FROM bus_events WHERE id NOT IN (
                    SELECT id FROM bus_events ORDER BY id DESC LIMIT ?1
                )",
                params![keep as i64],
            )
            .map_err(|e| PvError::Database(format!("prune_events failed: {e}")))?;
        Ok(deleted)
    }

    /// Get database file paths.
    #[must_use]
    pub fn field_db_path(&self) -> &Path {
        &self.field_db_path
    }

    /// Get bus database file path.
    #[must_use]
    pub fn bus_db_path(&self) -> &Path {
        &self.bus_db_path
    }
}

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

/// Open a `SQLite` connection with WAL mode and busy timeout.
fn open_wal_connection(path: &Path) -> PvResult<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| PvError::Database(format!("open {} failed: {e}", path.display())))?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| PvError::Database(format!("WAL mode failed: {e}")))?;
    conn.pragma_update(None, "busy_timeout", BUSY_TIMEOUT_MS)
        .map_err(|e| PvError::Database(format!("busy_timeout failed: {e}")))?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| PvError::Database(format!("synchronous failed: {e}")))?;

    Ok(conn)
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> PersistenceManager {
        PersistenceManager::temporary().unwrap()
    }

    // ── Construction ──

    #[test]
    fn persistence_manager_creation() {
        let pm = test_manager();
        assert!(pm.field_db_path().exists());
        assert!(pm.bus_db_path().exists());
    }

    #[test]
    fn persistence_manager_paths() {
        let pm = test_manager();
        assert!(pm.field_db_path().to_str().unwrap().contains("field_tracking"));
        assert!(pm.bus_db_path().to_str().unwrap().contains("bus_tracking"));
    }

    // ── Snapshot operations ──

    #[test]
    fn save_snapshot_success() {
        let pm = test_manager();
        let id = pm
            .save_snapshot(1, 0.95, 1.5, 3, 100, 1.0, r#"{"tick":1}"#)
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn load_snapshot_success() {
        let pm = test_manager();
        pm.save_snapshot(10, 0.8, 1.0, 5, 200, 1.1, r#"{"tick":10}"#)
            .unwrap();
        let (tick, json) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 10);
        assert!(json.contains("tick"));
    }

    #[test]
    fn load_snapshot_empty_fails() {
        let pm = test_manager();
        assert!(pm.load_snapshot().is_err());
    }

    #[test]
    fn load_snapshot_at_tick() {
        let pm = test_manager();
        pm.save_snapshot(5, 0.5, 0.5, 2, 50, 1.0, r#"{"tick":5}"#)
            .unwrap();
        pm.save_snapshot(10, 0.8, 1.0, 3, 100, 1.0, r#"{"tick":10}"#)
            .unwrap();
        let json = pm.load_snapshot_at_tick(5).unwrap();
        assert!(json.contains("5"));
    }

    #[test]
    fn load_snapshot_at_tick_not_found() {
        let pm = test_manager();
        assert!(pm.load_snapshot_at_tick(999).is_err());
    }

    #[test]
    fn load_latest_snapshot() {
        let pm = test_manager();
        pm.save_snapshot(1, 0.5, 0.5, 2, 50, 1.0, r#"{"tick":1}"#)
            .unwrap();
        pm.save_snapshot(5, 0.7, 0.8, 3, 80, 1.0, r#"{"tick":5}"#)
            .unwrap();
        pm.save_snapshot(10, 0.9, 1.2, 5, 150, 1.0, r#"{"tick":10}"#)
            .unwrap();
        let (tick, _) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 10);
    }

    #[test]
    fn save_multiple_snapshots() {
        let pm = test_manager();
        for i in 0..10 {
            let json = format!(r#"{{"tick":{i}}}"#);
            pm.save_snapshot(i, 0.5, 0.5, 1, 10, 1.0, &json)
                .unwrap();
        }
        let (tick, _) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 9);
    }

    // ── Event operations ──

    #[test]
    fn save_event_success() {
        let pm = test_manager();
        let id = pm.save_event("field.tick", r#"{"r":0.95}"#, 1).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn recent_events_returns_latest() {
        let pm = test_manager();
        pm.save_event("field.tick", r#"{"r":0.5}"#, 1).unwrap();
        pm.save_event("field.tick", r#"{"r":0.6}"#, 2).unwrap();
        pm.save_event("sphere.registered", r#"{"id":"a"}"#, 3)
            .unwrap();
        let events = pm.recent_events(2).unwrap();
        assert_eq!(events.len(), 2);
        // Most recent first
        assert_eq!(events[0].2, 3);
    }

    #[test]
    fn recent_events_empty() {
        let pm = test_manager();
        let events = pm.recent_events(10).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn recent_events_caps_at_limit() {
        let pm = test_manager();
        for i in 0..10 {
            pm.save_event("test", "data", i).unwrap();
        }
        let events = pm.recent_events(5).unwrap();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn event_count_by_type() {
        let pm = test_manager();
        pm.save_event("field.tick", "d", 1).unwrap();
        pm.save_event("field.tick", "d", 2).unwrap();
        pm.save_event("sphere.registered", "d", 3).unwrap();
        assert_eq!(pm.event_count_by_type("field.tick").unwrap(), 2);
        assert_eq!(pm.event_count_by_type("sphere.registered").unwrap(), 1);
        assert_eq!(pm.event_count_by_type("nonexistent").unwrap(), 0);
    }

    // ── Sphere history ──

    #[test]
    fn save_sphere_history() {
        let pm = test_manager();
        pm.save_sphere_history(1, "sphere-a", 1.5, 0.1, 10, 0.5, 0.8, "Working")
            .unwrap();
    }

    #[test]
    fn save_sphere_history_multiple() {
        let pm = test_manager();
        for i in 0..5 {
            #[allow(clippy::cast_precision_loss)]
            pm.save_sphere_history(i, "sphere-a", i as f64 * 0.1, 0.1, 10, 0.5, 0.8, "Working")
                .unwrap();
        }
    }

    // ── Pruning ──

    #[test]
    fn prune_snapshots() {
        let pm = test_manager();
        for i in 0..20 {
            let json = format!(r#"{{"tick":{i}}}"#);
            pm.save_snapshot(i, 0.5, 0.5, 1, 10, 1.0, &json)
                .unwrap();
        }
        let deleted = pm.prune_snapshots(5).unwrap();
        assert!(deleted > 0);
    }

    #[test]
    fn prune_snapshots_nothing_to_prune() {
        let pm = test_manager();
        pm.save_snapshot(1, 0.5, 0.5, 1, 10, 1.0, "{}")
            .unwrap();
        let deleted = pm.prune_snapshots(10).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn prune_events_success() {
        let pm = test_manager();
        for i in 0..20 {
            pm.save_event("test", "data", i).unwrap();
        }
        let deleted = pm.prune_events(5).unwrap();
        assert!(deleted > 0);
    }

    #[test]
    fn prune_events_nothing_to_prune() {
        let pm = test_manager();
        pm.save_event("test", "data", 1).unwrap();
        let deleted = pm.prune_events(10).unwrap();
        assert_eq!(deleted, 0);
    }

    // ── WAL mode verification ──

    #[test]
    fn wal_mode_enabled() {
        let pm = test_manager();
        let conn = pm.open_field_db().unwrap();
        let mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    // ── Schema idempotency ──

    #[test]
    fn schema_idempotent() {
        let pm = test_manager();
        // Calling init again should not fail
        pm.init_field_schema().unwrap();
        pm.init_bus_schema().unwrap();
    }

    // ── Full lifecycle ──

    #[test]
    fn full_persistence_lifecycle() {
        let pm = test_manager();

        // Save snapshots
        for i in 0..5 {
            let json = format!(r#"{{"tick":{i},"r":0.5}}"#);
            pm.save_snapshot(i, 0.5, 0.5, 3, 50, 1.0, &json)
                .unwrap();
        }

        // Save events
        for i in 0..10 {
            pm.save_event("field.tick", &format!(r#"{{"tick":{i}}}"#), i)
                .unwrap();
        }

        // Save sphere history
        pm.save_sphere_history(5, "s1", 1.0, 0.1, 20, 0.6, 0.9, "Working")
            .unwrap();

        // Load latest snapshot
        let (tick, json) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 4);
        assert!(json.contains("tick"));

        // Recent events
        let events = pm.recent_events(5).unwrap();
        assert_eq!(events.len(), 5);

        // Prune
        pm.prune_snapshots(2).unwrap();
        pm.prune_events(3).unwrap();

        // Load after prune
        let (tick, _) = pm.load_snapshot().unwrap();
        assert!(tick >= 3); // Most recent should survive
    }

    #[test]
    fn concurrent_read_write() {
        let pm = test_manager();
        // Save from "writer"
        pm.save_snapshot(1, 0.5, 0.5, 1, 10, 1.0, r#"{"tick":1}"#)
            .unwrap();

        // Read from "reader"
        let (tick, _) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 1);

        // Write again
        pm.save_snapshot(2, 0.6, 0.6, 2, 20, 1.0, r#"{"tick":2}"#)
            .unwrap();

        // Read again
        let (tick, _) = pm.load_snapshot().unwrap();
        assert_eq!(tick, 2);
    }

    // ── Edge cases ──

    #[test]
    fn save_empty_json_snapshot() {
        let pm = test_manager();
        pm.save_snapshot(1, 0.0, 0.0, 0, 0, 0.0, "{}").unwrap();
        let (_, json) = pm.load_snapshot().unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn save_large_json_snapshot() {
        let pm = test_manager();
        let large_json = format!(r#"{{"data":"{}"}}"#, "x".repeat(10000));
        pm.save_snapshot(1, 0.5, 0.5, 1, 1, 1.0, &large_json)
            .unwrap();
        let (_, json) = pm.load_snapshot().unwrap();
        assert!(json.len() > 10000);
    }

    #[test]
    fn save_event_unicode() {
        let pm = test_manager();
        pm.save_event("test.unicode", r#"{"msg":"日本語テスト"}"#, 1)
            .unwrap();
        let events = pm.recent_events(1).unwrap();
        assert!(events[0].1.contains("日本語"));
    }
}
