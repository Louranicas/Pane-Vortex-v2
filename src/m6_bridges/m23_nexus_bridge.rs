//! # M23: Nexus Bridge (SAN-K7)
//! Nested Kuramoto bridge to SAN-K7 :8100. Fetches strategy coherence, `r_outer`.
//! 20 nexus commands in 6 categories. Strategy: Aligned|Partial|Diverging|Incoherent.
//! ## Layer: L6 | Module: M23 | Dependencies: L1, L3
//! ## Pattern: Raw TCP HTTP, deep poll every 60 ticks, consent-gated `k_adj`
//! ## Related: [[Nexus Controller V2]], [[Executor and Nested Kuramoto Bridge — Session 028]]
