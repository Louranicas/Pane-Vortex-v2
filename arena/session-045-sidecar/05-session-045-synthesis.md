# Session 045 — Sidecar Exploration Synthesis

> 1 hour deep exploration of sidecar, IPC bus, swarm plugin, cross-service intelligence
> Claude Opus 4.6 (1M context) | 2026-03-21

## Top 10 Discoveries

### 1. RALPH Loop in WASM (Novel)
The swarm-orchestrator.wasm has a complete 5-phase quality loop:
Reflect → Analyze → Learn → Plan → Harmonize → (iterate if quality < 0.80)
Max 5 iterations. Auto-trigger configurable. This is an **autonomous quality ratchet**
embedded in the Zellij plugin itself.

### 2. Intelligence Router (3-Service Fusion)
IntelligenceRouter fuses POVM pathways + RM context + Nexus r_outer into a
composite route score: `0.3×hebbian + 0.3×receptivity + 0.2×strategy + 0.2×history`.
This is **cross-service intelligence** that the V1 codebase never had.

### 3. DistributedPlan (v3.0 Phase 7 — Dormant)
The swarm lib has a full distributed planning system: decompose → dispatch subtasks
across fleet panes → collect → merge. Exists in code but never exercised in production.
**This is the next frontier** for fleet coordination.

### 4. V1/V2 Wire Protocol Gap (Critical Finding)
Sidecar V1 binary hits V2 handshake envelope mismatch. Ring file 29K ticks stale.
The sidecar CONNECTS but fails to parse. Fix: rebuild sidecar or add V1-compat
mode to V2 daemon handshake.

### 5. 3 Dispatch Modes
PvExecutor (Zellij panes via run_command), IpcBus (socket via sidecar), LegacyClaude
(subprocess). Default is PvExecutor. IpcBus mode requires live sidecar.

### 6. Worker Roles (Auto-Assigned)
First spawned = Leader (decomposes), last = Validator (checks quality), middle = Workers.
Max 8 concurrent. This enables structured parallel execution.

### 7. 74 Strong POVM Pathways
Pathways form naturally from Hebbian STDP during real usage. Strongest:
nexus-bus:cs-v7 → synthex (w=1.05, supercritical). These represent
**emergent information routes** the system learned are high-value.

### 8. Service Response Distribution
All 16 services sub-2ms. Median 0.23ms. RM slowest at 1.2ms (SQLite I/O).
ME at 0.55ms (12D tensor + RALPH cycles). NAIS fastest at 0.18ms.

### 9. 57 Learned Patterns (B1-B10)
Independently discovered and reinforced across sessions. B1 (SQLite state query)
and B2 (quality gate chain) have 10 reinforcements each. These are **validated
operational knowledge** stored in the service_tracking DB.

### 10. Backpressure Architecture (5-Layer Defense)
Health gate → exponential backoff → rate limiter → circuit breaker → event
summarization. Prevents IPC broken pipe flood from killing Zellij (BUG-014).

## Architecture Schematic

```
┌────────────────────────────────────────────────────────────┐
│                    THE HABITAT                              │
│                                                            │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐            │
│  │ SYNTHEX  │    │  SAN-K7  │    │   ME     │            │
│  │ (cortex) │    │ (ganglia)│    │ (ANS)    │            │
│  │  :8090   │    │  :8100   │    │  :8080   │            │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘            │
│       │               │               │                   │
│       └───────────┬───┘───────────────┘                   │
│                   ▼                                        │
│           ┌──────────────┐                                │
│           │ ConsentGate  │ ← 6 bridges (GAP-3 fixed)     │
│           │  (M28)       │                                │
│           └──────┬───────┘                                │
│                  ▼                                         │
│  ┌───────────────────────────────┐                        │
│  │     Pane-Vortex V2 (:8132)   │                        │
│  │  ┌────────────────────────┐  │                        │
│  │  │ Tick Orchestrator (M35)│  │                        │
│  │  │  1→Steps 2→Coupling   │  │                        │
│  │  │  3→Field 3.5→GOV      │  │  ← GAP-1 actuator     │
│  │  │  4→Conductor 5→Persist│  │                        │
│  │  └────────────────────────┘  │                        │
│  │  ┌────────────────────────┐  │                        │
│  │  │   IPC Bus (M29)        │  │                        │
│  │  │   Unix socket          │──┼──→ /run/user/1000/     │
│  │  │   NDJSON wire          │  │    pane-vortex-bus.sock │
│  │  └────────────────────────┘  │                        │
│  └───────────────────────────────┘                        │
│                  │                                         │
│      ┌───────────┼────────────┐                           │
│      ▼           ▼            ▼                           │
│  ┌────────┐ ┌────────┐ ┌──────────────┐                  │
│  │Client  │ │Sidecar │ │ WASM Plugin  │                  │
│  │(native)│ │(native)│ │ (Zellij)     │                  │
│  │connect │ │FIFO→UDS│ │ RALPH loop   │                  │
│  │submit  │ │ring→   │ │ Intel Router │                  │
│  │cascade │ │events  │ │ DistribPlan  │                  │
│  └────────┘ └────────┘ └──────────────┘                  │
│                              │                            │
│                   ┌──────────┼──────────┐                 │
│                   ▼          ▼          ▼                 │
│              ┌────────┐ ┌────────┐ ┌────────┐            │
│              │Fleet-α │ │Fleet-β │ │Fleet-γ │            │
│              │Tab 4   │ │Tab 5   │ │Tab 6   │            │
│              │Monitor │ │3 panes │ │3 panes │            │
│              └────────┘ └────────┘ └────────┘            │
│                                                            │
│  Memory Systems:                                           │
│  ┌──────┐ ┌──────┐ ┌────┐ ┌─────┐ ┌────┐ ┌──────┐      │
│  │POVM  │ │ RM   │ │VMS │ │Vault│ │Auto│ │SQLite│      │
│  │:8125 │ │:8130 │ │8120│ │ Obs │ │Mem │ │166DBs│      │
│  │2427pw│ │3250+ │ │    │ │215+ │ │    │ │360MB │      │
│  └──────┘ └──────┘ └────┘ └─────┘ └────┘ └──────┘      │
└────────────────────────────────────────────────────────────┘
```

## What's Next

1. **Rebuild sidecar** against V2 wire format — ring file currently stale
2. **Exercise DistributedPlan** — the dormant v3.0 Phase 7 command
3. **Test RALPH loop** with real fleet tasks — quality ratchet never production-tested
4. **Deploy V2 binary** — governance routes are 404 on live daemon
5. **Wire Intelligence Router** to live POVM/RM data — currently initialized but stale

## Arena Files

| File | Content |
|------|---------|
| 01-sidecar-architecture.md | 5-layer data flow, wire protocol issue, runtime state |
| 02-api-wiring-map.md | 18 API routes, intelligence router, cross-service flow |
| 03-intelligence-synthesis.md | POVM pathways, learned patterns, synergy map |
| 04-advanced-dispatch-architecture.md | 3 dispatch modes, RALPH, DistributedPlan, response times |
| 05-session-045-synthesis.md | This file — top 10 discoveries, architecture schematic |
