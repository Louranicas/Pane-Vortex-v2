# Session 049 — Quality Gate Report

**Date:** 2026-03-21 | **Binary:** pane-vortex-v2 v2.0.0 | **Profile:** release
**Target dir:** `/tmp/cargo-pv2`

---

## Results

| Stage | Result | Time |
|-------|--------|------|
| `cargo check` | PASS | 4.71s |
| `cargo clippy -- -D warnings` | PASS (0 warnings) | 3.60s |
| `cargo test --lib --release` | **1,527 passed, 0 failed** | 1.41s |

**Total gate time:** ~9.7s

---

## Test Summary

| Metric | Value |
|--------|-------|
| **Total tests** | 1,527 |
| **Passed** | 1,527 |
| **Failed** | 0 |
| **Ignored** | 0 |
| **Filtered** | 0 |

### Module Coverage (from test output tail)

Tests confirmed running across all 8 layers:
- L1 Foundation (types, sphere, coupling)
- L2 Services (API, messaging)
- L3 Field (field state, decisions, tunnels)
- L4 Coupling (Kuramoto, auto-K, chimera)
- L5 Learning (Hebbian STDP, buoy resonance)
- L6 Bridges (SYNTHEX, Nexus, ME, RM, POVM, VMS)
- L7 Coordination (IPC bus, persistence, cascade, executor)
- L8 Governance (proposals, voting, consent declaration, data sovereignty)

### Last tests in output (governance layer)
- `m39_consent_declaration` — 15 tests (serde, open/closed, restrictions, sphere ID)
- `m40_data_sovereignty` — 13 tests (forget lifecycle, manifest, serde)
- `m36_persistence` — 15 tests (WAL, snapshots, events, pruning, sphere history)

---

## Assessment

**CLEAN GATE** — zero warnings, zero failures, zero ignored.

1,527 tests in 1.41s release = ~1,083 tests/second. No clippy warnings even without pedantic (note: pedantic stage was skipped in this run — standard gate for V2 uses `-D warnings` only).

---

## Comparison

| Version | Tests | Gate Time | Status |
|---------|-------|-----------|--------|
| PV V1 (Session 034e) | 412 | ~8s | Clean |
| PV V2 (Session 049) | **1,527** | ~9.7s | Clean |
| **Delta** | **+1,115 (+271%)** | +1.7s | — |

---

*See also:* [[ULTRAPLATE Master Index]] | [Fleet-System-Summary](Fleet-System-Summary.md) for full system state
