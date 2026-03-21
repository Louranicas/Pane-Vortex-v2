# Session 044 — Fleet Orchestration Pioneer

## Overview
First systematic exploration of Claude-as-orchestrator across 5 surfaces (2 Claude Code instances + 2 shell panes + orchestrator). Discovered capacities, limits, and cascading patterns.

## Surfaces Used
| Surface | Type | Tab:Pos | Role |
|---------|------|---------|------|
| Claude-PV2-Command | Orchestrator (ME) | 1:left | Coordination, synthesis, RM writes |
| Claude-PV1-Sidecar | Claude Code | 1:top-right | IPC flow analysis, atuin mining |
| Claude-PV2-Main | Claude Code | 1:bottom-right | Bridge comparison, consent analysis |
| BETA-Shell-1 | Shell | 5:top-right | Health scans, DB mines, module census |
| GAMMA-Shell-1 | Shell | 6:top-right | Bridge probes, relay consumer, field mining |

## Cascading Patterns Discovered

### Pattern 1: Signal-File Chaining
```
BETA writes results → /tmp/arena/tier1.signal
GAMMA polls for signal → reads tier1 results → writes tier2
Orchestrator polls tier2.signal → synthesizes
```
**Verdict:** Works. Needs 15-20s poll timeout.

### Pattern 2: Producer-Consumer Relay
```
BETA samples PV tick 5x at 1s intervals → relay-N.json
GAMMA sleeps, then reads all samples → computes velocity
Orchestrator verifies prediction accuracy
```
**Result:** Measured velocity = 1.0 ticks/sec. Prediction accurate.

### Pattern 3: Parallel Fan-Out
```
Orchestrator dispatches to BETA + GAMMA + Claude-Main + Sidecar simultaneously
Each writes to /tmp/arena/mega-SN-*.txt
Orchestrator collects when signals arrive
```
**Result:** 7/7 data files delivered. Shell <1s, Claude 30-60s.

### Pattern 4: Claude-to-Claude Intelligence
```
Claude-Main reads 2 source files → writes comparative analysis
Orchestrator reads analysis → records to RM → dispatches follow-up
```
**Result:** 12-line bridge comparison + 7-gap consent analysis produced.

## Capacity Findings

| Capability | Shell Panes | Claude Instances |
|-----------|-------------|-----------------|
| Execution speed | <1s | 30-60s |
| Command format | Bash (max ~250 chars) | Natural language prompts |
| Reliability | High (short commands) | High (once submitted) |
| Best for | Data gathering, DB queries | Code analysis, synthesis |
| Submission | fleet-ctl dispatch or write-chars + Enter | write-chars + key 13 (CR) |
| Gotcha | Long commands garble >300 chars | Need "use bash to run:" prefix |

## Limits Discovered

1. **Claude Code instances are NOT shell panes** — `write-chars` sends prompts, not commands
2. **Long dispatch commands corrupt** — keep under 250 chars or use file-based handoff
3. **Claude prompt submission** — key code 13 (CR) works, key code 10 (LF) does not
4. **RM JSON escaping** — complex inline python parsing fails; use file-based approach
5. **Signal file polling** — 10s too short for multi-service probes
6. **field_tracking.db** — V1 data only (ticks up to ~27K), V2 daemon doesn't write here

## Key Intelligence Produced

### [[Bridge Patterns|Bridge Comparison]] (from Claude-Main)
- SYNTHEX: bidirectional, 6-tick poll, simple thermal response, 8KB buffer
- Nexus: read-only, 60-tick deep poll, strategy enum (Aligned/Partial/Diverging/Incoherent), 16KB buffer
- Shared: both duplicate `extract_body()` and `raw_http_get` TCP boilerplate — refactor candidate

### [[Consent Flow Analysis]] (from Claude-Main)
- M28 ConsentGate and M37 Proposals operate on **separate consent planes** with no wiring
- **7 gaps identified:**
  1. GAP-1: No actuator — `approved_unapplied()` returns proposals but nothing executes them
  2. GAP-2: Proposal→Gate feedback missing — governance can't widen/narrow K_MOD_BUDGET
  3. GAP-3: POVM/RM/VMS bridges bypass consent entirely
  4. GAP-4: `divergence_requested` flag declared but never checked
  5. GAP-5: Per-sphere override mechanism unconnected to proposals
  6. GAP-6: No proposable opt-out variant
  7. GAP-7: 5-tick voting window too short for 60-tick Nexus poll intervals

### Module Census (from BETA shell)
- 29,226 lines, 1,439 tests, 111 structs, 44 pub_fns across 41 modules
- L6 Bridges largest: 6,244 lines, 372 tests
- m10_api_server biggest single module: 1,985 lines
- m30_bus_types highest test count: 62 tests

### Field State Analysis
- 2 sync clusters: 5 members (local_r=0.936) and 11 members (local_r=0.870)
- Clusters phase-gapped at ~155 degrees (5.95 vs 3.24 rad)
- k rising (0.224→0.350) but global r stuck at 0 — coupling energy absorbed by gap
- 100 tunnels, 18 spheres, fleet_mode=Full

## Arena Statistics
- 51+ files, 272K+ data
- Sources: 16-port health scan, deep service probes, POVM pathways, RM entries, tick relay, module census, bridge analysis, consent flow, field tracking
- All written to `/tmp/arena/`

## Tools Used
- `fleet-ctl dispatch/read/status/broadcast` — shell pane orchestration
- `fleet-vortex conductor` — field-aware coordination
- `habitat-probe pulse` — 30ms system overview
- `zj_action` (fleet-nav.sh) — IPC-safe Zellij navigation with 150ms pacing
- SAN-K7 Nexus commands — service-health, compliance, best-practice, module-status
- `nvim --server /tmp/nvim.sock` — remote buffer operations
- Reasoning Memory (TSV PUT + search) — cross-session persistence
- `room.wasm` (Ctrl+y) and `harpoon.wasm` (Alt+v) — quick pane jumping

## Links
- [[Session 043 — Plan-Driven Scaffold Engine]]
- [[The Habitat — Integrated Master Plan V3]]
- [[ANTIPATTERNS]]
- [[CONSENT_SPEC]]

## Late-Session Discoveries

### K Anomaly Explained (from Claude-Main)
- Root cause: `auto_scale_k()` uses Kc = (2 * freq_spread / π) * N
- New sphere's hash-derived frequency expanded spread from 0.032 to 0.678 (21x)
- Combined with N: 17→18 = 22x total k increase
- **Not a bug** — it's a sensitivity cliff in the Kuramoto critical coupling formula
- Fix: dampen spread sensitivity or use median-based spread estimate

### R Dynamics Observed
- Field timeseries showed r climbing from 0 to 0.61 over 10 seconds
- Then dropped back to 0 — field oscillates between coherent and decoherent
- 2 sync clusters persist (5 + 11 members) with high local r (0.87-0.94)
- Global r fluctuates because clusters are phase-gapped ~155 degrees

### 7-Surface Coordination Achieved
- 4 Claude Code instances (Main, Sidecar, ALPHA, BETA) + 3 shell panes
- ALPHA and BETA were fresh instances (0 tokens) dispatched via write-chars + CR
- Each Claude analyses independently, writes to /tmp/arena/, orchestrator collects
- Session produced: 65+ arena files, 400K+ data, 6 Obsidian notes

### BETA Test Plan (25KB) — Key Findings
- Shell test count was WRONG: missed `#[tokio::test]` async tests
- m10_api_server actual: 53 tests (12 sync + 41 async), not 12
- 3 completely untested handlers: `phase_handler`, `steer_handler`, `bus_suggestions_handler`
- Phase handler (50 lines) has NaN guard + phase wrapping — untested = risky
- Steer handler has consent-gated receptivity — untested consent path

### God-Tier Coordination Achieved
- All 7 surfaces dispatched simultaneously (4 Claude + 3 shell)
- Shell chains: bash+git+nvim+atuin+sqlite+curl linked in single commands
- Arena: 85+ files, 532K+ data
- DB inventory: 24 DBs discovered, service_tracking.db has 20 tables (richest)
- Synergy top pair: SAN-K7 + SYNTHEX (score 98.7, 59 integration points)

### PI Controller Analysis (17KB from Sidecar)
- **NOT a PI controller** — it's a P-controller with emergent breathing blend
- No integral term; "I-like" behavior from 30% emergent breathing signal (R7)
- r_target: 0.93 (small fleet) → 0.85 (50+ spheres), EMA hysteresis
- At k=7.77: K_eff exceeds natural frequencies, sync achievable but oscillates
- Error r=-0.93, step clamped to -0.06/tick (reaching ceiling 1.5 in ~30 ticks)
- k_mod at ceiling 1.5 → K_eff = 7.77 × 1.5 = 11.66 — controller has NO MORE authority

### Service Tracking DB (20 tables!)
coordination_patterns, cross_agent_learnings, orchestration_graph,
service_communication_paths, inter_service_synergy — richest DB discovered
