//! # M31: Conductor
//! PI controller targeting `r_target` (dynamic, fleet-negotiated). Breathing blend 30%.
//! Divergence cooldown: suppress coherence boost for 3 ticks when sphere requests divergence.
//! ## Layer: L7 | Module: M31 | Dependencies: L1, L3 (M12 field state), L5 (M19 learning)
//! ## NA: NA-P-5 (conductor cooldown deployed), NA-P-3 (fleet `r_target` deployed)
//! ## Risk: 39 unwraps in v1 conductor.rs — v2 eliminates ALL (C4)
//! ## Related: [L7 Spec](../../ai_specs/layers/L7_COORDINATION_SPEC.md)
