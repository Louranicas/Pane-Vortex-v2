//! # M36: Persistence
//! `SQLite` WAL mode. `field_tracking.db` + `bus_tracking.db`. Snapshots every 60 ticks.
//! `busy_timeout` 5000ms. Migrations applied idempotently on startup.
//! ## Layer: L7 | Module: M36 | Dependencies: L1, L3 (M15 `app_state`)
//! ## Feature Gate: #[cfg(feature = "persistence")]
//! ## Related: [Database Spec](../../ai_specs/DATABASE_SPEC.md), [migrations/](../../migrations/)
