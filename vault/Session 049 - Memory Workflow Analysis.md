# Session 049 — Memory Workflow Analysis

> **2 memory cycles mapped: Crystallize-Hydrate + Hebbian Learning**
> **Both cycles are OPEN — data written but not read back**
> **Captured:** 2026-03-21

---

## Cycle 1: Crystallize → Hydrate (Cross-Session)

```mermaid
sequenceDiagram
    participant SE as session_end.sh
    participant POVM as POVM :8125
    participant RM as RM :8130
    participant SS as session_start.sh
    participant Claude as Next Session

    Note over SE: Session N ends
    SE->>POVM: POST /snapshots {sphere_id, r, event}
    SE->>RM: PUT /put (pv2:done, task summary)
    SE->>RM: PUT /put (pv2:status, session-end)
    SE->>POVM: POST /sphere/{id}/deregister

    Note over SS: Session N+1 starts
    SS->>POVM: GET /hydrate
    POVM-->>SS: {memory_count: 82, pathway_count: 2427}
    SS->>RM: GET /search?q=discovery
    RM-->>SS: [discovery entries]
    SS->>RM: GET /search?q=pv2:task
    RM-->>SS: [pending fleet tasks]
    SS-->>Claude: "[HABITAT] Hydrated: 82 memories, 2427 pathways, N discoveries"
```

### Verdict: CYCLE IS BROKEN

| Written by session_end | Read by session_start? |
|------------------------|------------------------|
| POVM snapshots (sphere, r, event) | **NO** — reads /hydrate (aggregate counts only) |
| RM pv2:done records | **NO** — searches discovery + pv2:task, not pv2:done |
| RM pv2:status heartbeats | **NO** — never queried |
| Sphere memories (tool names) | **NO** — no API to retrieve |
| Coupling weights | **NO** — lost on restart (in-memory) |

**What survives:** Only aggregate counts (memory_count, pathway_count) and discovery/task entries. Fine-grained session context is crystallized but **never hydrated**.

---

## Cycle 2: Hebbian Learning (In-Session)

```mermaid
sequenceDiagram
    participant PTU as PostToolUse Hook
    participant POVM as POVM :8125
    participant Tick as PV2 Tick (5s)
    participant STDP as apply_stdp()
    participant Net as CouplingNetwork

    Note over PTU: Tool completes
    PTU->>POVM: POST /pathways {source: prev_tool, target: tool, weight}
    Note over POVM: Stores tool transitions (2,427 pathways)

    Note over Tick: Every 5 seconds
    Tick->>STDP: Phase 2.5: tick_hebbian()
    STDP->>Net: Read sphere statuses (Working/Idle)
    Note over STDP: Both Working? LTP +0.01<br/>Otherwise? LTD -0.002
    STDP->>Net: Update weights (clamp 0.15-1.0)

    Note over POVM,STDP: NO CONNECTION
    POVM--xSTDP: hydrate_pathways() exists but NEVER CALLED
```

### Verdict: LOOP IS OPEN

| Step | Status |
|------|--------|
| PostToolUse → POVM pathways | **Connected** (writes tool transitions) |
| Tick Phase 2.5 → apply_stdp() | **Connected** (runs every 5s) |
| POVM pathways → apply_stdp() | **DISCONNECTED** — hydrate_pathways() exists in code but is never called |
| Namespace bridge | **MISSING** — POVM uses tool names ("Read", "Edit"), STDP uses sphere IDs ("fleet-alpha") |

**Two independent systems:** POVM stores tool-level transitions (2,427 pathways). STDP operates on sphere-level co-activation (12 edges at w=0.6). They share the word "Hebbian" but have zero data exchange.

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - LTP LTD Balance]] — STDP dynamics analysis
- [[Session 049 - Cross-Hydration Analysis]] — POVM+RM relationship
- [[Session 049 - Persistence Cluster]] — persistence layer map
