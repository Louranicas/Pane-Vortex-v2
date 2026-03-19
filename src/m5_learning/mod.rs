//! # Layer 5: Learning
//!
//! Hebbian STDP, LTP/LTD, burst detection, buoy network.
//! Depends on L1 (Foundation), L3 (Field), L4 (Coupling).
//!
//! ## Design Constraints: C1 C11 C12
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m19_hebbian_stdp` | ~300 | LTP=0.01, LTD=0.002, burst 3x, newcomer 2x |
//! | `m20_buoy_network` | ~200 | 3D buoy positions, activation thresholds, pruning |
//! | `m21_memory_manager` | ~250 | Sphere memory storage, amortised prune at MAX+50 |

pub mod m19_hebbian_stdp;
pub mod m20_buoy_network;
pub mod m21_memory_manager;
