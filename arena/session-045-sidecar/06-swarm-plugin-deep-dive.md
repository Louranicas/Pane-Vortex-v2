# Swarm Orchestrator Plugin — Deep Dive

> 1278 lines of WASM Zellij plugin | Session 045

## Architecture Overview

The swarm-orchestrator is a Zellij WASM plugin that:
1. Receives commands via named pipe (FIFO)
2. Dispatches work to fleet panes (3 modes)
3. Runs RALPH quality loops (5-phase autonomous ratchet)
4. Integrates with PV field state, POVM pathways, RM context, Nexus strategy
5. Manages distributed planning across fleet panes
6. Auto-launches and monitors the sidecar bridge

## Plugin State (SwarmOrchestrator struct)

```rust
struct SwarmOrchestrator {
    workers: BTreeMap<String, Worker>,      // Fleet workers
    ralph: Ralph,                            // RALPH loop state
    log: VecDeque<String>,                   // UI log (50 max)
    pv_cache: PvFieldCache,                  // Cached PV field state
    dispatch_mode: DispatchMode,             // PvExecutor|IpcBus|LegacyClaude
    capacity: CapacityManager,               // Max 8 workers
    bus_status: BusStatus,                   // IPC bus connection state
    intelligence: IntelligenceRouter,        // POVM + RM + Nexus fusion
    distributed_plan: DistributedPlan,       // Phase 7 distributed planning
}
```

## Event Loop (ZellijPlugin::update)

```
Timer (2s bus / 10s fallback):
  ├── check_ralph_progress()
  ├── read_event_buffer()        (Phase 1: sidecar ring file)
  ├── poll_pv_field()            (HTTP fallback every 5th tick)
  ├── pv_heartbeat_workers()     (every 2nd tick)
  ├── poll_intelligence()        (every 6th tick → POVM, RM, Nexus)
  ├── check_sidecar_health()     (every 5th tick)
  ├── check_pv_suggestions()     (Phase 5: auto-RALPH trigger)
  └── check_distributed_plan()   (Phase 7: cascade ack monitoring)
```

## Worker Lifecycle

```
Spawn → pv_register_sphere() → dispatch_task() → Working
  → handle_run_result() → Done → pv_update_sphere_status()
  → Kill → pv_deregister_sphere() → ghost trace created
```

All workers are registered as PV spheres. Status syncs to PV.
Heartbeats prevent ghost sweep during long tasks.

## RALPH Loop Implementation

```
ralph_start(task, max_iter=5, threshold=0.80)
  └── Phase: Reflect
      ├── dispatch_task() to all workers
      ├── pending = worker_count
      └── wait for all to report

All workers reported → ralph_advance()
  └── Phase: Analyze
      ├── build analysis prompt (includes all worker outputs)
      ├── run_command(claude analysis)
      └── handle_ralph_analysis()

Analysis result parsed → ralph_advance()
  └── Phase: Learn
      └── Extract insights from analysis

  └── Phase: Plan
      ├── if quality >= threshold → Complete
      └── else → refine task with insights → back to Reflect

  └── Phase: Harmonize
      └── merge best outputs, emit final result
```

## Intelligence-Weighted Dispatch

```
dispatch_intelligent(task, min_score)
  for each worker:
    score = compute_route_score(
      worker_id,
      task_hint,
      povm_pathways,      // Hebbian affinity
      receptivity,         // Idle=0.8, Working=0.2, Unknown=0.5
      nexus_strategy,      // Aligned=1.0 ... Incoherent=0.0
      historical_success   // Default 0.5
    )
  sort by score descending
  dispatch to all above threshold (or fallback to best)
```

## Distributed Planning (Phase 7)

```
start_distributed_plan(task, panes)
  ├── generate subtasks with rotating focus:
  │   1. architecture and API design
  │   2. implementation patterns and code structure
  │   3. testing strategy and edge cases
  │   4. security considerations and error handling
  │   5. performance optimization and scalability
  ├── send BusCascade to each target pane
  ├── each target writes to ~/projects/shared-context/planning/{plan_id}/
  └── monitor cascade.ack events until all collected
```

## PV Integration Points (10)

1. `pv_register_sphere()` — register worker as PV sphere
2. `pv_deregister_sphere()` — remove (triggers ghost trace)
3. `pv_update_sphere_status()` — sync worker status
4. `pv_heartbeat_workers()` — prevent ghost sweep
5. `poll_pv_field()` — HTTP poll /field/decision + /bus/suggestions
6. `handle_event_buffer()` — process bus events from ring file
7. `check_pv_suggestions()` — auto-RALPH from field suggestions
8. `dispatch_via_pv()` — POST to /executor/dispatch
9. `build_prompt()` — inject PV field state into worker prompts
10. `pv_status_line()` — dashboard display

## Cross-Service Integration Points (3)

1. **POVM** (:8125) — poll /pathways?min_weight=0.5 every 6 ticks
2. **RM** (:8130) — poll /recent?limit=10 every 6 ticks
3. **Nexus** (:8100) — poll /health for r_outer every 6 ticks

## POVM Pathway Topology (strongest hubs)

| Sphere | Connections | Max Weight |
|--------|-------------|------------|
| fascinating-tambourine:0 | 5 | 0.95 |
| 15 (internal) | 4 | 1.00 |
| beta-left | 4 | 1.00 |
| 14 (internal) | 3 | 1.00 |
| GAMMA-synthesizer | 3 | 0.995 |
| gamma-bot-right | 3 | 1.00 |
| ALPHA-explorer | 2 | 0.995 |
| BETA-analyst | 2 | 0.995 |

## Cross-Service Bridge State (Live)

```
SYNTHEX (:8090)
  Temperature: 0.5724 (target 0.50, slightly hot)
  Heat sources:
    Hebbian:   1.0 × 0.30 = 0.300
    Cascade:   0.0 × 0.35 = 0.000
    Resonance: 0.612 × 0.20 = 0.122
    CrossSync: 1.0 × 0.15 = 0.150
    Total heat contribution: 0.572

ME (:8080)
  Fitness: 0.616 (Degraded, Stable)
  Tick: 14400

All 6 bridges: NOT stale (all fresh)
```

## Key Code Patterns

1. **WASI→Shell bridge** — all network ops via `run_command` (curl) because WASI can't hold sockets
2. **Context-tagged results** — every `run_command` has a `BTreeMap<String,String>` context to route results
3. **Graceful degradation** — PV dispatch fails → fallback to legacy `claude -p` subprocess
4. **Event dedup** — `last_event_seq` prevents re-processing events from ring file tail
5. **Prompt injection** — POVM pathways, RM context, Nexus strategy injected into worker prompts
