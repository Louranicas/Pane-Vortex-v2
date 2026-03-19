//! # M06: Input Validation
//!
//! Validates all external inputs at system boundary.
//! Every API handler calls validation before processing.
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M06
//! ## Dependencies: M01 (types), M02 (errors), M04 (constants)
//!
//! ## Validators
//! - `validate_phase(f64)` → clamp to [0, 2π) via `rem_euclid(TAU)`
//! - `validate_frequency(f64)` → clamp to [0.001, 10.0], NaN guard
//! - `validate_strength(f64)` → clamp to [0.0, 2.0], NaN guard
//! - `validate_pane_id(&str)` → regex [a-zA-Z0-9_.:-]{1,128}
//! - `validate_persona(&str)` → max 256 chars, UTF-8 safe
//! - `validate_tool_name(&str)` → max 128 chars, `chars().take()`
//!
//! ## Design Constraints
//! - C3: Phase wrapping via `rem_euclid(TAU)`
//! - C11: `is_finite()` check on ALL f64 inputs
//! - C12: String truncation via `chars().take()` not byte slicing (R15)
//!
//! ## Related Documentation
//! - [Security Spec](../../ai_specs/SECURITY_SPEC.md)
