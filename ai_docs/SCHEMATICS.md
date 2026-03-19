---
title: "Pane-Vortex V2 — Architecture Schematics"
date: 2026-03-19
tags: [schematics, mermaid, architecture, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[Session 036 — Complete Architecture Schematics]]"
  - "[[Session 039 — Architectural Schematics and Refactor Safety]]"
  - "[[Pane-Vortex System Schematics — Session 027c]]"
---

# Pane-Vortex V2 — Architecture Schematics

> 6 Mermaid diagrams covering the core architectural views.
> See also: `[[Session 036 — Complete Architecture Schematics]]` for 60+ V1 diagrams.

---

## 1. Tick Orchestrator (5 Phases)

The tick loop runs every 5 seconds and passes through 5 sequential phases:

```mermaid
flowchart TD
    START([tick_orchestrator]) --> P1

    subgraph P1["Phase 1: Bridge Polling"]
        P1a[Poll SYNTHEX thermal<br>every 6 ticks]
        P1b[Poll Nexus strategy<br>every 12 ticks]
        P1c[Poll ME fitness<br>every 12 ticks]
        P1a & P1b & P1c --> P1d[Accumulate k_adjustments]
    end

    P1 --> P2

    subgraph P2["Phase 2: Field Update"]
        P2a[Kuramoto coupling<br>15 steps x dt=0.01]
        P2b[Phase wrapping<br>.rem_euclid TAU]
        P2c[Chimera detection<br>O N log N]
        P2d[Tunnel detection]
        P2a --> P2b --> P2c --> P2d
    end

    P2 --> P3

    subgraph P3["Phase 3: Learning"]
        P3a[Hebbian LTP<br>co-active pairs]
        P3b[Hebbian LTD<br>anti-correlated]
        P3c[Burst detection<br>3x multiplier]
        P3d[Amortised prune<br>at threshold+50]
        P3a --> P3b --> P3c --> P3d
    end

    P3 --> P4

    subgraph P4["Phase 4: Decision"]
        P4a[Compute FieldState<br>r, spread, idle ratio]
        P4b[Decision engine<br>priority chain]
        P4c[PI controller<br>gain=0.15]
        P4d[Consent gate<br>all k_adjustments]
        P4e[Generate suggestions]
        P4a --> P4b --> P4c --> P4d --> P4e
    end

    P4 --> P5

    subgraph P5["Phase 5: Persistence"]
        P5a[Field snapshot<br>every 60 ticks]
        P5b[POVM weights<br>every 60 ticks]
        P5c[RM TSV post<br>every 60 ticks]
        P5d[Bus events persist]
        P5a & P5b & P5c & P5d
    end

    P5 --> DONE([TickMetrics])

    style P1 fill:#e1f5fe
    style P2 fill:#f3e5f5
    style P3 fill:#e8f5e9
    style P4 fill:#fff3e0
    style P5 fill:#fce4ec
```

---

## 2. Module Dependency Graph (8 Layers)

Strict downward-only dependency flow between layers:

```mermaid
graph TB
    subgraph L8["L8 Governance (feature-gated)"]
        m37[m37 proposals]
        m38[m38 voting]
        m39[m39 consent_decl]
        m40[m40 data_sovereignty]
        m41[m41 evolution]
    end

    subgraph L7["L7 Coordination"]
        m29[m29 ipc_bus]
        m30[m30 bus_types]
        m31[m31 conductor]
        m32[m32 executor]
        m33[m33 cascade]
        m34[m34 suggestions]
        m35[m35 tick]
        m36[m36 persistence]
    end

    subgraph L6["L6 Bridges"]
        m22[m22 synthex]
        m23[m23 nexus]
        m24[m24 me]
        m25[m25 povm]
        m26[m26 rm]
        m27[m27 vms]
        m28[m28 consent_gate]
    end

    subgraph L5["L5 Learning"]
        m19[m19 hebbian]
        m20[m20 buoy]
        m21[m21 memory_mgr]
    end

    subgraph L4["L4 Coupling"]
        m16[m16 coupling_net]
        m17[m17 auto_k]
        m18[m18 topology]
    end

    subgraph L3["L3 Field"]
        m11[m11 sphere]
        m12[m12 field_state]
        m13[m13 chimera]
        m14[m14 messaging]
        m15[m15 app_state]
    end

    subgraph L2["L2 Services"]
        m07[m07 registry]
        m08[m08 health]
        m09[m09 lifecycle]
        m10[m10 api_server]
    end

    subgraph L1["L1 Foundation"]
        m01[m01 core_types]
        m02[m02 errors]
        m03[m03 config]
        m04[m04 constants]
        m05[m05 traits]
        m06[m06 validation]
    end

    L8 --> L7
    L8 --> L3
    L7 --> L6
    L7 --> L5
    L7 --> L3
    L6 --> L3
    L5 --> L4
    L5 --> L3
    L4 --> L3
    L3 --> L1
    L2 --> L1
    L6 --> L1

    style L1 fill:#e8eaf6
    style L2 fill:#e3f2fd
    style L3 fill:#f3e5f5
    style L4 fill:#e8f5e9
    style L5 fill:#fff8e1
    style L6 fill:#fce4ec
    style L7 fill:#e0f2f1
    style L8 fill:#fff3e0
```

---

## 3. Sphere Lifecycle FSM

State transitions for a PaneSphere from registration to deregistration:

```mermaid
stateDiagram-v2
    [*] --> Registered: POST /register

    Registered --> Idle: initial state
    Idle --> Working: status_update(working)
    Working --> Idle: status_update(idle)
    Working --> Complete: status_update(complete)
    Working --> Blocked: status_update(blocked)
    Blocked --> Working: status_update(working)
    Blocked --> Idle: status_update(idle)
    Complete --> Idle: status_update(idle)
    Complete --> Working: status_update(working)

    Idle --> Newcomer: step_count < 50
    Newcomer --> Established: step_count >= 50
    Established --> Senior: step_count >= 200

    Idle --> Deregistered: POST /deregister
    Working --> Deregistered: POST /deregister
    Complete --> Deregistered: POST /deregister
    Blocked --> Deregistered: POST /deregister

    Deregistered --> Ghost: create GhostTrace
    Ghost --> Registered: re-register\n(weight inheritance)

    note right of Newcomer
        2x Hebbian LTP boost
        for first 50 steps
    end note

    note right of Ghost
        Max 20 ghost traces.
        Preserves weights + memories.
        Consent-gated re-entry.
    end note
```

---

## 4. Decision Engine FSM

The conductor decision priority chain:

```mermaid
stateDiagram-v2
    [*] --> CheckBlocked

    CheckBlocked --> HasBlockedAgents: any sphere blocked
    CheckBlocked --> CheckCoherence: no blocked spheres

    CheckCoherence --> NeedsCoherence: r > 0.3 AND\nr falling AND\nspheres >= 2
    CheckCoherence --> CheckDivergence: coherence not needed

    CheckDivergence --> NeedsDivergence: r > 0.8 AND\nidle > 60% AND\nspheres >= 2
    CheckDivergence --> CheckIdle: divergence not needed

    CheckIdle --> IdleFleet: all spheres idle
    CheckIdle --> CheckFresh: not all idle

    CheckFresh --> FreshFleet: none have worked yet
    CheckFresh --> CheckWarmup: some have worked

    CheckWarmup --> Recovering: warmup_remaining > 0
    CheckWarmup --> Stable: normal operation

    HasBlockedAgents --> [*]: EmergencyCoherence
    NeedsCoherence --> [*]: Increase K
    NeedsDivergence --> [*]: Decrease K
    IdleFleet --> [*]: No action
    FreshFleet --> [*]: No action
    Recovering --> [*]: Suppress decisions
    Stable --> [*]: Normal tick

    note right of HasBlockedAgents
        Highest priority.
        Steer blocked spheres
        toward working cluster.
    end note

    note right of NeedsDivergence
        Multi guard: spheres >= 2
        prevents false signal
        from single-sphere r=1.0.
    end note
```

---

## 5. Bridge Data Flow

How external services connect to PV through the consent gate:

```mermaid
flowchart LR
    subgraph External["External Services"]
        SX["SYNTHEX :8090<br>Cerebral Cortex"]
        NX["Nexus :8100<br>Basal Ganglia"]
        ME["ME :8080<br>Autonomic NS"]
        PO["POVM :8125<br>Spinal Cord"]
        RM["RM :8130<br>Prefrontal Cortex"]
        VM["VMS :8120<br>Hippocampus"]
    end

    subgraph Bridges["L6: Bridge Modules"]
        B22["m22<br>synthex_bridge"]
        B23["m23<br>nexus_bridge"]
        B24["m24<br>me_bridge"]
        B25["m25<br>povm_bridge"]
        B26["m26<br>rm_bridge"]
        B27["m27<br>vms_bridge"]
    end

    subgraph Gate["L6: Consent Gate"]
        B28["m28<br>consent_gate<br>k_mod budget<br>[0.85, 1.15]"]
    end

    subgraph Core["L7: Conductor"]
        COND["m31 conductor<br>PI controller<br>gain=0.15"]
    end

    SX <-->|thermal| B22
    NX <-->|strategy| B23
    ME -->|fitness| B24
    PO <--|snapshots| B25
    RM <--|TSV| B26
    VM <--|memory| B27

    B22 -->|k_adj| B28
    B23 -->|k_adj| B28
    B24 -->|k_adj| B28

    B28 -->|consent-scaled<br>k_mod_total| COND

    COND -->|effective_K| FIELD["L4: Coupling<br>Network"]

    style Gate fill:#ffcdd2
    style Core fill:#c8e6c9
```

---

## 6. IPC Bus Architecture

Unix domain socket bus with NDJSON wire protocol:

```mermaid
flowchart TB
    subgraph Daemon["Pane-Vortex Daemon :8132"]
        TICK["m35 tick_orchestrator"]
        BUS["m29 ipc_bus<br>UnixListener"]
        TASKS["m30 bus_types<br>Task Queue"]
        EVENTS["Event Broadcast<br>mpsc channel"]
        DB["m36 persistence<br>SQLite WAL"]
    end

    subgraph Socket["Unix Socket"]
        SOCK["/run/user/1000/<br>pane-vortex-bus.sock"]
    end

    subgraph Clients["Claude Code Instances"]
        C1["Sphere Alpha<br>pane-vortex-client"]
        C2["Sphere Beta<br>pane-vortex-client"]
        C3["Sphere Gamma<br>pane-vortex-client"]
    end

    C1 <-->|NDJSON| SOCK
    C2 <-->|NDJSON| SOCK
    C3 <-->|NDJSON| SOCK
    SOCK <--> BUS

    BUS --> TASKS
    BUS --> EVENTS
    TICK --> EVENTS

    EVENTS -->|broadcast| BUS
    TASKS -->|persist| DB
    EVENTS -->|persist| DB

    subgraph Protocol["Wire Protocol"]
        direction LR
        HS["handshake"] --> SUB["subscribe"]
        SUB --> SUBMIT["submit/claim/complete"]
        SUBMIT --> EVT["event stream"]
    end

    style Socket fill:#e1f5fe
    style Protocol fill:#f5f5f5
```

---

## Cross-References

- **[ARCHITECTURE_DEEP_DIVE.md](ARCHITECTURE_DEEP_DIVE.md)** — Detailed architecture narrative
- **[STATE_MACHINES.md](STATE_MACHINES.md)** — FSM formal definitions
- **[MESSAGE_FLOWS.md](MESSAGE_FLOWS.md)** — Sequence diagrams
- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Module-level details
- **Obsidian:** `[[Session 036 — Complete Architecture Schematics]]` (60+ V1 Mermaid diagrams)
- **Obsidian:** `[[Session 039 — Architectural Schematics and Refactor Safety]]` (tick decomposition)
- **Obsidian:** `[[Pane-Vortex System Schematics — Session 027c]]` (V1 system schematics)
