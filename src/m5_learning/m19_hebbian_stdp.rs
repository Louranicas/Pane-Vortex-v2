//! # M19: Hebbian STDP Learning
//!
//! Spike-timing dependent plasticity adapted for Kuramoto oscillators.
//! LTP=0.01 for co-active pairs, LTD=0.002 otherwise. Burst detection (3x).
//! Newcomer boost (2x for first 50 steps). Weight floor at 0.15.
//!
//! ## Layer: L5 (Learning) | Module: M19 | Dependencies: L1, L3 (M11), L4 (M16)
//!
//! ## Learning Rule
//! - Co-active spheres (both Working in same tick): LTP += 0.01
//! - Burst detection (>3 memories in 5 ticks): LTP *= 3.0
//! - Newcomer (<50 steps): LTP *= 2.0
//! - Non-co-active: LTD -= 0.002
//! - Weight floor: `max(new_weight`, 0.15)
//!
//! ## POVM Insight: Bimodal Distribution
//! Pathway weights cluster at >0.9 or <0.3 (phase-transitive learning).
//! The transition zone (0.3-0.8) is almost empty — learning crystallizes, not drifts.
//! See [[Session 039 — What I Learned]] for analysis.
//!
//! ## Design Constraints: C11, C12 (weight floor enforcement)
//! ## Related: [STDP Wikipedia](https://en.wikipedia.org/wiki/Spike-timing-dependent_plasticity)
