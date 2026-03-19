//! # M08: Health Monitor
//! Async health polling for all 16 services. Staleness detection. Circuit breaker per service.
//! ME v2 pattern: `HealthMonitoring` trait (11 methods), tiered polling intervals.
//! ## Layer: L2 | Module: M08 | Dependencies: L1, M07
//! ## Design Constraints: C2, C6 (signal emission after lock), C14 (fire-and-forget polling)
