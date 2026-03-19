//! # M01: Core Types
//!
//! Foundational types used across all layers: `PaneId`, `Phase`, `Frequency`,
//! `Point3D`, `SphereMemory`, `Buoy`, `ModuleId`.
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M01
//! ## Dependencies: None (leaf module)
//!
//! ## Design Constraints
//! - C1: No upward imports (this is L1 — no dependencies)
//! - C4: Zero unsafe/unwrap/expect
//! - C5: Copy semantics for `Point3D` (24 bytes = 3×f64)
//! - C7: Newtype wrappers for type safety (`PaneId`, `TaskId`, `Phase`)
//!
//! ## Related Documentation
//! - [Layer Specification](../../ai_specs/layers/L1_FOUNDATION_SPEC.md)
//! - [Module Documentation](../../ai_docs/modules/L1_FOUNDATION.md)
//! - [Design Constraints](../../ai_specs/DESIGN_CONSTRAINTS.md)
