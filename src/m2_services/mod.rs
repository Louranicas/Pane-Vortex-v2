//! # Layer 2: Services
//!
//! Service registry, health monitoring, and lifecycle management.
//! Depends on L1 (Foundation).
//!
//! ## Design Constraints: C1 C2 C6 C7 C9 C13 C14
//! See `ai_specs/DESIGN_CONSTRAINTS.md` for full definitions.
//!
//! ## Modules
//!
//! | Module | LOC Target | Purpose |
//! |--------|-----------|---------|
//! | `m07_service_registry` | ~200 | Track 16 ULTRAPLATE services: id, port, health path, batch |
//! | `m08_health_monitor` | ~250 | Async health polling, staleness detection, circuit breaker |
//! | `m09_lifecycle` | ~200 | Service start/stop/restart, PID tracking, devenv integration |
//! | `m10_api_server` | ~400 | Axum HTTP server, CORS, routes, body limits |

pub mod m07_service_registry;
pub mod m08_health_monitor;
pub mod m09_lifecycle;

#[cfg(feature = "api")]
pub mod m10_api_server;
