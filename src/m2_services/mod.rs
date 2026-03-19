//! # Layer 2: Services
//!
//! Service registry, health monitoring, lifecycle management, and API server.
//! Depends on L1 (Foundation).

pub mod m07_service_registry;
pub mod m08_health_monitor;
pub mod m09_lifecycle;

#[cfg(feature = "api")]
pub mod m10_api_server;

// ── Ergonomic re-exports ──

pub use m07_service_registry::{ServiceDefinition, ServiceDefinitionBuilder, ServiceRegistry};
pub use m08_health_monitor::{CircuitState, HealthMonitor, ServiceHealth};
pub use m09_lifecycle::{LifecycleManager, ServiceLifecycle, ServiceState};

#[cfg(feature = "api")]
pub use m10_api_server::AppContext;
