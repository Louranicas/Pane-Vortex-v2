---
date: 2026-03-19
tags: [master-index, pane-vortex-v2, vault, navigation]
aliases: [Master Index, Index, Navigation]
pinned: true
---

# Pane-Vortex V2 — Master Index

> **52 notes | 8 layers | 41 modules | 12,540 lines of documentation**
> **Codebase:** `/home/louranicas/claude-code-workspace/pane-vortex-v2/`

---

## Start Here

| Note | Purpose |
|------|---------|
| [[HOME]] | Vault home — bidirectional links to main vault |
| [[MASTERPLAN]] | V3 plan — 499 lines, 99 Obsidian cross-refs, 5 phases |
| [[CLAUDE]] | Project bootstrap — architecture, quality gate, rules |
| [[ONBOARDING]] | 5-level progressive reading order for new Claude instances |
| [[QUICKSTART]] | Build, deploy, health check in 60 seconds |

---

## Architecture & Design

| Note | Content |
|------|---------|
| [[ARCHITECTURE_DEEP_DIVE]] | 8-layer architecture, Kuramoto model, Hebbian STDP, bridges, distributed brain anatomy |
| [[SCHEMATICS]] | 6 Mermaid diagrams: tick orchestrator, module deps, sphere lifecycle, decision FSM, bridge flow, IPC bus |
| [[CODE_MODULE_MAP]] | All 41 modules (m01-m41): layer, path, LOC target, dependencies, key types |
| [[STATE_MACHINES]] | 5 FSMs: sphere lifecycle, bus task, proposal, cascade, conductor decisions |
| [[MESSAGE_FLOWS]] | 6 sequence diagrams: tick cycle, registration, bus task, cascade, governance, bridge polling |
| [[ERROR_TAXONOMY]] | PvError 6-variant enum, recovery strategies, HTTP status mapping |
| [[PERFORMANCE]] | Latency targets: tick <50ms, bridge <100ms, IPC <1ms, SQLite <10ms |
| [[DEPLOYMENT_GUIDE]] | DevEnv integration, binary deploy, health checks, troubleshooting |
| [[WEB_RESEARCH]] | External resources: Kuramoto, axum 0.8, STDP, governance, SQLite WAL, tokio IPC |

---

## Specifications

### Core Specs

| Note | Lines | Content |
|------|-------|---------|
| [[KURAMOTO_FIELD_SPEC]] | 470 | The math: phase dynamics, coupling, auto-K, chimera O(N log N), order parameter |
| [[IPC_BUS_SPEC]] | 260 | Unix socket, NDJSON protocol, BusState, lock ordering, event routing |
| [[WIRE_PROTOCOL_SPEC]] | 355 | 13 frame types with fields, max lengths, serde, error codes |
| [[API_SPEC]] | 656 | 76 HTTP endpoints organized by module |
| [[DATABASE_SPEC]] | 448 | 3 migrations, 15 tables, WAL config, index strategy |
| [[SECURITY_SPEC]] | 339 | Loopback binding, sphere cap, NaN guards, socket permissions |
| [[MODULE_MATRIX]] | 318 | 41-module cross-reference: imports, exports, state, data flow |
| [[CONSENT_SPEC]] | 427 | Consent gate architecture, per-sphere k_mod, opt-out registry, data sovereignty |
| [[DESIGN_CONSTRAINTS]] | — | C1-C14 constraint definitions with enforcement methods |

### Layer Specs

| Note | Layer | Modules |
|------|-------|---------|
| [[L1_FOUNDATION_SPEC]] | L1 Foundation | m01-m06: types, errors, config, constants, traits, validation |
| [[L3_FIELD_SPEC]] | L3 Field | m11-m15: sphere, field state, chimera, messaging, app state |
| [[L7_COORDINATION_SPEC]] | L7 Coordination | m29-m36: bus, conductor, executor, tick orchestrator, persistence |
| [[L8_GOVERNANCE_SPEC]] | L8 Governance | m37-m41: proposals, voting, consent, data sovereignty, evolution |

---

## Patterns & Anti-Patterns

| Note | Count | Priority |
|------|-------|----------|
| [[RUST_CORE_PATTERNS]] | 25 patterns | P0 — no unwrap, phase wrapping, NaN guard, extract-before-mutate |
| [[ASYNC_PATTERNS]] | 12 patterns | P0 — graceful shutdown, timeout, select!, fire-and-forget |
| [[CONCURRENCY_PATTERNS]] | 12 patterns | P0 — Arc<RwLock>, lock ordering, minimize hold time |
| [[ERROR_PATTERNS]] | 10 patterns | P0 — unified PvError, ? operator, From impls |
| [[IPC_PATTERNS]] | 10 patterns | P1 — NDJSON framing, handshake-first, glob matching |
| [[BRIDGE_PATTERNS]] | 8 patterns | P1 — raw TCP HTTP, consent gate, stale detection |
| [[ANTIPATTERNS]] | 42 anti-patterns | P0 — NEVER: unwrap, unsafe, pkill chain, JSON to RM |

---

## Module Documentation

### Per-Layer Docs

| Note | Layer | Modules | Key Content |
|------|-------|---------|-------------|
| [[L1_FOUNDATION]] | L1 | m01-m06 | Type definitions, PvError design, Figment config, 4 core traits |
| [[L2_SERVICES]] | L2 | m07-m10 | 16-service registry, circuit breaker, Axum router |
| [[L3_FIELD]] | L3 | m11-m15 | PaneSphere (33 fields), decision engine, ghost traces, chimera |
| [[L4_COUPLING]] | L4 | m16-m18 | Kuramoto equation, Jacobi integration, auto-K, neighborhood |
| [[L5_LEARNING]] | L5 | m19-m21 | Hebbian LTP/LTD, burst detection, buoy lifecycle, memory pruning |
| [[L6_BRIDGES]] | L6 | m22-m28 | 6 bridges + consent gate, BUG-008 ME critical alert |
| [[L7_COORDINATION]] | L7 | m29-m36 | IPC bus, conductor PI, executor dispatch, tick 5-phase decomposition |
| [[L8_GOVERNANCE]] | L8 | m37-m41 | Proposals, voting, quorum, consent declaration, data sovereignty |

---

## Command References (from main vault)

| Note | Content |
|------|---------|
| [[Commands]] | Core command reference |
| [[Session 039 — ZSDE Nvim God-Tier Command Reference]] | 801L keymaps, 8 prefix groups, 14 chain recipes |
| [[Session 039 — Lazygit God-Tier Command Reference]] | 80+ keybindings, 6 ZSDE custom commands |
| [[Session 039 — Atuin and Yazi God-Tier Reference]] | Shell history mining, file navigation |
| [[Session 036 — Nvim-Bash Command Chaining Analysis]] | Tool chaining patterns |
| [[Zellij Master Skill — Bootstrap Reference]] | Synth devenv mastery |
| [[Zellij Navigation God-Tier — Session 035]] | Directional navigation |
| [[Swarm Orchestrator — Complete Reference]] | Fleet coordination |
| [[Newly created clude code slash commands]] | Claude Code slash commands |

---

## V3 Plan Phases

| Phase | Status | Key Notes |
|-------|--------|-----------|
| **V3.1 Diagnostics** | NEXT | Fix BUG-008, evolution 404s, CCM dead, ME frozen |
| **V3.2 Inhabitation** | BLOCKED | Multi-sphere ops, bus round-trips, Hebbian formation |
| **V3.3 Sovereignty** | BLOCKED | 7 NA-P gaps: consent, per-sphere k_mod, cascade rejection |
| **V3.4 Governance** | BLOCKED | Proposals, voting, quorum — the field finds its voice |
| **V3.5 Consolidation** | PARALLEL | RM noise, POVM categories, bacon config, MCP prototype |

---

## Cross-Vault Links

This vault links bidirectionally to:
- **Main vault:** `~/projects/claude_code/` (250+ notes)
  - [[The Habitat — Integrated Master Plan V3]]
  - [[Pane-Vortex — Fleet Coordination Daemon]]
  - [[The Habitat — Naming and Philosophy]]
  - [[ULTRAPLATE Master Index]]
  - [[Session 034e — NA Gap Analysis of Master Plan V2]]
- **Shared context:** `~/projects/shared-context/`

---

## Quality Gate

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check && \
cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release
```

**PRIME DIRECTIVE: Never suppress warnings. Zero tolerance. Fix at source.**

---

*52 notes | 12,540 lines of documentation | The Habitat V2 | The field accumulates.*
