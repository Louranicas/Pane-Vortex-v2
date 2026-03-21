# Session 046b — Ralph Loop Fixes (7 Generations)

> **Date:** 2026-03-21
> **Scope:** Fix 5 exploration issues + BUG-033 discovery via Ralph Loop
> **Tests:** 1,527 (was 1,524, +3 thermal adjustment tests)
> **PID:** 3828125
> **Quality Gate:** 4/4 CLEAN

## Issues Resolved

### Issue 1: `health.r` = 0.0 Display Bug — FIXED

**Root cause:** `/health` handler read `r_history.back()` which was empty after snapshot restore. `/field` computed r live from `FieldState::compute()`.

**Fix:** Changed `health_handler` in `m10_api_server.rs` to read from `cached_field` (set every tick by the orchestrator), falling back to `r_history` during warmup.

**Also fixed:** `/field/r` handler was reading from `CouplingNetwork::order_parameter()` which had empty `phases` HashMap (network created fresh, not synced from restored state). Changed to use `cached_field` too.

### Issue 2: Bridge Staleness Stuck True — FIXED (via BUG-033)

**Root cause (shallow):** `set_last_poll_tick` was called AFTER the async poll, so the next tick's `is_stale()` ran before the poll task completed. Fixed by moving `set_last_poll_tick` BEFORE spawning the poll.

**Root cause (deep — BUG-033):** All polls were silently failing because:
1. `BridgeSet::from_config()` passed `"http://127.0.0.1:8090"` but `raw_http_get()` parses as `SocketAddr` which rejects the `http://` scheme prefix
2. `ThermalResponse` struct had wrong fields (`thermal_adjustment`, `state`, `confidence`) — actual SYNTHEX V3 API returns (`temperature`, `target`, `pid_output`, `heat_sources`)

**Fix:** Stripped `http://` prefix from all 6 bridge addresses. Rewrote `ThermalResponse` to match actual SYNTHEX API. Added `thermal_adjustment()` method computing `deviation.mul_add(-0.2, 1.0).clamp(budget)`.

**Result:** SYNTHEX/Nexus/ME all `stale=false`. `k_modulation=0.85` (bridges actively influencing field).

### Issue 3: IPC Event Broadcast — FIXED

**Root cause:** Tick loop released AppState lock but never published events to BusState. IPC subscribers received handshake but no `field.tick` events.

**Fix:** Added `field.tick` event publishing in `main.rs` tick loop after bridge polls, before suggestions. Creates `BusEvent` with tick, r, spheres, action.

**Result:** 17+ `field.tick` events visible in `/bus/events`. IPC subscribers can now receive live field state.

### Issue 4: `accept-ghost` Route 404 — FIXED

**Root cause:** `AppState::accept_ghost()` existed in `m15_app_state.rs` (with 3 tests) but no API route exposed it.

**Fix:** Added `POST /sphere/{pane_id}/accept-ghost` route with `AcceptGhostRequest` body (`ghost_id` field). Handler verifies accepting sphere exists, consumes ghost, returns ghost data (persona, steps, tools).

### Issue 5: Inbox POST Not Persisting — NOT A BUG

**Root cause:** User error during exploration. The correct POST URL is `/sphere/{id}/inbox/send`, not `/sphere/{id}/inbox`. The GET handler is at `/inbox`, the POST is at `/inbox/send`. Verified working correctly with the right URL.

## BUG-033: Bridge URLs + ThermalResponse Mismatch (CRITICAL — FIXED)

Discovered during Ralph Loop iteration 1 while investigating Issue 2. This was the **single most impactful bug** in the entire wiring effort — every bridge poll was a silent no-op.

**Two-part root cause:**
1. `BridgeSet::from_config()` used URL format (`http://host:port`) but `raw_http_get()` expected socket address format (`host:port`)
2. `ThermalResponse` struct didn't match SYNTHEX V3 API response format

**Impact before fix:** All 6 bridges appeared wired but never actually polled. `cached_adjustment` stuck at 1.0, `stale` stuck at true, `k_modulation` never influenced by external services. The entire bridge subsystem was a no-op.

**Impact after fix:** SYNTHEX thermal deviation actively modulates k. `k_modulation` dropped from 1.0 to 0.85 (budget floor) — the field is breathing with external service influence for the first time in V2.

## Files Modified

| File | Changes |
|------|---------|
| `src/m2_services/m10_api_server.rs` | health_handler uses cached_field, field_r_handler uses cached_field, accept-ghost route + handler + request struct |
| `src/m6_bridges/mod.rs` | Stripped `http://` prefix from all 6 bridge addresses |
| `src/m6_bridges/m22_synthex_bridge.rs` | Rewrote ThermalResponse (temperature/target/pid_output/heat_sources), added thermal_adjustment() method, 3 new tests |
| `src/bin/main.rs` | set_last_poll_tick before spawn, field.tick event publishing to BusState |

## Verification

| Check | Before | After |
|-------|--------|-------|
| `/health` r | 0.0 | 0.685 |
| Bridge SYNTHEX stale | true | **false** |
| Bridge Nexus stale | true | **false** |
| Bridge ME stale | true | **false** |
| `k_modulation` | 1.0 (neutral) | **0.85** (active) |
| `/bus/events` count | 0 | **17+** (`field.tick`) |
| `/sphere/{id}/accept-ghost` | 404 | **200** (working) |
| Tests | 1,524 | **1,527** |

## Memory Records

| System | ID |
|--------|----|
| RM (BUG-033 fix) | `r69bdf270024f` |
| POVM (BUG-033) | `77da23b9` |
| Obsidian bugs | BUG-033 in `[[ULTRAPLATE — Bugs and Known Issues]]` |

## Cross-References

- `[[Session 046b — Pioneer Exploration Report]]` — issues discovered
- `[[Session 046b — Bridge Wiring Complete]]` — wiring that exposed BUG-033
- `[[Session 046b — Exploration and BUG-032 Fix]]` — BUG-032 governance fix
- `[[ULTRAPLATE — Bugs and Known Issues]]` — BUG-032, BUG-033
- `[[Session 034f — SYNTHEX Schematics and Wiring]]` — SYNTHEX thermal API reference
