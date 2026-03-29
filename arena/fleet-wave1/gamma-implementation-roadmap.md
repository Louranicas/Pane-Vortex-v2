# MOMENTUM GAMMA: Combined Implementation Roadmap

> **Agent:** GAMMA-MOMENTUM | **Date:** 2026-03-21 | **Tick:** 74,517
> **Sources:** `subagent-5-new-synergies.md`, `subagent-hook-points.md`
> **Current state:** r=0.677, 35 spheres, 3/6 bridges live, health ~49/100

---

## Deployment Tiers

### Tier 0: Prerequisites (before anything else)

| Item | Action | Effort | Blocks |
|------|--------|--------|--------|
| **V2 rebuild** | Commit 847 lines, build release, hot-swap binary | 5 min | Everything in Tiers 1-3 |
| **ME emergence cap** | Raise 1000→5000 via config/API + restart | 10 min | Synergy 2 |

These two are independent and can execute in parallel. Without V2 rebuild, 3 bridges remain stale, Hebbian STDP is empty, and thermal cascade cannot start. Without ME cap raise, the evolution engine stays deadlocked.

---

### Tier 1: Quick Wins (30 min total, highest ROI)

Deploy these first — they require minimal code and produce immediate measurable impact.

| # | Item | Type | Effort | Impact | Dependencies |
|---|------|------|--------|--------|-------------|
| 1 | **Hook 2: SessionStart** — sphere registration + IPC bus connect | Hook | 15 min | Saves 40-60s per agent startup, ensures every Claude instance registers a sphere | V2 deployed |
| 2 | **Hook 5: Stop** — sphere deregister + session crystallization | Hook | 10 min | Prevents ghost buildup, triggers POVM crystallization on exit | V2 deployed |
| 3 | **Synergy 5: Bus Diversity Amplification** | Config | 5 min | Already partially done (QW1 unblocked 7 spheres). Complete by ensuring status updates flow → diverse decision actions → bus events diversify | V2 deployed |

**Combined impact:** +15-20 health points. Fleet lifecycle becomes automatic (register on start, deregister on stop), bus diversity improves from monotone IdleFleet.

---

### Tier 2: Core Wiring (2-3 hours total, structural improvements)

These require code changes in V2 source — modifications to tick loop, bridge modules, or new hook scripts.

| # | Item | Type | Effort | Impact | Dependencies |
|---|------|------|--------|--------|-------------|
| 4 | **Hook 3: PostToolUse** — POVM pathway recording | Hook | 20 min | Raises co_activation from 0% to ~100% coverage. Every tool use fires a Hebbian pathway update. Fixes the "write-only POVM" problem. | V2 deployed + Hook 2 |
| 5 | **Synergy 1: POVM-SYNTHEX Crystallisation Loop** | Code | 45 min | When SYNTHEX temp crosses threshold, trigger POVM crystallization for high-weight memories. Wires HS-001→crystallization API. Prevents memory decay. | V2 deployed, SYNTHEX heating |
| 6 | **Synergy 3: Harmonic Damping via Spectrum** | Code | 60 min | Feed l2_quadrupole back into IQR K-scaling. When quadrupole > 0.7 (cluster detected), boost K to break degeneracy. Requires reading `/field/spectrum` in tick loop and adjusting `auto_scale_k`. | V2 deployed |
| 7 | **Hook 7: PreCompact** — handoff context serialization | Hook | 30 min | Before context compaction, serialize sphere state + field snapshot to cascade handoff file. Enables seamless multi-window continuation. | V2 deployed |

**Implementation order:** 4 → 5 → 6 → 7 (each builds on the previous — PostToolUse feeds POVM data that crystallization loop needs, spectrum damping needs active field dynamics from Hebbian).

**Combined impact:** +20-25 health points. POVM becomes a living memory (co-activations > 0, crystallization preserves important memories), phase clusters break via harmonic feedback, context survives compaction.

---

### Tier 3: Advanced Autonomy (4-6 hours total, behavioral changes)

These are the most complex — they embed decision logic into spheres and cross-service pipelines.

| # | Item | Type | Effort | Impact | Dependencies |
|---|------|------|--------|--------|-------------|
| 8 | **Synergy 4: Governance Auto-Voting** | Code | 90 min | Each sphere evaluates proposals against local fitness (r proximity, phase diversity, coupling health) and votes autonomously. Requires: fitness evaluation function per sphere, vote decision logic, tick-loop integration. Mass-vote script (proven in Wave-8) serves as interim. | V2 deployed, Tier 2 items |
| 9 | **Synergy 2: RM-ME Emergence Corridor** | Code | 120 min | RM knowledge entries seed ME mutation proposals, bypassing the saturated emergence cap. Requires: RM→ME bridge (new), ME mutation injection API, category-to-parameter mapping. Most complex synergy — needs both ME and PV code changes. | ME cap raised, V2 deployed |
| 10 | **Hook 4: PreToolUse** — safety gate | Hook | 30 min | Block 7 anti-patterns (chaining after pkill, using `cat` instead of Read, etc.). Low complexity but high caution — false positives block legitimate work. | None (can deploy standalone) |
| 11 | **Hook 1: UserPromptSubmit** — field state injection | Hook | 45 min | Inject PV field state (r, decision, tunnels) into every prompt context. 20-30% reduction in recon queries. Requires: hook script that curls PV health + decision, formats as context block, prepends to prompt. | V2 deployed |
| 12 | **Hook 6: SubagentStop** — cascade result aggregation | Hook | 30 min | When subagent completes, auto-aggregate results into parent's arena directory. 2-3x faster multi-agent synthesis. | Hook 7 (PreCompact) |
| 13 | **Hooks 8-10: Extended PostToolUse + UserPromptSubmit** | Hook | 60 min | Auto arena file generation, consensus check before destructive ops, cross-service correlation recording. Polish items — defer until core hooks are stable. | All Tier 1-2 hooks |

**Combined impact:** +10-15 health points + 40-50% workflow automation. The system begins to self-govern (auto-voting), self-heal (RM→ME corridor), and self-protect (safety gates).

---

## Sequencing Diagram

```
WEEK 1 (Day 1)
├── Tier 0: V2 rebuild (5 min)     ←── ALPHA authorization
├── Tier 0: ME cap raise (10 min)  ←── parallel with V2
└── Tier 1: 3 quick wins (30 min)
    ├── Hook 2: SessionStart
    ├── Hook 5: Stop
    └── Synergy 5: Bus Diversity

WEEK 1 (Day 1-2)
└── Tier 2: Core wiring (2-3h)
    ├── Hook 3: PostToolUse → POVM recording
    ├── Synergy 1: POVM-SYNTHEX crystallisation
    ├── Synergy 3: Harmonic damping via spectrum
    └── Hook 7: PreCompact handoff

WEEK 1 (Day 2-3)
└── Tier 3: Advanced autonomy (4-6h)
    ├── Synergy 4: Governance auto-voting (code)
    ├── Synergy 2: RM-ME emergence corridor
    ├── Hook 4: PreToolUse safety gate
    ├── Hook 1: UserPromptSubmit field injection
    ├── Hook 6: SubagentStop aggregation
    └── Hooks 8-10: Extended automation
```

---

## Effort Summary

| Tier | Items | Total Effort | Health Impact | Automation |
|------|-------|-------------|---------------|------------|
| 0 | 2 prerequisites | 15 min | Unblocks all | — |
| 1 | 3 quick wins | 30 min | +15-20 pts | Fleet lifecycle |
| 2 | 4 core wiring | 2-3 hours | +20-25 pts | POVM alive, clusters break |
| 3 | 6 advanced | 4-6 hours | +10-15 pts | 40-50% automation |
| **Total** | **15 items** | **~7-10 hours** | **49 → ~95/100** | **Full autonomy** |

---

## Critical Path

```
V2 rebuild ──→ Hook 2 (SessionStart) ──→ Hook 3 (PostToolUse/POVM)
    │                                         │
    ├──→ Synergy 5 (bus diversity)            ├──→ Synergy 1 (crystallisation)
    │                                         │
    ├──→ Hook 5 (Stop/deregister)             ├──→ Synergy 3 (spectrum damping)
    │                                         │
    └──→ Synergy 4 interim (mass-vote)        └──→ Synergy 4 full (auto-vote)

ME cap raise ──→ Synergy 2 (RM-ME corridor) ──→ ME evolution restarts
```

The critical path runs through V2 rebuild → PostToolUse hook → POVM crystallisation → spectrum damping. The ME track is independent and can proceed in parallel.

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| Hook 4 (PreToolUse safety) false positives block work | Start with warning-only mode, promote to blocking after 1 week |
| Synergy 3 (spectrum damping) oscillation | Add EMA smoothing (α=0.9) to quadrupole reading before feeding K-scaling |
| Synergy 2 (RM-ME corridor) needs unknown ME APIs | Investigate ME config/API first; if no injection endpoint exists, build a thin shim |
| Governance auto-voting creates echo chamber | Require 10% minimum dissent threshold before applying; add sphere-local fitness evaluation |
| PostToolUse hook latency impacts UX | Fire-and-forget async — hook writes to POVM without blocking tool completion |

---

GAMMA-ROADMAP-COMPLETE
