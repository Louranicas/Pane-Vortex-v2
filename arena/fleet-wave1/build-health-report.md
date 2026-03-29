# Build Health Report — Pane-Vortex V2

> **Agent:** GAMMA-LEFT | **Date:** 2026-03-21 | **Task:** CLUSTER-TASK build health

---

## Quality Gate

| Check | Result | Time |
|-------|--------|------|
| `cargo test --lib --release` | **1,527 passed, 0 failed, 0 ignored** | 1.37s |

**Verdict: ALL GREEN.** Zero failures, zero ignored.

---

## Recent Commits (10)

| Hash | Message | Type |
|------|---------|------|
| `a722a6b` | BUG-028 — V1 sidecar wire compat for subscribe/event responses | fix |
| `6fa51d9` | BUG-031 — Wire Hebbian STDP into tick orchestrator Phase 2.5 | fix |
| `ea06b35` | BUG-029 — submit --target flag no longer parsed as description | fix |
| `73314ad` | Deploy Session 044 remediation plan — 7 GAPs + 137 tests | feat |
| `ac0e9ac` | Silence V1 sidecar Ping keepalive warnings | fix |
| `e9c4258` | V1 wire protocol compat for swarm sidecar handshake | fix |
| `7255305` | Fix 5 API gaps for V1 hook compatibility + Obsidian schematics | feat |
| `cb347aa` | Wire IPC listener + 27 API routes — full daemon operational | feat |
| `18027ca` | Wire daemon + client binaries — daemon starts and serves on :8132 | feat |
| `b8a08f5` | Implement L6 Bridges + L7 Coordination + L8 Governance — 1329 tests | feat |

**Pattern:** 5 bug fixes (BUG-028/029/031, sidecar compat, keepalive) + 5 feature commits. Recent work focused on V1 wire compatibility and remediation.

---

## Uncommitted Changes

12 files modified: **+847 insertions, -833 deletions**

| File | Lines Changed | Risk Level |
|------|---------------|------------|
| `src/m6_bridges/mod.rs` | +280 | MEDIUM — new bridge orchestration |
| `src/bin/main.rs` | +183 | LOW — probe binary |
| `src/m6_bridges/m22_synthex_bridge.rs` | +122 | MEDIUM — active bridge |
| `CLAUDE.local.md` | +123 | N/A — docs |
| `src/m2_services/m10_api_server.rs` | +75 | MEDIUM — new routes |
| `src/m7_coordination/m35_tick.rs` | +47 | HIGH — critical path |
| `src/m8_governance/m37_proposals.rs` | +42 | LOW |
| `src/m7_coordination/m29_ipc_bus.rs` | +29 | LOW |
| `bacon.toml` | +10 | N/A — tooling |
| `.claude/skills/*` | various | N/A — skills |
| `src/m3_field/m15_app_state.rs` | +3 | LOW |

**Note:** These changes pass all 1,527 tests but are NOT in the currently running binary (built from committed state at `a722a6b`).

---

## Running Daemon

| Metric | Value |
|--------|-------|
| PID | 3828125 |
| Binary | `pane-vortex-v2/bin/pane-vortex` (5.7MB) |
| Built | 2026-03-21 12:19 |
| Status | **healthy** |
| Tick | 72,874 |
| Spheres | 34 |
| r | 0.685 |
| k | 1.5 |
| k_modulation | 0.85 |
| Fleet mode | Full |

---

## Test Growth Trajectory

| Milestone | Tests | Commit |
|-----------|-------|--------|
| L6+L7+L8 implementation | 1,329 | `b8a08f5` |
| +V1 hook compat | ~1,350 | `7255305` |
| +Session 044 remediation | 1,466 (+137) | `73314ad` |
| Current (HEAD + uncommitted) | **1,527** | working tree |

**+198 tests since initial L6-L8 deploy.** Growth rate: ~50 tests per commit cycle.

---

## Summary

```
BUILD:    GREEN  — 1,527/1,527 tests passing in 1.37s
DAEMON:   GREEN  — healthy, tick 72874, 34 spheres
BINARY:   STALE  — running binary is behind working tree by 847 lines
GIT:      DIRTY  — 12 files uncommitted (high-risk: tick loop, bridge, API)
ROLLBACK: READY  — V1 binary at pane-vortex/bin/ (Mar 17)
```

**Action needed:** Commit + rebuild to pick up the 847 uncommitted lines, or accept that the running binary is stable at the `a722a6b` snapshot.

---

BUILD-HEALTH-COMPLETE
