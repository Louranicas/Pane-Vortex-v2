//! # Layer 3: Field
//!
//! Kuramoto oscillator field state, decision engine, tunnels, chimera detection.
//! Depends on L1 (Foundation).
//!
//! ## Design Constraints: C1 C2 C3 C5 C7 C8 C11 C12
//! See `ai_specs/DESIGN_CONSTRAINTS.md` for full definitions.
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m11_sphere` | ~500 | PaneSphere oscillator, memory, semantic phase, self-model |
//! | `m12_field_state` | ~400 | FieldState, FieldDecision, tunnels, ghost traces |
//! | `m13_chimera` | ~200 | Phase-gap cluster detection O(N log N) |
//! | `m14_messaging` | ~150 | PhaseMessage types (5 variants) |
//! | `m15_app_state` | ~500 | SharedState (AppState + BusState), snapshots, warmup |

pub mod m11_sphere;
pub mod m12_field_state;
pub mod m13_chimera;
pub mod m14_messaging;
pub mod m15_app_state;
