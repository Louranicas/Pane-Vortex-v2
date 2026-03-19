//! # Layer 4: Coupling
//!
//! Kuramoto coupling network with Jacobi integration and auto-K scaling.
//! Depends on L1 (Foundation), L3 (Field).
//!
//! ## Design Constraints: C1 C3 C11 C12
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m16_coupling_network` | ~300 | Weight matrix, Jacobi dt=0.01, phase stepping |
//! | `m17_auto_k` | ~200 | Auto-scale K by mean effective weight, period=20 ticks |
//! | `m18_topology` | ~150 | Weight² amplification, neighborhood queries |

pub mod m16_coupling_network;
pub mod m17_auto_k;
pub mod m18_topology;
