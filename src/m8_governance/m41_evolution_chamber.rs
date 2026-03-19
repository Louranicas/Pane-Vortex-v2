//! # M41: Evolution Chamber
//! Pattern detection, anomaly scoring, emergence events. Feature-gated: evolution.
//! 8 endpoints: analytics/patterns, anomalies, baseline, observe, summary + evolution/emergence, regime, status.
//! ## Layer: L8 | Module: M41 | Dependencies: L1, L3
//! ## Feature Gate: #[cfg(feature = "evolution")]
//! ## NA: NA-P-8 (observation opt-out deployed in V2)
//! ## Alert: Evolution endpoints return 404 (ALERT-8) — may need route verification
