//! # M17: Auto-Scale K
//!
//! Periodically adjusts coupling strength K by dividing by mean effective weight.
//! Period: every 20 ticks. Multiplier: 0.5. Prevents over-synchronization.
//!
//! ## Layer: L4 | Module: M17 | Dependencies: L1, M16
//! ## Design Constraints: C11 (guard against division by zero mean weight)
//! ## Bug History: M11-M14 fixes in PV v1 — K multiplier 0.5 (was 1.5), fixed w² exponent
//! ## Related: CLAUDE.md "All Known Bugs FIXED" section 15-17
