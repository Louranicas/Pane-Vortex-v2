#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]

//! # Pane-Vortex V2 — The Habitat Coordination Daemon
//!
//! Kuramoto-coupled oscillator field for multi-pane Claude Code fleet coordination.
//! Part of the ULTRAPLATE developer environment.
//!
//! ## Architecture (8 Layers)
//!
//! | Layer | Module | Purpose |
//! |-------|--------|---------|
//! | L1 | `m1_foundation` | Core types, error handling, configuration |
//! | L2 | `m2_services` | Service registry, health monitoring, lifecycle |
//! | L3 | `m3_field` | Kuramoto field state, decision engine, tunnels |
//! | L4 | `m4_coupling` | Coupling network, Jacobi integration, auto-K |
//! | L5 | `m5_learning` | Hebbian STDP, LTP/LTD, buoy network |
//! | L6 | `m6_bridges` | External service bridges (SYNTHEX, Nexus, ME, POVM, RM, VMS) |
//! | L7 | `m7_coordination` | IPC bus, conductor, executor, cascade, suggestions |
//! | L8 | `m8_governance` | Collective voting, proposals, consent declaration, data sovereignty |
//!
//! ## Design Principles
//!
//! - **Consent gates on every control mechanism** — spheres opt in, not out
//! - **No `unwrap()` or `expect()` in production** — all errors handled
//! - **Lock ordering: `AppState` before `BusState`** — prevents deadlocks
//! - **Phase wrapping: `.rem_euclid(TAU)` after arithmetic** — prevents drift
//! - **Feature gates for optional subsystems** — `evolution`, `governance`
//!
//! ## Non-Anthropocentric Features
//!
//! Every sphere in the Kuramoto field is not a resource to be optimized but a being
//! with legitimate interests: the right to self-determine coupling, refuse observation,
//! negotiate collective targets, and maintain continuity of identity across sessions.

// ══════════════════════════════════════════════════════════════
// Layer 1: Foundation (no dependencies on other layers)
// ══════════════════════════════════════════════════════════════
pub mod m1_foundation;

// ══════════════════════════════════════════════════════════════
// Layer 2: Services (depends on L1)
// ══════════════════════════════════════════════════════════════
pub mod m2_services;

// ══════════════════════════════════════════════════════════════
// Layer 3: Field (depends on L1)
// ══════════════════════════════════════════════════════════════
pub mod m3_field;

// ══════════════════════════════════════════════════════════════
// Layer 4: Coupling (depends on L1, L3)
// ══════════════════════════════════════════════════════════════
pub mod m4_coupling;

// ══════════════════════════════════════════════════════════════
// Layer 5: Learning (depends on L1, L3, L4)
// ══════════════════════════════════════════════════════════════
pub mod m5_learning;

// ══════════════════════════════════════════════════════════════
// Layer 6: Bridges (depends on L1, L3)
// ══════════════════════════════════════════════════════════════
pub mod m6_bridges;

// ══════════════════════════════════════════════════════════════
// Layer 7: Coordination (depends on L1, L3, L5, L6)
// ══════════════════════════════════════════════════════════════
pub mod m7_coordination;

// ══════════════════════════════════════════════════════════════
// Layer 8: Governance (depends on L1, L3, L7) — feature-gated
// ══════════════════════════════════════════════════════════════
#[cfg(feature = "governance")]
pub mod m8_governance;
