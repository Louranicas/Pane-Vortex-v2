//! # M12: Field State & Decision Engine
//!
//! `FieldState` holds the Kuramoto field configuration. `FieldDecision` is the
//! conductor's output: Stable, `NeedsCoherence`, `NeedsDivergence`, `HasBlockedAgents`,
//! `IdleFleet`, `FreshFleet`, Recovering.
//!
//! ## Layer: L3 (Field) | Module: M12 | Dependencies: L1 (M01, M02, M04)
//!
//! ## Decision Priority Chain
//! `HasBlockedAgents` > `NeedsCoherence` (r>0.3, falling, N≥2) > `NeedsDivergence` (r>0.8, idle>60%, N≥2) > `IdleFleet` > `FreshFleet` > Stable
//!
//! ## Design Constraints: C3 (phase wrapping), C11 (NaN guard), C12 (bounded tunnel list)
//! ## Related: [L3 Spec](../../ai_specs/layers/L3_FIELD_SPEC.md), [[Vortex Sphere Brain-Body Architecture]]
