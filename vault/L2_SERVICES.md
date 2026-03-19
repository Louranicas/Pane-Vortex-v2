---
title: "Layer 2: Services — Module Documentation"
date: 2026-03-19
tags: [modules, services, L2, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
layer: L2
modules: [m07, m08, m09, m10]
---

# Layer 2: Services (m07-m10)

> Service registry, health monitoring, lifecycle management, and HTTP API server.
> **Depends on:** L1 (Foundation)
> **Target LOC:** ~1,050 | **Target tests:** 41+

---

## Overview

L2 provides the infrastructure shell around the field dynamics:
- A registry of all 16 ULTRAPLATE services with ports and health paths
- Async health polling with circuit breakers for stale services
- Process lifecycle management with graceful shutdown (SIGTERM handling)
- An Axum HTTP server that composes routes from higher layers

**Implementation order:** m07 (registry) -> m08 (health) -> m09 (lifecycle) -> m10 (API server, last because it needs routes from L3+)

---

## m07 — Service Registry

**Source:** `src/m2_services/m07_service_registry.rs`
**LOC Target:** ~200

### Purpose

Tracks all 16 ULTRAPLATE services. Pre-populated at startup with service metadata from the known ULTRAPLATE ecosystem. Used by health monitor, bridge modules, and integration matrix.

### Service Registry Table

| Service ID | Port | Health Path | Batch | Notes |
|-----------|------|-------------|-------|-------|
| maintenance-engine | 8080 | /api/health | 2 | 12D tensor, PBFT |
| devops-engine | 8081 | /health | 1 | Neural orchestration |
| synthex | 8090 | /api/health | 2 | REST + WS |
| san-k7-orchestrator | 8100 | /health | 2 | 59 modules |
| nais | 8101 | /health | 3 | Neural adaptive intelligence |
| bash-engine | 8102 | /health | 3 | 45 safety patterns |
| tool-maker | 8103 | /health | 3 | v1.55.0 |
| claude-context-manager | 8104 | /health | 4 | 41 crates |
| tool-library | 8105 | /health | 4 | 65 tools |
| codesynthor-v7 | 8110 | /health | 1 | 62 modules |
| vortex-memory-system | 8120 | /health | 5 | OVM + POVM bridge |
| povm-engine | 8125 | /health | 1 | Persistent OVM |
| reasoning-memory | 8130 | /health | 4 | TSV format |
| pane-vortex | 8132 | /health | 5 | This service |
| architect-agent | 9001 | /health | 2 | Pattern library |
| prometheus-swarm | 10001 | /health | 2 | CVA-NAM 40 agents |

### Key Design

- Registry is immutable after construction (no runtime registration of new services)
- `by_batch(n)` returns services ordered for startup dependency resolution
- `health_url(id)` constructs the full URL for health checking

### V3 Alignment

V3.1 (Diagnostics) uses this registry to verify all 16 services are healthy. ALERT-7 (Tool Library port mapping anomaly) originates from registry data.

---

## m08 — Health Monitor

**Source:** `src/m2_services/m08_health_monitor.rs`
**LOC Target:** ~250

### Purpose

Async health polling for all ULTRAPLATE services. Detects staleness, tracks latency, and provides circuit breakers to avoid hammering unresponsive services.

### Circuit Breaker States

```
Closed (healthy) --[3 failures]--> Open (tripped)
Open --[60s elapsed]--> HalfOpen (probe)
HalfOpen --[probe succeeds]--> Closed
HalfOpen --[probe fails]--> Open
```

### Key Functions

- `check_health(entry)` — Single HTTP GET to service health endpoint. Returns HTTP status code + latency.
- `check_all(registry)` — Parallel health check of all services using `tokio::join!`.
- `is_stale(status, max_age_secs)` — Returns true if last successful check is older than threshold.

### Health Check Pattern

```rust
// CORRECT: use -o /dev/null -w '%{http_code}' pattern (not -sf which swallows errors)
let response = raw_http_get(&health_url, timeout).await?;
```

The bridge pattern (raw TCP HTTP) is used here too — no hyper dependency just for health checks.

### V3 Alignment

V3.1 items V3.1.5 (Prometheus Swarm) and V3.1.8 (VMS restart) use health monitoring to verify repair.

---

## m09 — Lifecycle

**Source:** `src/m2_services/m09_lifecycle.rs`
**LOC Target:** ~200

### Purpose

Process lifecycle management: graceful shutdown with SIGTERM handling, PID file management, and startup coordination.

### Graceful Shutdown (V1 Bug Fix RG-1)

V1 lost state on SIGTERM because the shutdown handler did not wait for the write lock. V2 implements:

```rust
pub async fn install_signal_handlers() -> PvResult<ShutdownSignal> {
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
        let _ = tx.send(());
    });

    Ok(ShutdownSignal(rx))
}
```

On shutdown signal:
1. Stop accepting new connections
2. Drain IPC bus event queue
3. Flush pending persistence writes (with 5-second timeout)
4. Write final field snapshot
5. Remove PID file
6. Exit cleanly

### PID File Management

```rust
pub fn write_pid_file(path: &Path) -> PvResult<()> { ... }
pub fn remove_pid_file(path: &Path) -> PvResult<()> { ... }
```

PID files are written to `~/.local/share/devenv/pids/pane-vortex.pid` when running under DevEnv.

### V3 Alignment

All deployment and restart operations (V3.1, V3.2) depend on clean lifecycle management.

---

## m10 — API Server

**Source:** `src/m2_services/m10_api_server.rs`
**LOC Target:** ~400
**Feature gate:** `api`

### Purpose

Axum HTTP server that composes routes from all layers into a single application. Handles CORS, body size limits, and state injection.

### Router Composition

```rust
pub fn build_router(state: ApiState) -> Router {
    Router::new()
        // L2: Infrastructure
        .route("/health", get(health_handler))

        // L3: Field
        .nest("/sphere", sphere_router())
        .nest("/field", field_router())

        // L6: Bridges
        .nest("/synthex", synthex_router())
        .nest("/nexus", nexus_router())

        // L7: Coordination
        .nest("/bus", bus_router())
        .nest("/executor", executor_router())
        .nest("/cascade", cascade_router())
        .nest("/integration", integration_router())

        // L8: Governance (feature-gated)
        .merge(governance_router())  // Only if feature = "governance"

        // Middleware
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(65_536))  // 65KB
        .with_state(state)
}
```

### Bind Address Security

Default: `127.0.0.1:8132` (loopback only). Override with `PV2_SERVER__BIND_ADDR=0.0.0.0` for external access. This was a critical security fix in V1 (Session 016c).

### Body Limit

65KB max request body. This prevents memory exhaustion from large payloads. The limit matches V1.

### Route Count Target

V1 had 56 endpoints (32 original + 24 from sprints). V2 targets similar coverage organized by layer.

### V3 Alignment

V3.1.2 (fix evolution endpoint 404s) requires verifying router composition. V3.4 adds governance routes (feature-gated).

---

## Implementation Notes

### Error Handling in API Handlers

All API handlers return `Result<Json<Value>, StatusCode>` using the mapping from [ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md):

```rust
async fn register_sphere(
    State(state): State<ApiState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, StatusCode> {
    let sphere = state.register_sphere(body.id, body.persona)
        .await
        .map_err(|e| match e {
            PvError::Field(FieldError::SphereLimitExceeded) => StatusCode::TOO_MANY_REQUESTS,
            PvError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(sphere).unwrap_or_default()))
}
```

### Axum State Injection

All handlers receive `State<ApiState>` where `ApiState = Arc<AppState>`. The state is shared across all handler invocations via Axum's state layer.

---

## Cross-References

- **[CODE_MODULE_MAP.md](../CODE_MODULE_MAP.md)** — m07-m10 type and function index
- **[DEPLOYMENT_GUIDE.md](../DEPLOYMENT_GUIDE.md)** — Startup, PID tracking, health checks
- **[ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md)** — HTTP status code mapping
- **[MASTERPLAN.md](../../MASTERPLAN.md)** — V3.1 diagnostic items
- **Obsidian:** `[[ULTRAPLATE Developer Environment]]` (service registry), `[[ULTRAPLATE Master Index]]` (port assignments)
