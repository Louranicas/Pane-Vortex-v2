# Pane-Vortex V2 — The Habitat (Private Project Instructions)

## BOOTSTRAP PROTOCOL (New Context Window)

**MANDATORY — execute these 3 steps at the start of EVERY new context window:**

1. **Run `/primehabitat`** — loads The Habitat: Zellij tabs, 16 services, IPC bus, memory systems, tool chains, NEVER list
2. **Run `/deephabitat`** — loads deep substrate: yazi, btm, bacon, MCP, pipe protocol, autocmds, cross-DB, vault nav, 55 custom binaries, all bridge details
3. **Read this file** — `CLAUDE.local.md` — implementation status, phase tracking, traps to avoid

**After bootstrap, WAIT for user to type `start coding` before writing ANY Rust implementation code.**

Bootstrap gives you god-tier understanding. But code changes require explicit authorization.

---

## Session 040 — Scaffold Phase (2026-03-19)

**Status:** SCAFFOLDED — 198 files, 8 layers, 41 modules, 17,120 lines docs, 26 diagrams, quality gate 4/4 CLEAN
**Plan:** `MASTERPLAN.md` in this directory (499 lines, 99 Obsidian cross-refs)
**Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`
**Workflow:** `vault/PV2 Scaffolding Workflow.md` — 17-step reusable process
**Schematics:** 3 files in ai_docs/ (26 Mermaid diagrams covering all bridges, IPC, field, governance)

## V3 Plan Status

| Phase | Focus | Status |
|-------|-------|--------|
| V3.1 | Diagnostics & Repair | NEXT (after `start coding`) |
| V3.2 | Inhabitation | BLOCKED by V3.1 |
| V3.3 | Sovereignty | BLOCKED by V3.2 |
| V3.4 | Governance | BLOCKED by V3.3 |
| V3.5 | Consolidation | PARALLEL with V3.2+ |

## The `start coding` Command

When user types **`start coding`**, execute in this order:

1. Verify quality gate passes (cargo check + clippy + pedantic + test)
2. Read `MASTERPLAN.md` Phase V3.1 items
3. Begin implementation layers bottom-up (see Implementation Order below)
4. Quality gate after EVERY module — zero tolerance, never suppress warnings
5. Commit after each completed layer

## Implementation Order (within `start coding`)

### Phase 1: L1 Foundation (m01-m06)
1. m01_core_types — Point3D, PaneId, SphereMemory, Buoy, Phase, Frequency
2. m02_error_handling — PvError enum, PvResult type alias, From impls
3. m03_config — Figment-based config from default.toml + env vars
4. m04_constants — All named constants from config/default.toml
5. m05_traits — Oscillator, Learnable, Bridgeable, Consentable traits
6. m06_validation — Input validators (phase bounds, freq clamp, string limits)
**Quality gate after each module. 50+ tests for L1.**

### Phase 2: L3 Field (m11-m15) — before L2 because L2 needs L3 types
1. m11_sphere — PaneSphere (33 fields from v1, refactored)
2. m12_field_state — FieldState, FieldDecision, tunnels
3. m13_chimera — Phase-gap detection
4. m14_messaging — PhaseMessage variants
5. m15_app_state — SharedState with Arc<RwLock>

### Phase 3: L4 Coupling + L5 Learning
1. m16-m18 coupling network
2. m19-m21 Hebbian STDP

### Phase 4: L2 Services (m07-m10)
1. m07 service registry
2. m08 health monitor
3. m09 lifecycle
4. m10 API server (axum routes)

### Phase 5: L6 Bridges (m22-m28)
All 6 service bridges + consent gate

### Phase 6: L7 Coordination (m29-m36)
IPC bus, conductor, executor, tick orchestrator, persistence

### Phase 7: L8 Governance (m37-m41) — feature-gated
Proposals, voting, consent declaration, data sovereignty

## Gold Standard Reference

**ME v2:** `~/claude-code-workspace/the_maintenance_engine_v2/`
- Module naming: `m01_`, `m02_`, etc.
- Layer structure: `src/m1_foundation/`, `src/m2_services/`
- Error handling: Custom error enum with `thiserror`
- Config: Figment with TOML + env overlay
- Tests: In-file `#[cfg(test)]` modules

**PV v1:** `~/claude-code-workspace/pane-vortex/`
- 79 patterns in `ai_specs/patterns/`
- 42 anti-patterns in `ai_specs/patterns/ANTIPATTERNS.md`
- .claude folder: context.json, patterns.json, anti_patterns.json, queries/, schemas/
- Operational hooks: session_start.sh, post_tool_use.sh, session_end.sh

## Traps to Avoid

1. Never chain after pkill (exit 144)
2. Always `\cp -f` (cp aliased)
3. TSV only for Reasoning Memory
4. Lock ordering: AppState before BusState
5. Phase wrapping: `.rem_euclid(TAU)` always
6. No stdout in daemons (SIGPIPE death)
7. BUG-008: ME EventBus has zero publishers — highest-impact fix in V3.1

## Working Directory
`/home/louranicas/claude-code-workspace/pane-vortex-v2`

## Quality Gate
```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```
