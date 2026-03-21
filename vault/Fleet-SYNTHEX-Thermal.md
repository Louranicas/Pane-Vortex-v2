# Fleet-SYNTHEX Thermal Exploration Report

**Date:** 2026-03-21T04:27:13Z
**Source:** `localhost:8090/v3/thermal` + `localhost:8090/v3/diagnostics`
**Probed by:** Command tab (Session 049+)

---

## Thermal State

| Parameter | Value | Target/Threshold | Status |
|-----------|-------|------------------|--------|
| **Temperature** | 0.03 | 0.50 | COLD (6% of target) |
| **PID Output** | -0.335 | — | Negative output despite undershoot |
| **Damping Adjustment** | 0.0167 | — | Active |
| **Decay Rate Multiplier** | 0.8995 | — | Slightly below 1.0 (slow decay) |
| **Signal Maintenance** | true | — | OK |
| **Pattern GC Triggered** | false | — | OK |

### Heat Sources

| ID | Name | Reading | Weight | Contribution |
|----|------|---------|--------|-------------|
| HS-001 | **Hebbian** | 0.0 | 0.30 | DEAD — no Hebbian learning activity |
| HS-002 | **Cascade** | 0.0 | 0.35 | DEAD — no cascade propagation |
| HS-003 | **Resonance** | 0.0 | 0.20 | DEAD — no field resonance |
| HS-004 | **CrossSync** | 0.2 | 0.15 | WEAK — only active heat source |

**Weighted thermal input:** (0.0 x 0.30) + (0.0 x 0.35) + (0.0 x 0.20) + (0.2 x 0.15) = **0.03**

This confirms temperature = weighted sum of heat source readings.

---

## Diagnostics

| Probe | Value | Warning | Critical | Severity |
|-------|-------|---------|----------|----------|
| PatternCount | 0.0 | 50.0 | 75.0 | OK |
| CascadeAmplification | 1.0 | 150.0 | 500.0 | OK |
| Latency | 10ms | 500ms | 1000ms | OK |
| **Synergy** | **0.5** | 0.9 | **0.7** | **CRITICAL** |

- **Overall Health:** 0.75
- **Critical Count:** 1 (Synergy)
- **Warning Count:** 0

---

## Analysis

### Why SYNTHEX Is Cold

1. **Hebbian heat source = 0.0**: PV Hebbian STDP is active (LTP/LTD running in tick loop), but the Hebbian→SYNTHEX bridge is not generating thermal readings. Either the bridge isn't firing thermal updates, or SYNTHEX isn't receiving them.

2. **Cascade heat source = 0.0**: No cascade events propagating. The bus has cascade support but it's not thermally active.

3. **Resonance heat source = 0.0**: Despite r=0.454 in the PV field (moderate coherence), the resonance heat source reads zero. The SYNTHEX thermal bridge (`synthex_bridge.rs`) may not be reporting field resonance as thermal input.

4. **CrossSync = 0.2**: The only activity comes from cross-synchronization (likely the PV↔SYNTHEX bridge health check pings).

### PID Controller Anomaly

- Temperature (0.03) is far below target (0.50)
- PID output is **-0.335** (negative = cooling)
- This suggests the PID controller may have inverted gain, or the PID is measuring error as `target - temperature` but applying it inverted
- Expected behavior: T < target → positive PID output → increase heat
- Actual behavior: T < target → negative PID output → decrease heat further

This could be a **PID sign error** or the PID output represents something other than a heating/cooling command.

### Synergy Crisis

Synergy at 0.5 is below the critical threshold of 0.7. This means SYNTHEX's cross-service integration score is degraded. Possible causes:
- Services not reporting state changes to SYNTHEX
- SYNTHEX thermal model starved of input (confirmed by heat sources)
- Circular dependency: low heat → low synergy → less integration → less heat

---

## Recommendations

### Immediate (V3.1 Diagnostics)
1. **Investigate SYNTHEX thermal bridge** (`src/synthex_bridge.rs`): verify it POSTs heat source updates
2. **Check PID controller sign**: the negative output with T << target is suspicious
3. **Verify SYNTHEX `/v3/thermal` accepts external heat updates**: may need POST endpoint

### Short-term (V3.2 Inhabitation)
4. **Wire Hebbian heat**: PV tick loop should POST Hebbian activity level to SYNTHEX
5. **Wire Resonance heat**: PV field `r` value should map to resonance heat reading
6. **Wire Cascade heat**: Bus cascade events should generate thermal input

### Medium-term (V3.5 Consolidation)
7. **Create SYNTHEX MCP adapter**: expose thermal state as Claude Code tool
8. **Add bacon.toml thermal job**: auto-check SYNTHEX health on PV recompile
9. **Dashboard integration**: add SYNTHEX thermal to `dashboard.sh`

---

## Raw Data

### /v3/thermal
```json
{
  "adjustments": {
    "damping_adjustment": 0.016749999999999998,
    "decay_rate_multiplier": 0.8995,
    "signal_maintenance": true,
    "trigger_pattern_gc": false
  },
  "heat_sources": [
    {"id": "HS-001", "name": "Hebbian", "reading": 0.0, "weight": 0.3},
    {"id": "HS-002", "name": "Cascade", "reading": 0.0, "weight": 0.35},
    {"id": "HS-003", "name": "Resonance", "reading": 0.0, "weight": 0.2},
    {"id": "HS-004", "name": "CrossSync", "reading": 0.2, "weight": 0.15}
  ],
  "pid_output": -0.33499999999999996,
  "target": 0.5,
  "temperature": 0.03
}
```

### /v3/diagnostics
```json
{
  "critical_count": 1,
  "overall_health": 0.75,
  "probes": [
    {"name": "PatternCount", "severity": "Ok", "value": 0.0, "warning_threshold": 50.0, "critical_threshold": 75.0},
    {"name": "CascadeAmplification", "severity": "Ok", "value": 1.0, "warning_threshold": 150.0, "critical_threshold": 500.0},
    {"name": "Latency", "severity": "Ok", "value": 10.0, "warning_threshold": 500.0, "critical_threshold": 1000.0},
    {"name": "Synergy", "severity": "Critical", "value": 0.5, "warning_threshold": 0.9, "critical_threshold": 0.7}
  ],
  "timestamp": "2026-03-21T04:27:13.142726878+00:00",
  "warning_count": 0
}
```
