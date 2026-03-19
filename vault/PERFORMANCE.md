---
title: "Pane-Vortex V2 — Performance Targets"
date: 2026-03-19
tags: [performance, latency, complexity, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
---

# Pane-Vortex V2 — Performance Targets

> Latency budgets, complexity constraints, and benchmark targets.
> Based on V1 measured performance and V3 plan requirements.

---

## Latency Budget

### Tick Loop (5-second interval)

The tick_orchestrator must complete well within the 5-second tick interval:

| Phase | Target | V1 Measured | Notes |
|-------|--------|-------------|-------|
| Bridge Polling | <100ms | ~50ms | Fire-and-forget spawns, non-blocking |
| Field Update (coupling) | <20ms | ~5ms at N=10 | 15 Jacobi steps x dt=0.01 |
| Learning (Hebbian) | <10ms | ~3ms | LTP/LTD scan of active pairs |
| Decision Engine | <5ms | ~1ms | Priority chain + PI controller |
| Persistence | <10ms | ~5ms | SQLite WAL write |
| **tick_orchestrator total** | **<50ms** | **~15ms** | V1 was ~5ms for tick_once |

### HTTP API

| Endpoint Category | Target | Notes |
|-------------------|--------|-------|
| Health check | <5ms | In-memory, no DB read |
| Sphere summary | <10ms | Read from RwLock |
| Field state | <10ms | Read from RwLock |
| Sphere registration | <20ms | Write + ghost check |
| Status update | <10ms | Write to existing sphere |
| Bus suggestions | <15ms | Computed on request |
| Governance vote | <20ms | SQLite write (WAL) |
| Data manifest | <50ms | Multi-table scan |

### IPC Bus

| Operation | Target | Notes |
|-----------|--------|-------|
| Handshake RTT | <1ms | In-process, no I/O |
| Task submit RTT | <5ms | Create + persist |
| Task claim RTT | <5ms | Update + persist |
| Event delivery | <1ms | mpsc channel broadcast |
| Cascade handoff | <10ms | Consent check + forward |

### Bridge Polling

| Bridge | Target | Notes |
|--------|--------|-------|
| SYNTHEX thermal | <100ms | TCP connect + GET + JSON parse |
| Nexus strategy | <100ms | TCP connect + POST + JSON parse |
| ME fitness | <100ms | TCP connect + GET + JSON parse |
| POVM snapshot write | <50ms | Fire-and-forget POST |
| RM TSV post | <50ms | Fire-and-forget POST |
| VMS memory seed | <50ms | Fire-and-forget POST |

### SQLite Persistence

| Operation | Target | Notes |
|-----------|--------|-------|
| Field snapshot write | <10ms | Single INSERT, WAL mode |
| Bus event write | <5ms | Single INSERT, WAL mode |
| Sphere history write | <5ms | Single INSERT, WAL mode |
| Governance vote write | <10ms | INSERT + quorum check |
| Snapshot read (restore) | <50ms | Single SELECT, cold start only |
| Migration apply | <100ms | Schema creation, startup only |

---

## Complexity Constraints

### Per-Module Complexity

| Module | Hotpath Complexity | Constraint | Notes |
|--------|-------------------|-----------|-------|
| m13 chimera | O(N log N) | N = sphere_count | Sort + gap scan |
| m16 coupling_network | O(N^2) | N = sphere_count | Pairwise coupling |
| m19 hebbian_stdp | O(N^2) | N = sphere_count | Pairwise LTP/LTD |
| m21 memory_manager | O(M) per sphere | M = memory_count | Amortised prune |
| m29 ipc_bus | O(C) per event | C = connections | Broadcast to subscribers |
| m34 suggestions | O(N) | N = sphere_count | Linear scan |
| m35 tick | O(N^2) | N = sphere_count | Dominated by coupling |

### Scaling Limits

| Parameter | Max Value | Constraint Source | Impact at Max |
|-----------|-----------|------------------|---------------|
| Sphere count | 200 (SPHERE_CAP) | Security + O(N^2) | 200^2 = 40K pairs in coupling step |
| Memory per sphere | 500 (MEMORY_MAX) | Memory budget | 200 * 500 = 100K total memories |
| Ghost traces | 20 | VecDeque FIFO | Bounded by constant |
| Bus connections | 50 | Config | 50 subscribers per event |
| Event buffer | 256 | mpsc channel | Backpressure if exceeded |
| Active proposals | 10 | Governance | Bounded iteration |
| Cascade rate | 10/min/source | Rate limiter | Per-source, not global |

### Memory Budget

Estimated memory usage at full capacity (200 spheres):

| Component | Estimate | Notes |
|-----------|----------|-------|
| 200 PaneSpheres | ~50MB | 500 memories each, ~250 bytes per memory |
| Coupling weights | ~1.6MB | 200^2 * 8 bytes (f64) |
| Field state | ~1KB | Fixed size |
| Ghost traces | ~5MB | 20 ghosts with memories |
| Bus state | ~10MB | Task queue + event buffer + subscriptions |
| SQLite connections | ~2MB | 2 databases, WAL buffers |
| **Total** | **~70MB** | Well within typical 512MB-1GB daemon budget |

---

## Benchmark Targets

### Benchmark Configuration

Benchmarks use Criterion and run with:
```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo bench --bench tick_loop 2>&1 | tail -30
```

### Target Benchmarks

| Benchmark | Target p50 | Target p99 | Spheres |
|-----------|-----------|-----------|---------|
| tick_orchestrator (empty field) | <1ms | <5ms | 0 |
| tick_orchestrator (5 spheres) | <5ms | <15ms | 5 |
| tick_orchestrator (50 spheres) | <30ms | <100ms | 50 |
| tick_orchestrator (200 spheres) | <200ms | <500ms | 200 |
| coupling_step (5 spheres) | <100us | <500us | 5 |
| coupling_step (50 spheres) | <5ms | <15ms | 50 |
| chimera_detect (50 spheres) | <500us | <2ms | 50 |
| hebbian_step (50 spheres) | <2ms | <10ms | 50 |
| field_snapshot_write | <5ms | <15ms | - |
| bus_event_broadcast (50 subs) | <500us | <2ms | - |

### V1 Baseline

V1 measured performance (for comparison):
- tick_once with 6 spheres: ~5ms
- Health check: <1ms
- Sphere registration: ~2ms
- Bus task submit: ~3ms
- Dispatch latency: 911ms (Zellij IPC bottleneck, not PV)

---

## Performance Anti-Patterns

| Anti-Pattern | Why It Hurts | Correct Pattern |
|-------------|-------------|----------------|
| Blocking bridge calls in tick loop | 100ms+ TCP connect blocks entire tick | Fire-and-forget with tokio::spawn |
| Vec for message logs | Unbounded growth, O(N) shift on front removal | VecDeque with cap |
| Prune every tick | O(M) scan on every 5s tick | Amortised: prune at threshold+50 |
| String alloc in tight loop | Allocation pressure in coupling step | Pre-allocate, reuse buffers |
| Lock held across I/O | RwLock contention blocks API handlers | Extract values, drop lock, then I/O |
| Sync SQLite in tick loop | WAL write blocks coupling step | Background persist with channel |
| Full field clone for snapshot | 70MB clone at 200 spheres | Extract only needed fields |

---

## Profiling Strategy

When performance issues arise:

1. **Identify hotspot:** Add `tracing::instrument` to phase functions in m35_tick
2. **Measure:** Use Criterion benchmarks with realistic sphere counts
3. **Profile:** `cargo flamegraph --bench tick_loop` for CPU hotspots
4. **Validate:** Ensure fix passes all tests before and after
5. **Document:** Add benchmark result to this file

### Tracing Spans

Key spans to instrument:
```rust
#[tracing::instrument(skip_all, fields(tick))]
async fn tick_orchestrator(state: &SharedState) -> PvResult<TickMetrics> { ... }

#[tracing::instrument(skip_all)]
fn phase_field_update(state: &AppState) -> PvResult<()> { ... }

#[tracing::instrument(skip_all)]
fn phase_learning(state: &AppState) -> PvResult<()> { ... }
```

---

## Cross-References

- **[ARCHITECTURE_DEEP_DIVE.md](ARCHITECTURE_DEEP_DIVE.md)** — Tick loop architecture
- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Module responsibilities
- **[config/default.toml](../config/default.toml)** — Timing parameters
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan: "Don't optimize tick_once performance (~5ms, not the bottleneck)"
- **Obsidian:** `[[Session 039 — Architectural Schematics and Refactor Safety]]` (tick_once profiling)
