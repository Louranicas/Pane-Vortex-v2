---
date: 2026-03-19
tags: [workflow, scaffolding, pane-vortex-v2, session-040, gold-standard, reusable]
aliases: [Scaffolding Workflow, PV2 Workflow]
pinned: true
---

# PV2 Scaffolding Workflow

> **17-step reusable workflow producing 198 files, 17,120 lines of documentation.**
> **Session 040 | Duration: ~4 hours | Agents: 8 parallel | Ralph Loop: 7 generations**
> **RM:** `r69bbcf4100bc` | **POVM:** stored as workflow memory

---

## The 17 Steps

### Phase A: Research (30 min)

| Step | Action | Output |
|------|--------|--------|
| 1 | **Read ME v2 gold standard** — `the_maintenance_engine_v2/src/m1_foundation/` and `m2_services/`. Extract: `parking_lot::RwLock`, trait-first design, builder pattern, C1-C12 constraints, `TensorContributor`, `SignalBus`, module naming (m01_, m02_) | ME v2 pattern catalog |
| 2 | **Read PV v1 patterns** — `pane-vortex/ai_specs/patterns/` (79 patterns + 42 anti-patterns), `.claude/` folder (context, patterns, anti_patterns, queries, schemas) | Pattern + anti-pattern catalog |

### Phase B: Structure (30 min)

| Step | Action | Output |
|------|--------|--------|
| 3 | **Create directory structure** — 8 layer dirs (`src/m1_foundation` through `src/m8_governance`), `ai_docs/modules/`, `ai_specs/layers/`, `ai_specs/patterns/`, `.claude/queries/`, `.claude/schemas/`, `config/`, `migrations/`, `data/`, `hooks/`, `tests/`, `benches/`, `scripts/` | 28 directories |
| 4 | **Write Cargo.toml** — `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used/expect_used)]`, `parking_lot`, `thiserror`, `axum 0.8`, `rusqlite bundled`, `figment`, feature gates (`api`, `persistence`, `bridges`, `evolution`, `governance`) | Cargo.toml with lint rules |
| 5 | **Write lib.rs + 8 mod.rs + 41 module stubs** — ME v2-style doc comments with Layer/Module/Dependencies/Design Constraints/Related Documentation. Each mod.rs has single constraint block. | 52 Rust files |

### Phase C: Configuration (20 min)

| Step | Action | Output |
|------|--------|--------|
| 6 | **Write CLAUDE.md + CLAUDE.local.md** — Architecture overview, quality gate, rules, constants, implementation order, V3 plan status | 2 bootstrap files |
| 7 | **Write .claude folder** — `context.json` (module inventory), `status.json` (heartbeat), `patterns.json` (15 patterns), `anti_patterns.json` (15 anti-patterns), `queries/` (3 SQL files), `schemas/` (2 JSON schemas) | 10 operational files |
| 8 | **Write config/default.toml + production.toml** — 10 TOML sections, all constants documented | 2 config files |
| 9 | **Write 3 migration SQL files** — field tables (3), bus tables (6), governance tables (4) = 13 total | 3 SQL files |
| 10 | **Write 3 hooks** — `session_start.sh` (register sphere + bus listener), `post_tool_use.sh` (memory + frequency), `session_end.sh` (deregister + cleanup) | 3 executable scripts |

### Phase D: Documentation (2 hours, parallelized)

| Step | Action | Output |
|------|--------|--------|
| 11 | **Web search** — Kuramoto Rust, axum 0.8, Hebbian STDP, collective governance, SQLite WAL, tokio IPC | `WEB_RESEARCH.md` |
| 12 | **Ralph Loop 7 generations** — Verify scaffold, fix constraint duplicates, clean mod.rs, verify cross-refs, add .gitignore/README/benchmark/tests, improve stubs | Iterative improvement |
| 13 | **Launch ai_docs agent** — 20 files: INDEX, ONBOARDING, QUICKSTART (519L), ARCHITECTURE_DEEP_DIVE, CODE_MODULE_MAP, SCHEMATICS, ERROR_TAXONOMY, STATE_MACHINES, MESSAGE_FLOWS, PERFORMANCE, DEPLOYMENT_GUIDE + 8 module docs (L1-L8) | 20 files, 5,985 lines |
| 14 | **Launch ai_specs agent** — 28 files: 9 core specs (KURAMOTO, IPC, WIRE, API, DB, SECURITY, MODULE_MATRIX, CONSENT, CONSTRAINTS) + 8 layer specs (L1-L8) + 8 pattern files + INDEX + TASK_QUEUE + EVENT_SYSTEM | 28 files, 11,135 lines |

### Phase E: Quality (30 min)

| Step | Action | Output |
|------|--------|--------|
| 15 | **Fix ALL clippy warnings at source** — `cargo clippy --fix` for doc_markdown, manual fix for empty-line-after-doc-comments, lint priority in Cargo.toml. **NEVER suppress.** | Zero warnings |
| 16 | **Create Obsidian vault + MASTER INDEX** — Copy all docs/specs to vault/, create MASTER INDEX with 52 notes organized by category, bidirectional links to main vault | 60+ notes |
| 17 | **Create architecture diagrams** — 20 Mermaid diagrams covering bridges, IPC, tick loop, Kuramoto, governance | 2 schematic files |

### Phase F: Verification

```bash
# Scaffold: 35/35 PASS
bash scripts/verify-scaffold.sh

# Quality gate: 4/4 CLEAN
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check && \
cargo clippy -- -D warnings && \
cargo clippy -- -D warnings -W clippy::pedantic && \
cargo test --lib --release
```

---

## Key Patterns Applied

| Pattern | Source | Application |
|---------|--------|-------------|
| `parking_lot::RwLock` | ME v2 | Interior mutability for all shared state |
| Trait-first design | ME v2 | Every module exposes trait with `&self` methods |
| Builder pattern | ME v2 | `#[must_use]` on all builder methods |
| C1-C14 constraints | ME v2 + PV v1 | Documented in every mod.rs |
| 79 patterns + 42 anti-patterns | PV v1 | Catalogued in .claude/ and ai_specs/patterns/ |
| Fire-and-forget bridges | PV v1 | `tokio::spawn` for all bridge writes |
| Consent gates | PV v1 NA | `consent_gated_k_adjustment()` on all external influence |
| Feature gates | Design | `#[cfg(feature)]` for evolution, governance |
| Zero warning tolerance | PRIME DIRECTIVE | Fix at source, never suppress |

---

## Final Metrics

| Metric | Value |
|--------|-------|
| Total files | 198 |
| Rust stubs | 59 |
| ai_docs | 20 (5,985L) |
| ai_specs | 28 (11,135L) |
| Total documentation | 17,120 lines |
| Obsidian vaults | 2 (60 + 63 notes) |
| Architecture diagrams | 28 (8 existing + 20 new) |
| Quality gate | 4/4 CLEAN |
| Scaffold verification | 35/35 PASS |
| Gaps remaining | Zero |

---

## Reuse Instructions

To scaffold a new ULTRAPLATE service using this workflow:

1. Clone the directory structure from `pane-vortex-v2/`
2. Rename layers and modules for the target service
3. Update `Cargo.toml` dependencies for the service domain
4. Update `config/default.toml` with service-specific constants
5. Write migrations for the service's database schema
6. Run the 17-step workflow, adapting steps 11-14 for the service's architecture
7. Verify with quality gate (zero tolerance)

---

## Related

- [[The Habitat — Integrated Master Plan V3]] — the plan this scaffold implements
- [[MASTER INDEX]] — vault navigation hub
- [[QUICKSTART]] — comprehensive deployment guide
- [[CLAUDE]] — project bootstrap
- [[DESIGN_CONSTRAINTS]] — C1-C14 rules
- `[[Maintenance Engine — Architecture Schematic]]` — ME v2 gold standard source
- `[[Pane-Vortex — Fleet Coordination Daemon]]` — PV v1 pattern source
