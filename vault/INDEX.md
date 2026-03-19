# Pane-Vortex V2 — Specification Index

> Navigation hub for all technical specification documents.
> Convention: `{CONCEPT}_SPEC.md` | Layer specs in `layers/` | Patterns in `patterns/`
> Supersedes: `pane-vortex/ai_specs/INDEX.md` (v1)
> Plan: `MASTERPLAN.md` | Obsidian: `[[The Habitat — Integrated Master Plan V3]]`

## Architecture Overview

| Layer | Modules | Purpose | Spec |
|-------|---------|---------|------|
| L1 Foundation | m01-m06 | Core types, errors, config, validation | [L1_FOUNDATION_SPEC](layers/L1_FOUNDATION_SPEC.md) |
| L2 Services | m07-m10 | Registry, health, lifecycle, API server | [L2_SERVICES_SPEC](layers/L2_SERVICES_SPEC.md) |
| L3 Field | m11-m15 | Sphere, field state, chimera, messaging | [L3_FIELD_SPEC](layers/L3_FIELD_SPEC.md) |
| L4 Coupling | m16-m18 | Kuramoto network, auto-K, topology | [L4_COUPLING_SPEC](layers/L4_COUPLING_SPEC.md) |
| L5 Learning | m19-m21 | Hebbian STDP, buoy network, memory | [L5_LEARNING_SPEC](layers/L5_LEARNING_SPEC.md) |
| L6 Bridges | m22-m28 | SYNTHEX, Nexus, ME, POVM, RM, VMS, consent gate | [L6_BRIDGES_SPEC](layers/L6_BRIDGES_SPEC.md) |
| L7 Coordination | m29-m36 | IPC bus, conductor, executor, tick | [L7_COORDINATION_SPEC](layers/L7_COORDINATION_SPEC.md) |
| L8 Governance | m37-m41 | Proposals, voting, consent, data sovereignty | [L8_GOVERNANCE_SPEC](layers/L8_GOVERNANCE_SPEC.md) |

## Specification Map

### Core Specifications (Priority 0 — read before writing code)

| File | Purpose | Audience | Priority |
|------|---------|----------|----------|
| [KURAMOTO_FIELD_SPEC.md](KURAMOTO_FIELD_SPEC.md) | Phase dynamics, coupling, chimera, Hebbian STDP, conductor | Architect, Implementer | P0 |
| [IPC_BUS_SPEC.md](IPC_BUS_SPEC.md) | Unix socket bus architecture, BusState, lock ordering | Implementer | P0 |
| [WIRE_PROTOCOL_SPEC.md](WIRE_PROTOCOL_SPEC.md) | NDJSON frame types, handshake, examples | Implementer, Client | P0 |
| [API_SPEC.md](API_SPEC.md) | 60+ HTTP endpoints, request/response schemas | Client, Hook author | P0 |
| [CONSENT_SPEC.md](CONSENT_SPEC.md) | Consent gate, k_mod isolation, sovereignty framework | Architect, NA reviewer | P0 |

### Coordination Specifications (Priority 0 — read for IPC bus implementation)

| File | Purpose | Audience | Priority |
|------|---------|----------|----------|
| [TASK_QUEUE_SPEC.md](TASK_QUEUE_SPEC.md) | Task lifecycle FSM, targeting, routing, expiry, dependencies | Implementer | P0 |
| [EVENT_SYSTEM_SPEC.md](EVENT_SYSTEM_SPEC.md) | 24 event types, subscriptions, delivery, backpressure, persistence | Implementer, Client | P0 |

### Infrastructure Specifications (Priority 1 — read during implementation)

| File | Purpose | Audience | Priority |
|------|---------|----------|----------|
| [DATABASE_SPEC.md](DATABASE_SPEC.md) | 3 migrations, table schemas, WAL, cross-DB queries | Implementer, Ops | P1 |
| [SECURITY_SPEC.md](SECURITY_SPEC.md) | Binding, caps, validation, socket permissions | Auditor, Deployer | P1 |
| [MODULE_MATRIX.md](MODULE_MATRIX.md) | 41-module cross-reference, imports, state access | Architect, Reviewer | P1 |

### Layer Specifications (Priority 1 — read when implementing each layer)

| File | Purpose |
|------|---------|
| [layers/L1_FOUNDATION_SPEC.md](layers/L1_FOUNDATION_SPEC.md) | m01-m06: types, errors, config, constants, traits, validation |
| [layers/L2_SERVICES_SPEC.md](layers/L2_SERVICES_SPEC.md) | m07-m10: service registry, health monitor, lifecycle, API server |
| [layers/L3_FIELD_SPEC.md](layers/L3_FIELD_SPEC.md) | m11-m15: sphere, field state, chimera, messaging, app state |
| [layers/L4_COUPLING_SPEC.md](layers/L4_COUPLING_SPEC.md) | m16-m18: Kuramoto network, auto-K, topology |
| [layers/L5_LEARNING_SPEC.md](layers/L5_LEARNING_SPEC.md) | m19-m21: Hebbian STDP, buoy network, memory manager |
| [layers/L6_BRIDGES_SPEC.md](layers/L6_BRIDGES_SPEC.md) | m22-m28: SYNTHEX, Nexus, ME, POVM, RM, VMS, consent gate |
| [layers/L7_COORDINATION_SPEC.md](layers/L7_COORDINATION_SPEC.md) | m29-m36: bus, conductor, executor, tick orchestrator |
| [layers/L8_GOVERNANCE_SPEC.md](layers/L8_GOVERNANCE_SPEC.md) | m37-m41: proposals, voting, consent, data sovereignty |

### Pattern Libraries (Priority 0 — read BEFORE writing any code)

| File | Count | Purpose |
|------|-------|---------|
| [patterns/RUST_CORE_PATTERNS.md](patterns/RUST_CORE_PATTERNS.md) | 25 | Core Rust idioms in this codebase |
| [patterns/ASYNC_PATTERNS.md](patterns/ASYNC_PATTERNS.md) | 12 | Tokio async: shutdown, cancellation, locks |
| [patterns/CONCURRENCY_PATTERNS.md](patterns/CONCURRENCY_PATTERNS.md) | 12 | Arc/RwLock, channels, spawn safety |
| [patterns/ERROR_PATTERNS.md](patterns/ERROR_PATTERNS.md) | 10 | PvError, ?, From impl, context |
| [patterns/IPC_PATTERNS.md](patterns/IPC_PATTERNS.md) | 10 | NDJSON framing, backpressure, reconnection |
| [patterns/BRIDGE_PATTERNS.md](patterns/BRIDGE_PATTERNS.md) | 8 | Raw TCP, fire-and-forget, consent gate |
| [patterns/ANTIPATTERNS.md](patterns/ANTIPATTERNS.md) | 42 | What NOT to do (learned from v1) |

## Reading Orders

### New contributor
1. `KURAMOTO_FIELD_SPEC.md` — understand the physics
2. `MODULE_MATRIX.md` — understand the 41-module architecture
3. `CONSENT_SPEC.md` — understand the sovereignty philosophy
4. `API_SPEC.md` — understand the external interface
5. `patterns/ANTIPATTERNS.md` — know what to avoid

### IPC bus implementation
1. `IPC_BUS_SPEC.md` — bus architecture
2. `WIRE_PROTOCOL_SPEC.md` — frame types
3. `TASK_QUEUE_SPEC.md` — task lifecycle FSM + routing
4. `EVENT_SYSTEM_SPEC.md` — event types + subscriptions + backpressure
5. `layers/L7_COORDINATION_SPEC.md` — coordination layer detail
6. `patterns/IPC_PATTERNS.md` — NDJSON + socket patterns
7. `patterns/ASYNC_PATTERNS.md` — shutdown + cancellation

### Governance implementation
1. `CONSENT_SPEC.md` — sovereignty framework
2. `layers/L8_GOVERNANCE_SPEC.md` — governance layer detail
3. `DATABASE_SPEC.md` — governance tables
4. `API_SPEC.md` — governance endpoints
5. `patterns/BRIDGE_PATTERNS.md` — consent gate pattern

### Security review
1. `SECURITY_SPEC.md` — hardening measures
2. `API_SPEC.md` — endpoint validation
3. `WIRE_PROTOCOL_SPEC.md` — input limits
4. `CONSENT_SPEC.md` — data sovereignty

## Quick Reference: Key Constants

| Constant | Value | Module | Spec |
|----------|-------|--------|------|
| TICK_INTERVAL | 5s | m04 | KURAMOTO_FIELD_SPEC |
| COUPLING_STEPS_PER_TICK | 15 (adaptive) | m04 | KURAMOTO_FIELD_SPEC |
| KURAMOTO_DT | 0.01 | m04 | KURAMOTO_FIELD_SPEC |
| HEBBIAN_LTP | 0.01 (3x burst) | m04 | KURAMOTO_FIELD_SPEC |
| HEBBIAN_LTD | 0.002 | m04 | KURAMOTO_FIELD_SPEC |
| R_TARGET | 0.93 (dynamic) | m04 | KURAMOTO_FIELD_SPEC |
| PHASE_GAP_THRESHOLD | pi/3 rad | m04 | KURAMOTO_FIELD_SPEC |
| SYNC_THRESHOLD | 0.5 | m04 | KURAMOTO_FIELD_SPEC |
| K_MOD_BUDGET | [0.85, 1.15] | m04 | CONSENT_SPEC |
| SPHERE_CAP | 200 | m04 | SECURITY_SPEC |
| MEMORY_MAX_COUNT | 500 | m04 | L3_FIELD_SPEC |
| SNAPSHOT_INTERVAL | 60 ticks | m04 | DATABASE_SPEC |

## Project Pointers

| Resource | Location |
|----------|----------|
| Source code (v2) | `pane-vortex-v2/src/` (41 modules, 52 files) |
| Source code (v1) | `pane-vortex/src/` (21,569 LOC, 412 tests) |
| Config | `pane-vortex-v2/config/default.toml` (87 lines) |
| Migrations | `pane-vortex-v2/migrations/` (3 SQL files) |
| Master Plan | `pane-vortex-v2/MASTERPLAN.md` |
| Bus Schema | `pane-vortex-v2/.claude/schemas/bus_frame.schema.json` |
| Obsidian | `[[The Habitat — Integrated Master Plan V3]]` |
| v1 Specs | `pane-vortex/ai_specs/` (21 files, ~5,900+ lines) |

## Conventions

- Every spec has: Overview, Design Principles, Architecture, Data Structures, Algorithms, Testing Strategy, Anti-Patterns
- Code examples use Rust for types, shell for usage
- Constants reference module + name + value
- Phase values always in radians unless stated otherwise
- All struct field lists sorted: identity, oscillator, memory, coupling, status, consent
- Mermaid for diagrams; ASCII fallback in terminals

## Versioning

Specs track the scaffolded state of pane-vortex-v2 as of Session 040 (2026-03-19).
Update specs when implementation diverges. Mark sections with `[IMPLEMENTED]` or `[PLANNED]`.
