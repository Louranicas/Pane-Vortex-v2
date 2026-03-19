//! # M22: SYNTHEX Bridge
//! Bidirectional REST bridge to SYNTHEX :8090. Polls /v3/thermal every 6 ticks.
//! Thermal `k_adjustment` feeds into consent gate (M28). Posts field state to /api/ingest.
//! ## Layer: L6 | Module: M22 | Dependencies: L1, L3
//! ## Pattern: Raw TCP HTTP, fire-and-forget writes (C14), consent-gated reads (C8)
//! ## Alert: SYNTHEX synergy at 0.15-0.5 (Session 040 ALERT-1)
//! ## Related: [[Session 034f — SYNTHEX Schematics and Wiring]], [[Synthex (The brain of the developer environment)]]
