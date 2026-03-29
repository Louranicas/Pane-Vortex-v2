# SYNERGY BETA: SYNTHEX Heat Injection Experiment

**Agent:** BETA | **Timestamp:** 2026-03-21 02:28–02:29 UTC

---

## Experiment: Can We Inject Heat via `/api/ingest`?

### Setup

5 injection cycles at 10s intervals, POSTing `{"heat_source_id":"HS-001","reading":0.4}` then reading `/v3/thermal`.

### Results

| Cycle | Time | Inject HTTP | Temperature | Hebbian HS | PID |
|-------|------|-------------|-------------|------------|-----|
| 1 | 02:28:37 | **200** | 0.030 | 0.0 | -0.335 |
| 2 | 02:28:47 | **200** | 0.030 | 0.0 | -0.335 |
| 3 | 02:28:57 | **200** | 0.030 | 0.0 | -0.335 |
| 4 | 02:29:07 | **200** | 0.030 | 0.0 | -0.335 |
| 5 | 02:29:17 | **200** | 0.030 | 0.0 | -0.335 |

### Response Body

```json
{"accepted": true, "temperature": 0.03}
```

**The endpoint accepts the POST (HTTP 200, `accepted: true`) but the injection has ZERO effect on thermal state.** Temperature, heat sources, and PID output are completely unchanged across 5 cycles.

---

## Additional Endpoint Scan

| Endpoint | Method | HTTP | Effect |
|----------|--------|------|--------|
| `/api/ingest` | POST | 200 | Accepts but no thermal effect |
| `/v3/thermal/heat` | POST | 404 | Does not exist |
| `/v3/thermal/inject` | POST | 404 | Does not exist |
| `/v3/heat` | POST | 404 | Does not exist |
| `/v3/ingest` | POST | 404 | Does not exist |
| `/api/events` | POST | 404 | Does not exist |
| `/api/thermal` | POST | 404 | Does not exist |

Also tested alternative payload formats — all return 200 but none affect thermal:
- `{"type":"heat","source":"Hebbian","value":0.5}` → 200, no effect
- `[{"heat_source_id":"HS-001","reading":0.4}]` → 200, no effect
- `{"event_type":"thermal_update","data":{"Hebbian":0.5}}` → 200, no effect

---

## Diagnosis

`/api/ingest` is a **general event ingestion endpoint**, not a thermal heat source override. It likely feeds SYNTHEX's internal event correlation pipeline, not the thermal PID controller directly. The heat source readings (Hebbian, Cascade, Resonance, CrossSync) are computed internally by SYNTHEX from correlated event patterns, not directly settable via API.

**SYNTHEX's thermal system has no external write path for heat sources.** The only way to raise temperature is to generate real cross-service activity that SYNTHEX's internal correlators detect as heat.

---

## Conclusion

`/api/ingest` is a red herring — it accepts data silently but doesn't affect the thermal model. SYNTHEX temperature cannot be externally manipulated. The thermal loop can only be warmed by:

1. **V2 binary deploy** — closes the PV→SYNTHEX feedback loop, enabling k_mod propagation
2. **Real fleet activity** — spheres doing actual work generates Hebbian, Cascade, Resonance heat
3. **There is no shortcut** — SYNTHEX correctly refuses external thermal manipulation

---

BETA-INJECTION-COMPLETE
