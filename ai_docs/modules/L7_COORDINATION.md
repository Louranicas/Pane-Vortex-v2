---
title: "Layer 7: Coordination — Module Documentation"
date: 2026-03-19
tags: [documentation, l7_coordination, pane-vortex-v2, ipc, conductor, tick]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Pane-Vortex IPC Bus — Session 019b]]"
  - "[[Session 039 — Architectural Schematics and Refactor Safety]]"
layer: L7
modules: [m29, m30, m31, m32, m33, m34, m35, m36]
---

# Layer 7: Coordination (m29-m36)

> IPC bus, conductor, executor, cascade, tick orchestrator, and persistence.
> **Depends on:** L1, L3, L5, L6
> **Target LOC:** ~2,350 | **Target tests:** 91+

## Modules: m29_ipc_bus m30_bus_types m31_conductor m32_executor m33_cascade m34_suggestions m35_tick m36_persistence

## Purpose

L7 is the coordination layer that orchestrates all field dynamics. It contains the tick loop (the heartbeat of the system), the IPC bus for inter-sphere communication, the conductor (PI controller for coupling), the executor (Zellij dispatch), cascade handoffs, field suggestions, and SQLite persistence.

## Design Constraints

- Lock ordering: AppState before BusState (pattern P02)
- Tick loop must complete in <50ms (see PERFORMANCE.md)
- IPC bus uses NDJSON over Unix domain socket
- Handshake required before any other frame type
- Task TTL default 3600s with auto-expiry
- Cascade rate limit 10/min per source sphere
- Persistence uses SQLite WAL mode (pattern P15)
- Never write tracing to stdout in daemon (BUG-018)
- Feature gate: persistence behind `#[cfg(feature = "persistence")]`

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list.

## Implementation Status: STUB (awaiting implementation)

---

## m29 -- IPC Bus

**Source:** `src/m7_coordination/m29_ipc_bus.rs` | **LOC Target:** ~500

### Purpose

Unix domain socket server for inter-sphere communication. Uses NDJSON (Newline-Delimited JSON) wire protocol.

### Architecture

```
UnixListener at /run/user/1000/pane-vortex-bus.sock
    |
    +-- Connection 1 (Sphere Alpha)
    |     |-- subscriber patterns: ["field.*", "task.*"]
    |     |-- writer: UnixStream write half
    |
    +-- Connection 2 (Sphere Beta)
    |     |-- subscriber patterns: ["*"]
    |     |-- writer: UnixStream write half
    |
    +-- Event broadcast channel (mpsc, capacity 256)
```

### Connection Lifecycle

1. Client connects to Unix socket
2. Client sends Handshake frame with sphere_id + version
3. Server validates, sends HandshakeAck
4. Client can now: subscribe, submit tasks, claim tasks, send cascades
5. Server broadcasts matching events to subscribed clients
6. On disconnect: remove from connections map

### Key Functions

- `start_bus_listener(socket_path, state) -> JoinHandle` -- Start listener in tokio task
- `handle_connection(stream, state)` -- Per-connection handler loop
- `process_frame(frame, state) -> Option<BusFrame>` -- Process one frame, return response
- `broadcast_event(event, state)` -- Send event to all matching subscribers
- `match_subscription(pattern, event_type) -> bool` -- Glob matching

### Event Buffer

Uses mpsc channel with capacity 256. If full, oldest events are dropped (not blocking). This prevents a slow subscriber from backpressuring the tick loop.

---

## m30 -- Bus Types

**Source:** `src/m7_coordination/m30_bus_types.rs` | **LOC Target:** ~300

### Purpose

All types for the IPC bus wire protocol.

### Frame Types

```rust
pub enum FrameType {
    Handshake,          // Client -> Server: {sphere_id, version}
    HandshakeAck,       // Server -> Client: {status}
    Submit,             // Client -> Server: {task_id, source, target, desc}
    SubmitAck,          // Server -> Client: {task_id}
    Claim,              // Client -> Server: {task_id, sphere_id}
    ClaimAck,           // Server -> Client: {task_id}
    Complete,           // Client -> Server: {task_id, result}
    CompleteAck,        // Server -> Client: {task_id}
    Fail,               // Client -> Server: {task_id, error}
    Subscribe,          // Client -> Server: {pattern}
    SubscribeAck,       // Server -> Client: {pattern}
    Unsubscribe,        // Client -> Server: {pattern}
    Event,              // Server -> Client: {event_type, source, data, tick}
    CascadeHandoff,     // Client -> Server: {source, target, brief, depth}
    CascadeAck,         // Client -> Server: {source, target, status}
    CascadeReject,      // Server -> Client: {reason}
    Error,              // Server -> Client: {code, message}
}
```

### Task Routing

```rust
pub enum TaskTarget {
    Specific(PaneId),   // Named sphere
    AnyIdle,            // First idle sphere
    FieldDriven,        // Based on field suggestions
    Willing,            // Consent-aware routing
}
```

### Task Lifecycle

Submitted -> Claimed -> Completed/Failed (terminal). Also: Submitted/Claimed -> Expired (TTL).

---

## m31 -- Conductor

**Source:** `src/m7_coordination/m31_conductor.rs` | **LOC Target:** ~400

### Purpose

PI controller that adjusts global coupling strength K based on field state. Implements the decision engine priority chain.

### PI Controller

```rust
pub fn pi_control(r_current: f64, r_target: f64, gain: f64) -> f64 {
    gain * (r_target - r_current)
}
```

- Gain: 0.15
- r_target: 0.93 (dynamic, fleet-negotiable via governance)
- Breathing blend: 0.3 (emergent breathing contribution)

### Divergence Cooldown

After NeedsDivergence, suppress coherence for 3 ticks:

```rust
pub fn apply_divergence_cooldown(state: &mut ConductorState, decision: FieldDecision) -> FieldDecision {
    if decision == NeedsDivergence {
        state.divergence_cooldown = DIVERGENCE_COOLDOWN_TICKS;
    }
    if state.divergence_cooldown > 0 && decision == NeedsCoherence {
        state.divergence_cooldown -= 1;
        return Stable;  // Suppress coherence during cooldown
    }
    decision
}
```

### Decision Attribution (NA-P-9)

Each decision records its modulation sources:

```rust
pub struct DecisionRecord {
    pub action: DecisionAction,
    pub rationale: String,
    pub attribution: Vec<(String, f64)>,  // (source, k_adj)
    pub tick: u64,
    pub modulation_breakdown: ModulationBreakdown,
}
```

---

## m32 -- Executor

**Source:** `src/m7_coordination/m32_executor.rs` | **LOC Target:** ~200

### Purpose

Thin Zellij dispatch for executing commands in fleet panes. 5-step dispatch process.

### Dispatch Steps

1. Look up target sphere in pane map
2. Switch to target tab: `zellij action go-to-tab N`
3. Navigate to target pane: `zellij action move-focus direction`
4. Send command: `zellij action write-chars "command"`
5. Return to command tab

### Pane Mapping

```rust
pub struct PaneMapping {
    pub tab: usize,
    pub pane: String,    // Directional from tab root
    pub label: String,   // Descriptive label
    pub sphere_id: PaneId,
}
```

### V1 Measured Latency

Dispatch latency in V1: 911ms (dominated by Zellij IPC, not PV logic).

---

## m33 -- Cascade

**Source:** `src/m7_coordination/m33_cascade.rs` | **LOC Target:** ~200

### Purpose

Inter-sphere work delegation with rate limiting, consent checking, and depth tracking.

### Key Types

- `CascadeHandoff` -- source, target, brief, depth
- `CascadeAck` -- source, target, status (Acked/Rejected)

### Rate Limiting

Max 10 cascades per minute per source sphere. Tracked in a sliding window.

### Fallback Brief

When no bus connection exists for the target:
- Write markdown to `~/projects/shared-context/tasks/cascade-{uuid}.md`
- Target picks up brief on next session start

### V3 Alignment

V3.3.3 (Cascade rejection): target sphere can reject cascades (NA-P-7).

---

## m34 -- Suggestions

**Source:** `src/m7_coordination/m34_suggestions.rs` | **LOC Target:** ~200

### Purpose

Field-driven task suggestions based on field state. Suggestions are generated in the tick loop and exposed via `/bus/suggestions`.

### Suggestion Types

```rust
pub enum SuggestionType {
    Diverge,      // Field over-synchronized, suggest exploration
    Cohere,       // Field under-synchronized, suggest alignment
    Rebalance,    // Workload imbalance detected
    Investigate,  // Anomaly detected (chimera, blocked sphere)
}
```

### Sphere Autonomy Filtering (NA-P-8)

Suggestions are filtered through `filter_eligible_spheres()`:
- Receptivity gate: sphere.activation > 0.3
- Opt-out: respect `opt_out_cross_activation`
- Status: exclude idle/complete spheres for work suggestions

---

## m35 -- Tick Orchestrator

**Source:** `src/m7_coordination/m35_tick.rs` | **LOC Target:** ~300

### Purpose

The heartbeat of the system. Runs every 5 seconds, orchestrating all field dynamics through 5 sequential phases.

### 5-Phase Decomposition

This is the V2 decomposition of V1's 829-line `tick_once` god function:

```rust
pub async fn tick_orchestrator(state: &SharedState) -> PvResult<TickMetrics> {
    let mut metrics = TickMetrics::new();

    metrics.record(TickPhase::BridgePoll, phase_bridge_poll(state).await);
    metrics.record(TickPhase::FieldUpdate, phase_field_update(state).await);
    metrics.record(TickPhase::Learning, phase_learning(state).await);
    metrics.record(TickPhase::Decision, phase_decision(state).await);
    metrics.record(TickPhase::Persistence, phase_persistence(state).await);

    Ok(metrics)
}
```

See [SCHEMATICS.md](../SCHEMATICS.md) for the tick orchestrator Mermaid diagram.

### V1 Reference

V1's `tick_once` was 829 lines with 65 branches. Session 039 analyzed it as a "god function" but determined it was born that way (58% of main.rs at commit 1), not a regression. V2 decomposes it cleanly into 5 phases + orchestrator.

---

## m36 -- Persistence

**Source:** `src/m7_coordination/m36_persistence.rs` | **LOC Target:** ~250
**Feature gate:** `persistence`

### Purpose

SQLite persistence with WAL mode. Manages two databases:
- `data/field_tracking.db` -- field_snapshots, sphere_history, coupling_history
- `data/bus_tracking.db` -- bus_tasks, bus_events, event_subscriptions, cascade_events, task_tags, task_dependencies

### Migrations

Auto-applied on startup from `migrations/` directory:
- 001_field_tables.sql (3 tables + 2 indexes)
- 002_bus_tables.sql (6 tables + 4 indexes)
- 003_governance_tables.sql (4 tables + 2 indexes)

### WAL Mode (Pattern P15)

All connections open with:
```sql
PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;
```

WAL mode enables concurrent reads during writes. Busy timeout prevents immediate failure under contention.

---

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan
- [ARCHITECTURE_DEEP_DIVE.md](../ARCHITECTURE_DEEP_DIVE.md) -- Tick loop, IPC bus
- [SCHEMATICS.md](../SCHEMATICS.md) -- Tick orchestrator and IPC bus diagrams
- [STATE_MACHINES.md](../STATE_MACHINES.md) -- Task lifecycle, cascade lifecycle, conductor decisions
- [MESSAGE_FLOWS.md](../MESSAGE_FLOWS.md) -- Bus task submission, cascade handoff sequences
- [PERFORMANCE.md](../PERFORMANCE.md) -- Tick loop timing budget
- Obsidian: `[[Pane-Vortex IPC Bus -- Session 019b]]`, `[[Session 039 -- Architectural Schematics and Refactor Safety]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/main.rs` (tick_once), `bus.rs`, `ipc.rs`, `persistence.rs`
