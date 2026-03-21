# Session 046b — Pioneer Exploration Report

> **Date:** 2026-03-21
> **Scope:** Deep feature exploration + capacity testing of fully-wired PV2 daemon
> **Spheres tested:** 37 (33 persistent + 4 pioneer)
> **PID:** 3724323 | **Ticks:** 68144–69400+

## Field Dynamics Observed

| Metric | Value | Notes |
|--------|-------|-------|
| Order parameter r | 0.549–0.994 | 0.994 raw phase r, 0.549 field r (cluster geometry) |
| Sync clusters | 2 (5 + 32) | Smaller cluster near phase 5.95, larger near 2.96 |
| Chimera | Not detected | Both clusters above sync threshold |
| Tunnels | 100 | All overlap=1.0, hub node `4:bottom-right` |
| Harmonics | l0=-0.70, l1=0.71, l2=0.78 | Quadrupole dominant — indicates bimodal phase distribution |
| K (auto-scaled) | 0.791 | Down from 1.5 due to IQR freq_spread estimation |
| k_modulation | 1.0 | Bridges polling but cached_adj still neutral |

## Hebbian STDP — Weight Differentiation Confirmed

| Pair | Initial | After 3 ticks co-Working | After idle | Mechanism |
|------|---------|--------------------------|------------|-----------|
| pioneer-1 ↔ pioneer-2 | 0.09 | 0.554 (LTP burst) | 0.499 (LTD decay) | Co-active LTP × 3 burst, then LTD at 0.002/tick |
| pioneer-1 ↔ pioneer-3b | 0.09 | 0.09 (no change) | 0.09 | Never co-active, no LTP |

This proves Hebbian STDP (BUG-031 fix) is live and differentiating. The coupling network adapts based on co-activity.

## Governance Pipeline — Full Lifecycle Verified

| Step | Result |
|------|--------|
| Submit proposal (`k_mod_budget_max → 1.25`) | `e23f0a8a` created |
| 20 spheres vote `approve` | 20/37 = 54% > 50% quorum |
| Phase 3.5 governance actuator | Auto-applied on next tick |
| Final status | `Applied` |

Previous proposals: 2 `Expired` (voting window elapsed before quorum).

## Suggestion Engine — 3,955 Generated

All `SuggestReseed` targeting 7 blocked fleet panes (4:left, 5:left, 5:top-right, 5:bottom-right, 6:left, 6:top-right, 6:bottom-right). Confidence 0.7. Ring buffer holds latest 50 in BusState.

## IPC Bus Operations

| Operation | Result |
|-----------|--------|
| Connect (handshake) | OK — session ID assigned |
| Submit task (AnyIdle) | Pending, routed correctly |
| Subscribe (field.*) | Connected but no events received (broadcast not wired) |
| Cascade (pioneer-1→2) | Dispatched, visible in `/bus/cascades` |

## Ghost Lifecycle

| Ghost | Persona | Steps Lived | Created At |
|-------|---------|-------------|------------|
| deploy-v2-test | deployment-test | 15 | tick ~68400 |
| explore-test | explorer | 15 | tick ~68426 |
| pioneer-3 | pioneer-3 | 23 | tick 69192 |

Ghost reincarnation (`accept-ghost`) returns 404 — V1-only route not in V2.

## SQLite Persistence

| DB | Table | Records |
|----|-------|---------|
| bus_tracking.db (4.5MB) | bus_events | 1,214 |
| field_tracking.db (36KB) | field_snapshots | 3 |
| field_tracking.db | sphere_history | 0 |
| field_tracking.db | coupling_history | 0 |

sphere_history empty because persistence only writes snapshots at SNAPSHOT_INTERVAL, not per-tick sphere data.

## Cross-Service Bridge State

| Service | Port | Temperature/Fitness | Heat Sources |
|---------|------|---------------------|-------------|
| SYNTHEX | 8090 | T=0.572 (target 0.5, running hot) | HS-001:1.0(w0.3), HS-002:0.0(w0.35), HS-003:0.612(w0.2), HS-004:1.0(w0.15) |
| ME | 8080 | fitness=0.609, tick 14580 | — |
| Nexus/K7 | 8100 | synergy check responded | — |

## API Surface Audit

### LIVE (24 routes)
`/health` `/spheres` `/ghosts` `/field` `/field/r` `/field/decision` `/field/decisions` `/field/chimera` `/field/tunnels` `/field/k` `/field/spectrum` `/field/proposals` `/field/propose`(POST) `/sphere/{id}/neighbors` `/sphere/{id}/consent` `/sphere/{id}/data-manifest` `/sphere/{id}/inbox` `/bus/info` `/bus/tasks` `/bus/events` `/bus/suggestions` `/bus/cascades` `/bus/submit`(POST) `/bridges/health`

### 404 in V2 (12 V1-only routes)
`/sphere/{id}/narrative` `/sphere/{id}/associations` `/sphere/{id}/preferences` `/sphere/{id}/recall` `/sphere/{id}/decouple` `/sphere/{id}/recouple` `/sphere/{id}/request-divergence` `/sphere/{id}/accept-ghost` `/nexus/state` `/nexus/metrics` `/synthex/thermal` `/integration/matrix`

## Issues Discovered

| # | Issue | Severity | Detail |
|---|-------|----------|--------|
| 1 | `health.r` = 0.0 | MEDIUM | `/health` shows r=0.0 but `/field` shows r=0.549. Display bug in health handler — likely returns cached outer r or stale value. |
| 2 | Bridge staleness stuck | LOW | All bridges show stale=true despite smoke test passing. `set_last_poll_tick` in spawned task may not propagate correctly to staleness check window. |
| 3 | IPC event broadcast | MEDIUM | `subscribe` receives no events. Tick loop doesn't call `publish_event()` on BusState after tick. |
| 4 | Sphere inbox POST | LOW | Messages don't persist — POST returns empty body, GET returns empty messages. |
| 5 | `accept-ghost` 404 | INFO | V1-only route, not ported to V2 API server. Ghost reincarnation is coded in m12 but not exposed via API. |

## Memory Records

| System | ID | Content |
|--------|-----|---------|
| RM | `r69bdedc0024c` | Exploration findings |
| POVM | `7147442f` | Pioneer exploration complete |
| POVM | `30120621` | Exploration + BUG-032 |

## Cross-References

- `[[Session 046b — Bridge Wiring Complete]]` — 10 modules wired
- `[[Session 046b — Exploration and BUG-032 Fix]]` — BUG-032 governance fix
- `[[Session 046 — V2 Binary Deployed]]` — initial deploy
- `[[ULTRAPLATE — Bugs and Known Issues]]` — BUG-032
- `[[The Habitat — Integrated Master Plan V3]]` — V3.2 Inhabitation (partially validated)
- `[[Vortex Sphere Brain-Body Architecture]]` — Kuramoto dynamics confirmed
