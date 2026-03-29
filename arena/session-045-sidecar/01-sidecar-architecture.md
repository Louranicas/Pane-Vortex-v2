# Sidecar Architecture — Session 045 Deep Exploration

> Explored 2026-03-21 by Claude Opus 4.6 (1M context)

## Data Flow (5 Layers)

```
Layer 1: WASM Plugin (Zellij)
  swarm-orchestrator.wasm (1.1M) / v2 (992K)
  Writes JSON commands to FIFO pipe
  Reads events from ring file tail
  Has RALPH loop: Reflect→Analyze→Learn→Plan→Harmonize
    |
    v
Layer 2: FIFO Named Pipe
  /tmp/swarm-commands.pipe (created Mar 18)
  One-way: plugin → sidecar
    |
    v
Layer 3: Sidecar (native Rust binary)
  ~/.local/bin/swarm-sidecar (822K, stripped ELF x86-64)
  PID: 22419 (running since Mar 15)
  Log: /tmp/swarm-sidecar.log (179KB)
  Maintains persistent Unix socket connection to PV bus
  Bridges WASI limitation (no socket access from WASM)
    |
    v
Layer 4: Unix Domain Socket
  /run/user/1000/pane-vortex-bus.sock
  NDJSON wire protocol, srwx------ (0700)
  SO_PEERCRED UID check, 200 connection cap
  Backlog: 4096
    |
    v
Layer 5: Events Ring File
  /tmp/swarm-events.jsonl (405 lines, 62KB)
  Sidecar writes events from bus → file
  Plugin reads tail for status updates
  Event types: field.tick (257), field.suggestion (140), field.decision (4), field.cascade (4)
```

## Wire Protocol V1/V2 Compatibility Issue

**FINDING:** Sidecar was compiled against V1 wire protocol. PV daemon is now V2.

V1 sidecar expects:
```json
{"tick": N, "peers": N, "r": 0.xxx}
```

V2 daemon returns:
```json
{"type": "handshake_ok", "protocol_version": 1, "peer_count": 1, "r": 0.0, "tick": N}
```

Result: Sidecar enters reconnect loop. The sidecar CONNECTS (TCP established) but fails to parse the handshake response because V2 wraps it in a typed envelope with `"type": "handshake_ok"`.

The V2 client binary (pane-vortex-client) handles this correctly — it was updated in Session 043 for V1 compat.

## Pipe Command Protocol (14 Commands)

From swarm-orchestrator/src/lib.rs:

| Command | Description | Phase |
|---------|-------------|-------|
| `Spawn` | Launch new worker with persona + task | Core |
| `Dispatch` | Send task to specific worker by ID | Core |
| `DispatchAll` | Broadcast task to all workers | Core |
| `Ralph` | Start RALPH loop (max_iter, threshold) | Core |
| `Advance` | Advance RALPH to next phase | Core |
| `Status` | Query fleet status | Core |
| `Kill` | Terminate specific worker | Core |
| `KillAll` | Terminate all workers | Core |
| `SetThreshold` | Change quality threshold | Config |
| `SetDispatchMode` | Change dispatch strategy | Config |
| `AutoRalph` | Enable/disable automatic RALPH on triggers | Config |
| `BusSubmit` | Submit task via IPC bus (v3.0 Phase 2) | IPC |
| `BusCascade` | Cascade handoff via IPC bus (v3.0 Phase 2) | IPC |
| `DistributedPlan` | Distributed planning across fleet panes (v3.0 Phase 7) | IPC |

## Backpressure Architecture (5 Defenses)

From sidecar-backpressure.sh:

1. **Health gate** — waits for PV daemon before attempting IPC
2. **Exponential backoff** — 2s initial, 2x multiplier, 120s max
3. **Rate limiter** — max N events/second to protect Zellij terminal buffers
4. **Circuit breaker** — stops retrying after 10 consecutive failures
5. **Event summarization** — aggregates rapid events into single-line digests

BUG-014 mitigation: prevents IPC broken pipe flood from killing Zellij.

## Synergy Map

| System A | System B | Score | Integration Points |
|----------|----------|-------|-------------------|
| swarm-orchestrator | reasoning-memory | 98.0 | 12 |
| pane-ctl | fleet-ctl | 97.0 | 6 |
| swarm-stack-v2.1 | ultraplate-devenv | 96.5 | 11 |
| swarm-orchestrator | obsidian-vault | 92.0 | 5 |

## Live State (at exploration time)

- PV daemon: V1 binary, PID 279371, tick 56743, 31 spheres, r=0.992
- Sidecar: PID 22419, uptime ~6 days, reconnect loop (V2 wire compat)
- Bus: 19 subscribers, 2 pending tasks, 1 pending cascade
- WASM plugins: V1 (1.1M) + V2 (992K), active in Zellij
- Fleet: 6 tabs, 17 panes, 4 working Claude instances

## Key Findings

1. **Sidecar V1/V2 wire mismatch** — highest-priority fix for next session
2. **RALPH loop in WASM** — 5-phase quality loop built into the swarm plugin
3. **DistributedPlan command** — v3.0 Phase 7 distributed planning exists but untested
4. **Ring file is stale** — latest tick 27768 vs live tick 56743 (sidecar not forwarding due to reconnect loop)
5. **Backpressure is production-grade** — 5-layer defense in the bash wrapper
6. **Synergy data shows swarm↔RM is the strongest integration** (98.0, 12 points)
