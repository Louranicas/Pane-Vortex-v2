//! # M04: Constants
//!
//! All magic numbers as named constants. Values sourced from config/default.toml.
//! Constants that are configurable at runtime live in M03; compile-time constants live here.
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M04
//! ## Dependencies: None
//!
//! ## Constant Groups
//! - Tick timing: `TICK_INTERVAL`, `COUPLING_STEPS_PER_TICK`
//! - Kuramoto: `KURAMOTO_DT`, `DEFAULT_WEIGHT`, `WEIGHT_EXPONENT`
//! - Hebbian: LTP, LTD, `BURST_MULTIPLIER`, `WEIGHT_FLOOR`
//! - Thresholds: `PHASE_GAP`, SYNC, TUNNEL, `R_TARGET`, `R_HIGH`, `R_LOW`
//! - Limits: `SPHERE_CAP`, `MEMORY_MAX`, `GHOST_MAX`, `LOG_MAX`
//! - Budget: `K_MOD_MIN`, `K_MOD_MAX`, `K_MOD_BUDGET_MIN`, `K_MOD_BUDGET_MAX`
//!
//! ## Related Documentation
//! - [Design Constraints](../../ai_specs/DESIGN_CONSTRAINTS.md)
