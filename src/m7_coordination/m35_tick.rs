//! # M35: Tick Orchestrator
//!
//! Decomposes the v1 829-line god function `tick_once()` into 5 phase functions
//! plus an orchestrator. This is the heartbeat of the system.
//!
//! ## Layer: L7 (Coordination)
//! ## Module: M35
//! ## Dependencies: L1, L3 (field), L4 (coupling), L5 (learning), L6 (bridges)
//!
//! ## Architecture (from Session 039 refactor plan)
//! ```text
//! tick_orchestrator() ~30L
//!   ├── tick_coupling()    ~120L | ~8br  | Kuramoto steps + auto-K + frequency
//!   ├── tick_learning()    ~150L | ~12br | Hebbian STDP + newcomer + burst
//!   ├── tick_bridging()    ~200L | ~15br | 6 service polls + consent gate
//!   ├── tick_decisions()   ~180L | ~18br | Conductor + suggestions + cascade
//!   └── tick_persistence() ~100L | ~8br  | Snapshots + sphere history + events
//! ```
//!
//! ## Design Constraints
//! - C5: Lock ordering — `AppState` acquired once at start, `BusState` after decisions
//! - C6: All bridge calls (`tokio::spawn`) happen in `tick_bridging`, locks released first
//! - C14: Fire-and-forget for bridge writes
//! - Max branch count per function: 18 (was 65 in v1 monolith)
//!
//! ## V1 → V2 Migration
//! - v1: `tick_once()` in main.rs — 829 lines, 65 branches, 32-space nesting
//! - v2: 5 functions in `m35_tick.rs` — max 200 lines each, max 18 branches
//! - Identical behavior, testable phases, clear lock scoping
//!
//! ## Related Documentation
//! - [[Session 039 — Architectural Schematics and Refactor Safety]] — decomposition plan
//! - [Coordination Layer Spec](../../ai_specs/layers/L7_COORDINATION_SPEC.md)
