//! # M14: Phase Messaging
//!
//! 5 `PhaseMessage` types for inter-sphere communication:
//! Steer, `EmergencyCoherence`, `CrossActivation`, `DivergenceRequest`, `PhaseQuery`.
//!
//! ## Layer: L3 (Field) | Module: M14 | Dependencies: L1 (M01)
//! ## Design Constraints: C8 (consent gate on Steer), C12 (`EmergencyCoherence` targets capped at 50)
//! ## Related: [Wire Protocol Spec](../../ai_specs/WIRE_PROTOCOL_SPEC.md)
