# Layer 6: Bridges Specification

> Detailed spec for m22-m28: external service bridges and the consent gate.
> The bridge layer — how pane-vortex reads from and writes to 6 ULTRAPLATE services.
> Source: `src/m6_bridges/` | Plan: `MASTERPLAN.md` Phase 5
> v1 Source: `synthex_bridge.rs` (300 LOC), `nexus_bridge.rs` (450 LOC), `me_bridge.rs` (250 LOC),
>   `povm_bridge.rs` (200 LOC), `bridge.rs` (RM, 100 LOC), `vms_bridge.rs` (100 LOC)
> Consent: `CONSENT_SPEC.md` | Obsidian: `[[Session 034f — Database Architecture Schematics]]`

## Overview

Layer 6 provides bidirectional bridges to 6 external ULTRAPLATE services plus the
consent gate that controls all external influence on the Kuramoto field. Each bridge
follows the same raw TCP HTTP pattern (no hyper dependency) with fire-and-forget writes
and cached polling reads.

L6 depends on L1 (types, constants) and L3 (sphere data for consent checks).
L7 (Coordination) calls L6 from the tick loop for polling and writing.

### Design Constraints

| ID | Constraint | Application in L6 |
|----|-----------|-------------------|
| C1 | No upward imports | L6 imports only from L1 and L3 |
| C8 | Consent gate on all external k_mod | Every bridge influence passes through m28 |
| C14 | Fire-and-forget for writes | All POST operations use `tokio::spawn`, never block |

### The Raw TCP HTTP Pattern

All bridges use the same raw TCP pattern to avoid adding HTTP client dependencies (hyper,
reqwest). This keeps the binary small (5.6MB) and avoids transitive dependency complexity:

```rust
/// Send a raw HTTP GET request via TCP. Returns response body or None on any error.
async fn raw_http_get(addr: &str, path: &str) -> Option<String> {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    );

    // Connect with timeout (TIMEOUT_SECS = 3)
    let stream = match tokio::time::timeout(
        Duration::from_secs(TIMEOUT_SECS),
        tokio::net::TcpStream::connect(addr),
    ).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => { debug!("connect failed: {e}"); return None; }
        Err(_) => { debug!("connect timeout"); return None; }
    };

    let (reader, mut writer) = stream.into_split();

    // Write request
    if let Err(e) = writer.write_all(request.as_bytes()).await {
        debug!("write failed: {e}");
        return None;
    }
    // NOTE: Do NOT call writer.shutdown() — half-closing the write side causes
    // axum-based servers to drop the connection before sending the response.
    // Connection: close in the request header is sufficient. (BUG fix from v1)

    // Read response with timeout
    let mut buf_reader = BufReader::new(reader);
    let mut response = String::new();
    match tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), async {
        // Skip HTTP headers (read until empty line)
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).await?;
            if line.trim().is_empty() { break; }
        }
        // Read body
        buf_reader.read_to_string(&mut response).await
    }).await {
        Ok(Ok(_)) => Some(response),
        _ => None,
    }
}

/// Send a raw HTTP POST with JSON body. Fire-and-forget.
async fn raw_http_post(addr: &str, path: &str, body: &serde_json::Value) {
    let body_str = body.to_string();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body_str}",
        body_str.len()
    );

    let mut stream = match tokio::time::timeout(
        Duration::from_secs(TIMEOUT_SECS),
        tokio::net::TcpStream::connect(addr),
    ).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => { debug!("connect failed: {e}"); return; }
        Err(_) => { debug!("connect timeout"); return; }
    };

    if let Err(e) = stream.write_all(request.as_bytes()).await {
        debug!("post write failed: {e}");
    }
}
```

### Bridge Data Flow Overview

```
External Services              Pane-Vortex L6                   Impact
=================              =============                   ======

SYNTHEX (:8090) --GET /v3/thermal--> m22 --thermal_k_adj--> m28 --> conductor
                <--POST /api/ingest-- m22 (field state)

Nexus (:8100) --GET /health--------> m23 --nexus_k_adj---> m28 --> conductor
              <--POST /api/field---- m23 (inner field r)

ME (:8080) ----GET /api/health-----> m24 --me_k_adj------> m28 --> conductor
                                          (BUG-008: fitness frozen)

POVM (:8125) <--POST /snapshot------ m25 (field snapshots every 12 ticks)
             <--POST /weights------- m25 (Hebbian weights every 60 ticks)
             --GET /hydrate--------> m25 (startup hydration)
             --GET /pathways-------> m25 (startup weight seeding)

RM (:8130) <----POST /put----------- m26 (TSV, conductor decisions)
           ----GET /search---------> m26 (startup bootstrap)

VMS (:8120) <---POST /memory-------- m27 (field state every 60 ticks)

All bridges ----through-----------> m28 (consent_gated_k_adjustment)
```

## 1. m22_synthex_bridge (~300 LOC)

### 1.1 Service: SYNTHEX (:8090)

SYNTHEX is the thermal homeostasis engine — the "brain" of the developer environment.
It maintains a target temperature via PID control. Pane-vortex reads thermal state to
modulate coupling and writes field state so fleet dynamics feed the thermal model.

### 1.2 Read Path: GET /v3/thermal

```rust
const SYNTHEX_ADDR: &str = "127.0.0.1:8090";
const THERMAL_POLL_INTERVAL: u64 = 6;  // Every 30s at 5s/tick
const THERMAL_POLL_WALL_SECS: u64 = 25; // Wall-clock fallback

/// Parsed SYNTHEX v3 thermal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalState {
    pub temperature: f64,
    pub target: f64,
    pub pid_output: f64,
    pub heat_sources: Vec<HeatSource>,
    pub fetched_at: f64,  // epoch seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatSource {
    pub id: String,
    pub name: String,
    pub reading: f64,
    pub weight: f64,
}

impl ThermalState {
    /// Thermal deviation from target, normalized to [-1.0, 1.0].
    /// Negative = under target (cold), positive = over target (hot).
    pub fn thermal_deviation(&self) -> f64 {
        if self.target == 0.0 { return 0.0; }
        ((self.temperature - self.target) / self.target).clamp(-1.0, 1.0)
    }

    /// Whether the thermal state is stale (> 60s since fetch).
    /// 2 x THERMAL_POLL_WALL_SECS = 50s, rounded up to 60s for margin.
    pub fn is_stale(&self, now: f64) -> bool {
        now - self.fetched_at > 60.0
    }
}

pub type SharedThermalState = Arc<RwLock<Option<ThermalState>>>;
```

### 1.3 K_mod Influence

SYNTHEX thermal deviation maps to a coupling adjustment:

```rust
/// Compute k_adjustment from thermal deviation.
/// Hot field -> reduce coupling (allow divergence for cooldown).
/// Cold field -> increase coupling (encourage synchronization for warmth).
pub fn thermal_k_adj(thermal: &ThermalState) -> f64 {
    let dev = thermal.thermal_deviation();
    (-dev * 0.05).clamp(-0.05, 0.05)
}
```

### 1.4 Write Path: POST /api/ingest

Writes field state back to SYNTHEX so fleet dynamics contribute to the thermal model:

```rust
pub async fn post_field_to_synthex(r: f64, k_mod: f64, sphere_count: usize, tick: u64) {
    let body = serde_json::json!({
        "source": "pane-vortex",
        "data": { "r": r, "k_modulation": k_mod, "sphere_count": sphere_count, "tick": tick }
    });
    tokio::spawn(raw_http_post(synthex_addr(), "/api/ingest", &body));
}
```

### 1.5 Wall-Clock Fallback

The bridge tracks last poll time with an `AtomicU64` to decouple from tick interval
variability. If the tick interval becomes irregular, the wall-clock check ensures
SYNTHEX is still polled at approximately 25-second intervals:

```rust
static LAST_THERMAL_POLL: AtomicU64 = AtomicU64::new(0);

pub fn should_poll_thermal(tick: u64) -> bool {
    if tick % THERMAL_POLL_INTERVAL == 0 { return true; }
    let now = now_secs() as u64;
    let last = LAST_THERMAL_POLL.load(Ordering::Relaxed);
    now.saturating_sub(last) >= THERMAL_POLL_WALL_SECS
}
```

### 1.6 Tests (15 target)

- `thermal_deviation()`: above target returns positive
- `thermal_deviation()`: below target returns negative
- `thermal_deviation()`: zero target returns 0.0
- `thermal_deviation()`: clamps to [-1.0, 1.0]
- `thermal_k_adj()`: hot field returns negative k_adj
- `thermal_k_adj()`: cold field returns positive k_adj
- `is_stale()`: fresh reading is not stale
- `is_stale()`: old reading is stale (>60s)
- Poll timing: fires every 6 ticks
- Shared state: None when never polled
- Shared state: updated after successful poll
- Failed connection leaves state unchanged
- Timeout (3s) does not block tick loop
- POST field data serializes correctly
- Wall-clock fallback when ticks are irregular

## 2. m23_nexus_bridge (~400 LOC)

### 2.1 Service: SAN-K7 Orchestrator (:8100)

SAN-K7 runs the outer Kuramoto field (12 oscillators) for strategy coordination.
Pane-vortex reads the outer field to align inner dynamics with strategic goals.
This creates a two-layer nested Kuramoto architecture:
- Inner field (pane-vortex): up to 200 spheres, `r_inner` governs fleet dynamics
- Outer field (NexusForge): 12 oscillators, `r_outer` governs strategy

### 2.2 Read Path: GET /health (Nexus)

```rust
const NEXUS_ADDR: &str = "127.0.0.1:8100";
const NEXUS_POLL_INTERVAL: u64 = 12;  // Every 60s at 5s/tick

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusFieldState {
    pub r_outer: f64,           // Outer Kuramoto order parameter [0, 1]
    pub psi_outer: f64,         // Mean phase angle (radians)
    pub is_coherent: bool,
    pub memory_count: usize,
    pub avg_relevance: f64,     // [0, 1]
    pub synapse_count: usize,
    pub active_routes: usize,
    pub pending_signals: usize,
    pub fetched_at: f64,
    pub outer_frequency: Option<f64>,  // For inter-scale frequency ratio
}
```

### 2.3 Strategy Coherence

The outer field r maps to a qualitative strategy coherence level:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyCoherence {
    Aligned,     // r >= 0.8: dispatch confidently
    Partial,     // r >= 0.5: dispatch with caution
    Diverging,   // r >= 0.2: reduce dispatch rate
    Incoherent,  // r < 0.2: pause dispatch
}
```

### 2.4 K_mod Influence

```rust
pub fn nexus_k_adj(nexus: &NexusFieldState) -> f64 {
    match nexus.strategy_coherence() {
        StrategyCoherence::Aligned => 0.03,
        StrategyCoherence::Partial => 0.01,
        StrategyCoherence::Diverging => -0.02,
        StrategyCoherence::Incoherent => -0.05,
    }
}
```

### 2.5 Nexus Commands (20 Available)

SAN-K7 exposes 20 nexus commands via `POST /api/v1/nexus/command`:

| Command | Module | Purpose |
|---------|--------|---------|
| `service-health` | M6 | 16-service health matrix |
| `synergy-check` | M45 | Cross-service synergy score |
| `best-practice` | M44 | Pattern recommendations |
| `deploy-swarm` | M40 | Swarm deployment control |
| `module-status` | M3 | Per-module health |
| `route-query` | M15 | Signal routing paths |

See `[[ULTRAPLATE Master Index]]` for the full 20-command reference.

### 2.6 Tests (15 target)

- `strategy_coherence()`: Aligned at r=0.8
- `strategy_coherence()`: Incoherent at r=0.1
- `nexus_k_adj()`: Aligned returns positive
- `nexus_k_adj()`: Incoherent returns negative
- `is_stale()`: >120s is stale (2x poll interval)
- Poll timing: fires every 12 ticks
- Failed connection leaves state unchanged
- Write path: POST inner field r to nexus
- NexusFieldState: serde roundtrip
- FleetMode: single sphere is not "fleet coherence"
- Inter-scale frequency ratio: computed when outer_frequency present
- Strategy coherence boundary: r=0.5 exactly is Partial
- Wall-clock fallback at NEXUS_POLL_WALL_SECS (55s)
- Concurrent polling does not duplicate requests
- Address override from NEXUS_ADDR env

## 3. m24_me_bridge (~250 LOC)

### 3.1 Service: Maintenance Engine (:8080)

The ME runs a 7-layer architecture with 12D tensor dynamics, RALPH evolution, and
589K health check records. It provides a fitness score that influences coupling.

**BUG-008 (CRITICAL):** ME EventBus has zero publishers. This means the ME's internal
fitness evolution is frozen at 0.3662 since 2026-03-06. The bridge reads the same
value every poll. Fixing BUG-008 is the highest-impact item in MASTERPLAN.md V3.1.1.

### 3.2 Read Path: GET /api/health

```rust
const ME_ADDR: &str = "127.0.0.1:8080";
const ME_POLL_INTERVAL: u64 = 12;  // Every 60s at 5s/tick

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeHealthState {
    pub fitness: f64,          // [0.0, 1.0]
    pub overall_healthy: bool,
    pub status: String,        // "healthy" | "degraded" | "unhealthy"
    pub fetched_at: f64,
}
```

### 3.3 K_mod Influence

```rust
pub fn me_k_adj(me: &MeHealthState) -> f64 {
    if me.fitness >= 0.8 {
        0.03   // High fitness: boost coupling (confident)
    } else if me.fitness >= 0.5 {
        0.0    // Medium fitness: neutral
    } else if me.fitness >= 0.3 {
        -0.02  // Low fitness: reduce coupling (conserve)
    } else {
        -0.05  // Critical fitness: strong reduction
    }
}
```

### 3.4 The ME Health Endpoint Trap

ME uses `/api/health` (not `/health`). This is one of only two services with the `/api/`
prefix on health endpoints (the other is SYNTHEX at `/api/health`). The registry must
use the correct path or health checks will return 404.

### 3.5 Observer Endpoint (Alternative)

For richer data, GET `/api/observer` returns:
```json
{
  "metrics": { "correlations_found": 42, "patterns_detected": 7 },
  "last_report": { "current_fitness": 0.3662, "trend": "stable" }
}
```

### 3.6 Tests (10 target)

- `me_k_adj()`: high fitness returns positive
- `me_k_adj()`: low fitness returns negative
- `is_stale()`: >120s is stale
- Poll timing: fires every 12 ticks
- Failed connection leaves state unchanged
- Status parsing: "healthy", "degraded", "unhealthy"
- Fitness clamped to [0.0, 1.0]
- NaN fitness treated as 0.0
- MeHealthState serde roundtrip
- Default health when ME never polled: None

## 4. m25_povm_bridge (~200 LOC)

### 4.1 Service: POVM Engine (:8125)

Persistent Oscillating Vortex Memory — stores field snapshots and Hebbian pathway weights
across daemon restarts. Bidirectional: writes periodic snapshots, reads on startup for hydration.

### 4.2 Write Path: POST /snapshot (Every 12 Ticks)

```rust
const POVM_ADDR: &str = "127.0.0.1:8125";
const POVM_SYNC_INTERVAL: u64 = 12;          // Every ~60s
const HEBBIAN_SYNC_MULTIPLIER: u64 = 5;      // Weights every 60 ticks (~5min)

pub async fn post_field_snapshot(r: f64, psi: f64, k_mod: f64, sphere_count: usize, tick: u64) {
    let body = serde_json::json!({
        "type": "field_snapshot",
        "data": { "r": r, "psi": psi, "k_modulation": k_mod, "sphere_count": sphere_count, "tick": tick },
        "source": "pane-vortex",
        "timestamp": now_secs()
    });
    tokio::spawn(raw_http_post(povm_addr(), "/ingest", &body));
}
```

### 4.3 Write Path: POST /pathways (Every 60 Ticks)

```rust
pub async fn post_hebbian_weights(connections: &[Connection]) {
    let pathways: Vec<serde_json::Value> = connections.iter()
        .map(|c| serde_json::json!({
            "pre_id": c.from, "post_id": c.to,
            "weight": c.weight, "type_weight": c.type_weight
        }))
        .collect();

    let body = serde_json::json!({
        "type": "hebbian_weights",
        "pathways": pathways,
        "source": "pane-vortex"
    });
    tokio::spawn(raw_http_post(povm_addr(), "/pathways/batch", &body));
}
```

### 4.4 Read Path: Startup Hydration

On daemon startup (after RM bootstrap), the bridge reads from POVM to seed state:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PovmHydrationSummary {
    pub memory_count: i64,
    pub pathway_count: i64,
    pub latest_r: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PovmPathway {
    pub pre_id: String,
    pub post_id: String,
    pub weight: f64,
}

pub async fn hydrate_summary() -> Option<PovmHydrationSummary> {
    let response = raw_http_get(&povm_addr(), "/hydrate").await?;
    serde_json::from_str(&response).ok()
}

pub async fn hydrate_pathways() -> Vec<PovmPathway> {
    match raw_http_get(&povm_addr(), "/pathways").await {
        Some(response) => serde_json::from_str(&response).unwrap_or_default(),
        None => vec![],
    }
}
```

### 4.5 Shutdown Flush

On graceful shutdown (SIGTERM), the bridge posts a final snapshot and weight dump:

```rust
pub async fn shutdown_flush(r: f64, psi: f64, k_mod: f64, connections: &[Connection], tick: u64) {
    post_field_snapshot(r, psi, k_mod, 0, tick).await;
    post_hebbian_weights(connections).await;
}
```

### 4.6 Tests (10 target)

- `should_sync()`: true at tick 12, 24, 36
- `should_sync()`: false at tick 0 (skip first tick)
- `should_sync_hebbian()`: true at tick 60
- `hydrate_summary()`: parses valid JSON
- `hydrate_pathways()`: returns empty on connection failure
- Field snapshot POST serializes correctly
- Hebbian weights POST serializes all connections
- POVM address from env override
- Timeout does not block daemon startup
- Shutdown triggers final flush (snapshot + weights)

## 5. m26_rm_bridge (~150 LOC)

### 5.1 Service: Reasoning Memory (:8130)

**CRITICAL: Reasoning Memory uses TSV format, NEVER JSON.**

```
category\tagent\tconfidence\tttl\tcontent
```

The RM stores cross-session context as key-value entries with category, agent attribution,
confidence score, and TTL. Pane-vortex writes conductor decisions and reads on startup
for context bootstrap.

### 5.2 Write Path: POST /put (TSV)

```rust
const RM_ADDR: &str = "127.0.0.1:8130";

/// POST a TSV entry to Reasoning Memory. Fire-and-forget.
/// CRITICAL: TSV format. NEVER send JSON to this endpoint.
pub async fn post_to_rm(category: &str, content: &str) {
    let addr = std::env::var("REASONING_MEMORY_ADDR")
        .unwrap_or_else(|_| RM_ADDR.into());

    // Sanitize tabs and newlines to prevent TSV format corruption (E9)
    let safe_content = content.replace(['\t', '\n'], " ");
    let safe_category = category.replace(['\t', '\n'], " ");

    let body = format!("{safe_category}\tpane-vortex\t0.90\t604800\t{safe_content}");
    let request = format!(
        "POST /put HTTP/1.1\r\nHost: {addr}\r\nContent-Type: text/plain\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    let stream = match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(&addr),
    ).await {
        Ok(Ok(s)) => s,
        _ => return,
    };

    let (_, mut writer) = stream.into_split();
    writer.write_all(request.as_bytes()).await.ok();
}
```

### 5.3 TSV Field Layout

| Field | Value | Notes |
|-------|-------|-------|
| category | e.g., "conductor_decision" | No tabs or newlines (sanitized) |
| agent | "pane-vortex" | Always this value |
| confidence | "0.90" | Fixed confidence |
| ttl | "604800" | 7 days in seconds |
| content | Decision text | Sanitized (tabs/newlines replaced with spaces) |

### 5.4 Read Path: GET /search (Startup Bootstrap)

```rust
pub async fn bootstrap_from_rm() -> Vec<String> {
    let addr = std::env::var("REASONING_MEMORY_ADDR")
        .unwrap_or_else(|_| RM_ADDR.into());
    match raw_http_get(&addr, "/search?q=pane-vortex&limit=10").await {
        Some(response) => response.lines().filter(|l| !l.is_empty()).map(String::from).collect(),
        None => vec![],
    }
}
```

### 5.5 RM Noise Problem (SG-4)

Session 040 exploration revealed 67% of RM entries are PV field state updates with 7-day
TTL. MASTERPLAN.md V3.5.1 recommends reducing TTL to 600s for field state entries.

### 5.6 Tests (5 target)

- TSV format: tabs properly separate fields
- Content sanitization: embedded tabs replaced with spaces
- Content sanitization: embedded newlines replaced with spaces
- POST uses `text/plain` content type (not `application/json`)
- Bootstrap returns empty on connection failure

## 6. m27_vms_bridge (~150 LOC)

### 6.1 Service: Vortex Memory System (:8120)

VMS stores spatial memories using fractal topology. Pane-vortex seeds VMS with field state
so it starts with non-zero memory count and coherence after cold boot.

### 6.2 Write Path: POST /memory (Every 60 Ticks)

```rust
const VMS_ADDR: &str = "127.0.0.1:8120";
const VMS_SYNC_INTERVAL: u64 = 60;  // Every ~5min

pub async fn post_field_memory(
    r: f64, k_mod: f64, sphere_count: usize, decision: &str, tick: u64,
) {
    let body = serde_json::json!({
        "content": format!(
            "pane-vortex field: r={r:.3} k_mod={k_mod:.3} spheres={sphere_count} \
             decision={decision} tick={tick}"
        ),
        "tags": ["pane-vortex", "field-state", decision],
        "source": "pane-vortex",
        "metadata": { "r": r, "k_modulation": k_mod, "sphere_count": sphere_count }
    });
    tokio::spawn(raw_http_post(vms_addr(), "/api/memories", &body));
}
```

### 6.3 VMS is Write-Only from PV

Unlike SYNTHEX, Nexus, and POVM bridges, VMS bridge is write-only. PV does not read
from VMS. VMS reads are handled by other services that consume VMS memories.

### 6.4 VMS Health Note (BUG-017)

VMS was dormant (r=0.0, 0 memories) at time of Session 040 exploration. MASTERPLAN.md
V3.1.8 recommends restarting via `devenv restart vortex-memory-system`.

### 6.5 Tests (5 target)

- `should_sync()`: true at tick 60, 120
- `should_sync()`: false at tick 0
- POST body contains all required fields
- VMS address from env override
- Connection failure is silent (fire-and-forget)

## 7. m28_consent_gate (~200 LOC)

### 7.1 The Central Consent Function

Every external bridge influence passes through this module. It is the enforcement point
for sphere sovereignty over the coupling field.

```rust
/// Gate external k_mod influence through fleet consent.
/// This function MUST be called before any bridge influence reaches the conductor.
pub fn consent_gated_k_adjustment(
    raw_influence: f64,
    spheres: &HashMap<PaneId, PaneSphere>,
    budget: (f64, f64),  // (K_MOD_BUDGET_MIN, K_MOD_BUDGET_MAX)
) -> f64 {
    let consent_scale = fleet_mean_consent(spheres);
    let scaled = raw_influence * consent_scale;
    scaled.clamp(budget.0, budget.1)
}

/// Mean of all spheres' max_k_adjustment, excluding opted-out spheres.
/// Returns 0.0 if all spheres opt out (no external influence).
pub fn fleet_mean_consent(spheres: &HashMap<PaneId, PaneSphere>) -> f64 {
    let eligible: Vec<f64> = spheres.values()
        .filter(|s| s.consent.accept_external_modulation)
        .map(|s| s.consent.max_k_adjustment)
        .collect();

    if eligible.is_empty() { return 0.0; }

    #[allow(clippy::cast_precision_loss)]
    let mean = eligible.iter().sum::<f64>() / eligible.len() as f64;
    mean
}
```

### 7.2 K_mod Budget

The budget constrains the combined external influence:

| Parameter | Value | Source |
|-----------|-------|--------|
| `K_MOD_BUDGET_MIN` | 0.85 | m04_constants |
| `K_MOD_BUDGET_MAX` | 1.15 | m04_constants |

These bounds are fleet-negotiable via governance proposals (NA-P-2, V3.4).

### 7.3 Call Chain in tick_once

```
tick_once():
  1. poll_synthex() -> ThermalState
  2. poll_nexus() -> NexusFieldState
  3. poll_me() -> MeHealthState

  4. synthex_k_adj = thermal_k_adj(&thermal)    // [-0.05, 0.05]
  5. nexus_k_adj = nexus_k_adj(&nexus)          // [-0.05, 0.03]
  6. me_k_adj = me_k_adj(&me)                   // [-0.05, 0.03]

  7. raw_influence = synthex_k_adj + nexus_k_adj + me_k_adj
  8. budgeted = consent_gated_k_adjustment(raw_influence, spheres, budget)
  9. k_modulation += budgeted  // Applied to conductor
```

### 7.4 Per-Sphere K_mod Isolation (NA-P-4)

Global consent scaling is the minimum viable implementation. Full per-sphere isolation
computes effective coupling per sphere in the step function:

```rust
/// Per-sphere effective K scaling. Spheres that opt out of external modulation
/// receive K_eff without bridge influence. (NA-P-4)
pub fn per_sphere_k_scale(
    sphere: &PaneSphere,
    global_k_mod: f64,
    bridge_contribution: f64,
) -> f64 {
    if sphere.opt_out_external_modulation {
        global_k_mod - bridge_contribution
    } else {
        let scale = sphere.consent.max_k_adjustment / 0.15; // Normalized to default
        global_k_mod - bridge_contribution + (bridge_contribution * scale)
    }
}
```

### 7.5 Open NA-P Gaps Affecting Bridges

From `[[Session 034e — NA Gap Analysis of Master Plan V2]]`:

| Gap | Priority | Summary | Bridge Module Affected |
|-----|----------|---------|----------------------|
| NA-P-1 | P2 | Consent observed, not declared | m28 (addressed by m39 ConsentDeclaration) |
| NA-P-2 | P3 | k_mod budget fixed, not fleet-adaptive | m28 (V3.4 governance) |
| NA-P-4 | P1 | Thermal influence global, not per-sphere | m22 + m28 |
| NA-P-9 | P2 | No decision attribution | m28 (ModulationBreakdown) |
| NA-SG-1 | P1 | RM logging without sphere consent | m26 |
| NA-SG-2 | P1 | Nvim autocmds can't opt out | Hooks (consent-adjacent) |
| NA-SG-3 | P1 | Fleet dispatch targets unwilling spheres | m32 executor (L7) |
| NA-SG-4 | P2 | Bus listener persists without sphere knowing | m29 bus (L7) |
| NA-P-13 | P2 | No data sovereignty across bridge targets | m25, m26 (V3.3) |

### 7.6 Tests (10 target)

- `consent_gated_k_adjustment()`: raw influence scaled by consent
- `consent_gated_k_adjustment()`: clamped to budget bounds
- `fleet_mean_consent()`: all opted out returns 0.0
- `fleet_mean_consent()`: default consent returns 0.15
- `fleet_mean_consent()`: mixed consent averaged correctly
- `per_sphere_k_scale()`: opted-out sphere removes bridge contribution
- `per_sphere_k_scale()`: max_k_adjustment=0 zeroes bridge effect
- Budget bounds: influence never exceeds K_MOD_BUDGET_MAX
- Budget bounds: influence never falls below K_MOD_BUDGET_MIN
- Empty spheres map: consent_scale = 0.0

## Summary

| Module | LOC Target | Service | Read/Write Pattern | Tests |
|--------|-----------|---------|-------------------|-------|
| m22_synthex_bridge | 300 | SYNTHEX :8090 | GET /v3/thermal (6 ticks), POST /api/ingest | 15 |
| m23_nexus_bridge | 400 | SAN-K7 :8100 | GET /health (12 ticks), POST /api/field | 15 |
| m24_me_bridge | 250 | ME :8080 | GET /api/health (12 ticks), read-only | 10 |
| m25_povm_bridge | 200 | POVM :8125 | POST /snapshot (12 ticks), POST /weights (60 ticks), GET /hydrate (startup) | 10 |
| m26_rm_bridge | 150 | RM :8130 | POST /put (TSV, per-decision), GET /search (startup) | 5 |
| m27_vms_bridge | 150 | VMS :8120 | POST /memory (60 ticks), write-only | 5 |
| m28_consent_gate | 200 | (internal) | consent_gated_k_adjustment(), fleet_mean_consent() | 10 |
| **L6 Total** | **1,650** | | | **70** |

## Anti-Patterns

- **AP-1:** `writer.shutdown()` after HTTP request — causes axum to drop connection before response.
  Use `Connection: close` header instead. (V1 BUG fix, documented in v1 me_bridge.rs)
- **AP-2:** JSON POST to Reasoning Memory — RM accepts TSV ONLY. This trap recurs every session.
  The content type must be `text/plain`, not `application/json`.
- **AP-3:** Bridge influence without consent gate — violates C8. Every k_adjustment MUST pass
  through `consent_gated_k_adjustment()`.
- **AP-4:** Blocking bridge reads in tick loop — use `tokio::spawn` for all bridge IO (C14).
- **AP-5:** `curl -sf | jq` for bridge health checks — `-sf` silently fails. Use
  `-s -o /dev/null -w '%{http_code}'`.
- **AP-6:** Global k_mod without per-sphere option — violates NA-P-4 sphere sovereignty.
- **AP-7:** RM POST without content sanitization — embedded tabs corrupt TSV format.
  Always replace `\t` and `\n` with spaces.
- **AP-8:** Assuming ME fitness is dynamic — BUG-008 means fitness is frozen at 0.3662.
  Until fixed, ME bridge reads are effectively constant.

## Related

- `CONSENT_SPEC.md` — Full sovereignty framework
- `KURAMOTO_FIELD_SPEC.md` Section 4 — Conductor and k_mod budget
- `layers/L7_COORDINATION_SPEC.md` Section 7 — tick_once bridge integration
- `MASTERPLAN.md` V3.1 — Diagnostics and repair (BUG-008, alerts)
- `MASTERPLAN.md` V3.3 — Sovereignty (NA-P gaps)
- `[[Session 034d — NA Consent Gate Implementation]]` — Consent gate pattern origin
- `[[Session 034e — NA Gap Analysis of Master Plan V2]]` — 18 NA-P gaps
- `[[Session 034f — SYNTHEX Schematics and Wiring]]` — SYNTHEX thermal model
- `[[Maintenance Engine — Architecture Schematic]]` — ME 7-layer architecture
- `[[POVM Engine]]` — POVM persistence model
- `[[Session 039 — What I Learned]]` — POVM bimodal weight discovery
