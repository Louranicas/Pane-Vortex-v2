//! # M03: Configuration
//!
//! Figment-based configuration loading with TOML defaults + environment overrides.
//! Load priority: config/default.toml → config/production.toml → PV2_* env vars.
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M03
//! ## Dependencies: M02 (`PvError`)
//!
//! ## Design Constraints
//! - C1: No upward imports
//! - C13: Builder pattern for Config construction
//! - Serde defaults for backward compatibility (S04)
//!
//! ## Configuration Sections (11)
//! server, field, sphere, coupling, learning, bridges, conductor, ipc, persistence, governance, security
//!
//! ## Related Documentation
//! - [config/default.toml](../../config/default.toml) — all defaults
//! - [Layer Specification](../../ai_specs/layers/L1_FOUNDATION_SPEC.md)
