//! # M07: Service Registry
//! Track 16 ULTRAPLATE services: id, port, health path, batch, dependencies.
//! ME v2 pattern: `ServiceDefinition` builder, `ServiceDiscovery` trait (14 methods).
//! ## Layer: L2 | Module: M07 | Dependencies: L1
//! ## Design Constraints: C2 (trait &self), C7 (owned returns), C13 (builder)
//! ## Related: [[ULTRAPLATE Master Index]], [[ULTRAPLATE Developer Environment]]
