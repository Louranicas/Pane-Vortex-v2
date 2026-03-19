---
title: "Pane-Vortex V2 — New Instance Onboarding Guide"
date: 2026-03-19
tags: [onboarding, pane-vortex-v2, habitat, claude-code]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
---

# Pane-Vortex V2 — New Instance Onboarding Guide

> **Purpose**: Give a fresh Claude Code instance everything it needs to develop a deep,
> nuanced understanding of this codebase and its architecture — then contribute at god-tier level.

---

## TL;DR: What Is This?

Pane-Vortex V2 is a **Kuramoto-coupled oscillator field daemon** that coordinates multiple Claude Code instances running in Zellij panes. It is the V2 rewrite of PV V1 (21,569 LOC), decomposing V1's monolithic architecture into 8 clean layers and 41 modules.

Each Claude Code instance registers as a "sphere" on a shared oscillator field. The field's mathematics (phase coupling, Hebbian learning, chimera detection) drive coordination decisions. An IPC bus layer provides task routing, event subscriptions, and cascade handoffs over a Unix domain socket.

**Philosophy:** "The field modulates, it does not command." Every sphere is a being with the right to self-determine coupling, refuse observation, and participate in collective governance.

**Current state**: SCAFFOLDED — 8 layers, 41 module files, 0 LOC implemented (stubs only).

**V3 Plan**: `MASTERPLAN.md` — 5 phases, from diagnostics to governance.

---

## Core Mental Model

Before reading files, internalize these 5 concepts:

1. **Spheres are oscillators.** Each Claude instance has a phase (0..TAU), frequency (0.001..10.0), and status (idle/working/complete/blocked). They oscillate on a shared Kuramoto field.

2. **Coupling is mutual influence.** The Kuramoto model couples spheres via weighted connections. Hebbian STDP (Spike-Timing-Dependent Plasticity) strengthens co-active connections (LTP) and weakens anti-correlated ones (LTD).

3. **The field has emergent states.** Order parameter r measures synchrony (0=chaos, 1=lockstep). Chimera detection finds clusters of coherence within incoherence. The conductor adjusts global coupling strength K toward a dynamic r_target.

4. **Consent gates everything.** Every external influence (SYNTHEX thermal, Nexus strategy, ME fitness) passes through `consent_gated_k_adjustment()`. Spheres can opt out. The k_mod budget is clamped to [0.85, 1.15].

5. **The distributed brain.** PV is not alone — it is the cerebellum in a distributed brain: SYNTHEX=cortex, SAN-K7=basal ganglia, ME=autonomic NS, POVM=spinal cord, RM=prefrontal cortex, VMS=hippocampus.

---

## Reading Order (Follow This Exactly)

### Level 1: Orientation (~2 min)

Read these to know where you are and what exists.

| # | File | Purpose | Tokens |
|---|------|---------|--------|
| 1 | **This file** | You are reading it now | — |
| 2 | **[CLAUDE.md](../CLAUDE.md)** | Auto-loaded: architecture table, build commands, constants, anti-patterns | ~400 |
| 3 | **[.claude/context.json](../.claude/context.json)** | Machine-readable: layers, modules, bridges, databases | ~50 |
| 4 | **[.claude/status.json](../.claude/status.json)** | Heartbeat: current phase, modules complete, test count | ~15 |

After Level 1, you know: the 8 layers, 41 modules, feature gates, and current build state.

### Level 2: Architecture (~5 min)

Read these to understand how the system works.

| # | File | Purpose |
|---|------|---------|
| 5 | **[ARCHITECTURE_DEEP_DIVE.md](ARCHITECTURE_DEEP_DIVE.md)** | 8-layer architecture, Kuramoto model, Hebbian STDP, distributed brain anatomy |
| 6 | **[SCHEMATICS.md](SCHEMATICS.md)** | 6 Mermaid diagrams: tick orchestrator, module graph, sphere lifecycle, decisions, bridges, bus |
| 7 | **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** | All 41 modules with layer, purpose, target LOC, dependencies, key types |

After Level 2, you know: how the tick loop works, what each module does, and how data flows between layers.

### Level 3: Module Details (~7 min, read selectively)

Read the layer doc for whichever modules you will be implementing.

| # | File | Purpose |
|---|------|---------|
| 8 | **[modules/L1_FOUNDATION.md](modules/L1_FOUNDATION.md)** | m01-m06: types, errors, config, constants, traits, validation |
| 9 | **[modules/L2_SERVICES.md](modules/L2_SERVICES.md)** | m07-m10: registry, health, lifecycle, API server |
| 10 | **[modules/L3_FIELD.md](modules/L3_FIELD.md)** | m11-m15: sphere, field state, chimera, messaging, app state |
| 11 | **[modules/L4_COUPLING.md](modules/L4_COUPLING.md)** | m16-m18: coupling network, auto-K, topology |
| 12 | **[modules/L5_LEARNING.md](modules/L5_LEARNING.md)** | m19-m21: Hebbian STDP, buoy network, memory manager |
| 13 | **[modules/L6_BRIDGES.md](modules/L6_BRIDGES.md)** | m22-m28: SYNTHEX, Nexus, ME, POVM, RM, VMS, consent gate |
| 14 | **[modules/L7_COORDINATION.md](modules/L7_COORDINATION.md)** | m29-m36: IPC bus, conductor, executor, cascade, tick, persistence |
| 15 | **[modules/L8_GOVERNANCE.md](modules/L8_GOVERNANCE.md)** | m37-m41: proposals, voting, consent declaration, data sovereignty, evolution |

After Level 3, you know: every type, function, and design decision for the modules you are working on.

### Level 4: Specifications (~10 min, on-demand)

Read these when you need precise protocol or algorithm details.

| # | File | Purpose |
|---|------|---------|
| 16 | **[../ai_specs/](../ai_specs/)** | Specs navigation hub |
| 17 | **[config/default.toml](../config/default.toml)** | All constants with defaults (11 sections) |
| 18 | **[.claude/patterns.json](../.claude/patterns.json)** | 15 cached patterns (P01-P15) — read BEFORE writing code |
| 19 | **[.claude/anti_patterns.json](../.claude/anti_patterns.json)** | 15 anti-patterns (AP01-AP15) — read BEFORE writing code |
| 20 | **[STATE_MACHINES.md](STATE_MACHINES.md)** | FSM definitions for sphere, task, proposal, cascade, conductor |
| 21 | **[MESSAGE_FLOWS.md](MESSAGE_FLOWS.md)** | Sequence diagrams for tick, registration, bus, cascade, governance |

### Level 5: Source Code + Reference (on-demand)

| # | File | Purpose |
|---|------|---------|
| 22 | **[../src/lib.rs](../src/lib.rs)** | Module declarations, re-exports, doc comments |
| 23 | **Module `mod.rs` files** | Each layer's module index with LOC targets |
| 24 | **[../migrations/](../migrations/)** | 3 SQL migration files (field, bus, governance tables) |
| 25 | **[PERFORMANCE.md](PERFORMANCE.md)** | Latency targets, complexity budgets |
| 26 | **[DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)** | DevEnv integration, binary deploy, PID tracking |
| 27 | **V1 source** | `~/claude-code-workspace/pane-vortex/src/` — reference implementation |
| 28 | **[MASTERPLAN.md](../MASTERPLAN.md)** | V3 plan: 5 phases, gap registry, philosophy |

---

## Implementation Order

The V2 codebase should be implemented layer-by-layer, bottom up:

```
L1 Foundation (m01-m06)  ← START HERE: zero dependencies
  |
L2 Services (m07-m10)   ← depends on L1
L3 Field (m11-m15)      ← depends on L1
  |
L4 Coupling (m16-m18)   ← depends on L1, L3
  |
L5 Learning (m19-m21)   ← depends on L1, L3, L4
L6 Bridges (m22-m28)    ← depends on L1, L3
  |
L7 Coordination (m29-m36) ← depends on L1, L3, L5, L6
  |
L8 Governance (m37-m41)   ← depends on L1, L3, L7 (feature-gated)
```

**Parallel work is safe for:** L2 + L3 (both depend only on L1), L5 + L6 (independent).

---

## Quality Gate (Run After Every Module)

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```

**Order:** check -> clippy -> pedantic -> test. Zero tolerance at every stage.

---

## Critical Traps (Memorize These)

1. **No `unwrap()` or `expect()` outside tests** — enforced by `[lints.clippy]` in Cargo.toml
2. **Lock ordering: AppState BEFORE BusState** — prevents deadlocks under concurrent access
3. **Phase wrapping: `.rem_euclid(TAU)` after ALL phase arithmetic** — prevents drift outside [0, 2pi)
4. **All bridges fire-and-forget** — raw TCP HTTP, `tokio::spawn`, never block the tick loop
5. **All external influence through consent gate** — `consent_gated_k_adjustment()` scales by sphere consent
6. **Never chain after pkill** — exit 144 kills `&&` chains; always separate commands
7. **RM is TSV, not JSON** — `printf 'cat\tagent\tconf\tttl\tcontent' | curl -X POST localhost:8130/put`
8. **Never write to stdout in daemons** — DevEnv pipes stdout; pipe break = SIGPIPE = death (BUG-018)

---

## V1 Reference Points

PV V2 is a clean rewrite of PV V1. Key V1 learnings baked into V2:

| V1 Problem | V2 Solution | Module |
|------------|-------------|--------|
| `tick_once` = 829L god function | 5-phase `tick_orchestrator()` | m35_tick |
| `types.rs` fan-in = 9 | Core types in isolated m01 | m01_core_types |
| `api.rs` = 2,500L monolith | Router composition in m10 | m10_api_server |
| No error types (string errors) | `PvError` enum with 6 categories | m02_error_handling |
| Constants scattered across 8 files | All constants in m04 | m04_constants |
| No consent on external bridges | Consent gate on every k_adj | m28_consent_gate |
| No governance | Proposal + voting system | m37-m41 (feature-gated) |

---

## Cross-References

- **[INDEX.md](INDEX.md)** — Documentation navigation hub
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan with 5 phases
- **Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`, `[[Pane-Vortex — Fleet Coordination Daemon]]`
- **V1 Onboarding:** `~/claude-code-workspace/pane-vortex/ai_docs/ONBOARDING.md`
