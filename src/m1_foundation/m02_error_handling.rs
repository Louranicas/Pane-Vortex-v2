//! # M02: Error Handling
//!
//! Unified error enum `PvError` with `ErrorClassifier` trait for intelligent
//! retry, severity classification, and structured error codes.
//!
//! ## Layer: L1 (Foundation)
//! ## Module: M02
//! ## Dependencies: None
//!
//! ## Design Constraints
//! - C4: All errors handled — no panics, no unwrap
//! - C7: thiserror for structured library errors
//! - C8: Errors classified by retryability, severity, and category
//!
//! ## Error Categories
//! - Config (1000-1099): Configuration validation failures
//! - Validation (1100-1199): Input validation failures  
//! - Field (1200-1299): Kuramoto field computation errors
//! - Bridge (1300-1399): External service communication failures
//! - Bus (1400-1499): IPC bus protocol errors
//! - Persistence (1500-1599): SQLite/WAL errors
//! - Governance (1600-1699): Proposal/voting errors
//!
//! ## Related Documentation
//! - [Error Patterns](../../ai_specs/patterns/ERROR_PATTERNS.md)
//! - [Layer Specification](../../ai_specs/layers/L1_FOUNDATION_SPEC.md)
