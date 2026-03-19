//! # Layer 1: Foundation
//!
//! Core types, error handling, and configuration. No dependencies on other layers.
//! This is the bedrock — every other module depends on L1.
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m01_core_types` | ~300 | `Point3D`, `PaneId`, `SphereMemory`, `Buoy`, `Phase`, `Frequency` |
//! | `m02_error_handling` | ~200 | `PvError` enum, Result type, error conversion |
//! | `m03_config` | ~250 | Figment-based config loading, defaults, validation |
//! | `m04_constants` | ~100 | All magic numbers as named constants |
//! | `m05_traits` | ~150 | Core traits: Oscillator, Learnable, Bridgeable, Consentable |
//! | `m06_validation` | ~100 | Input validation: phase bounds, frequency clamp, string limits |

//! ## Design Constraints (C1-C14) applicable to L1:
//! - C1: No upward imports (this IS the base layer)
//! - C2: Trait methods always &self (interior mutability)
//! - C4: Zero unsafe/unwrap/expect (#![forbid] at crate level)
//! - C7: Owned returns through `RwLock`
//! - C11: NaN guard on all f64 inputs (M06)
//! - C12: Bounded collections always
//! - C13: Builder pattern for >2 parameters
//!
//! ## Enforced at compile-time:
//! `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`

pub mod m01_core_types;
pub mod m02_error_handling;
pub mod m03_config;
pub mod m04_constants;
pub mod m05_traits;
pub mod m06_validation;

// Re-exports will be added when modules are implemented.
// Planned exports:
//   pub use m01_core_types::{PaneId, Phase, Frequency};
//   pub use m02_error_handling::{PvError, PvResult};
//   pub use m03_config::PvConfig;
//   pub use m04_constants::*;
