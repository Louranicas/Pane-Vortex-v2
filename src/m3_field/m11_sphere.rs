//! # M11: `PaneSphere` Oscillator
//!
//! The core oscillator entity. Each Claude Code instance registers as a sphere.
//! `PaneSphere` has 33 fields (god object from v1 — to be decomposed in v2).
//!
//! ## Layer: L3 (Field)
//! ## Module: M11
//! ## Dependencies: L1 (M01 types, M02 errors, M04 constants, M05 traits)
//!
//! ## Key Types
//! - `PaneSphere`: 33-field oscillator with phase, frequency, memory, coupling state
//! - `SphereStatus`: Working | Idle | Blocked | Complete
//! - `MaturityLevel`: Newcomer (<50 steps) | Established | Senior (>1000 steps)
//!
//! ## Design Constraints
//! - C2: Interior mutability via `parking_lot::RwLock`<InnerSphereState>
//! - C3: Phase always wrapped via `rem_euclid(TAU)` after update
//! - C8: Consent gate checked before phase injection
//! - C11: NaN guard on all phase/frequency updates
//! - C12: Memory capped at `MEMORY_MAX_COUNT` (500), pruned amortised at +50
//!
//! ## NA Features (35 total from v1)
//! NA-1 semantic phase, NA-2 frequency discovery, NA-8 auto-status,
//! NA-12 self-model, NA-14 auto-receptivity, NA-15 self-frequency,
//! NA-16 voluntary decoupling, NA-33 coupling preferences, NA-34 opt-out flags
//!
//! ## Risk: God Object
//! Session 039 identified `PaneSphere` as the #1 structural risk (33 fields).
//! V2 should decompose into: `SphereCore` (phase/freq), `SphereMemory`, `SphereCoupling`,
//! `SphereConsent`, `SphereSelfModel`. Keep flat serialization for wire compat.
//!
//! ## Related Documentation
//! - [Field Layer Spec](../../ai_specs/layers/L3_FIELD_SPEC.md)
//! - [[Session 039 — Reflections and Learnings]] — god object risk analysis
//! - [[Vortex Sphere Brain-Body Architecture]] — theoretical foundation
