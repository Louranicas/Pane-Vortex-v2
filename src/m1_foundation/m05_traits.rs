//! # M05: Core Traits
//!
//! Dependency-inversion traits for cross-layer abstractions.
//! All trait methods use `&self` with interior mutability (C2).
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M05
//! ## Dependencies: M01 (types), M02 (errors)
//!
//! ## Traits
//! - `Oscillator`: `phase()`, `frequency()`, `step()`, `reset()`
//! - `Learnable`: `ltp()`, `ltd()`, `weight()`, `decay()`
//! - `Bridgeable`: `poll()`, `post()`, `health()`, `is_stale()`
//! - `Consentable`: `receptivity()`, `opt_out()`, `consent_posture()`
//! - `Persistable`: `snapshot()`, `restore()`, `migrate()`
//!
//! ## Design Constraints
//! - C2: All methods `&self` — interior mutability via `parking_lot::RwLock`
//! - C7: Owned returns through `RwLock` (never return references)
//! - All traits require `Send + Sync + std::fmt::Debug`
//!
//! ## Related Documentation
//! - [Rust Core Patterns](../../ai_specs/patterns/RUST_CORE_PATTERNS.md)
//! - [ME v2 Trait Design](../../ai_docs/WEB_RESEARCH.md)
