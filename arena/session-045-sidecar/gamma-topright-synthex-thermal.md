# SYNTHEX Thermal Dynamics Probe — Session 045

**Probe:** `localhost:8090/v3/thermal` x12 @ 5s intervals (60s window)
**Cross-ref:** `localhost:8080/api/fitness` (Maintenance Engine)
**Timestamp:** 2026-03-21 23:07:20–23:08:15 UTC
**Agent:** gamma-topright sidecar

---

## Executive Finding

**The thermal field is FROZEN.** Zero oscillation across 12 samples over 60 seconds. Temperature locked at 0.5724 (target 0.5). The PID controller outputs a constant +0.1362 correction that produces no visible effect. This is not thermal dynamics — it's thermal rigor mortis.

---

## Raw Timeseries

| # | Time (UTC) | Temp | PID Out | Hebbian | Cascade | Resonance | CrossSync | ME Fitness | ME Trend |
|---|-----------|------|---------|---------|---------|-----------|-----------|------------|----------|
| 1 | 23:07:20 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 2 | 23:07:25 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 3 | 23:07:30 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 4 | 23:07:35 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 5 | 23:07:40 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 6 | 23:07:45 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Stable |
| 7 | 23:07:50 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | **Declining** |
| 8 | 23:07:55 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Declining |
| 9 | 23:08:00 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Declining |
| 10 | 23:08:05 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Declining |
| 11 | 23:08:10 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Declining |
| 12 | 23:08:15 | 0.5724 | 0.1362 | 1.000 | 0.000 | 0.612 | 1.000 | 0.6089 | Declining |

**Variance across all 12 samples: 0.0 on every metric.** The only change in the entire window is ME trend flipping from Stable to Declining at sample 7.

---

## Heat Source Analysis

### HS-001: Hebbian (weight 0.30)
- **Reading:** 1.000 (SATURATED)
- **Contribution:** 0.300
- **Status:** Pegged at ceiling. Zero dynamic range. Hebbian learning has driven this source to maximum and it cannot respond to PID corrections. This is consistent with LTP dominance when LTD rate (0.002) is 5x weaker than LTP (0.01).

### HS-002: Cascade (weight 0.35)
- **Reading:** 0.000 (displayed), actual ~1e-132 (DEAD)
- **Contribution:** 0.000
- **Status:** Effectively extinct. The highest-weighted heat source (35% of thermal budget) contributes nothing. This is the single largest contributor to thermal dysfunction. The Cascade pathway has decayed to heat death — no cascade events are firing or their thermal contribution has underflowed to zero.

### HS-003: Resonance (weight 0.20)
- **Reading:** 0.612 (ONLY source with headroom)
- **Contribution:** 0.1224
- **Status:** The sole source operating in its dynamic range. Locked at 0.612 over the full window — no oscillation, but at least not saturated. If any recovery is possible, it routes through this source.

### HS-004: CrossSync (weight 0.15)
- **Reading:** 1.000 (SATURATED)
- **Contribution:** 0.150
- **Status:** Pegged at ceiling like Hebbian. Cross-synchronization coupling is at maximum. Combined with Hebbian saturation, 45% of thermal weight is locked at max with zero regulatory capacity.

### Thermal Budget Decomposition

```
Temperature = sum(reading * weight)
            = (1.000 * 0.30) + (0.000 * 0.35) + (0.612 * 0.20) + (1.000 * 0.15)
            = 0.300 + 0.000 + 0.1224 + 0.150
            = 0.5724  <-- exact match, confirming deterministic model
```

The temperature IS the weighted sum — no stochastic component, no noise floor, no thermal diffusion. Pure arithmetic.

---

## PID Controller State

| Parameter | Value | Interpretation |
|-----------|-------|----------------|
| PID output | +0.1362 | Trying to cool (temp > target) |
| Damping adjustment | -0.0136 | Weak negative damping |
| Decay rate multiplier | 1.1362 | Accelerating decay (trying to pull temp down) |
| Signal maintenance | false | No signal boosting active |
| Pattern GC trigger | false | No garbage collection needed |

The PID sees temp=0.5724 vs target=0.5, produces a cooling correction of 0.1362, but **nothing changes**. The PID output has no actuator path to modify the heat source readings. The controller is spinning its wheels — output is non-zero but effect is zero.

---

## ME Fitness Cross-Reference

| Metric | Value |
|--------|-------|
| Current fitness | 0.6089 |
| System state | **Degraded** |
| Trend | Stable (samples 1-6) → **Declining** (samples 7-12) |
| History length | 20 snapshots |

### Dimension Scores (from ME /api/fitness)

| Dimension | Score | Assessment |
|-----------|-------|------------|
| service_id | 1.000 | Healthy |
| uptime | 1.000 | Healthy |
| latency | 1.000 | Healthy |
| agents | 0.917 | Good |
| synergy | 0.833 | Moderate |
| protocol | 0.750 | Moderate |
| temporal | 0.587 | Weak — drifting |
| health | 0.583 | Weak |
| error_rate | 0.583 | Weak |
| tier | 0.486 | Poor |
| port | 0.123 | Critical |
| deps | 0.083 | **Critical** — dependency health near zero |

**Key correlation:** ME trend shifted from Stable to Declining at the same time thermal was frozen. The system is degrading while the thermal controller is inert. deps=0.083 confirms dependency chain health is nearly collapsed — consistent with Cascade heat death (Cascade tracks inter-service dependency propagation).

---

## Diagnosis

### Root Cause: Cascade Heat Death Collapses Thermal Regulation

1. **Cascade (HS-002) is dead at 1e-132** — 35% of thermal weight contributes zero heat
2. **Hebbian + CrossSync are saturated at 1.0** — 45% of weight has no dynamic range
3. **Only Resonance (20% weight) can move** — but it's also frozen at 0.612
4. **PID controller has no actuator path** — its output doesn't feed back into heat source readings
5. **Result:** Temperature locked at 0.5724, +14.5% above target, with zero oscillation

### Why It Matters

- A healthy thermal field should oscillate — the PID target of 0.5 implies a homeostatic set point that the system should orbit
- Frozen thermal = frozen adaptation. SYNTHEX V3 homeostasis requires thermal dynamics to drive signal maintenance and pattern GC
- The ME sees this as degradation (trend → Declining) but has no lever to fix it because the thermal subsystem is self-referential
- Cascade death at 1e-132 is not a slow decay — it's an underflow extinction event that happened some time ago and hasn't recovered

### Implications for PV2

- **Consent-gated K adjustment** routes through SYNTHEX thermal for modulation signals — if thermal is frozen, K modulation from SYNTHEX is constant, reducing PV2's adaptive capacity
- **Bridge m22 (SYNTHEX bridge)** will read the same thermal state every tick — it should detect thermal stasis and flag it
- **GAP-1 governance actuator** needs thermal feedback for voting weight normalization — frozen thermal means static voting weights

---

## Recommendations

1. **Revive Cascade (HS-002):** Investigate why reading decayed to 1e-132. This is likely a multiplicative decay without floor — add `max(reading, 1e-6)` floor to prevent underflow extinction
2. **Add thermal noise:** Inject small stochastic perturbation (~0.01) to break static equilibrium
3. **PID actuator wiring:** The PID output must feed back into heat source readings (currently disconnected). Without this, the PID is advisory-only
4. **Saturation caps:** Hebbian and CrossSync at 1.0 need soft ceiling (e.g., tanh clamping at 0.95) to preserve dynamic range
5. **Stasis detection:** SYNTHEX should self-detect when variance(temperature, window=12) < epsilon and trigger signal_maintenance=true

---

*Probe complete. 12/12 samples. Zero data loss. File: `arena/session-045-sidecar/gamma-topright-synthex-thermal.md`*
