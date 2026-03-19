# Changelog — Pane-Vortex V2

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased] — Scaffolded (2026-03-19)

### Added
- 8-layer architecture with 41 module stubs (m01-m41)
- Cargo.toml with 15 dependencies, 5 feature gates, clippy deny rules
- `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]`
- MASTERPLAN.md — V3 plan (499 lines, 99 Obsidian cross-references)
- CLAUDE.md + CLAUDE.local.md — project bootstrap and implementation guide
- ai_docs/ — 20 documentation files (5,714 lines)
- ai_specs/ — 22+ specification files (6,826+ lines)
- config/default.toml — 10 sections, all constants documented
- config/production.toml — production overrides
- migrations/ — 3 SQL files, 13 tables (field, bus, governance)
- .claude/ — context.json, status.json, patterns.json, anti_patterns.json, 3 query files, 2 schemas
- hooks/ — session_start.sh, post_tool_use.sh, session_end.sh (with V3 consent stubs)
- tests/ — 6 integration test stubs (per-layer + cross-layer + bridge)
- benches/ — tick_loop.rs benchmark stub
- scripts/verify-scaffold.sh — 35-check verification script
- bacon.toml — 5-job config with CARGO_TARGET_DIR=/tmp/cargo-pv2
- vault/ — 53-note Obsidian vault with MASTER INDEX
- README.md, .gitignore

### Design Decisions
- ME v2 gold standard: parking_lot::RwLock, trait-first design, builder pattern
- PV v1 learnings: 79 patterns, 42 anti-patterns, C1-C14 design constraints
- Quality gate: check → clippy → pedantic → test, zero tolerance, never suppress
- Feature gates: `api`, `persistence`, `bridges`, `evolution`, `governance`

### Quality Gate
- cargo check: CLEAN
- cargo clippy -D warnings: CLEAN
- cargo clippy pedantic: CLEAN
- cargo test: CLEAN (0 tests — stubs only)

## V1 Reference

See `~/claude-code-workspace/pane-vortex/` — 21,569 LOC, 412 tests, 22 modules.
Sessions 012-039 development history in Obsidian vault.
