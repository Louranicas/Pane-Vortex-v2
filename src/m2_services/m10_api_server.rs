//! # M10: API Server
//! Axum 0.8 HTTP server. CORS enabled. 65KB body limit. 60+ routes across all layers.
//! Path syntax: /{param} (not /:param — Axum 0.8 change).
//! ## Layer: L2 | Module: M10 | Dependencies: L1, all other layers (route registration)
//! ## Feature Gate: #[cfg(feature = "api")]
//! ## Design Constraints: C7 (JSON responses always owned), E03 (`IntoResponse` for errors)
//! ## Related: [API Spec](../../ai_specs/API_SPEC.md)
