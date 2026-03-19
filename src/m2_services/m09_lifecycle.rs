//! # M09: Service Lifecycle
//! Start/stop/restart, PID tracking, devenv integration. Graceful shutdown with timeout.
//! ME v2 pattern: `LifecycleOps` trait (13 methods), FSM transitions with backoff.
//! ## Layer: L2 | Module: M09 | Dependencies: L1, M07, M08
//! ## Design Constraints: C6, C8 (timeout on all operations)
