# Intelligence Synthesis — Sidecar + Cross-Service Tissue

## POVM Pathway Analysis (74 strong pathways, weight > 0.5)

Top connections:
1. `nexus-bus:cs-v7 → synthex` (w=1.05) — CodeSynthor feeds SYNTHEX
2. `nexus-bus:devenv-patterns → pane-vortex` (w=1.02) — DevEnv patterns drive PV
3. `operator-028 → alpha-left` (w=1.0) — Human→fleet coupling
4. `5:top-right → opus-explorer` (w=1.0) — Cross-pane Hebbian pathway

**Insight:** The POVM pathway graph shows emergent topology from real usage. Pathways
form naturally from Hebbian STDP — no manual wiring. The strongest connections
(w>1.0, supercritical) show information routes that the system has learned are
high-value.

## Orchestration Graph (25 edges)

Two clusters:
1. **ME internal** — SYNTHEX_ORCHESTRATOR hub connecting M1 (Hebbian), M3 (Tensor),
   M6 (DevOps), M15 (System), M20 (Discovery)
2. **SAN-K7 external** — san-k7 modules connecting to synthex-core, service-mesh,
   analytics-engine, workflow-manager (weights 97-99)

**Missing:** No pane-vortex edges in the orchestration graph. PV's integration
is entirely through the consent gate bridge system, not the orchestration layer.

## Learned Patterns (57 — B1 through B10)

Key patterns with high reinforcement:
- B1: SQLite state query (125x token reduction) — reinforced 10 times
- B2: Quality gate chain (mandatory order) — reinforced 10 times
- B5: Output filtering (save bulk to /tmp then tail) — reinforced 9 times
- B3: Health check minimal (3-token) — reinforced 8 times

**Novel insight:** These patterns were independently discovered and reinforced
across sessions. They represent VALIDATED operational knowledge, not theoretical.

## Cross-Agent Learnings (3 universal)

1. **Token optimization** — cache knowledge, progressive disclosure, trust state
2. **Bash optimization** — B1-B10 integrated across 12 CLAUDE.md + 8 DBs
3. **Security** — Greptile dual-credential pattern

## Coordination Patterns (2)

- Task Orchestration: DevOps → SYNTHEX (request-response, 30/min)
- Status Broadcast: SYNTHEX → DevOps (pub-sub, 60/min)

## Sidecar Synergy Map

```
Sidecar (822K) ←→ Bus Socket ←→ PV Daemon ←→ 6 Bridges ←→ 16 Services
     ↕                                              ↕
WASM Plugin ←→ Zellij ←→ Fleet Panes ←→ Claude Instances
     ↕                                              ↕
RALPH Loop ←→ IntelligenceRouter ←→ POVM + RM + Nexus
```

## Key Discoveries

1. **RALPH in WASM** — The swarm plugin has a complete 5-phase quality loop that
   dispatches tasks, collects results, analyzes quality, and iterates until threshold met

2. **IntelligenceRouter** — Composite route scoring using 4 signals:
   - 0.3 × Hebbian pathway weight (from POVM)
   - 0.3 × Sphere receptivity
   - 0.2 × Nexus strategy alignment (Aligned/Partial/Diverging/Incoherent)
   - 0.2 × Historical success rate

3. **DistributedPlan** — v3.0 Phase 7 command exists but is untested. Decomposes
   tasks into subtasks, dispatches across fleet panes, collects + merges results.

4. **Wire protocol mismatch** — V1 sidecar can't parse V2 handshake envelope.
   Ring file stale at tick 27768 vs live tick 56743. Sidecar is functional but
   disconnected from the live bus.

5. **BusCommand protocol** — Submit, Cascade, Request commands with target/payload/prompt
   fields. The plugin can drive both fleet panes AND IPC bus from a single WASM process.

6. **Event categorization** — BusEvent.category() splits on first `.` — enables
   wildcard subscription (`field.*`, `sphere.*`, `task.*`)

## Architecture Tension

The sidecar exists because WASI cannot hold sockets. But the V2 pane-vortex-client
CAN hold sockets (native binary). The sidecar adds a layer of indirection
(FIFO → socket → JSONL) that introduces:
- Stale data (ring file 29K ticks behind live)
- Reconnect storms (V1/V2 mismatch)
- File I/O overhead (FIFO + JSONL writes)

**Question for V3:** Could the V2 client replace the sidecar for non-WASM use cases?
The client already handles handshake, subscribe, submit, cascade. The sidecar is only
needed for the WASM plugin bridge.
