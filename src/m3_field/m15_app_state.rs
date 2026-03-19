//! # M15: Application State
//!
//! `AppState` (24 fields) and `BusState` (12 fields) wrapped in `Arc<parking_lot::RwLock<T>>`.
//! Snapshots, ghost traces, warmup period, `r_history` rolling window.
//!
//! ## Layer: L3 (Field) | Module: M15 | Dependencies: L1 (M01, M02, M04), M11, M12
//!
//! ## Lock Ordering (C5): `AppState` BEFORE `BusState`. ALWAYS.
//! ## Warmup (C12): 5-tick warmup after snapshot restore suspends Hebbian/conductor
//! ## Ghost Traces: FIFO capped at `GHOST_MAX` (20), weight inheritance requires consent (NA-P-11)
//!
//! ## Risk: God Object
//! `AppState` has 24 fields. Consider decomposing: `FieldSnapshot`, `ConductorState`, `BridgeState`, `BusState`.
//!
//! ## Related: [[Session 039 — Architectural Schematics and Refactor Safety]], [L3 Spec](../../ai_specs/layers/L3_FIELD_SPEC.md)
