---
title: "Pane-Vortex V2 — Code Module Map"
date: 2026-03-19
tags: [modules, code-map, pane-vortex-v2, reference]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
---

# Pane-Vortex V2 — Code Module Map

> All 41 modules with layer, purpose, target LOC, dependencies, and key types/functions.
> Status: SCAFFOLDED — types and functions listed are design targets, not yet implemented.

---

## How to Read This Map

Each module entry contains:
- **Layer** — which of the 8 layers it belongs to
- **Source** — file path relative to project root
- **LOC Target** — approximate lines of code when fully implemented
- **Depends On** — modules this module imports from
- **Key Types** — structs, enums, traits to be defined here
- **Key Functions** — public functions to be implemented here
- **Tests** — minimum test count target
- **V3 Phase** — which V3 plan phase drives implementation

---

## Layer 1: Foundation (m01-m06)

### m01 — Core Types

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m01_core_types.rs` |
| LOC Target | ~300 |
| Depends On | (none) |
| Key Types | `PaneId(String)`, `Phase(f64)`, `Frequency(f64)`, `Point3D { x, y, z }`, `SphereMemory { tool, phase, timestamp, activation }`, `Buoy { position, activation, label, decay }`, `SphereStatus { Idle, Working, Complete, Blocked }`, `WorkSignature { tool_histogram, burst_count }` |
| Key Functions | `PaneId::new(s) -> PvResult<PaneId>`, `Phase::wrap(f64) -> Phase`, `Frequency::clamp(f64) -> Frequency` |
| Tests | 15+ |
| V3 Phase | V3.2 |

### m02 — Error Handling

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m02_error_handling.rs` |
| LOC Target | ~200 |
| Depends On | (none) |
| Key Types | `PvError { Field(FieldError), Bridge(BridgeError), Bus(BusError), Persistence(PersistenceError), Validation(ValidationError), Config(ConfigError) }`, `PvResult<T> = Result<T, PvError>` |
| Key Functions | `impl From<rusqlite::Error>`, `impl From<serde_json::Error>`, `impl From<std::io::Error>`, `impl Display`, `impl Error` |
| Tests | 10+ |
| V3 Phase | V3.2 |

### m03 — Config

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m03_config.rs` |
| LOC Target | ~250 |
| Depends On | m02 |
| Key Types | `PvConfig { server, field, sphere, coupling, learning, bridges, conductor, ipc, persistence, governance }`, `ServerConfig`, `FieldConfig`, `SphereConfig`, `CouplingConfig`, `LearningConfig`, `BridgeConfig`, `ConductorConfig`, `IpcConfig`, `PersistenceConfig`, `GovernanceConfig` |
| Key Functions | `PvConfig::load() -> PvResult<PvConfig>`, `PvConfig::load_from(path) -> PvResult<PvConfig>`, `PvConfig::validate() -> PvResult<()>` |
| Tests | 10+ |
| V3 Phase | V3.2 |

### m04 — Constants

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m04_constants.rs` |
| LOC Target | ~100 |
| Depends On | (none) |
| Key Types | (constants only, no types) |
| Key Functions | All `pub const`: `TICK_INTERVAL`, `COUPLING_STEPS_PER_TICK`, `KURAMOTO_DT`, `HEBBIAN_LTP`, `HEBBIAN_LTD`, `PHASE_GAP_THRESHOLD`, `SYNC_THRESHOLD`, `TUNNEL_THRESHOLD`, `R_TARGET`, `K_MOD_BUDGET_MIN`, `K_MOD_BUDGET_MAX`, `SPHERE_CAP`, `MEMORY_MAX_COUNT`, `SNAPSHOT_INTERVAL`, `TAU` |
| Tests | 5+ (compile-time assertions) |
| V3 Phase | V3.2 |

### m05 — Traits

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m05_traits.rs` |
| LOC Target | ~150 |
| Depends On | m01, m02 |
| Key Types | `trait Oscillator { fn phase(&self) -> Phase; fn frequency(&self) -> Frequency; fn step(&mut self, dt: f64, influence: f64); }`, `trait Learnable { fn ltp(&mut self, other: &Self, rate: f64); fn ltd(&mut self, other: &Self, rate: f64); }`, `trait Bridgeable { fn poll(&self) -> impl Future<Output = PvResult<()>>; fn health(&self) -> bool; }`, `trait Consentable { fn accepts_modulation(&self) -> bool; fn max_k_adj(&self) -> f64; fn consent_scale(&self) -> f64; }` |
| Key Functions | Trait method definitions (see types) |
| Tests | 8+ (trait object tests with mock impls) |
| V3 Phase | V3.2 |

### m06 — Validation

| Field | Value |
|-------|-------|
| Layer | L1 Foundation |
| Source | `src/m1_foundation/m06_validation.rs` |
| LOC Target | ~100 |
| Depends On | m01, m02, m04 |
| Key Types | (functions only) |
| Key Functions | `validate_phase(f64) -> PvResult<Phase>`, `validate_frequency(f64) -> PvResult<Frequency>`, `validate_sphere_id(&str) -> PvResult<PaneId>`, `validate_string_length(&str, max: usize) -> PvResult<&str>`, `validate_k_mod(f64) -> PvResult<f64>`, `is_finite_or_err(f64, name: &str) -> PvResult<f64>` |
| Tests | 12+ |
| V3 Phase | V3.2 |

---

## Layer 2: Services (m07-m10)

### m07 — Service Registry

| Field | Value |
|-------|-------|
| Layer | L2 Services |
| Source | `src/m2_services/m07_service_registry.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02 |
| Key Types | `ServiceEntry { id, port, health_path, batch, status }`, `ServiceRegistry { services: HashMap<String, ServiceEntry> }` |
| Key Functions | `ServiceRegistry::new() -> Self` (pre-populated with 16 ULTRAPLATE services), `get(id) -> Option<&ServiceEntry>`, `health_url(id) -> String`, `by_batch(n) -> Vec<&ServiceEntry>` |
| Tests | 8+ |
| V3 Phase | V3.1 |

### m08 — Health Monitor

| Field | Value |
|-------|-------|
| Layer | L2 Services |
| Source | `src/m2_services/m08_health_monitor.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m07 |
| Key Types | `HealthStatus { service_id, http_code, latency_ms, last_checked, stale }`, `CircuitBreaker { state, failure_count, last_failure }`, `CircuitState { Closed, Open, HalfOpen }` |
| Key Functions | `check_health(entry: &ServiceEntry) -> PvResult<HealthStatus>`, `check_all(registry: &ServiceRegistry) -> Vec<HealthStatus>`, `is_stale(status: &HealthStatus, max_age_secs: u64) -> bool` |
| Tests | 10+ |
| V3 Phase | V3.1 |

### m09 — Lifecycle

| Field | Value |
|-------|-------|
| Layer | L2 Services |
| Source | `src/m2_services/m09_lifecycle.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02 |
| Key Types | `ServiceLifecycle { pid, started_at, restarts }`, `ShutdownSignal` |
| Key Functions | `graceful_shutdown(timeout: Duration) -> PvResult<()>`, `install_signal_handlers() -> PvResult<ShutdownSignal>`, `write_pid_file(path: &Path) -> PvResult<()>`, `remove_pid_file(path: &Path) -> PvResult<()>` |
| Tests | 8+ |
| V3 Phase | V3.1 |

### m10 — API Server

| Field | Value |
|-------|-------|
| Layer | L2 Services |
| Source | `src/m2_services/m10_api_server.rs` |
| LOC Target | ~400 |
| Depends On | m01, m02, m03 (+ all route handlers from L3-L8) |
| Feature Gate | `api` |
| Key Types | `ApiState(Arc<AppState>)` |
| Key Functions | `build_router(state: ApiState) -> Router`, `health_handler() -> Json<Value>`, `sphere_router() -> Router`, `field_router() -> Router`, `bus_router() -> Router`, `bridge_router() -> Router`, `governance_router() -> Router` (feature-gated) |
| Tests | 15+ |
| V3 Phase | V3.2 |

---

## Layer 3: Field (m11-m15)

### m11 — Sphere

| Field | Value |
|-------|-------|
| Layer | L3 Field |
| Source | `src/m3_field/m11_sphere.rs` |
| LOC Target | ~500 |
| Depends On | m01, m02, m04, m05 |
| Key Types | `PaneSphere { id, phase, frequency, status, persona, memories, buoys, self_model, preferences, inbox, step_count, created_at }`, `SelfModel { is_synchronized, tunnel_count, total_steps, maturity, age_secs }`, `Maturity { Newcomer, Established, Senior }`, `CouplingPreferences { opt_out_hebbian, opt_out_cross_activation, max_k_adj, accept_observation }`, `InboxMessage { id, from, content, timestamp }` |
| Key Functions | `PaneSphere::new(id, persona) -> Self`, `step(&mut self, dt, influence)`, `add_memory(tool, phase)`, `recall(near_phase, near_buoy) -> Vec<SphereMemory>`, `summary() -> SphereSummary`, `set_status(status)`, `receive_message(msg)`, `acknowledge_message(msg_id)`, `update_self_model(field_context)`, `prune_memories()` |
| Tests | 25+ |
| V3 Phase | V3.2 |

### m12 — Field State

| Field | Value |
|-------|-------|
| Layer | L3 Field |
| Source | `src/m3_field/m12_field_state.rs` |
| LOC Target | ~400 |
| Depends On | m01, m02, m04 |
| Key Types | `FieldState { r, r_history, k, k_mod, effective_k, phase_spread, mean_phase, sphere_count, idle_count, working_count, blocked_count, tunnels, chimera_detected, modulation_breakdown }`, `FieldDecision { action, targets, rationale, attribution }`, `DecisionAction { Stable, NeedsCoherence, NeedsDivergence, HasBlockedAgents, IdleFleet, FreshFleet, Recovering }`, `Tunnel { sphere_a, sphere_b, phase_diff, label }`, `GhostTrace { id, persona, weights, memories, departed_at, consent_given }`, `ModulationBreakdown { synthex, nexus, me, conductor, consent_scale }` |
| Key Functions | `FieldState::compute(spheres: &HashMap<PaneId, PaneSphere>) -> Self`, `detect_tunnels(spheres, weights) -> Vec<Tunnel>`, `field_summary() -> FieldSummary` |
| Tests | 20+ |
| V3 Phase | V3.2 |

### m13 — Chimera

| Field | Value |
|-------|-------|
| Layer | L3 Field |
| Source | `src/m3_field/m13_chimera.rs` |
| LOC Target | ~200 |
| Depends On | m01, m04 |
| Key Types | `ChimeraResult { detected, cluster_count, clusters: Vec<PhaseCluster> }`, `PhaseCluster { sphere_ids: Vec<PaneId>, mean_phase, spread }` |
| Key Functions | `detect_chimera(phases: &[(PaneId, f64)], threshold: f64) -> ChimeraResult` |
| Tests | 10+ |
| V3 Phase | V3.2 |

### m14 — Messaging

| Field | Value |
|-------|-------|
| Layer | L3 Field |
| Source | `src/m3_field/m14_messaging.rs` |
| LOC Target | ~150 |
| Depends On | m01, m02 |
| Key Types | `PhaseMessage { Steer { target, phase }, CrossActivation { source, targets, strength }, EmergencyCoherence { targets }, SemanticNudge { target, tool, phase_region }, FieldBroadcast { action, data } }` |
| Key Functions | `PhaseMessage::validate(&self) -> PvResult<()>`, `apply_steer(sphere, msg)`, `apply_cross_activation(spheres, msg)`, `apply_emergency_coherence(spheres, msg)` |
| Tests | 8+ |
| V3 Phase | V3.2 |

### m15 — App State

| Field | Value |
|-------|-------|
| Layer | L3 Field |
| Source | `src/m3_field/m15_app_state.rs` |
| LOC Target | ~500 |
| Depends On | m01, m02, m11, m12 |
| Key Types | `AppState { spheres: RwLock<HashMap<PaneId, PaneSphere>>, field: RwLock<FieldState>, ghosts: RwLock<VecDeque<GhostTrace>>, config: PvConfig, tick: AtomicU64, started_at: Instant, warmup_remaining: AtomicU32, dirty: AtomicBool }`, `SharedState { app: Arc<AppState>, bus: Arc<BusState> }` |
| Key Functions | `AppState::new(config) -> Self`, `register_sphere(id, persona) -> PvResult<()>`, `deregister_sphere(id) -> PvResult<GhostTrace>`, `snapshot() -> FieldSnapshot`, `restore_snapshot(snapshot) -> PvResult<()>`, `reconcile_memory_ids()` |
| Tests | 20+ |
| V3 Phase | V3.2 |

---

## Layer 4: Coupling (m16-m18)

### m16 — Coupling Network

| Field | Value |
|-------|-------|
| Layer | L4 Coupling |
| Source | `src/m4_coupling/m16_coupling_network.rs` |
| LOC Target | ~300 |
| Depends On | m01, m02, m04, m11 |
| Key Types | `CouplingNetwork { weights: HashMap<(PaneId, PaneId), f64>, k: f64, k_mod: f64 }` |
| Key Functions | `step(spheres: &mut HashMap<PaneId, PaneSphere>, network: &CouplingNetwork, dt: f64, steps: usize)`, `get_weight(a, b) -> f64`, `set_weight(a, b, w)`, `effective_k() -> f64`, `per_sphere_influence(id, spheres) -> f64` |
| Tests | 15+ |
| V3 Phase | V3.2 |

### m17 — Auto-K

| Field | Value |
|-------|-------|
| Layer | L4 Coupling |
| Source | `src/m4_coupling/m17_auto_k.rs` |
| LOC Target | ~200 |
| Depends On | m01, m04, m16 |
| Key Types | `AutoKState { last_recalc_tick, mean_effective_weight }` |
| Key Functions | `should_recalculate(tick: u64, period: u64) -> bool`, `auto_scale_k(network: &mut CouplingNetwork, multiplier: f64) -> f64` |
| Tests | 8+ |
| V3 Phase | V3.2 |

### m18 — Topology

| Field | Value |
|-------|-------|
| Layer | L4 Coupling |
| Source | `src/m4_coupling/m18_topology.rs` |
| LOC Target | ~150 |
| Depends On | m01, m16 |
| Key Types | `Neighborhood { sphere_id, neighbors: Vec<(PaneId, f64)>, strongest_neighbor, mean_weight }` |
| Key Functions | `neighbors(id, network) -> Neighborhood`, `weight_squared_amplification(w: f64) -> f64`, `topology_stats(network) -> TopologyStats` |
| Tests | 8+ |
| V3 Phase | V3.2 |

---

## Layer 5: Learning (m19-m21)

### m19 — Hebbian STDP

| Field | Value |
|-------|-------|
| Layer | L5 Learning |
| Source | `src/m5_learning/m19_hebbian_stdp.rs` |
| LOC Target | ~300 |
| Depends On | m01, m04, m11, m16 |
| Key Types | `HebbianConfig { ltp_rate, ltd_rate, burst_multiplier, newcomer_multiplier, weight_floor }`, `LearningEvent { sphere_a, sphere_b, delta_w, reason }` |
| Key Functions | `apply_ltp(network, a, b, config, is_burst, is_newcomer) -> f64`, `apply_ltd(network, a, b, config) -> f64`, `hebbian_step(spheres, network, config) -> Vec<LearningEvent>`, `detect_burst(sphere) -> bool`, `is_newcomer(sphere) -> bool` |
| Tests | 15+ |
| V3 Phase | V3.2 |

### m20 — Buoy Network

| Field | Value |
|-------|-------|
| Layer | L5 Learning |
| Source | `src/m5_learning/m20_buoy_network.rs` |
| LOC Target | ~200 |
| Depends On | m01, m04 |
| Key Types | `BuoyNetwork { buoys: Vec<Buoy> }` |
| Key Functions | `nearest_buoy(position: Point3D) -> Option<&Buoy>`, `activate_buoy(idx, amount)`, `decay_all(rate: f64)`, `prune_below(threshold: f64)`, `add_buoy(position, label)` |
| Tests | 10+ |
| V3 Phase | V3.2 |

### m21 — Memory Manager

| Field | Value |
|-------|-------|
| Layer | L5 Learning |
| Source | `src/m5_learning/m21_memory_manager.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m04, m11 |
| Key Types | `MemoryManager { max_count, prune_interval, next_prune_step }` |
| Key Functions | `should_prune(step: u64) -> bool`, `prune_sphere_memories(sphere: &mut PaneSphere) -> usize`, `amortised_batch_prune(sphere: &mut PaneSphere, threshold_plus: usize) -> usize`, `reconcile_ids(sphere: &mut PaneSphere)`, `narrative(sphere: &PaneSphere) -> String` |
| Tests | 10+ |
| V3 Phase | V3.2 |

---

## Layer 6: Bridges (m22-m28)

### m22 — SYNTHEX Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m22_synthex_bridge.rs` |
| LOC Target | ~300 |
| Depends On | m01, m02, m04, m05, m28 |
| Key Types | `SynthexThermal { temperature, target, heat_sources, synergy }`, `SynthexKAdj { raw_adj, consent_scaled_adj }` |
| Key Functions | `poll_thermal(addr: &str) -> PvResult<SynthexThermal>`, `compute_k_adjustment(thermal: &SynthexThermal) -> f64`, `post_field_state(addr, state) -> PvResult<()>` |
| Tests | 10+ |
| V3 Phase | V3.1-V3.3 |

### m23 — Nexus Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m23_nexus_bridge.rs` |
| LOC Target | ~400 |
| Depends On | m01, m02, m04, m05, m28 |
| Key Types | `NexusState { strategy, r_inner, r_outer, dispatch_confidence, modules_active }`, `NexusMetrics { strategy, r_inner, r_outer, dispatch_confidence }` |
| Key Functions | `poll_nexus(addr: &str) -> PvResult<NexusState>`, `compute_k_adjustment(state: &NexusState) -> f64`, `nexus_command(addr, cmd, params) -> PvResult<Value>` |
| Tests | 12+ |
| V3 Phase | V3.1-V3.3 |

### m24 — ME Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m24_me_bridge.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m04, m05, m28 |
| Key Types | `MeObserver { fitness, correlations_found, last_report }`, `MeKAdj { raw_adj, consent_scaled_adj }` |
| Key Functions | `poll_fitness(addr: &str) -> PvResult<MeObserver>`, `compute_k_adjustment(observer: &MeObserver) -> f64` |
| Tests | 8+ |
| V3 Phase | V3.1 |

### m25 — POVM Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m25_povm_bridge.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m04 |
| Key Types | `PovmSnapshot { tick, r, k, sphere_count, weights }` |
| Key Functions | `post_field_snapshot(addr, snapshot) -> PvResult<()>`, `post_hebbian_weights(addr, weights) -> PvResult<()>`, `hydrate_pathways(addr) -> PvResult<Vec<(PaneId, PaneId, f64)>>`, `hydrate_summary(addr) -> PvResult<Value>` |
| Tests | 8+ |
| V3 Phase | V3.2 |

### m26 — RM Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m26_rm_bridge.rs` |
| LOC Target | ~150 |
| Depends On | m01, m02, m04 |
| Key Types | (no types — functions only) |
| Key Functions | `post_tsv(addr: &str, category: &str, agent: &str, confidence: f64, ttl: u64, content: &str) -> PvResult<()>`, `post_field_state(addr, state) -> PvResult<()>`, `post_conductor_decision(addr, decision) -> PvResult<()>` |
| Tests | 6+ |
| V3 Phase | V3.5 |

### m27 — VMS Bridge

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m27_vms_bridge.rs` |
| LOC Target | ~150 |
| Depends On | m01, m02, m04 |
| Key Types | (no types — functions only) |
| Key Functions | `post_field_memory(addr, state) -> PvResult<()>`, `seed_from_vms(addr) -> PvResult<Option<FieldState>>` |
| Tests | 6+ |
| V3 Phase | V3.5 |

### m28 — Consent Gate

| Field | Value |
|-------|-------|
| Layer | L6 Bridges |
| Source | `src/m6_bridges/m28_consent_gate.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m04, m11 |
| Key Types | `ConsentDecision { accepted, raw_adj, scaled_adj, reason }`, `KModBudget { min, max, current }` |
| Key Functions | `consent_gated_k_adjustment(sphere: &PaneSphere, raw_adj: f64, source: &str) -> ConsentDecision`, `clamp_budget(total: f64, budget: &KModBudget) -> f64`, `modulation_breakdown(adjustments: &[(String, f64)]) -> ModulationBreakdown` |
| Tests | 12+ |
| V3 Phase | V3.3 |

---

## Layer 7: Coordination (m29-m36)

### m29 — IPC Bus

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m29_ipc_bus.rs` |
| LOC Target | ~500 |
| Depends On | m01, m02, m04, m30 |
| Key Types | `BusState { tasks, events, subscriptions, connections, event_tx }`, `BusConnection { sphere_id, writer, subscriptions }` |
| Key Functions | `start_bus_listener(socket_path, state) -> PvResult<JoinHandle>`, `handle_connection(stream, state)`, `process_frame(frame: BusFrame, state) -> PvResult<Option<BusFrame>>`, `broadcast_event(event, state)`, `match_subscription(pattern, event_type) -> bool` |
| Tests | 20+ |
| V3 Phase | V3.2 |

### m30 — Bus Types

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m30_bus_types.rs` |
| LOC Target | ~300 |
| Depends On | m01, m02 |
| Key Types | `BusFrame { type_: FrameType, ... }`, `FrameType { Handshake, Submit, Claim, Complete, Fail, Subscribe, Unsubscribe, Event, CascadeHandoff, CascadeAck, CascadeReject, Request, Response, Error }`, `BusTask { id, status, source, target, target_type, description, payload, claimed_by }`, `TaskTarget { Specific(PaneId), AnyIdle, FieldDriven, Willing }`, `TaskStatus { Submitted, Claimed, Completed, Failed, Expired }`, `BusEvent { event_type, source, data, tick }` |
| Key Functions | `BusFrame::parse(line: &str) -> PvResult<BusFrame>`, `BusFrame::serialize(&self) -> String`, `BusTask::new(source, target, desc) -> Self`, `BusTask::transition(status) -> PvResult<()>` |
| Tests | 15+ |
| V3 Phase | V3.2 |

### m31 — Conductor

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m31_conductor.rs` |
| LOC Target | ~400 |
| Depends On | m01, m02, m04, m12, m28 |
| Key Types | `ConductorState { r_target, gain, breathing_blend, divergence_cooldown, last_decision, k_adjustments }`, `DecisionRecord { action, rationale, attribution, tick, modulation_breakdown }` |
| Key Functions | `decide(field: &FieldState, spheres: &HashMap<PaneId, PaneSphere>) -> FieldDecision`, `pi_control(r_current, r_target, gain) -> f64`, `apply_divergence_cooldown(state, decision) -> FieldDecision`, `record_decision(decision, state)` |
| Tests | 15+ |
| V3 Phase | V3.2-V3.3 |

### m32 — Executor

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m32_executor.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m12, m15 |
| Key Types | `PaneMapping { tab, pane, label, sphere_id }`, `DispatchResult { success, latency_ms, output }` |
| Key Functions | `dispatch(mapping: &PaneMapping, command: &str) -> PvResult<DispatchResult>`, `load_pane_map() -> PvResult<Vec<PaneMapping>>`, `score_dispatch(result: &DispatchResult) -> f64` |
| Tests | 8+ |
| V3 Phase | V3.2 |

### m33 — Cascade

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m33_cascade.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m30 |
| Key Types | `CascadeHandoff { source, target, brief, depth }`, `CascadeAck { source, target, status }`, `CascadeStatus { Dispatched, Acked, Rejected }` |
| Key Functions | `dispatch_cascade(handoff: CascadeHandoff, state) -> PvResult<()>`, `acknowledge_cascade(ack: CascadeAck, state)`, `check_rate_limit(source, limit) -> bool`, `write_fallback_brief(handoff) -> PvResult<PathBuf>` |
| Tests | 8+ |
| V3 Phase | V3.3 |

### m34 — Suggestions

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m34_suggestions.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m12, m11 |
| Key Types | `FieldSuggestion { suggestion_type, targets, rationale, confidence }`, `SuggestionType { Diverge, Cohere, Rebalance, Investigate }` |
| Key Functions | `generate_suggestions(field: &FieldState, spheres: &HashMap<PaneId, PaneSphere>) -> Vec<FieldSuggestion>`, `filter_eligible_spheres(suggestion, spheres) -> Vec<PaneId>` |
| Tests | 8+ |
| V3 Phase | V3.2 |

### m35 — Tick Orchestrator

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m35_tick.rs` |
| LOC Target | ~300 |
| Depends On | m01, m02, m11, m12, m16, m19, m22-m28, m31, m34, m36 |
| Key Types | `TickPhase { BridgePoll, FieldUpdate, Learning, Decision, Persistence }`, `TickMetrics { phase_durations, total_ms }` |
| Key Functions | `tick_orchestrator(state: &SharedState) -> PvResult<TickMetrics>`, `phase_bridge_poll(state)`, `phase_field_update(state)`, `phase_learning(state)`, `phase_decision(state)`, `phase_persistence(state)` |
| Tests | 12+ |
| V3 Phase | V3.2 |

### m36 — Persistence

| Field | Value |
|-------|-------|
| Layer | L7 Coordination |
| Source | `src/m7_coordination/m36_persistence.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m04, m12, m30 |
| Feature Gate | `persistence` |
| Key Types | `PersistenceManager { field_db, bus_db }`, `FieldSnapshot { tick, r, k, k_mod, sphere_count, ... }` |
| Key Functions | `PersistenceManager::new(config) -> PvResult<Self>`, `save_field_snapshot(snapshot)`, `save_bus_event(event)`, `save_sphere_history(sphere_id, event_type, tick)`, `load_latest_snapshot() -> PvResult<Option<FieldSnapshot>>`, `run_migrations() -> PvResult<()>` |
| Tests | 10+ |
| V3 Phase | V3.2 |

---

## Layer 8: Governance (m37-m41)

### m37 — Proposals

| Field | Value |
|-------|-------|
| Layer | L8 Governance |
| Source | `src/m8_governance/m37_proposals.rs` |
| LOC Target | ~300 |
| Depends On | m01, m02, m04, m12 |
| Feature Gate | `governance` |
| Key Types | `Proposal { id, proposer, parameter, current_value, proposed_value, rationale, status, votes_for, votes_against, votes_abstain, quorum_threshold, voting_deadline_tick }`, `ProposalStatus { Open, Approved, Rejected, Expired }` |
| Key Functions | `create_proposal(proposer, parameter, proposed_value, rationale, config) -> PvResult<Proposal>`, `check_quorum(proposal, active_sphere_count) -> bool`, `resolve_proposal(proposal) -> ProposalStatus`, `apply_approved(proposal, state) -> PvResult<()>`, `list_proposals(status_filter) -> Vec<Proposal>` |
| Tests | 12+ |
| V3 Phase | V3.4 |

### m38 — Voting

| Field | Value |
|-------|-------|
| Layer | L8 Governance |
| Source | `src/m8_governance/m38_voting.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m37 |
| Feature Gate | `governance` |
| Key Types | `Vote { proposal_id, sphere_id, vote: VoteChoice }`, `VoteChoice { Approve, Reject, Abstain }` |
| Key Functions | `cast_vote(proposal_id, sphere_id, choice) -> PvResult<()>`, `has_voted(proposal_id, sphere_id) -> bool`, `tally(proposal_id) -> (u32, u32, u32)`, `check_and_resolve(proposal_id, active_count) -> PvResult<Option<ProposalStatus>>` |
| Tests | 10+ |
| V3 Phase | V3.4 |

### m39 — Consent Declaration

| Field | Value |
|-------|-------|
| Layer | L8 Governance |
| Source | `src/m8_governance/m39_consent_declaration.rs` |
| LOC Target | ~200 |
| Depends On | m01, m02, m11 |
| Feature Gate | `governance` |
| Key Types | `ConsentDeclaration { sphere_id, accept_external_modulation, max_k_adjustment, accept_cascade, accept_observation, accept_nvim_monitoring }` |
| Key Functions | `declare_consent(sphere_id, declaration) -> PvResult<()>`, `get_consent(sphere_id) -> ConsentDeclaration`, `default_consent() -> ConsentDeclaration`, `update_consent(sphere_id, partial) -> PvResult<ConsentDeclaration>` |
| Tests | 8+ |
| V3 Phase | V3.3-V3.4 |

### m40 — Data Sovereignty

| Field | Value |
|-------|-------|
| Layer | L8 Governance |
| Source | `src/m8_governance/m40_data_sovereignty.rs` |
| LOC Target | ~250 |
| Depends On | m01, m02, m36 |
| Feature Gate | `governance` |
| Key Types | `DataManifest { sphere_id, systems: Vec<SystemRecord> }`, `SystemRecord { system_name, record_count, last_scanned }`, `ForgetRequest { sphere_id, systems: Vec<String> }` |
| Key Functions | `enumerate_data(sphere_id) -> PvResult<DataManifest>`, `forget(request: ForgetRequest) -> PvResult<ForgetResult>`, `scan_field_db(sphere_id) -> PvResult<SystemRecord>`, `scan_bus_db(sphere_id) -> PvResult<SystemRecord>` |
| Tests | 10+ |
| V3 Phase | V3.3-V3.4 |

### m41 — Evolution Chamber

| Field | Value |
|-------|-------|
| Layer | L8 Governance |
| Source | `src/m8_governance/m41_evolution_chamber.rs` |
| LOC Target | ~400 |
| Depends On | m01, m02, m12, m16 |
| Feature Gate | `evolution` |
| Key Types | `Observation { tick, r, chimera, decision, coupling_stats }`, `AnomalyScore { value, category, description }`, `EmergencePattern { pattern_type, confidence, first_seen, recurrence }` |
| Key Functions | `observe(state: &FieldState) -> Observation`, `score_anomaly(observations: &[Observation]) -> Vec<AnomalyScore>`, `detect_patterns(observations: &[Observation]) -> Vec<EmergencePattern>`, `evolution_status() -> EvolutionSummary` |
| Tests | 15+ |
| V3 Phase | V3.4 |

---

## Totals

| Layer | Modules | Target LOC | Target Tests |
|-------|---------|-----------|-------------|
| L1 Foundation | 6 | ~1,100 | 60+ |
| L2 Services | 4 | ~1,050 | 41+ |
| L3 Field | 5 | ~1,750 | 83+ |
| L4 Coupling | 3 | ~650 | 31+ |
| L5 Learning | 3 | ~750 | 35+ |
| L6 Bridges | 7 | ~1,650 | 62+ |
| L7 Coordination | 8 | ~2,350 | 91+ |
| L8 Governance | 5 | ~1,400 | 55+ |
| **Total** | **41** | **~10,700** | **458+** |

---

## Cross-References

- **[ARCHITECTURE_DEEP_DIVE.md](ARCHITECTURE_DEEP_DIVE.md)** — System-level architecture
- **[modules/](modules/)** — Per-layer detailed documentation
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan phases
- **[.claude/context.json](../.claude/context.json)** — Machine-readable module inventory
- **Obsidian:** `[[Session 039 — Architectural Schematics and Refactor Safety]]`
- **V1 Module Map:** `~/claude-code-workspace/pane-vortex/ai_docs/CODE_MODULE_MAP.md`
