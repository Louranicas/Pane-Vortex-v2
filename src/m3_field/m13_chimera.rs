//! # M13: Chimera Detection
//!
//! Phase-gap cluster detection O(N log N). Sorts phases, finds gaps > π/3,
//! classifies clusters as sync or desync based on `SYNC_THRESHOLD`.
//!
//! ## Layer: L3 (Field) | Module: M13 | Dependencies: L1 (M01, M04)
//! ## Algorithm: Sort phases → find gaps > `PHASE_GAP_THRESHOLD` → cluster assignment
//! ## Design Constraints: C3 (phase wrapping before sort), C11 (NaN filtered pre-sort)
//! ## Related: [Kuramoto Field Spec](../../ai_specs/KURAMOTO_FIELD_SPEC.md)
