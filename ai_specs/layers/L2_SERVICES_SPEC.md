# Layer 2: Services Specification

> Detailed spec for m07-m10: service registry, health monitor, lifecycle, API server.
> The service infrastructure — how pane-vortex discovers, monitors, and communicates with
> the 16 ULTRAPLATE services and exposes its own HTTP interface.
> Source: `src/m2_services/` | Plan: `MASTERPLAN.md` Phase 1
> v1 Source: `api.rs` (1,800 LOC, 32 routes), `main.rs` (devenv integration)

## Overview

Layer 2 provides the service infrastructure: a registry of all ULTRAPLATE services,
async health monitoring with circuit breakers, process lifecycle management through devenv,
and the axum HTTP server that exposes pane-vortex's 60+ endpoints. L2 depends only on L1.
All other layers depend on L2's API server for HTTP route composition.

### Design Constraints

| ID | Constraint | Application in L2 |
|----|-----------|-------------------|
| C1 | No upward imports | L2 imports only from L1; never from L3-L8 (except m10 route registration) |
| C2 | Trait methods `&self` | `ServiceDiscovery` and `HealthMonitoring` traits use interior mutability |
| C6 | Events after lock release | Health status changes emit events after dropping `RwLock` |
| C7 | Owned returns through `RwLock` | `get_service()` returns `ServiceInfo` (cloned), never a reference |
| C9 | 50+ tests per layer | Target: 35 unit + 15 integration = 50 tests |
| C13 | Builder for >2 params | `ServiceDefinition::builder()` for service construction |
| C14 | Fire-and-forget polling | Health checks use `tokio::spawn`, never block the tick loop |

### Dependency Note: m10_api_server

Module m10 is the exception to strict upward-only imports. It registers route handlers
from all layers (L3-L8) because it assembles the complete HTTP router. However, m10
does not import implementation details — it receives `SharedState`, `SharedBusState`,
and other shared handles via axum `State` extractors at startup. The actual route handler
logic lives in each layer's module; m10 only composes them into a `Router`.

## 1. m07_service_registry (~200 LOC)

### 1.1 ServiceDefinition

```rust
/// Complete definition of an ULTRAPLATE service for registry and lifecycle management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Unique service identifier (e.g., "pane-vortex", "synthex")
    pub id: String,
    /// Display name for dashboards
    pub display_name: String,
    /// TCP port the service listens on
    pub port: u16,
    /// HTTP path for health check (e.g., "/health", "/api/health")
    pub health_path: String,
    /// Startup batch number (1-5). Services within a batch start in parallel.
    pub batch: u8,
    /// Service IDs that must be healthy before this one starts.
    pub depends_on: Vec<String>,
    /// Binary path (absolute, e.g., "~/.local/bin/pane-vortex")
    pub binary_path: Option<String>,
    /// Whether the service is currently enabled in devenv config.
    pub enabled: bool,
    /// Feature flags this service provides (e.g., ["api", "persistence"])
    pub features: Vec<String>,
}
```

### 1.2 ServiceDefinition Builder (C13)

```rust
impl ServiceDefinition {
    #[must_use]
    pub fn builder(id: impl Into<String>) -> ServiceDefinitionBuilder {
        ServiceDefinitionBuilder {
            id: id.into(),
            display_name: String::new(),
            port: 0,
            health_path: "/health".into(),
            batch: 1,
            depends_on: vec![],
            binary_path: None,
            enabled: true,
            features: vec![],
        }
    }
}

#[must_use]
pub struct ServiceDefinitionBuilder {
    id: String,
    display_name: String,
    port: u16,
    health_path: String,
    batch: u8,
    depends_on: Vec<String>,
    binary_path: Option<String>,
    enabled: bool,
    features: Vec<String>,
}

impl ServiceDefinitionBuilder {
    pub fn display_name(mut self, name: impl Into<String>) -> Self { self.display_name = name.into(); self }
    pub fn port(mut self, port: u16) -> Self { self.port = port; self }
    pub fn health_path(mut self, path: impl Into<String>) -> Self { self.health_path = path.into(); self }
    pub fn batch(mut self, batch: u8) -> Self { self.batch = batch; self }
    pub fn depends_on(mut self, deps: Vec<String>) -> Self { self.depends_on = deps; self }
    pub fn binary_path(mut self, path: impl Into<String>) -> Self { self.binary_path = Some(path.into()); self }
    pub fn enabled(mut self, enabled: bool) -> Self { self.enabled = enabled; self }
    pub fn features(mut self, features: Vec<String>) -> Self { self.features = features; self }
    pub fn build(self) -> PvResult<ServiceDefinition> {
        if self.port == 0 { return Err(PvError::Config("port must be non-zero".into())); }
        if self.id.is_empty() { return Err(PvError::Config("id must not be empty".into())); }
        Ok(ServiceDefinition {
            id: self.id,
            display_name: self.display_name,
            port: self.port,
            health_path: self.health_path,
            batch: self.batch,
            depends_on: self.depends_on,
            binary_path: self.binary_path,
            enabled: self.enabled,
            features: self.features,
        })
    }
}
```

### 1.3 ServiceInfo (Runtime State)

```rust
/// Runtime state of a registered service — combines static definition with live data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub definition: ServiceDefinition,
    pub health: HealthStatus,
    pub pid: Option<u32>,
    pub started_at: Option<f64>,       // epoch seconds
    pub last_health_check: Option<f64>,
    pub consecutive_failures: u32,
}
```

### 1.4 ServiceDiscovery Trait (ME v2 Pattern)

```rust
/// Discovery interface for the 16-service ULTRAPLATE fleet.
/// ME v2 gold standard: 14 methods covering lookup, filtering, ordering, and batch queries.
pub trait ServiceDiscovery {
    /// Look up a service by ID. Returns cloned data (C7).
    fn get_service(&self, id: &str) -> Option<ServiceInfo>;
    /// List all registered services.
    fn list_services(&self) -> Vec<ServiceInfo>;
    /// List services in a specific batch.
    fn list_batch(&self, batch: u8) -> Vec<ServiceInfo>;
    /// List services that depend on a given service.
    fn dependents_of(&self, id: &str) -> Vec<ServiceInfo>;
    /// List services that a given service depends on.
    fn dependencies_of(&self, id: &str) -> Vec<ServiceInfo>;
    /// Filter services by health status.
    fn filter_by_health(&self, status: HealthStatus) -> Vec<ServiceInfo>;
    /// Get services sorted by batch number (start order).
    fn start_order(&self) -> Vec<ServiceInfo>;
    /// Get services sorted by reverse batch (stop order).
    fn stop_order(&self) -> Vec<ServiceInfo>;
    /// Check if all dependencies of a service are healthy.
    fn deps_satisfied(&self, id: &str) -> bool;
    /// Count services by health status.
    fn health_summary(&self) -> HashMap<HealthStatus, usize>;
    /// Get the port for a service (shorthand for get_service + port).
    fn port_for(&self, id: &str) -> Option<u16>;
    /// Get all enabled service IDs.
    fn enabled_ids(&self) -> Vec<String>;
    /// Get service by port number (reverse lookup).
    fn service_at_port(&self, port: u16) -> Option<ServiceInfo>;
    /// Total registered service count.
    fn count(&self) -> usize;
}
```

### 1.5 ServiceRegistry Implementation

```rust
#[derive(Debug)]
pub struct ServiceRegistry {
    state: parking_lot::RwLock<HashMap<String, ServiceInfo>>,
}

impl ServiceRegistry {
    /// Create a registry pre-populated with the 16 ULTRAPLATE service definitions.
    pub fn with_ultraplate_defaults() -> Self;

    /// Register a new service (used by devenv integration).
    pub fn register(&self, definition: ServiceDefinition) -> PvResult<()>;

    /// Deregister a service.
    pub fn deregister(&self, id: &str) -> PvResult<()>;

    /// Update health status for a service (called by m08).
    pub fn update_health(&self, id: &str, health: HealthStatus);

    /// Update PID for a service (called by m09).
    pub fn update_pid(&self, id: &str, pid: Option<u32>);
}

impl ServiceDiscovery for ServiceRegistry { /* delegate to RwLock<HashMap> */ }
```

### 1.6 ULTRAPLATE Service Table

The registry is pre-populated with these 16 active services on construction:

| Service | ID | Port | Health Path | Batch | Depends On |
|---------|----|------|-------------|-------|------------|
| DevOps Engine | `devops-engine` | 8081 | `/health` | 1 | (none) |
| CodeSynthor V7 | `codesynthor-v7` | 8110 | `/health` | 1 | (none) |
| POVM Engine | `povm-engine` | 8125 | `/health` | 1 | (none) |
| SYNTHEX | `synthex` | 8090 | `/api/health` | 2 | devops-engine |
| SAN-K7 | `san-k7-orchestrator` | 8100 | `/health` | 2 | devops-engine |
| Maintenance Engine | `maintenance-engine` | 8080 | `/api/health` | 2 | devops-engine |
| Architect Agent | `architect-agent` | 9001 | `/health` | 2 | devops-engine |
| Prometheus Swarm | `prometheus-swarm` | 10001 | `/health` | 2 | devops-engine |
| NAIS | `nais` | 8101 | `/health` | 3 | synthex, san-k7 |
| Bash Engine | `bash-engine` | 8102 | `/health` | 3 | synthex, san-k7 |
| Tool Maker | `tool-maker` | 8103 | `/health` | 3 | synthex, san-k7 |
| Context Manager | `claude-context-manager` | 8104 | `/health` | 4 | nais, bash-engine |
| Tool Library | `tool-library` | 8105 | `/health` | 4 | nais, bash-engine |
| Reasoning Memory | `reasoning-memory` | 8130 | `/health` | 4 | nais, bash-engine |
| Vortex Memory System | `vortex-memory-system` | 8120 | `/health` | 5 | povm-engine |
| Pane-Vortex | `pane-vortex` | 8132 | `/health` | 5 | povm-engine, synthex |

Two disabled services (not in registry): `library-agent` (8083), `sphere-vortex` (8120, port conflict with VMS).

### 1.7 Tests (5 target)

- `with_ultraplate_defaults()`: populates exactly 16 services
- Builder: missing port returns `PvError::Config`
- `get_service()`: returns `None` for unregistered ID
- `deps_satisfied()`: returns false when dependency is unhealthy
- `start_order()`: batch 1 before batch 2, batch 2 before batch 3

## 2. m08_health_monitor (~250 LOC)

### 2.1 HealthStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service responded 200 within timeout
    Healthy,
    /// Service responded non-200 or timed out once
    Degraded,
    /// Circuit breaker open — polling paused
    Unhealthy,
    /// Service has never been checked
    Unknown,
}
```

### 2.2 Circuit Breaker FSM

Each service has an independent circuit breaker that controls polling frequency:

```
         success
  +--- Closed ←──────────────────+
  |     |                        |
  |  failure ×3                  | success
  |     ↓                        |
  |   Open ──── cooldown(30s) → HalfOpen
  |     ↑                        |
  |     +── failure ─────────────+
  +──── (normal polling) ────────+
```

States:
- **Closed**: Normal operation. Polls at tier interval. Failure counter incremented on non-200.
  Transitions to Open after 3 consecutive failures.
- **Open**: Polling paused for `CIRCUIT_BREAKER_COOLDOWN` (30s, 6 ticks). No HTTP requests sent.
  Transitions to HalfOpen after cooldown.
- **HalfOpen**: Single probe sent. Success -> Closed (counter reset). Failure -> Open (cooldown restarts).

```rust
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub last_transition_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn record_success(&mut self, tick: u64);
    pub fn record_failure(&mut self, tick: u64);
    pub fn should_poll(&self, tick: u64) -> bool;
}
```

### 2.3 Tiered Polling Intervals

Services are polled at different frequencies based on their role in the PV ecosystem:

| Tier | Services | Poll Interval | Rationale |
|------|----------|---------------|-----------|
| Critical (5s) | synthex, povm-engine, san-k7 | Every tick | Direct bridge dependencies |
| Standard (30s) | maintenance-engine, reasoning-memory, vms | Every 6 ticks | Bridge targets, slower cadence |
| Background (60s) | All others | Every 12 ticks | No direct bridge, status-only |

```rust
pub fn poll_tier(service_id: &str) -> PollTier {
    match service_id {
        "synthex" | "povm-engine" | "san-k7-orchestrator" => PollTier::Critical,
        "maintenance-engine" | "reasoning-memory" | "vortex-memory-system" => PollTier::Standard,
        _ => PollTier::Background,
    }
}
```

### 2.4 Staleness Detection

A health reading is considered stale when it exceeds 2x the poll interval:

```rust
pub fn is_stale(&self, service_id: &str, now: f64) -> bool {
    let info = self.get_service(service_id)?;
    let max_age = match poll_tier(service_id) {
        PollTier::Critical => 10.0,   // 2 × 5s
        PollTier::Standard => 60.0,   // 2 × 30s
        PollTier::Background => 120.0, // 2 × 60s
    };
    info.last_health_check.map_or(true, |t| now - t > max_age)
}
```

### 2.5 Async Health Check

```rust
/// Perform a single health check against a service. Runs as tokio::spawn (C14).
/// Timeout: 3 seconds. Returns HTTP status code or error.
pub async fn check_health(service: &ServiceDefinition) -> HealthCheckResult {
    let url = format!("http://127.0.0.1:{}{}", service.port, service.health_path);
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        raw_http_get(&format!("127.0.0.1:{}", service.port), &service.health_path),
    ).await;

    match result {
        Ok(Ok(status)) if status == 200 => HealthCheckResult::Healthy,
        Ok(Ok(status)) => HealthCheckResult::Degraded(status),
        Ok(Err(e)) => HealthCheckResult::Failed(e.to_string()),
        Err(_) => HealthCheckResult::Timeout,
    }
}
```

### 2.6 Health Monitor Trait

```rust
/// ME v2 pattern: centralized health monitoring across the 16-service fleet.
pub trait HealthMonitoring {
    /// Run one poll cycle for all services due for a check at this tick.
    async fn poll_cycle(&self, tick: u64, registry: &ServiceRegistry);
    /// Get health status for a service.
    fn health_of(&self, id: &str) -> HealthStatus;
    /// Get all unhealthy services.
    fn unhealthy_services(&self) -> Vec<String>;
    /// Get circuit breaker state for a service.
    fn circuit_state(&self, id: &str) -> CircuitState;
    /// Force a health check bypass (for HalfOpen probes).
    async fn probe(&self, id: &str, registry: &ServiceRegistry);
    /// Check if all critical-tier services are healthy.
    fn critical_services_healthy(&self) -> bool;
    /// Time since last successful check for a service (seconds).
    fn time_since_healthy(&self, id: &str) -> Option<f64>;
}
```

### 2.7 Tests (10 target)

- Circuit breaker: 3 failures -> Open
- Circuit breaker: cooldown expires -> HalfOpen
- Circuit breaker: HalfOpen + success -> Closed (counter reset)
- Circuit breaker: HalfOpen + failure -> Open (cooldown resets)
- Tiered polling: critical service polled every tick
- Tiered polling: background service skipped for 12 ticks
- Staleness: service with no check is stale
- Staleness: fresh check within tier is not stale
- `unhealthy_services()`: returns empty when all healthy
- `critical_services_healthy()`: false when synthex is unhealthy

## 3. m09_lifecycle (~200 LOC)

### 3.1 Lifecycle Operations

```rust
/// ME v2 pattern: lifecycle operations for ULTRAPLATE services.
pub trait LifecycleOps {
    /// Start a service by ID. Checks dependencies first.
    async fn start_service(&self, id: &str, registry: &ServiceRegistry) -> PvResult<u32>;
    /// Stop a service by ID. Sends SIGTERM, waits for graceful shutdown.
    async fn stop_service(&self, id: &str) -> PvResult<()>;
    /// Restart: stop + start with dependency verification.
    async fn restart_service(&self, id: &str, registry: &ServiceRegistry) -> PvResult<u32>;
    /// Graceful shutdown of all services in reverse batch order.
    async fn shutdown_all(&self, registry: &ServiceRegistry) -> PvResult<()>;
    /// Get PID of a running service from devenv PID directory.
    fn get_pid(&self, id: &str) -> Option<u32>;
    /// Check if a service process is alive (kill -0).
    fn is_alive(&self, id: &str) -> bool;
}
```

### 3.2 Graceful Shutdown Protocol

The shutdown sequence respects batch ordering in reverse (5 -> 1) to ensure
dependent services stop before their dependencies:

```
1. Collect services by batch, reverse sort
2. For each batch (5, 4, 3, 2, 1):
   a. Send SIGTERM to all services in batch (parallel)
   b. Wait up to SHUTDOWN_TIMEOUT (10s) for process exit
   c. If still alive after timeout, send SIGKILL
   d. Clean up PID files
3. Verify all ports are free (ss -tlnp check)
```

```rust
pub async fn graceful_shutdown(pid: u32) -> PvResult<()> {
    // Send SIGTERM
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid as i32),
        nix::sys::signal::Signal::SIGTERM,
    ).map_err(|e| PvError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

    // Wait for process exit with timeout
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if !process_alive(pid) { return Ok(()); }
        if tokio::time::Instant::now() >= deadline { break; }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Force kill if still alive
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid as i32),
        nix::sys::signal::Signal::SIGKILL,
    ).ok();
    Ok(())
}
```

### 3.3 Devenv Integration

PV integrates with devenv for process lifecycle. PID files live at
`~/.local/share/devenv/pids/{service-id}.pid`.

```rust
pub fn read_devenv_pid(id: &str) -> Option<u32> {
    let path = dirs::data_local_dir()?
        .join("devenv/pids")
        .join(format!("{id}.pid"));
    let contents = std::fs::read_to_string(path).ok()?;
    contents.trim().parse().ok()
}

pub fn write_devenv_pid(id: &str, pid: u32) -> PvResult<()> {
    let path = dirs::data_local_dir()
        .ok_or_else(|| PvError::Config("no data dir".into()))?
        .join("devenv/pids")
        .join(format!("{id}.pid"));
    std::fs::write(path, pid.to_string())
        .map_err(PvError::Io)
}
```

### 3.4 Known Issue: devenv stop Does Not Kill Processes

From v1 operational experience: `devenv stop` removes PID files but may leave processes
alive on ports. The lifecycle module includes a port-based kill fallback:

```rust
/// Kill any process occupying a port. Used when PID tracking fails.
pub async fn kill_port_occupant(port: u16) -> PvResult<bool> {
    let output = tokio::process::Command::new("ss")
        .args(["-tlnp", &format!("sport = :{port}")])
        .output().await?;
    // Parse PID from output, send SIGTERM
}
```

### 3.5 Tests (5 target)

- `read_devenv_pid()`: reads existing PID file
- `read_devenv_pid()`: returns None for missing file
- `is_alive()`: false for non-existent PID
- Shutdown: reverse batch ordering (5 before 1)
- Port kill fallback: frees port when PID file missing

## 4. m10_api_server (~400 LOC)

### 4.1 Server Construction

```rust
/// Build the complete axum router with all layer routes composed.
/// Feature-gated: governance routes require `#[cfg(feature = "governance")]`.
pub fn build_router(
    app_state: SharedState,
    bus_state: SharedBusState,
    dbs: Databases,
    thermal: SharedThermalState,
    nexus: SharedNexusState,
    me: SharedMeState,
    executor: SharedExecutorRegistry,
    registry: Arc<ServiceRegistry>,
) -> Router {
    let base = Router::new()
        // Health & Status (3)
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/version", get(version_handler))
        // Field (12)
        .route("/field", get(field_handler))
        .route("/field/r", get(r_handler))
        .route("/field/spectrum", get(spectrum_handler))
        .route("/field/chimera", get(chimera_handler))
        .route("/field/tunnels", get(tunnels_handler))
        .route("/field/decision", get(decision_handler))
        .route("/field/decisions", get(decision_history_handler))
        .route("/field/k", get(k_handler))
        .route("/field/warmup", get(warmup_handler))
        .route("/field/history", get(field_history_handler))
        .route("/field/ghosts", get(ghosts_handler))
        .route("/field/snapshot", get(snapshot_handler))
        // Spheres (20+)
        .route("/spheres", get(list_spheres_handler))
        .route("/sphere/{pane_id}", get(get_sphere_handler))
        .route("/sphere/{pane_id}/register", post(register_handler))
        .route("/sphere/{pane_id}/deregister", post(deregister_handler))
        .route("/sphere/{pane_id}/memory", post(memory_handler))
        .route("/sphere/{pane_id}/phase", post(phase_handler))
        .route("/sphere/{pane_id}/status", post(status_update_handler))
        .route("/sphere/{pane_id}/steer", post(steer_handler))
        .route("/sphere/{pane_id}/recall", get(recall_handler))
        .route("/sphere/{pane_id}/neighbors", get(neighbors_handler))
        .route("/sphere/{pane_id}/decouple", post(decouple_handler))
        .route("/sphere/{pane_id}/recouple", post(recouple_handler))
        .route("/sphere/{pane_id}/preferences", post(preferences_handler))
        .route("/sphere/{pane_id}/request-divergence", post(divergence_handler))
        .route("/sphere/{pane_id}/narrative", get(narrative_handler))
        .route("/sphere/{pane_id}/associations", get(associations_handler))
        .route("/sphere/{pane_id}/inbox", get(inbox_handler))
        .route("/sphere/{pane_id}/inbox/send", post(inbox_send_handler))
        .route("/sphere/{pane_id}/inbox/ack", post(inbox_ack_handler))
        .route("/sphere/{pane_id}/consent", post(consent_handler))
        .route("/sphere/{pane_id}/data-manifest", get(data_manifest_handler))
        .route("/sphere/{pane_id}/forget", post(forget_handler))
        .route("/sphere/{pane_id}/heartbeat", post(heartbeat_handler))
        .route("/sphere/{pane_id}/accept-ghost", post(accept_ghost_handler))
        // Bus (5)
        .route("/bus/tasks", get(bus_tasks_handler))
        .route("/bus/events", get(bus_events_handler))
        .route("/bus/suggestions", get(bus_suggestions_handler))
        .route("/bus/info", get(bus_info_handler))
        .route("/bus/submit", post(bus_submit_handler))
        // Bridges (3)
        .route("/bridges/health", get(bridges_health_handler))
        .route("/nexus/metrics", get(nexus_metrics_handler))
        .route("/synthex/thermal", get(synthex_thermal_handler))
        // Integration (2)
        .route("/integration/matrix", get(integration_matrix_handler))
        .route("/coupling/matrix", get(coupling_matrix_handler));

    // Feature-gated governance routes (V3.4)
    #[cfg(feature = "governance")]
    let base = base
        .route("/field/propose", post(propose_handler))
        .route("/field/proposals", get(proposals_handler))
        .route("/sphere/{pane_id}/vote/{proposal_id}", post(vote_handler))
        .route("/governance/consent_budget", post(consent_budget_handler));

    // Feature-gated evolution routes
    #[cfg(feature = "evolution")]
    let base = base
        .route("/analytics/observe", post(analytics_observe_handler))
        .route("/analytics/patterns", get(analytics_patterns_handler))
        .route("/analytics/anomalies", get(analytics_anomalies_handler))
        .route("/analytics/baseline", get(analytics_baseline_handler))
        .route("/analytics/summary", get(analytics_summary_handler))
        .route("/evolution/emergence", get(evolution_emergence_handler))
        .route("/evolution/regime", get(evolution_regime_handler))
        .route("/evolution/status", get(evolution_status_handler));

    base
        .layer(DefaultBodyLimit::max(65_536))  // 65KB
        .layer(CorsLayer::permissive())
        .with_state(AppStateBundle { app_state, bus_state, dbs, thermal, nexus, me, executor, registry })
}
```

### 4.2 State Bundle

Route handlers need access to multiple shared state objects. Rather than passing each
via separate `Extension`, V2 bundles them:

```rust
#[derive(Clone)]
pub struct AppStateBundle {
    pub app_state: SharedState,
    pub bus_state: SharedBusState,
    pub dbs: Databases,
    pub thermal: SharedThermalState,
    pub nexus: SharedNexusState,
    pub me: SharedMeState,
    pub executor: SharedExecutorRegistry,
    pub registry: Arc<ServiceRegistry>,
}
```

### 4.3 Server Binding

```rust
/// Start the HTTP server. Binds to 127.0.0.1 by default (loopback only).
/// Override with BIND_ADDR=0.0.0.0 for external access.
pub async fn serve(router: Router, port: u16) -> PvResult<()> {
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".into());
    let addr = format!("{bind_addr}:{port}");

    // Retry binding with exponential backoff (devenv may still be releasing port)
    let listener = bind_with_retry(&addr, BIND_MAX_RETRIES, BIND_INITIAL_DELAY_MS).await?;

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| PvError::Io(e.into()))
}
```

### 4.4 Path Parameter Syntax

Axum 0.8 changed path parameter syntax from `:param` to `{param}`:

```rust
// v1 (axum 0.7): .route("/sphere/:pane_id", ...)
// v2 (axum 0.8): .route("/sphere/{pane_id}", ...)
```

All route definitions use `{param}` syntax. Extractors use `Path(pane_id): Path<String>`.

### 4.5 CORS Configuration

Permissive CORS for local development. Production may restrict origins:

```rust
CorsLayer::permissive()
// Equivalent to: Access-Control-Allow-Origin: *
//                Access-Control-Allow-Methods: *
//                Access-Control-Allow-Headers: *
```

### 4.6 Body Limit

All POST endpoints enforce a 65KB body limit (`DefaultBodyLimit::max(65_536)`).
This prevents memory exhaustion from oversized payloads. The limit covers:
- Memory summaries (typically <1KB)
- Task payloads (typically <10KB)
- Consent declarations (<500B)

### 4.7 Tests (15 target)

- Router builds without panic (smoke test)
- GET /health returns 200 with valid JSON
- GET /field returns FieldState JSON
- POST /sphere/{id}/register with valid body returns 200
- POST /sphere/{id}/register with duplicate returns 409
- POST /sphere/{id}/memory with oversized body returns 413
- POST /sphere/{id}/consent updates consent flags
- GET /sphere/{id}/data-manifest returns manifest
- Feature gate: governance routes absent without feature
- Feature gate: evolution routes absent without feature
- CORS: preflight OPTIONS returns 200
- Invalid JSON body returns 400
- Non-existent sphere returns 404
- Path parameter extraction works with special characters
- Concurrent requests do not deadlock

## Summary

| Module | LOC Target | Key Exports | Tests |
|--------|-----------|-------------|-------|
| m07_service_registry | 200 | ServiceDefinition, ServiceRegistry, ServiceDiscovery | 5 |
| m08_health_monitor | 250 | HealthStatus, CircuitBreaker, HealthMonitoring, check_health() | 10 |
| m09_lifecycle | 200 | LifecycleOps, graceful_shutdown(), read_devenv_pid() | 5 |
| m10_api_server | 400 | build_router(), serve(), AppStateBundle | 15 |
| **L2 Total** | **1,050** | | **35** |

## Anti-Patterns

- **AP-1:** Polling all 16 services at the same interval — use tiered polling (critical/standard/background)
- **AP-2:** Circuit breaker without cooldown — causes retry storm on failing service
- **AP-3:** `devenv stop` without port verification — processes may survive (known issue)
- **AP-4:** `curl -sf | jq` in health checks — silent failure masks errors. Use `curl -s -o /dev/null -w '%{http_code}'`
- **AP-5:** `:param` syntax in Axum 0.8 — must use `{param}` (compilation error otherwise)
- **AP-6:** Returning references from `ServiceDiscovery` methods — must return owned data (C7)
- **AP-7:** Binding to `0.0.0.0` by default — security risk. Always default to 127.0.0.1.

## Related

- `MASTERPLAN.md` Phase 1 — Services layer implementation plan
- `API_SPEC.md` — Complete endpoint documentation (60+ routes)
- `SECURITY_SPEC.md` — Binding address, body limits, rate limiting
- `[[ULTRAPLATE Developer Environment]]` — Service ports, batches, devenv config
- `[[ULTRAPLATE Master Index]]` — Service registry and port mapping
- `[[Session 034g — NexusBus Wiring and ME-SYNTHEX Bridge]]` — 3 bug fixes affecting health checks
