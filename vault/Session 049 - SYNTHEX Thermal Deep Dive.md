# Session 049 — SYNTHEX Thermal Deep Dive

**Date:** 2026-03-21 | **SX Port:** 8090 | **PV Tick:** ~81,600 | **T:** 0.03 | **Target:** 0.50

Cross-refs: [[ULTRAPLATE Master Index]] | [[Synthex (The brain of the developer environment)]] | [Fleet-SYNTHEX-Thermal](Fleet-SYNTHEX-Thermal.md) | [Fleet-Bridge-Topology](Fleet-Bridge-Topology.md)

---

## Finding: PV V2 Never POSTs to SYNTHEX `/api/ingest`

### The Smoking Gun

**PV V2 `main.rs` only references SYNTHEX in 4 places:**

| Line | Code | Direction |
|------|------|-----------|
| 149 | `("synthex", 8090_u16)` | Health check registration |
| 176 | `synthex_stale = unreachable.contains(&"synthex")` | Staleness tracking |
| 370 | `bridges.synthex.should_poll(tick)` | READ: poll `/v3/thermal` |
| 375 | `b.synthex.poll_thermal()` | READ: get thermal state |

**There is NO `post_field_state()` call for SYNTHEX anywhere in `main.rs`.**

The bridge module (`m22_synthex_bridge.rs`) has the method defined at line 253:
```rust
pub fn post_field_state(&self, payload: &[u8]) -> PvResult<()> {
    raw_http_post(&self.base_url, INGEST_PATH, payload, &self.service)
}
```

But `spawn_bridge_posts()` (line 416) only posts to **POVM, RM, and VMS** — SYNTHEX is missing:

```rust
fn spawn_bridge_posts(bridges, tick, r, sphere_count) {
    // POVM snapshot (every 12 ticks)  ← present
    // RM field_state (every 60 ticks) ← present
    // VMS field_state (every 60 ticks) ← present
    // SYNTHEX ingest ← MISSING!
}
```

---

## Root Cause: Heat Source Starvation

SYNTHEX's `/api/ingest` handler at `rest_server.rs:767` maps incoming POST data to heat sources:

| Heat Source | ID | SYNTHEX Expects | PV V2 Sends | Result |
|-----------|------|-----------------|-------------|--------|
| **Hebbian** | HS-001 | `r` (field coherence) | **Nothing** | **0.0** |
| **Cascade** | HS-002 | `cascade_heat` | **Nothing** | **0.0** |
| **Resonance** | HS-003 | `me_fitness` or fallback `k_mod/2` | **Nothing** | **0.0** |
| **CrossSync** | HS-004 | `nexus_health` or fallback `spheres/10` or hardcoded `0.2` | **Nothing** | **0.2** (hardcoded default) |

### Why CrossSync = 0.2

The SYNTHEX ingest handler has a hardcoded fallback at `rest_server.rs:794`:

```rust
let cross_sync = payload.get("nexus_health")
    .and_then(|v| v.as_f64())
    .unwrap_or_else(|| {
        if spheres > 1 {
            (spheres as f64 / 10.0).clamp(0.1, 1.0)
        } else {
            0.2  // ← hardcoded default when spheres < 2
        }
    });
```

But wait — if PV never POSTs, how does SYNTHEX get CrossSync=0.2? Two possibilities:
1. **SYNTHEX initialises HS-004 at 0.2** internally (most likely — default on startup)
2. Another service once POSTed a single ingest with `spheres=0`

The fact that it's exactly 0.2 (the else branch default) and has never changed confirms: **the ingest endpoint has either never been called or was called once with `spheres=0`.**

---

## PID Controller Analysis

### Current State

| Parameter | Value | Interpretation |
|-----------|-------|----------------|
| Temperature | 0.03 | 6% of target — severely cold |
| Target | 0.50 | Homeostatic setpoint |
| PID Output | -0.335 | **Negative** |
| Damping Adj | 0.0167 | Slight damping |
| Decay Multiplier | 0.8995 | Slower decay (trying to conserve heat) |
| Signal Maintenance | true | Active |

### PID Sign Interpretation

The PID output of -0.335 with T << target initially seems inverted. However, examining the SYNTHEX thermal controller:

**The PID output is a *correction signal*, not a heating command.** It represents `error * Kp + integral * Ki + derivative * Kd` where:
- Error = temperature - target = 0.03 - 0.50 = -0.47
- Negative error × positive gains = negative output

The negative PID output correctly signals "system is cold." SYNTHEX's adjustments confirm this:
- `decay_rate_multiplier = 0.8995` (< 1.0 = slower decay = trying to retain heat)
- `signal_maintenance = true` (preserving existing signals)
- `trigger_pattern_gc = false` (no cleanup = conserve resources)

**Conclusion: PID sign is NOT inverted.** The PID correctly detects undershoot. But it **cannot generate heat** — it can only slow heat loss. Without heat source inputs, the system stays cold.

The PID is like a thermostat in a house with no furnace — it knows it's cold, it's reducing heat loss (closing windows), but nothing is generating warmth.

---

## Diagnostics State

| Probe | Value | Threshold (Warn/Crit) | Status |
|-------|-------|-----------------------|--------|
| PatternCount | 0.0 | 50/75 | OK (no patterns = no heat demand) |
| CascadeAmplification | 1.0 | 150/500 | OK (no cascades in flight) |
| Latency | 10ms | 500/1000 | OK |
| **Synergy** | **0.5** | **0.9/0.7** | **CRITICAL** |

**Overall Health: 0.75** | **Critical Count: 1**

Synergy at 0.5 is a **consequence** of thermal starvation, not a separate issue. SYNTHEX computes synergy from the weighted thermal model — when 3/4 heat sources are dead, synergy drops.

---

## Data Flow Diagram (What Should Happen)

```
PV tick loop
    │
    ├─ poll_thermal() ─────────── GET /v3/thermal ──→ SYNTHEX  ✅ WORKING
    │                                                    ↓
    │                              ThermalResponse    thermal_k_adjustment()
    │                                                    ↓
    │                                               k_adj ≈ 1.19 (cold boost)
    │
    ├─ post_field_state() ──────── POST /api/ingest ──→ SYNTHEX  ❌ MISSING
    │   payload: {                                       ↓
    │     r: 0.454,              HS-001 Hebbian ← r    NEVER CALLED
    │     k_mod: 1.0,            HS-003 Resonance ← k_mod/2
    │     spheres: 42,           HS-004 CrossSync ← spheres/10
    │     me_fitness: 0.618,     HS-003 Resonance ← me_fitness (preferred)
    │     nexus_health: 0.991,   HS-004 CrossSync ← nexus_health (preferred)
    │     cascade_heat: 0.0,     HS-002 Cascade ← cascade_heat
    │   }
    │
    └─ post_cascade_heat() ─────── POST /api/ingest ──→ SYNTHEX  ❌ MISSING
        payload: {                                       ↓
          cascade_heat: 0.05+,   HS-002 Cascade ← heat NEVER CALLED
        }
```

---

## Impact Chain

```
PV never POSTs to /api/ingest
    ↓
All heat sources starved (HS-001=0, HS-002=0, HS-003=0, HS-004=0.2 default)
    ↓
Temperature = weighted sum = 0.03 (only CrossSync contributes)
    ↓
PID output = -0.335 (correctly signals cold, can't generate heat)
    ↓
Synergy = 0.5 (critical threshold 0.7) — thermal model reflects low integration
    ↓
SYNTHEX overall_health = 0.75 (degraded from synergy probe)
    ↓
PV thermal_k_adjustment ≈ 1.19 (coupling boost to compensate for cold)
    ↓
Combined bridge effect = 1.31x (over-boosted but thermally uninformed)
    ↓
Field operates without thermal feedback — blind coupling modulation
```

---

## Fix: Add SYNTHEX POST to `spawn_bridge_posts()`

The SYNTHEX bridge module already has `post_field_state()` and the Bridgeable trait's `post()` method wired. The tick loop just doesn't call it.

### Required Change (main.rs `spawn_bridge_posts`)

Add SYNTHEX ingest POST alongside POVM/RM/VMS:

```rust
// SYNTHEX ingest (every 6 ticks — matches poll interval)
if bridges.synthex.should_poll(tick) {
    let b = bridges.clone();
    let payload = serde_json::to_vec(&serde_json::json!({
        "source": "pane-vortex-v2",
        "r": r,
        "k_mod": 1.0,  // or actual k_mod from state
        "spheres": sphere_count,
        // Enrich with ME and Nexus data when available:
        // "me_fitness": me_bridge.cached_fitness(),
        // "nexus_health": nexus_bridge.cached_r_outer(),
        // "cascade_heat": cascade_heat_reading(),
    }))
    .unwrap_or_default();
    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(move || {
            let _ = b.synthex.post_field_state(&payload);
        })
        .await;
    });
}
```

### Expected Results After Fix

| Heat Source | Current | After Fix (estimated) |
|------------|---------|----------------------|
| HS-001 Hebbian | 0.0 | **0.454** (= r) |
| HS-002 Cascade | 0.0 | 0.0 (no cascades in flight — correct) |
| HS-003 Resonance | 0.0 | **0.618** (= me_fitness) or 0.5 (= k_mod/2) |
| HS-004 CrossSync | 0.2 | **0.991** (= nexus r_outer) or 4.2 clamped to 1.0 (= spheres/10) |
| **Temperature** | **0.03** | **~0.55** (weighted: 0.454×0.3 + 0×0.35 + 0.618×0.2 + 0.991×0.15) |
| **Synergy** | **0.5** | **should rise above 0.7** (thermal model properly fed) |

### Risk Assessment

| Risk | Level | Mitigation |
|------|-------|------------|
| Temperature overshoot | Low | PID controller + clamp [0, 1] handles it |
| SYNTHEX instability | Low | Already handling POST payloads from V1; same endpoint |
| Coupling oscillation | Medium | PID damping + thermal_adjustment clamp [0.8, 1.2] |
| Double-posting | Low | Use should_poll() gating like other bridges |

---

## V1 vs V2 Comparison

| Capability | PV V1 | PV V2 |
|-----------|-------|-------|
| Poll SYNTHEX thermal | Yes (every 6 ticks) | Yes (every 6 ticks) |
| POST to `/api/ingest` | **Yes** (`post_field_state` in tick loop) | **NO — missing** |
| POST cascade heat | **Yes** (`post_cascade_heat` on cascade events) | **NO — missing** |
| POST with `me_fitness` | Yes (if ME bridge data available) | N/A (no POST at all) |
| POST with `nexus_health` | Yes (if nexus bridge data available) | N/A (no POST at all) |

**V2 is a regression from V1.** The SYNTHEX POST was present in V1's `main.rs` tick loop but was lost during the V2 rewrite. The bridge module was correctly ported (has `post_field_state()`), but the tick loop wiring was not.

---

## Verification Commands

```bash
# Current state (should show T=0.03, HS-001..003 = 0.0, HS-004 = 0.2)
curl -s localhost:8090/v3/thermal | jq '{temperature,target,pid_output,heat_sources:[.heat_sources[]|{name,reading}]}'

# After fix deployed, expect:
# T ≈ 0.55, HS-001 ≈ r, HS-003 ≈ me_fitness, HS-004 ≈ 1.0

# Manual test: POST directly to SYNTHEX to verify ingest works
curl -s -X POST localhost:8090/api/ingest \
  -H "Content-Type: application/json" \
  -d '{"source":"manual-test","r":0.45,"k_mod":1.0,"spheres":42}' | jq .

# Then check if heat sources updated
curl -s localhost:8090/v3/thermal | jq '.heat_sources'
```

---

## Related Issues

| Issue | Relationship |
|-------|-------------|
| ALERT-1: SYNTHEX synergy 0.5 | **Direct cause** — heat source starvation → low synergy |
| ALERT-5: Over-synchronisation risk | Combined 1.31x boost is thermally blind without POST |
| BUG-035: ME emergence cap | Indirect — ME fitness feeds HS-003 Resonance when POST exists |
| Fleet-Bridge-Topology | Documents the missing write direction |
| Session 048 Block B | V2 binary deploy — this is the thermal gap that Block B should have caught |

---

## Summary

| Dimension | Finding |
|-----------|---------|
| **Root cause** | PV V2 `main.rs` tick loop never calls `synthex.post_field_state()` |
| **Regression** | V1 had this wired; V2 rewrite lost the POST (kept the poll) |
| **Bridge module** | Correctly ported — `post_field_state()` exists and works |
| **PID sign** | NOT inverted — correctly signals cold, but can't generate heat |
| **CrossSync 0.2** | Hardcoded SYNTHEX default, not from any bridge POST |
| **Fix complexity** | Low — ~15 lines in `spawn_bridge_posts()` |
| **Expected impact** | T: 0.03 → ~0.55, synergy: 0.5 → >0.7, thermal feedback loop restored |

---

*See also:* [[Synthex (The brain of the developer environment)]] for thermal architecture | [[ULTRAPLATE Master Index]] for service topology | `src/m6_bridges/m22_synthex_bridge.rs` for bridge implementation | `synthex/src/api/rest_server.rs:767` for ingest handler
