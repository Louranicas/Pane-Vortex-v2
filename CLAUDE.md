# Pane-Vortex V2 — The Habitat Coordination Daemon

> **Kuramoto-coupled oscillator field for multi-pane Claude Code fleet coordination**
> **STATUS: SCAFFOLDED** — 8 layers, 41 modules, 52 Rust files, awaiting implementation
> **ULTRAPLATE Service ID:** `pane-vortex` | **Port:** 8132 | **Batch:** 5
> **Plan:** `MASTERPLAN.md` | **Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`

## Architecture (8 Layers, 41 Modules)

```
L1 Foundation (m01-m06):  Core types, errors, config, constants, traits, validation
L2 Services   (m07-m10):  Registry, health, lifecycle, API server
L3 Field      (m11-m15):  Sphere, field state, chimera, messaging, app state
L4 Coupling   (m16-m18):  Network, auto-K, topology
L5 Learning   (m19-m21):  Hebbian STDP, buoy network, memory manager
L6 Bridges    (m22-m28):  SYNTHEX, Nexus, ME, POVM, RM, VMS, consent gate
L7 Coord      (m29-m36):  IPC bus, bus types, conductor, executor, cascade, suggestions, tick, persistence
L8 Governance (m37-m41):  Proposals, voting, consent declaration, data sovereignty, evolution (feature-gated)
```

## Build & Quality Gate (MANDATORY)

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```

**Order:** check → clippy → pedantic → test. Zero tolerance at every stage.

## Rules (Non-Negotiable)

### Code Quality
- **No `unwrap()` or `expect()` outside tests** — enforced via `[lints.clippy]` in Cargo.toml
- **No `unsafe`** — zero tolerance
- **Doc comments on all public items** — `///` for functions, `//!` for modules
- **50+ tests per layer minimum** — unit tests in same file, integration in tests/
- **Phase wrapping** — always `.rem_euclid(TAU)` after phase arithmetic
- **Lock ordering** — AppState BEFORE BusState. Always.

### Patterns (from PV v1 + ME v2)
- **Borrow checker** — extract values before cross-field mutations
- **Error propagation** — `?` operator with `PvError` enum, no panics
- **Feature gates** — `#[cfg(feature = "X")]` for evolution, governance
- **Consent gates** — every external control must pass `consent_gated_k_adjustment()`
- **Fire-and-forget bridges** — raw TCP HTTP, no hyper overhead
- **Amortised cleanup** — batch prune at threshold+50, not every tick

### Anti-Patterns (NEVER)
- `unwrap()` / `expect()` in production
- Chaining after `pkill` (exit 144 kills the chain)
- `cp` without `\` prefix (aliased to interactive)
- JSON to Reasoning Memory (TSV only!)
- stdout in daemons (SIGPIPE → death, BUG-018)
- `git status -uall` (memory issues on large repos)
- Modifying code without reading first
- Global k_mod without per-sphere consent option

## Key Constants

| Constant | Value | Module |
|----------|-------|--------|
| TICK_INTERVAL | 5s | m04_constants |
| COUPLING_STEPS_PER_TICK | 15 (adaptive) | m04_constants |
| KURAMOTO_DT | 0.01 | m04_constants |
| HEBBIAN_LTP | 0.01 (3x burst) | m04_constants |
| HEBBIAN_LTD | 0.002 | m04_constants |
| PHASE_GAP_THRESHOLD | π/3 rad | m04_constants |
| SYNC_THRESHOLD | 0.5 | m04_constants |
| TUNNEL_THRESHOLD | 0.8 rad | m04_constants |
| R_TARGET | 0.93 (dynamic) | m04_constants |
| K_MOD_BUDGET | [0.85, 1.15] | m04_constants |
| SPHERE_CAP | 200 | m04_constants |
| MEMORY_MAX_COUNT | 500 | m04_constants |
| SNAPSHOT_INTERVAL | 60 ticks | m04_constants |

## Dependency Layers

```
L8 → L7 → L6 → L5 → L4 → L3 → L1
                              → L2 → L1
```

L1 has zero internal dependencies. Each layer only imports from lower layers.

## Related

- **V1 Source:** `~/claude-code-workspace/pane-vortex/` (21,569 LOC, 412 tests)
- **ME v2 Exemplar:** `~/claude-code-workspace/the_maintenance_engine_v2/`
- **Master Plan:** `MASTERPLAN.md` in this directory
- **Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`
- **Skills:** `.claude/skills/deephabitat/SKILL.md`
