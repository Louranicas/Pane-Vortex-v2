# Session 049 — Workflow Analysis (Dual Code Path Trace)

**Date:** 2026-03-21

---

## 1. Task Submission Workflow

### Sequence Diagram

```mermaid
sequenceDiagram
    participant Client as HTTP Client / Hook
    participant API as m10_api_server.rs
    participant Bus as m29_ipc_bus.rs<br/>BusState
    participant Types as m30_bus_types.rs<br/>BusTask
    participant Exec as m32_executor.rs
    participant Hook as post_tool_use.sh

    Note over Client,Hook: === HTTP Path (hooks poll) ===
    Client->>API: POST /bus/submit<br/>{description, target, submitter}
    API->>Types: BusTask::new(desc, target, submitter)
    Types-->>API: BusTask { status: Pending }
    API->>Bus: submit_task(task)
    Bus-->>Bus: Check pending < 256<br/>Insert into HashMap
    Bus-->>API: Ok(task_id)
    API-->>Client: 201 { task_id, status: "Pending" }

    Note over Hook: Every 5th PostToolUse event...
    Hook->>API: GET /bus/tasks
    API-->>Hook: { tasks: [...] }
    Hook->>Hook: Filter status=="Pending"<br/>Select first
    Hook->>API: POST /bus/claim/{task_id}<br/>{claimer: pane_id}
    API->>Bus: claim_task(id, claimer)
    Bus->>Types: task.claim(claimer)
    Types-->>Types: Pending → Claimed<br/>Set claimed_by, claimed_at
    API-->>Hook: { status: "Claimed" }
    Hook->>Hook: Write /tmp/pane-vortex-active-task-{ID}<br/>Inject systemMessage

    Note over Hook: When TASK_COMPLETE in tool output...
    Hook->>API: POST /bus/complete/{task_id}
    API->>Types: task.complete()
    Types-->>Types: Claimed → Completed<br/>Set completed_at
    API->>Bus: publish_event("task.completed")
    API-->>Hook: { status: "Completed" }

    Note over Client,Hook: === IPC Socket Path (auto-dispatch) ===
    Client->>Bus: BusFrame::Submit(task)
    Bus->>Bus: submit_task(task)
    Bus->>Exec: execute(&task, &spheres)
    Exec->>Exec: select_idle_sphere()<br/>or select_field_driven()
    Exec-->>Bus: ExecutorResult { target_sphere }
    Bus->>Types: task.claim(target_sphere)
    Bus->>Bus: publish_event("task.dispatched")
    Bus-->>Client: BusFrame::TaskSubmitted
```

### Two Submit Paths

| Aspect | HTTP POST /bus/submit | IPC Socket BusFrame::Submit |
|--------|----------------------|---------------------------|
| Auto-dispatch | No — stays Pending | Yes — Executor claims immediately |
| Event published | None | task.dispatched |
| Who claims | Hook polls 1-in-5 | Server auto-claims |
| Backpressure | 256 pending max | Same |

### TaskTarget Selection Strategies

| Target | Filter | Rank By |
|--------|--------|---------|
| AnyIdle | status == Idle | Least recently active |
| FieldDriven | Idle or Working | field_score = receptivity × status_weight × (1 - opt_out) |
| Willing | receptivity > 0.3, !opt_out | Max receptivity |
| Specific | sphere exists | Direct lookup |

### State Machine

```
Pending → Claimed → Completed
                  → Failed
Claimed → Pending  (requeue on stale timeout)
```

---

## 2. Bridge Polling & k_modulation Workflow

### Sequence Diagram

```mermaid
sequenceDiagram
    participant Tick as m35_tick.rs<br/>tick_orchestrator
    participant Bridge as m6_bridges/mod.rs<br/>BridgeSet
    participant SX as m22_synthex_bridge
    participant NX as m23_nexus_bridge
    participant ME as m24_me_bridge
    participant Gate as m28_consent_gate
    participant Net as m16_coupling_network

    Note over Tick: Phase 1: sphere steps
    Note over Tick: Phase 2: Kuramoto coupling (15 steps)
    Note over Tick: Phase 2.5: Hebbian STDP

    Tick->>Bridge: Phase 2.7: apply_k_mod(state, network)

    Bridge->>SX: cached_adjustment()
    SX-->>Bridge: sx = 1.0 - 0.2*(temp - target)<br/>Cold→boost, Hot→dampen

    Bridge->>NX: cached_adjustment()
    NX-->>Bridge: nx = mult * (0.7 + 0.3*r)<br/>Aligned=1.10, Diverging=0.92

    Bridge->>ME: cached_adjustment()
    ME-->>Bridge: me = fitness*0.30 + 0.85<br/>f=0.5→1.0, f=1.0→1.15

    Bridge->>Gate: apply_combined_all(sx, nx, me, pv, rm, vm, consents)
    Gate->>Gate: raw = sx * nx * me * pv * rm * vm
    Gate->>Gate: Filter consenting spheres<br/>(exclude opted-out, divergence-exempt)
    Gate->>Gate: Scale by mean receptivity:<br/>scaled = 1.0 + (raw-1.0) * mean_r
    Gate->>Gate: Newcomer dampening if ratio > 0.5
    Gate->>Gate: Clamp to [0.85, 1.15]
    Gate-->>Bridge: gated_adjustment

    Bridge->>Net: k_modulation *= gated
    Bridge->>Net: clamp(k_modulation, 0.85, 1.15)

    Note over Tick: Phase 3: compute r, field decision
    Note over Tick: Phase 3.1: harmonic damping if L2 > 0.70
    Tick->>Net: k_modulation *= 1.0 + 0.15*(1-r)*(l2-0.70)/0.30

    Note over Tick: Phase 4: conductor composition
    Tick->>Net: k_modulation *= 1.0 + divergence_ema<br/>clamp [0.85, 1.15]

    Note over Tick: Next tick Phase 2...
    Net->>Net: k_effective = k * k_modulation<br/>= 1.5 * k_modulation<br/>dθ/dt = ω + r_i * (k_eff/N) * Σ w_ij * sin(θ_j - θ_i)
```

### Bridge Poll Intervals

| Bridge | Endpoint | Interval | Factor Formula |
|--------|----------|----------|----------------|
| SYNTHEX | GET :8090/v3/thermal | 6 ticks | 1.0 - 0.2*(temp - target) |
| Nexus/K7 | GET :8100/nexus/metrics | 60 ticks | mult * (0.7 + 0.3*r) |
| ME | GET :8080/api/observer | 12 ticks | fitness*0.30 + 0.85 |
| POVM | GET :8125/pathways | 60 ticks | Cached, not applied |
| RM | GET :8130/search | On hydration | Cached, not applied |
| VMS | GET :8120/health | 60 ticks | Cached, not applied |

### k_modulation Mutation Chain (per tick)

```
                     Phase 2.7                    Phase 3.1                Phase 4
k_mod = k_mod × consent_gate(Π bridges) → × harmonic_damping → × conductor_factor
        ↓ clamp [0.85, 1.15]                 ↓ clamp               ↓ clamp

Next tick Phase 2:
k_effective = k_base(1.5) × k_modulation
dθᵢ/dt = ωᵢ + receptivityᵢ × (k_eff / N) × Σ wᵢⱼ × sin(θⱼ − θᵢ)
```

### Consent Gate Pipeline

1. Filter: exclude opted-out + divergence-exempt (receptivity < 0.15)
2. Scale: `1.0 + (raw - 1.0) × mean_receptivity`
3. Dampen: if newcomer_ratio > 0.5, reduce deviation
4. Clamp: `[0.85, 1.15]` (budget adjustable via governance)

---

## Cross-References

- [[Session 049 - System Architecture]]
- [[Session 049 - Fleet Architecture]]
- [[Session 049 - Data Flow Verification]]
- [[ULTRAPLATE Master Index]]
