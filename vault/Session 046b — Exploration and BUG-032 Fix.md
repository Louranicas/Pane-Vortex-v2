# Session 046b — Feature Exploration + BUG-032 Governance Fix

> **Date:** 2026-03-21
> **Scope:** Explore live PV2 features/capacity after 10-module wiring, discover + fix BUG-032
> **PID:** 3724323
> **Tests:** 1,524

## Live Field State (tick ~68500)

| Metric | Value |
|--------|-------|
| Spheres | 33 (7 blocked fleet panes, 26 idle ORAC7 sessions) |
| Order parameter r | 0.685 |
| Mean phase psi | 3.049 rad |
| Sync clusters | 2 (5 members + 28 members) |
| Chimera detected | No |
| Buoy tunnels | 100 (all overlap 1.0, hub node `4:bottom-right`) |
| Harmonics | l0=-0.68, l1=0.69, l2=0.83 (quadrupole dominant) |
| Fleet mode | Full |
| K | 1.5, k_modulation=1.0 |
| Decision action | HasBlockedAgents |
| Ghost traces | 2 (deploy-v2-test, explore-test) |

## Sphere Distribution

- **Blocked (7):** 6:top-right, 6:left, 4:left, 5:left, 5:top-right, 6:bottom-right, 5:bottom-right
- **Idle (26):** 4:bottom-right, orchestrator-044, 20× ORAC7:* sessions, 4:top-right

## Features Verified Working

### Suggestions Engine (NEW)
- 112+ suggestions generated across ticks
- All `SuggestReseed` type targeting blocked fleet panes
- Confidence 0.7, populated via BusState ring buffer (max 50)
- `/bus/suggestions` API returns live data (was hardcoded `[]`)

### SQLite Persistence (NEW)
- `field_tracking.db` (36KB) — schema created, 3 tables (field_snapshots, sphere_history, coupling_history)
- `bus_tracking.db` (4.5MB) — 287+ events, all `HasBlockedAgents` with full field state JSON
- Supplements JSON snapshot (not replacement)

### Bridge Polling (NEW)
- All 6 bridges reachable (smoke test confirms)
- Polls fire via `spawn_blocking` every 6/12/60 ticks
- `bridge_ms=0.01` per tick (Phase 2.7)
- Staleness tracking active

### IPC Bus
- 1 subscriber (sidecar, V1 wire compat connected)
- 0 pending tasks, 0 cascades
- Handshake: `PANE_VORTEX_ID="probe" pane-vortex-client connect` → OK

### Governance (FIXED — BUG-032)
- `/field/propose` → proposal created
- `/sphere/{id}/vote/{proposal_id}` → votes recorded
- `/field/proposals` → shows proposal with vote count
- Quorum: >50% of active spheres (17+ of 34 needed)
- Proposal `83efdf6d` submitted: `r_target → 0.88`, 2 votes (4:left, 5:left)

### Consent + Data Sovereignty
- `/sphere/4:left/consent` → all 4 opt-out flags visible, receptivity 1.0
- `/sphere/4:left/data-manifest` → buoys_count=3, memories=0, inbox=0, steps=66003

### Sphere Lifecycle
- Register → deregister → ghost trace creation verified
- Ghost shows `total_steps_lived`, persona, deregistered_at tick
- Re-registration creates fresh sphere

## BUG-032: ProposalManager `derive(Default)` Gives `max_active=0`

**Root cause:** `#[derive(Default)]` on `ProposalManager` produces `max_active=0, voting_window=0, quorum_threshold=0.0`. Serde deserializes the zero values from snapshot (they're present, not missing). Every `submit()` hits `0 >= 0` → permanently locked.

**Fix (3 parts):**
1. Replaced `#[derive(Default)]` with custom `impl Default` delegating to `new()` (100, 24, 0.5)
2. Added `#[serde(default = "fn")]` on each config field for missing-field case
3. Added `reconcile()` method on `ProposalManager` — called from `AppState::reconcile_after_restore()` — replaces zero values with operational defaults

**Files:** `m37_proposals.rs`, `m15_app_state.rs`
**Obsidian:** `[[ULTRAPLATE — Bugs and Known Issues]]` BUG-032

## Memory Records

| System | ID |
|--------|----|
| RM (wiring) | `r69bde8e20247` |
| RM (BUG-032) | `r69bdebaf0249` |
| POVM (wiring) | `d3c0c4b6` |
| POVM (exploration) | `30120621` |

## Cross-References

- `[[Session 046 — V2 Binary Deployed]]` — initial V2 deploy
- `[[Session 046b — Bridge Wiring Complete]]` — 10 modules wired
- `[[ULTRAPLATE — Bugs and Known Issues]]` — BUG-032
- `[[The Habitat — Integrated Master Plan V3]]` — V3.1 diagnostics + V3.4 governance
- `[[Pane-Vortex — Fleet Coordination Daemon]]` — parent project
