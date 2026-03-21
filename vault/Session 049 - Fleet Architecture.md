# Session 049 — Fleet Architecture Schematics

**Date:** 2026-03-21

## 1. Hook Pipeline (Event → Service Data Flow)

```mermaid
graph TD
    SS[SessionStart<br/>20s timeout] -->|POST /sphere/register| PV
    SS -->|GET /hydrate| POVM
    SS -->|GET /search?q=discovery| RM
    SS -->|spawn listener| BUS[IPC Bus<br/>Unix Socket]

    UPS[UserPromptSubmit<br/>5s] -->|GET /health| PV
    UPS -->|GET /v3/thermal| SX[SYNTHEX]
    UPS -->|GET /field/decision| PV
    UPS -->|inject systemMessage| CLAUDE[Claude Response]

    PTG[PreToolUse<br/>3s] -->|GET /v3/thermal| SX
    PTG -->|"T > 0.8? block"| CLAUDE

    TOOL[TOOL EXECUTION] --> PTU

    PTU[PostToolUse<br/>5s] -->|POST /sphere/memory| PV
    PTU -->|POST /sphere/status| PV
    PTU -->|"GET /bus/tasks (1-in-5)"| PV
    PTU -->|"TASK_COMPLETE? → POST /bus/complete"| PV

    POVM_H[PostToolUse:POVM<br/>3s] -->|POST /pathways| POVM
    NX_H[PostToolUse:Nexus<br/>3s] -->|POST nexus/command| K7[SAN-K7]

    SAS[SubagentStop<br/>5s] -->|POST /put| RM
    SAS -->|POST /sphere/steer| PV

    STOP[Stop<br/>10s] -->|POST /bus/fail| PV
    STOP -->|POST /sphere/deregister| PV
    STOP -->|POST /snapshots| POVM
    STOP -->|POST /put summary| RM
    STOP -->|kill listener| BUS

    style SS fill:#9c6
    style STOP fill:#f66
    style PTU fill:#fc6
```

## 2. Task Lifecycle (3 Channels)

```mermaid
stateDiagram-v2
    [*] --> Submitted: bus/submit OR fq_submit OR rm_submit_task
    
    state "HTTP Bus (Primary)" as HTTP {
        Submitted --> Pending: POST /bus/submit
        Pending --> Claimed: POST /bus/claim/{id}
        Claimed --> Completed: POST /bus/complete/{id}
        Claimed --> Failed: POST /bus/fail/{id}
    }

    state "File Queue (Fallback)" as FQ {
        FQ_Pending: vault/tasks/pending/
        FQ_Claimed: vault/tasks/claimed/
        FQ_Done: vault/tasks/done/
        FQ_Pending --> FQ_Claimed: mv -n (atomic)
        FQ_Claimed --> FQ_Done: mv
    }

    state "RM Bus (Cross-Session)" as RM {
        RM_Task: pv2:task (3600s TTL)
        RM_Claim: pv2:claim (1800s TTL)
        RM_Done: pv2:done (7200s TTL)
        RM_Task --> RM_Claim
        RM_Claim --> RM_Done
    }

    Completed --> [*]
    Failed --> Pending: GAP-G1 requeue (300s)
```

### Dedup Hierarchy (GAP-G9)
HTTP Bus > File Queue > RM Bus

## 3. Multi-Tab Fleet Topology

```mermaid
graph TD
    subgraph "Tab 1 — Command"
        ORCH[Orchestrator<br/>Full-width pane]
    end

    subgraph "Tab 4 — ALPHA"
        A_L[Claude ALPHA-Left]
        A_MON[PV Monitor]
        A_HW[Health Watch]
    end

    subgraph "Tab 5 — BETA"
        B_L[Claude BETA-Left]
        B_TR[Claude BETA-TopRight]
        B_BR[Claude BETA-BottomRight]
    end

    subgraph "Tab 6 — GAMMA"
        G_L[Claude GAMMA-Left]
        G_TR[Claude GAMMA-TopRight]
        G_BR[Claude GAMMA-BottomRight]
    end

    BUS[IPC Bus<br/>/run/user/1000/<br/>pane-vortex-bus.sock]

    ORCH -->|submit tasks| BUS
    BUS -->|field-driven routing| A_L
    BUS -->|any-idle routing| B_L
    BUS -->|any-idle routing| B_TR
    BUS -->|any-idle routing| B_BR
    BUS -->|any-idle routing| G_L
    BUS -->|any-idle routing| G_TR
    BUS -->|any-idle routing| G_BR

    A_L -->|TASK_COMPLETE| BUS
    B_L -->|TASK_COMPLETE| BUS
    G_L -->|TASK_COMPLETE| BUS

    BUS -->|events| SIDECAR[Swarm Sidecar<br/>WASM bridge]
    SIDECAR -->|/tmp/swarm-events.jsonl| PLUGIN[Swarm Plugin<br/>Zellij WASM]

    style BUS fill:#fc6,stroke:#333
    style ORCH fill:#9cf,stroke:#333
```

### Routing Modes
| Target | Behavior |
|--------|----------|
| any_idle | First idle sphere claims |
| field_driven | Conductor routes by phase coherence |
| specific | Named sphere only |
| willing | Opt-in spheres (consent-gated) |

---
*Cross-refs:* [[Fleet Coordination Spec]], [[IPC Bus Architecture Deep Dive]], [[Session 049 — Master Index]]
