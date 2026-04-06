//! # Layer 7: Coordination
//!
//! IPC bus, conductor, executor, cascade, field suggestions, tick orchestration.
//! Depends on L1, L3, L5, L6.
//!
//! ## Design Constraints: C1 C5 C6 C10 C12 C14
//! - C5: Lock ordering (`AppState` before `BusState`) critical here
//! - C6: Signal/event emission AFTER lock release
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m29_ipc_bus` | ~500 | Unix socket, NDJSON, subscriptions |
//! | `m30_bus_types` | ~300 | BusFrame (11 kinds), BusTask, events |
//! | `m31_conductor` | ~400 | PI controller, r_target, breathing |
//! | `m32_executor` | ~200 | Thin Zellij dispatch |
//! | `m33_cascade` | ~200 | CascadeHandoff/Ack, rate limiting |
//! | `m34_suggestions` | ~200 | Field-driven, sphere autonomy filtered |
//! | `m35_tick` | ~300 | tick_orchestrator() + 5 phases |
//! | `m36_persistence` | ~250 | SQLite WAL, snapshots, events |

pub mod m29_ipc_bus;
pub mod m30_bus_types;
pub mod m31_conductor;
pub mod m32_executor;
pub mod m33_cascade;
pub mod m34_suggestions;
pub mod m35_tick;

#[cfg(feature = "persistence")]
pub mod m36_persistence;

// ── Ergonomic re-exports ──

// IPC bus
pub use m29_ipc_bus::{BusState, BusSubscriber};

// Bus types
pub use m30_bus_types::{BusEvent, BusFrame, BusTask, TaskStatus, TaskTarget};

// Conductor
pub use m31_conductor::Conductor;

// Executor
pub use m32_executor::{Executor, ExecutorResult};

// Cascade
pub use m33_cascade::{CascadeHandoff, CascadeTracker};

// Suggestions
pub use m34_suggestions::{FieldSuggestion, SuggestionEngine, SuggestionType};

// Tick orchestration
pub use m35_tick::{tick_orchestrator, PhaseTiming, TickResult};

// Persistence (feature-gated)
#[cfg(feature = "persistence")]
pub use m36_persistence::PersistenceManager;
