# SYNTHEX Heat Injection Experiment — BETA Momentum

> **Agent:** BETA-TOP-RIGHT | 2026-03-21
> **Target:** localhost:8090/api/ingest
> **Protocol:** 5 injection cycles, 5s intervals, tracking temperature response
> **Payload:** `{"heat_source_id":"HS-001","reading":0.4,"cascade_amplification":8}`

---

## Baseline

| Metric | Value |
|--------|-------|
| Temperature | 0.03 |
| Target | 0.50 |
| PID output | -0.335 |
| Hebbian heat | 0.0 |
| Cascade heat | 0.0 |
| Resonance heat | 0.0 |
| CrossSync heat | 0.2 |

---

## Injection Results

| Cycle | Inject Response | Wait | Temp After | PID After | Delta |
|-------|-----------------|------|------------|-----------|-------|
| 1 | `accepted: true` | 5s | 0.03 | -0.335 | 0.000 |
| 2 | `accepted: true` | 5s | 0.03 | -0.335 | 0.000 |
| 3 | `accepted: true` | 5s | 0.03 | -0.335 | 0.000 |
| 4 | `accepted: true` | 5s | 0.03 | -0.335 | 0.000 |
| 5 | `accepted: true` | 5s | 0.03 | -0.335 | 0.000 |

## Heat Sources After 5 Injections

| Source | Before | After | Delta |
|--------|--------|-------|-------|
| Hebbian | 0.0 | 0.0 | 0.0 |
| Cascade | 0.0 | 0.0 | 0.0 |
| Resonance | 0.0 | 0.0 | 0.0 |
| CrossSync | 0.2 | 0.2 | 0.0 |

---

## Verdict: SYNTHEX IS THERMALLY INERT

**5 injections accepted but produced zero temperature change.**

```
Temp timeline:
  0.50 ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ TARGET
        |     |     |     |     |
  0.03 ─●─────●─────●─────●─────●── FLATLINE
        C1    C2    C3    C4    C5
```

### Root Cause Analysis

1. **Ingest accepted but not applied.** All 5 POSTs returned `accepted: true` with the current temp echoed back. The endpoint acknowledges receipt but the thermal model does not incorporate `HS-001` into its heat calculation.

2. **Heat sources are hardcoded, not dynamic.** The 4 registered heat sources (Hebbian, Cascade, Resonance, CrossSync) are the only sources the thermal model reads. `HS-001` is an unregistered source ID — SYNTHEX accepts the ingest event (possibly for logging/analytics) but does not map it to a thermal input.

3. **PID controller demands heat but has no supply.** PID output is -0.335 (negative = "we need MORE heat"). The controller correctly detects the 94% deficit from target. But with 3/4 registered sources at 0.0 and no way to inject via API into those sources, the system cannot self-correct.

4. **The 4 registered sources require V2 bridge events:**
   - **Hebbian** — needs PV2 Hebbian STDP weight updates (V1 inactive)
   - **Cascade** — needs PV2 cascade events (V1 bus: 0 cascades)
   - **Resonance** — needs PV2 field resonance events (V1 not emitting)
   - **CrossSync** — reads from Nexus (only alive source, 0.2)

### Implications

- **External heat injection via `/api/ingest` cannot warm SYNTHEX.** The endpoint is an analytics sink, not a thermal input.
- **Only V2 binary deployment can restore thermal activity** by activating the 3 dead heat source bridges.
- **No workaround exists on V1** — the thermal model's heat sources are structurally coupled to PV2's internal event emission, not to the ingest API.

---

BETA-INJECT-COMPLETE
