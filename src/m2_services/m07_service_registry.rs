//! # M07: Service Registry
//!
//! Track 16 ULTRAPLATE services with their configuration: ID, port,
//! health path, dependency batch, and startup command.
//!
//! ## Layer: L2 (Services)
//! ## Module: M07
//! ## Dependencies: L1 (M02 errors)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::m1_foundation::m02_error_handling::{PvError, PvResult};

// ──────────────────────────────────────────────────────────────
// Service definition
// ──────────────────────────────────────────────────────────────

/// Definition of an ULTRAPLATE service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Unique service identifier (e.g. "pane-vortex", "synthex").
    pub id: String,
    /// Network port.
    pub port: u16,
    /// Health check HTTP path (e.g. "/health", "/api/health").
    pub health_path: String,
    /// Dependency batch (1-5). Lower batches start first.
    pub batch: u8,
    /// Whether the service is currently enabled.
    pub enabled: bool,
    /// IDs of services this service depends on.
    pub dependencies: Vec<String>,
    /// Human-readable description.
    pub description: String,
}

impl ServiceDefinition {
    /// Builder for creating a service definition.
    #[must_use]
    pub fn builder(id: impl Into<String>, port: u16) -> ServiceDefinitionBuilder {
        ServiceDefinitionBuilder {
            id: id.into(),
            port,
            health_path: "/health".into(),
            batch: 1,
            enabled: true,
            dependencies: Vec::new(),
            description: String::new(),
        }
    }
}

/// Builder for `ServiceDefinition`.
pub struct ServiceDefinitionBuilder {
    id: String,
    port: u16,
    health_path: String,
    batch: u8,
    enabled: bool,
    dependencies: Vec<String>,
    description: String,
}

impl ServiceDefinitionBuilder {
    /// Set the health check path.
    #[must_use]
    pub fn health_path(mut self, path: impl Into<String>) -> Self {
        self.health_path = path.into();
        self
    }

    /// Set the dependency batch.
    #[must_use]
    pub const fn batch(mut self, batch: u8) -> Self {
        self.batch = batch;
        self
    }

    /// Set enabled/disabled.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add a dependency.
    #[must_use]
    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Build the service definition.
    #[must_use]
    pub fn build(self) -> ServiceDefinition {
        ServiceDefinition {
            id: self.id,
            port: self.port,
            health_path: self.health_path,
            batch: self.batch,
            enabled: self.enabled,
            dependencies: self.dependencies,
            description: self.description,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Service registry
// ──────────────────────────────────────────────────────────────

/// Registry of all ULTRAPLATE services.
#[derive(Debug, Clone, Default)]
pub struct ServiceRegistry {
    /// Services indexed by ID.
    services: HashMap<String, ServiceDefinition>,
}

impl ServiceRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with all 16 ULTRAPLATE services.
    #[must_use]
    pub fn ultraplate() -> Self {
        let mut reg = Self::new();
        reg.register_ultraplate_services();
        reg
    }

    /// Register a service.
    ///
    /// # Errors
    /// Returns `PvError::Internal` if a service with the same ID already exists.
    pub fn register(&mut self, def: ServiceDefinition) -> PvResult<()> {
        if self.services.contains_key(&def.id) {
            return Err(PvError::Internal(format!(
                "service '{}' already registered",
                def.id
            )));
        }
        self.services.insert(def.id.clone(), def);
        Ok(())
    }

    /// Get a service by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ServiceDefinition> {
        self.services.get(id)
    }

    /// Get all enabled services.
    #[must_use]
    pub fn enabled_services(&self) -> Vec<&ServiceDefinition> {
        self.services.values().filter(|s| s.enabled).collect()
    }

    /// Get services in a specific batch.
    #[must_use]
    pub fn batch(&self, batch: u8) -> Vec<&ServiceDefinition> {
        self.services
            .values()
            .filter(|s| s.batch == batch && s.enabled)
            .collect()
    }

    /// Total number of registered services.
    #[must_use]
    pub fn count(&self) -> usize {
        self.services.len()
    }

    /// Number of enabled services.
    #[must_use]
    pub fn enabled_count(&self) -> usize {
        self.services.values().filter(|s| s.enabled).count()
    }

    /// Get all service IDs.
    #[must_use]
    pub fn service_ids(&self) -> Vec<&str> {
        self.services.keys().map(String::as_str).collect()
    }

    /// Get the dependency batch order (batch numbers in ascending order).
    #[must_use]
    pub fn batch_order(&self) -> Vec<u8> {
        let mut batches: Vec<u8> = self.services.values().map(|s| s.batch).collect();
        batches.sort_unstable();
        batches.dedup();
        batches
    }

    /// Register all 16 ULTRAPLATE services + 2 disabled.
    fn register_ultraplate_services(&mut self) {
        let services = vec![
            ServiceDefinition::builder("devops-engine", 8081).batch(1).description("Neural orchestration").build(),
            ServiceDefinition::builder("codesynthor-v7", 8110).batch(1).description("62 modules, 17 layers").build(),
            ServiceDefinition::builder("povm-engine", 8125).batch(1).description("Persistent OVM store").build(),
            ServiceDefinition::builder("synthex", 8090).batch(2).health_path("/api/health").depends_on("devops-engine").description("REST + WS, V3 homeostasis").build(),
            ServiceDefinition::builder("san-k7-orchestrator", 8100).batch(2).depends_on("devops-engine").description("M1-M55, 59 modules").build(),
            ServiceDefinition::builder("maintenance-engine", 8080).batch(2).health_path("/api/health").depends_on("devops-engine").description("12D tensor, PBFT, 7 layers").build(),
            ServiceDefinition::builder("architect-agent", 9001).batch(2).depends_on("devops-engine").description("Pattern library & design").build(),
            ServiceDefinition::builder("prometheus-swarm", 10001).batch(2).depends_on("devops-engine").description("CVA-NAM 40 agents, PBFT").build(),
            ServiceDefinition::builder("nais", 8101).batch(3).depends_on("synthex").description("Neural adaptive intelligence").build(),
            ServiceDefinition::builder("bash-engine", 8102).batch(3).depends_on("synthex").description("45 safety patterns").build(),
            ServiceDefinition::builder("tool-maker", 8103).batch(3).depends_on("synthex").description("v1.55.0").build(),
            ServiceDefinition::builder("claude-context-manager", 8104).batch(4).depends_on("nais").description("41 crates").build(),
            ServiceDefinition::builder("tool-library", 8105).batch(4).depends_on("nais").description("65 tools").build(),
            ServiceDefinition::builder("reasoning-memory", 8130).batch(4).depends_on("nais").description("Cross-session TSV").build(),
            ServiceDefinition::builder("vortex-memory-system", 8120).batch(5).depends_on("povm-engine").description("OVM + POVM bridge").build(),
            ServiceDefinition::builder("pane-vortex", 8132).batch(5).depends_on("povm-engine").depends_on("synthex").description("Fleet coordination, Kuramoto").build(),
            // Disabled services
            ServiceDefinition::builder("library-agent", 8083).batch(2).enabled(false).description("Not migrated").build(),
            ServiceDefinition::builder("sphere-vortex", 8120).batch(5).enabled(false).description("VMS owns port").build(),
        ];

        for svc in services {
            self.services.insert(svc.id.clone(), svc);
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ──

    #[test]
    fn new_registry_empty() {
        let reg = ServiceRegistry::new();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn default_matches_new() {
        let reg = ServiceRegistry::default();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn ultraplate_has_18_services() {
        let reg = ServiceRegistry::ultraplate();
        assert_eq!(reg.count(), 18);
    }

    #[test]
    fn ultraplate_16_enabled() {
        let reg = ServiceRegistry::ultraplate();
        assert_eq!(reg.enabled_count(), 16);
    }

    // ── Registration ──

    #[test]
    fn register_service() {
        let mut reg = ServiceRegistry::new();
        let svc = ServiceDefinition::builder("test", 9999).build();
        assert!(reg.register(svc).is_ok());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn register_duplicate_fails() {
        let mut reg = ServiceRegistry::new();
        let svc1 = ServiceDefinition::builder("test", 9999).build();
        let svc2 = ServiceDefinition::builder("test", 9998).build();
        assert!(reg.register(svc1).is_ok());
        assert!(reg.register(svc2).is_err());
    }

    // ── Lookup ──

    #[test]
    fn get_existing_service() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("pane-vortex");
        assert!(svc.is_some());
        assert_eq!(svc.unwrap().port, 8132);
    }

    #[test]
    fn get_missing_service() {
        let reg = ServiceRegistry::ultraplate();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn get_disabled_service() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("library-agent");
        assert!(svc.is_some());
        assert!(!svc.unwrap().enabled);
    }

    // ── Batch queries ──

    #[test]
    fn batch_1_has_services() {
        let reg = ServiceRegistry::ultraplate();
        let b1 = reg.batch(1);
        assert!(b1.len() >= 3);
    }

    #[test]
    fn batch_5_has_pane_vortex() {
        let reg = ServiceRegistry::ultraplate();
        let b5 = reg.batch(5);
        assert!(b5.iter().any(|s| s.id == "pane-vortex"));
    }

    #[test]
    fn batch_order_ascending() {
        let reg = ServiceRegistry::ultraplate();
        let order = reg.batch_order();
        for i in 1..order.len() {
            assert!(order[i] >= order[i - 1]);
        }
    }

    #[test]
    fn batch_order_has_5_batches() {
        let reg = ServiceRegistry::ultraplate();
        let order = reg.batch_order();
        assert_eq!(order.len(), 5);
    }

    // ── Enabled services ──

    #[test]
    fn enabled_services_excludes_disabled() {
        let reg = ServiceRegistry::ultraplate();
        let enabled = reg.enabled_services();
        assert!(enabled.iter().all(|s| s.enabled));
    }

    #[test]
    fn enabled_services_count() {
        let reg = ServiceRegistry::ultraplate();
        assert_eq!(reg.enabled_services().len(), 16);
    }

    // ── Service IDs ──

    #[test]
    fn service_ids_not_empty() {
        let reg = ServiceRegistry::ultraplate();
        assert!(!reg.service_ids().is_empty());
    }

    #[test]
    fn service_ids_contains_pane_vortex() {
        let reg = ServiceRegistry::ultraplate();
        assert!(reg.service_ids().contains(&"pane-vortex"));
    }

    // ── Service definition builder ──

    #[test]
    fn builder_default_health_path() {
        let svc = ServiceDefinition::builder("test", 9999).build();
        assert_eq!(svc.health_path, "/health");
    }

    #[test]
    fn builder_custom_health_path() {
        let svc = ServiceDefinition::builder("test", 9999)
            .health_path("/api/health")
            .build();
        assert_eq!(svc.health_path, "/api/health");
    }

    #[test]
    fn builder_batch() {
        let svc = ServiceDefinition::builder("test", 9999).batch(3).build();
        assert_eq!(svc.batch, 3);
    }

    #[test]
    fn builder_disabled() {
        let svc = ServiceDefinition::builder("test", 9999)
            .enabled(false)
            .build();
        assert!(!svc.enabled);
    }

    #[test]
    fn builder_dependencies() {
        let svc = ServiceDefinition::builder("test", 9999)
            .depends_on("dep1")
            .depends_on("dep2")
            .build();
        assert_eq!(svc.dependencies.len(), 2);
    }

    #[test]
    fn builder_description() {
        let svc = ServiceDefinition::builder("test", 9999)
            .description("Test service")
            .build();
        assert_eq!(svc.description, "Test service");
    }

    // ── ULTRAPLATE specific ──

    #[test]
    fn synthex_has_api_health_path() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("synthex").unwrap();
        assert_eq!(svc.health_path, "/api/health");
    }

    #[test]
    fn me_has_api_health_path() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("maintenance-engine").unwrap();
        assert_eq!(svc.health_path, "/api/health");
    }

    #[test]
    fn pane_vortex_port_8132() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("pane-vortex").unwrap();
        assert_eq!(svc.port, 8132);
    }

    #[test]
    fn pane_vortex_batch_5() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("pane-vortex").unwrap();
        assert_eq!(svc.batch, 5);
    }

    #[test]
    fn pane_vortex_depends_on_povm() {
        let reg = ServiceRegistry::ultraplate();
        let svc = reg.get("pane-vortex").unwrap();
        assert!(svc.dependencies.contains(&"povm-engine".to_string()));
    }

    // ── Serde roundtrip ──

    #[test]
    fn service_definition_serde_roundtrip() {
        let svc = ServiceDefinition::builder("test", 9999)
            .health_path("/api/health")
            .batch(3)
            .depends_on("dep1")
            .build();
        let json = serde_json::to_string(&svc).unwrap();
        let back: ServiceDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "test");
        assert_eq!(back.port, 9999);
    }

    // ── Edge cases ──

    #[test]
    fn empty_batch_returns_empty() {
        let reg = ServiceRegistry::ultraplate();
        let b99 = reg.batch(99);
        assert!(b99.is_empty());
    }
}
