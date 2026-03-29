# Advanced Dispatch Architecture — Session 045

## 3 Dispatch Modes (swarm-orchestrator/src/lib.rs)

```
                          ┌──────────────────┐
                          │  Swarm WASM       │
                          │  Orchestrator     │
                          │  (main.rs 1278L)  │
                          └────────┬──────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
              ┌──────────┐  ┌──────────┐  ┌──────────┐
              │PvExecutor │  │ IpcBus   │  │ Legacy   │
              │ (default) │  │ (v3.0)   │  │ Claude   │
              └─────┬─────┘  └─────┬────┘  └─────┬────┘
                    │              │              │
                    ▼              ▼              ▼
              Zellij pane    Unix socket    claude -p
              write-chars    NDJSON wire    subprocess
              run_command    via sidecar    direct exec
```

## Worker Role Assignment

```
Workers:  [ 0: Leader ]  [ 1: Worker ]  ...  [ N-1: Validator ]
           ▲                                     ▲
           │ First spawned                       │ Last spawned
           │ Drives subtask                      │ Validates quality
           │ decomposition                       │ of merged results
```

Max 8 workers (DEFAULT_MAX_WORKERS). Leader decomposes. Workers execute. Validator checks quality.

## PV Dispatch Routing

```
Worker ID           → Tab  → Pane
─────────────────────────────────
alpha-*             → 4    → 0 (Fleet-ALPHA left)
beta-*              → 5    → 0 (Fleet-BETA left)
gamma-*             → 6    → 0 (Fleet-GAMMA left)
delta-*             → 6    → 1 (Fleet-GAMMA top-right)
(other)             → 4    → (auto)
```

## Intelligence-Weighted Route Scoring

```
route_score = 0.3 × hebbian_affinity(POVM pathways)
            + 0.3 × receptivity(sphere [0,1])
            + 0.2 × strategy_confidence(Nexus alignment)
            + 0.2 × historical_success(past completions)

Strategy alignment from Nexus r_outer:
  r_outer ≥ 0.8 → Aligned    (1.0)
  r_outer ≥ 0.5 → Partial    (0.7)
  r_outer ≥ 0.2 → Diverging  (0.3)
  r_outer < 0.2 → Incoherent (0.0)
```

## RALPH Loop Phases

```
Idle → Reflect → Analyze → Learn → Plan → Harmonize → Complete
  │                 ↑                          │
  │                 └──── if quality < threshold ──┘
  │                       (max_iter=5, threshold=0.80)
  │
  └── AutoRalph: auto-trigger on configurable events
```

## BusCommand Protocol

```json
// Submit a task via IPC bus
{"cmd": "submit", "description": "task text", "target": "any_idle"}

// Cascade handoff via IPC bus
{"cmd": "cascade", "target": "5:left", "task": "focus area", "prompt": "full context"}

// Request to specific sphere
{"cmd": "request", "to": "sphere-id", "payload": {"key": "value"}}
```

## DistributedPlan (v3.0 Phase 7 — Untested)

```
DistributedPlan {
    plan_id: String,
    subtasks: Vec<PlanSubtask>,
    results_collected: usize,
    active: bool,
}

PlanSubtask {
    handoff_id: String,    // Cascade ID
    target: String,         // Target pane
    focus: String,          // What to focus on
    acked: bool,            // Whether target acknowledged
}
```

Decompose task → dispatch subtasks across fleet panes → collect results → merge.
This is the holy grail of fleet coordination — exists in code but never exercised.

## Service Response Time Distribution (all 16)

```
8101 (NAIS):     0.185ms  ← fastest
8110 (CS-V7):    0.188ms
9001 (Architect): 0.188ms
10001 (Prometheus): 0.191ms
8081 (DevOps):    0.204ms
8100 (SAN-K7):    0.214ms
8103 (ToolMaker): 0.216ms
8102 (Bash):      0.217ms
8125 (POVM):      0.235ms
8104 (CCM):       0.251ms
8090 (SYNTHEX):   0.257ms
8105 (ToolLib):   0.270ms
8132 (PV):        0.288ms
8120 (VMS):       0.372ms
8080 (ME):        0.552ms  ← heaviest (12D tensor, RALPH)
8130 (RM):        1.201ms  ← slowest (SQLite I/O)
```

All sub-2ms. Median: 0.23ms. P99: 1.2ms (RM).

## Key Discovery: V1/V2 Wire Protocol Gap

The sidecar (V1 binary) connects to the V2 PV daemon but can't parse the V2
handshake response. The V2 response wraps everything in a typed envelope:

```
V1: {"tick": N, "peers": N, "r": 0.xxx}
V2: {"type": "handshake_ok", "protocol_version": 1, "peer_count": N, "r": 0.0, "tick": N}
```

The sidecar's ring file is 29K ticks stale. Fix: rebuild sidecar against V2 wire
format, or add V1-compat response mode to the V2 daemon's handshake.
