//! # M21: Memory Manager
//!
//! Per-sphere memory storage. Amortised batch prune at `MEMORY_MAX_COUNT+50`.
//! Memory pruning every 200 steps removes lowest-activation entries.
//!
//! ## Layer: L5 | Module: M21 | Dependencies: L1, L3 (M11)
//! ## Design Constraints: C12 (bounded at 500), P11 (amortised prune pattern)
//! ## Bug History: BUG-9 (v1) — unbounded memory growth fixed with amortised prune
