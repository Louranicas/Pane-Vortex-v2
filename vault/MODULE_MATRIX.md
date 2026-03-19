# Module Matrix

> 41-module cross-reference: imports, exports, state access, data flow.
> Read this to understand who depends on whom and who modifies what.
> Source: `src/lib.rs` module declarations, `.claude/context.json`
> Plan: `MASTERPLAN.md` | Obsidian: `[[Session 039 — Architectural Schematics and Refactor Safety]]`

## Overview

Pane-vortex v2 has 41 modules across 8 layers. Dependencies flow strictly downward:
higher layers import from lower layers, never upward. L1 has zero external dependencies.

```
L8 Governance  -> L7 -> L6 -> L5 -> L4 -> L3 -> L1
                                          -> L2 -> L1
```

## 1. Layer 1: Foundation (m01-m06) — Zero Dependencies

| Module | Exports | Imported By | State Modified | Tests Target |
|--------|---------|-------------|----------------|-------------|
| m01_core_types | PaneId, Phase, Frequency, Point3D, SphereMemory, Buoy | ALL modules | None (data types only) | 15 |
| m02_error_handling | PvError, PvResult | ALL modules | None (error types only) | 10 |
| m03_config | PvConfig, load_config() | m10, m35, main.rs | None (reads config files) | 10 |
| m04_constants | All named constants | ALL modules | None (const values only) | 5 |
| m05_traits | Oscillator, Learnable, Bridgeable, Consentable | m11, m16, m19, m22-m28, m39 | None (trait definitions) | 5 |
| m06_validation | validate_phase(), validate_frequency(), truncate_string() | m10, m11, m29 | None (pure functions) | 15 |

### m01_core_types — Fan-in: ALL (highest)

This is the most-imported module. Changes here ripple everywhere.

```rust
// Key exports
pub type PaneId = String;
pub type Phase = f64;       // radians, [0, TAU)
pub type Frequency = f64;   // Hz, [0.001, 10.0]
pub type TaskId = String;

pub struct Point3D { pub x: f64, pub y: f64, pub z: f64 }
pub struct SphereMemory { pub id: u64, pub position: Point3D, pub activation: f64, ... }
pub struct Buoy { pub position: Point3D, pub home: Point3D, pub activation_count: u64, ... }
pub struct OrderParameter { pub r: f64, pub psi: f64 }
```

### m05_traits — Core Trait Definitions

```rust
pub trait Oscillator {
    fn phase(&self) -> Phase;
    fn frequency(&self) -> Frequency;
    fn step(&mut self, dt: f64, coupling_sum: f64);
}

pub trait Learnable {
    fn record_memory(&mut self, tool_name: &str, summary: &str) -> u64;
    fn decay_memories(&mut self);
    fn prune_if_needed(&mut self);
}

pub trait Bridgeable {
    fn bridge_name(&self) -> &str;
    fn poll_interval_ticks(&self) -> u64;
    async fn poll(&mut self) -> PvResult<()>;
}

pub trait Consentable {
    fn accepts_external_modulation(&self) -> bool;
    fn max_k_adjustment(&self) -> f64;
    fn accepts_cascade(&self) -> bool;
    fn accepts_observation(&self) -> bool;
}
```

## 2. Layer 2: Services (m07-m10)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m07_service_registry | L1 | ServiceInfo, ServiceRegistry | Registry map | 5 |
| m08_health_monitor | L1, m07 | HealthStatus, check_health() | None | 5 |
| m09_lifecycle | L1, m07 | start(), stop(), graceful_shutdown() | Process state | 5 |
| m10_api_server | L1, L3, L4, L5, L6, L7 | Router, build_routes() | None (routes handlers read/write shared state) | 20 |

**m10_api_server** has the highest fan-in after m01. It imports from nearly every layer
to build route handlers. All handlers receive `State<SharedState>` and optionally
`State<SharedBusState>`.

## 3. Layer 3: Field (m11-m15)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m11_sphere | L1 | PaneSphere, PaneStatus, SphereFieldContext, WorkSignature | Sphere fields | 25 |
| m12_field_state | L1, m11, m13, m16 | FieldState, FieldDecision, FieldAction, Tunnel, HarmonicSpectrum | None (computed) | 15 |
| m13_chimera | L1, m16 | ChimeraState, Cluster | None (computed) | 15 |
| m14_messaging | L1 | PhaseMessage (5 variants) | None (data types) | 5 |
| m15_app_state | L1, m11, m16 | AppState, SharedState, GhostTrace | AppState (spheres, network, tick, r_history) | 10 |

### m11_sphere — Core Data Structure (Fan-in: 8)

PaneSphere is the central entity. ~33 fields in v1. V2 organizes into sub-structs:

```rust
pub struct PaneSphere {
    // Identity
    pub id: PaneId,
    pub persona: String,
    // Oscillator (impl Oscillator trait)
    pub phase: Phase,
    pub frequency: Frequency,
    pub momentum: f64,
    // Memory (impl Learnable trait)
    pub memories: Vec<SphereMemory>,
    pub buoys: Vec<Buoy>,
    // Status
    pub status: PaneStatus,
    pub last_tool: String,
    pub total_steps: u64,
    pub has_worked: bool,
    pub work_signature: WorkSignature,
    // Consent (impl Consentable trait)
    pub receptivity: f64,
    pub opt_out_hebbian: bool,
    pub opt_out_cross_activation: bool,
    pub opt_out_external_modulation: bool,
    pub opt_out_observation: bool,
}
```

### m15_app_state — Shared State (Fan-in: 6)

```rust
pub struct AppState {
    pub spheres: HashMap<PaneId, PaneSphere>,
    pub network: CouplingNetwork,
    pub tick: u64,
    pub r_history: VecDeque<f64>,
    pub message_log: VecDeque<String>,
    pub state_changes: u32,
    pub dirty: bool,
    pub warmup_remaining: u32,
    pub divergence_ema: f64,
    pub conductor_integral: f64,
    pub ghosts: VecDeque<GhostTrace>,
    pub decision_history: VecDeque<FieldDecision>,
}

pub type SharedState = Arc<RwLock<AppState>>;
```

## 4. Layer 4: Coupling (m16-m18)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m16_coupling_network | L1, m11 | CouplingNetwork, Connection | phases, frequencies, connections, k | 20 |
| m17_auto_k | L1, m16 | auto_scale_k() | CouplingNetwork.k | 10 |
| m18_topology | L1, m16 | AdjacencyIndex, rebuild_index() | CouplingNetwork.adj_index | 5 |

### m16_coupling_network — State Owner

Owns the Kuramoto coupling state. Modified by:
- m17 (auto_scale_k)
- m19 (Hebbian weight updates)
- m31 (conductor k_modulation)
- m35 (tick loop orchestration)

## 5. Layer 5: Learning (m19-m21)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m19_hebbian_stdp | L1, m11, m16 | hebbian_ltp(), hebbian_ltd() | Connection weights | 15 |
| m20_buoy_network | L1, m11 | drift_buoys(), compute_co_activation() | Buoy positions | 10 |
| m21_memory_manager | L1, m11 | record_memory(), decay(), prune() | SphereMemory list | 10 |

## 6. Layer 6: Bridges (m22-m28)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m22_synthex_bridge | L1, m11 | ThermalState, SharedThermalState, poll_thermal() | SharedThermalState | 15 |
| m23_nexus_bridge | L1, m11 | NexusState, SharedNexusState, poll_nexus() | SharedNexusState | 15 |
| m24_me_bridge | L1, m11 | MeState, SharedMeState, poll_me() | SharedMeState | 10 |
| m25_povm_bridge | L1 | post_field_snapshot(), post_hebbian_weights(), hydrate() | None (write-only) | 10 |
| m26_rm_bridge | L1 | post_to_rm(), bootstrap_from_rm() | None (write-only, TSV) | 5 |
| m27_vms_bridge | L1 | post_to_vms() | None (write-only) | 5 |
| m28_consent_gate | L1, m11, m15 | consent_gated_k_adjustment(), fleet_mean_consent() | None (pure computation) | 10 |

### Bridge Pattern (all bridges follow this)

```rust
// Raw TCP HTTP — no hyper dependency
async fn raw_http_get(addr: &str, path: &str) -> PvResult<String> {
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    // Read response, parse body
}
```

### m28_consent_gate — Critical Module

Every external bridge influence passes through this module:

```rust
pub fn consent_gated_k_adjustment(
    raw_influence: f64,
    spheres: &HashMap<PaneId, PaneSphere>,
    budget: (f64, f64), // (min, max)
) -> f64 {
    let consent_scale = fleet_mean_consent(spheres);
    (raw_influence * consent_scale).clamp(budget.0, budget.1)
}
```

## 7. Layer 7: Coordination (m29-m36)

| Module | Imports From | Exports | State Modified | Tests Target |
|--------|-------------|---------|----------------|-------------|
| m29_ipc_bus | L1, m30 | start_bus_listener(), handle_connection() | BusState | 10 |
| m30_bus_types | L1 | BusFrame, BusTask, BusEvent, BusState, SharedBusState | None (data types) | 10 |
| m31_conductor | L1, m15, m16, m28 | conduct_breathing(), hebbian_learning() | k_modulation, weights | 10 |
| m32_executor | L1, m15 | dispatch_to_pane(), ExecutorRegistry | None (dispatches commands) | 10 |
| m33_cascade | L1, m30 | dispatch_cascade(), handle_cascade_ack() | BusState.cascade_events | 5 |
| m34_suggestions | L1, m12, m15 | generate_suggestions() | None (pure computation) | 5 |
| m35_tick | L1, L3, L4, L5, L6, m31, m36 | tick_once() | AppState (orchestrates all) | 15 |
| m36_persistence | L1, m30, m15 | write_snapshot(), load_snapshot(), persist_task() | Database files | 10 |

### m35_tick — The Orchestrator (Fan-in: highest complexity)

`tick_once()` is the heart of the daemon. It runs every 5 seconds and orchestrates:

```
tick_once() phases:
  1. Bridge polls (SYNTHEX, Nexus, ME) — read external state
  2. Coupling steps — Kuramoto integration
  3. Sphere steps — memory decay, status update, frequency modulation
  4. Hebbian learning — LTP/LTD weight updates
  5. Field computation — order parameter, chimera, tunnels, decision
  6. Conductor — PI control, k_modulation update
  7. Bus events — broadcast field events to subscribers
  8. Persistence — snapshot if dirty + interval elapsed
  9. Bridge writes — POVM snapshot, RM post, VMS post
```

**v1 note:** tick_once was 829 lines (65 branches) in v1. V2 decomposes into
named functions called from m35_tick.

### Lock Acquisition Order (CRITICAL)

Any function that needs both locks MUST acquire in this order:

1. `app_state.read()` or `app_state.write()`
2. `bus_state.read()` or `bus_state.write()`

Functions that acquire both locks:
- `m35_tick::tick_once()` — writes AppState, reads BusState for events
- `m10_api_server::register_sphere()` — writes AppState, notifies BusState
- `m10_api_server::submit_task()` — reads AppState for routing, writes BusState
- `m33_cascade::dispatch_cascade()` — reads AppState for targets, writes BusState

## 8. Layer 8: Governance (m37-m41) — Feature-Gated

| Module | Imports From | Exports | State Modified | Feature Gate | Tests Target |
|--------|-------------|---------|----------------|-------------|-------------|
| m37_proposals | L1, m15, m36 | create_proposal(), resolve_proposals() | Database (proposals table) | governance | 10 |
| m38_voting | L1, m15, m37 | cast_vote(), tally_votes() | Database (votes table) | governance | 10 |
| m39_consent_declaration | L1, m11, m36 | declare_consent(), load_consent() | Database (consent_declarations) | governance | 10 |
| m40_data_sovereignty | L1, m36 | scan_manifests(), forget_sphere_data() | Database + external systems | governance | 10 |
| m41_evolution_chamber | L1, L3, L4 | observe(), population(), history() | In-memory evolution state | evolution | 10 |

## 9. Data Flow Diagram

```
External Services              Pane-Vortex V2                    Claude Instances
=================              ==============                    ================

SYNTHEX (:8090) ---thermal---> m22 ---> m28 (consent gate)
                                              |
Nexus (:8100) ----strategy---> m23 ---> m28 --+--> m31 (conductor)
                                              |         |
ME (:8080) -------fitness----> m24 ---> m28 --+    k_modulation
                                                    |
                                              m16 (coupling)
                                                    |
                                              m35 (tick) <---+--- m10 (API) <--- HTTP
                                                    |        |
                                              m12 (field) ---+--- m29 (bus) <--- IPC
                                                    |
                                              m36 (persist) --> SQLite
                                                    |
POVM (:8125) <----snapshots--- m25              m32 (executor) --> Zellij
RM (:8130) <------TSV--------- m26
VMS (:8120) <-----state------- m27
```

## 10. Risk Hotspots

From v1 Session 039 analysis:

| Module | Risk | Reason |
|--------|------|--------|
| m01_core_types | HIGH | Fan-in=ALL; any change ripples everywhere |
| m11_sphere | HIGH | Fan-in=8; god-object risk (33 fields in v1) |
| m35_tick | HIGH | Orchestrates everything; 829L in v1 |
| m15_app_state | MEDIUM | Central mutable state; lock contention |
| m16_coupling_network | MEDIUM | O(N^2) connections; performance sensitive |
| m12_field_state | MEDIUM | Test gap in v1; 3 tests for 310 LOC |
| m28_consent_gate | LOW | Pure computation; well-tested in v1 |

## 11. Summary Statistics

| Metric | Value |
|--------|-------|
| Total modules | 41 |
| Layers | 8 |
| Feature-gated modules | 5 (governance) + 1 (evolution) |
| Cross-layer imports | L1 imported by all; L3 imported by L4-L8 |
| Shared state types | 2 (SharedState, SharedBusState) |
| Bridge modules | 7 (6 services + consent gate) |
| Test target total | ~350+ |
