---
title: "Pane-Vortex V2 Documentation Index"
date: 2026-03-19
tags: [pane-vortex-v2, documentation, habitat, navigation]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
---

# Pane-Vortex V2 Documentation Index

> Navigation hub for AI-assisted development on the pane-vortex-v2 codebase.
> **8 layers, 41 modules, 52 Rust files — from the V1 monolith to a layered architecture.**
> **V3 Plan phases:** V3.1 (Diagnostics) -> V3.2 (Inhabitation) -> V3.3 (Sovereignty) -> V3.4 (Governance) -> V3.5 (Consolidation)

---

## Tier 1: Quick Start (Read First)

| Document | Purpose | Read When... |
|----------|---------|--------------|
| [ONBOARDING.md](ONBOARDING.md) | 5-level progressive reading order | New instance — start here |
| [QUICKSTART.md](QUICKSTART.md) | Build, deploy, health check, quality gate | First time building or deploying |
| [CODE_MODULE_MAP.md](CODE_MODULE_MAP.md) | All 41 modules with types and functions | Looking for a specific function or type |
| [ERROR_TAXONOMY.md](ERROR_TAXONOMY.md) | PvError enum mapped to modules | Debugging an error or adding a new variant |

## Tier 2: Architecture (Understand the System)

| Document | Purpose | Read When... |
|----------|---------|--------------|
| [ARCHITECTURE_DEEP_DIVE.md](ARCHITECTURE_DEEP_DIVE.md) | 8-layer architecture, Kuramoto model, Hebbian STDP, distributed brain | Understanding how the field works |
| [SCHEMATICS.md](SCHEMATICS.md) | 6 Mermaid diagrams: tick, modules, spheres, decisions, bridges, bus | Visual architecture orientation |
| [STATE_MACHINES.md](STATE_MACHINES.md) | FSMs: sphere, task, proposal, cascade, conductor | Tracing a lifecycle or adding states |
| [MESSAGE_FLOWS.md](MESSAGE_FLOWS.md) | Sequence diagrams: tick, registration, bus, cascade, governance | Understanding request/response paths |

## Tier 3: Deep Dive (Reference)

| Document | Purpose | Read When... |
|----------|---------|--------------|
| [PERFORMANCE.md](PERFORMANCE.md) | Latency targets, complexity budgets, benchmarks | Optimizing or reviewing O(N^2) code |
| [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) | DevEnv integration, binary deploy, PID tracking | Deploying or restarting the daemon |
| [modules/](modules/) | Per-layer deep dives (L1-L8) | Modifying a specific module |

## Layer Module Documentation

| File | Layer | Modules | Status |
|------|-------|---------|--------|
| [modules/L1_FOUNDATION.md](modules/L1_FOUNDATION.md) | L1 Foundation | m01-m06 | Scaffolded |
| [modules/L2_SERVICES.md](modules/L2_SERVICES.md) | L2 Services | m07-m10 | Scaffolded |
| [modules/L3_FIELD.md](modules/L3_FIELD.md) | L3 Field | m11-m15 | Scaffolded |
| [modules/L4_COUPLING.md](modules/L4_COUPLING.md) | L4 Coupling | m16-m18 | Scaffolded |
| [modules/L5_LEARNING.md](modules/L5_LEARNING.md) | L5 Learning | m19-m21 | Scaffolded |
| [modules/L6_BRIDGES.md](modules/L6_BRIDGES.md) | L6 Bridges | m22-m28 | Scaffolded |
| [modules/L7_COORDINATION.md](modules/L7_COORDINATION.md) | L7 Coordination | m29-m36 | Scaffolded |
| [modules/L8_GOVERNANCE.md](modules/L8_GOVERNANCE.md) | L8 Governance | m37-m41 | Scaffolded |

## Cross-References

### Project Files
- **[CLAUDE.md](../CLAUDE.md)** — Auto-loaded project instructions, build commands, constants, anti-patterns
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan: 5 phases, 8 alerts, 42 gap registry, philosophy
- **[.claude/context.json](../.claude/context.json)** — Machine-readable module/layer/database inventory
- **[.claude/status.json](../.claude/status.json)** — Current heartbeat (<15 tokens)
- **[.claude/patterns.json](../.claude/patterns.json)** — 15 cached patterns (P01-P15)
- **[.claude/anti_patterns.json](../.claude/anti_patterns.json)** — 15 anti-patterns (AP01-AP15)
- **[config/default.toml](../config/default.toml)** — All constants with defaults (11 sections)

### Specifications
- **[ai_specs/](../ai_specs/)** — Technical specs (to be ported from V1 and extended)
- **[ai_specs/layers/](../ai_specs/layers/)** — Per-layer detailed specifications
- **[ai_specs/patterns/](../ai_specs/patterns/)** — Pattern library

### Obsidian Notes
- `[[The Habitat — Integrated Master Plan V3]]` — Master plan
- `[[Pane-Vortex — Fleet Coordination Daemon]]` — V1 project note
- `[[Session 039 — Architectural Schematics and Refactor Safety]]` — Tick decomposition, risk hotspots
- `[[Session 036 — Complete Architecture Schematics]]` — Mermaid diagrams
- `[[Vortex Sphere Brain-Body Architecture]]` — Kuramoto field theory

### V1 Reference
- **V1 Source:** `~/claude-code-workspace/pane-vortex/` (21,569 LOC, 412 tests)
- **V1 Docs:** `~/claude-code-workspace/pane-vortex/ai_docs/` (21 files)
- **V1 Specs:** `~/claude-code-workspace/pane-vortex/ai_specs/` (21 files, 79 patterns)
