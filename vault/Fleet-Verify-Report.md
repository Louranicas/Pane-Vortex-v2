# Fleet Verify Report — Live Diagnostics

**Generated:** 2026-03-21T04:35Z | **Session:** 050 | **Tick:** 83,441

Cross-refs: [[Session 049 — Full Remediation Deployed]] | [[ULTRAPLATE Master Index]]

---

## Raw JSON Output

```json
{
  "spheres": 50,
  "working": 6,
  "idle": 44,
  "blocked": 0,
  "fleet_workers": 14,
  "subscribers": 1,
  "pending_tasks": 5,
  "bus_events": 1000,
  "cascades": 0,
  "sidecar": "UP",
  "sidecar_events": 405,
  "stale_bridges": 0,
  "stale_names": [],
  "confidence": 100,
  "pv_r": 0.454,
  "pv_tick": 83441,
  "pv_k_mod": 0.85,
  "pv_status": "healthy"
}
```

---

## Confidence Score: 100/100

Fleet-verify reports **maximum confidence**. This score aggregates:

| Factor | Status | Contribution |
|--------|--------|-------------|
| PV health | healthy | Pass |
| Stale bridges | 0 | Pass |
| Blocked spheres | 0 | Pass |
| Sidecar | UP | Pass |
| Bus connectivity | 1 subscriber | Pass |
| Pending tasks | 5 (low, below cap) | Pass |

**Interpretation:** All subsystems that fleet-verify checks are operational. No degraded dimensions. This is the first 100/100 score observed in Session 050.

---

## Field State

| Metric | Value | Context |
|--------|-------|---------|
| r (order parameter) | **0.454** | Mid-range coherence, field is breathing |
| Tick | 83,441 | +3,738 since first Session 050 probe (79,703) |
| k_mod | 0.85 | Near floor of K_MOD_BUDGET [0.85, 1.15] |
| Status | healthy | — |

**r at 0.454** is healthier than the 0.285 dip observed during governance/field probe and tracks the upward movement from earlier probes (0.409 → 0.454). Field is recovering coherence as fleet workers activate.

**k_mod at 0.85** is at the budget floor — coupling is at minimum allowed modulation. This is consistent with the auto-K scaling response to a mid-coherence field with mostly idle spheres.

---

## Sphere Distribution

| Category | Count | Notes |
|----------|-------|-------|
| Total spheres | **50** | +5 since initial probe (was 45) |
| Working | **6** | Fleet instances + orchestrator |
| Idle | **44** | Persistent ORAC7 registrations + fleet slots |
| Blocked | **0** | No stuck spheres |
| Fleet workers | **14** | Named fleet sphere registrations |

**5 new spheres** registered since session start (45 → 50). Fleet worker count of 14 exceeds the 6 working spheres — 8 fleet workers are registered but currently idle.

---

## IPC Bus

| Metric | Value | Notes |
|--------|-------|-------|
| Subscribers | 1 | Single active subscriber (likely sidecar) |
| Pending tasks | 5 | Down from 53 at session start — queue draining |
| Bus events | 1,000 | At buffer cap |
| Cascades | 0 | No cross-tab handoffs |

**Task queue recovered** from 53 malformed tasks → 5 pending. Either tasks expired or were consumed.

---

## Sidecar

| Metric | Value |
|--------|-------|
| Status | **UP** |
| Events processed | 405 |

Sidecar (WASM↔Bus bridge) is running and processing events. 405 events through the ring file since startup.

---

## Bridge Health

| Metric | Value |
|--------|-------|
| Stale bridges | **0** |
| Stale names | (none) |

**All 6 bridges fresh.** This is an improvement from the POVM stale flag observed in the bridge topology probe. POVM bridge has recovered its sync interval.

---

## Delta from Session Start

| Metric | Session Start | Now | Change |
|--------|--------------|-----|--------|
| Spheres | 45 | **50** | +5 |
| Working | 1 | **6** | +5 |
| r | 0.434 | **0.454** | +0.020 |
| Tick | 79,685 | **83,441** | +3,756 |
| Pending tasks | 53 | **5** | -48 (draining) |
| Stale bridges | 0 | **0** | Stable |
| Confidence | — | **100** | First measurement |

---

## Assessment

Fleet is in **optimal operational state** for an idle-dominant field:
- All infrastructure green (bridges, sidecar, bus)
- No blocked spheres, no stale bridges
- Task backlog cleared (53 → 5)
- r trending up (0.434 → 0.454) as fleet workers activate
- k_mod at budget floor (0.85) — coupling conservatively modulated

The system is ready for work dispatch. The 44 idle spheres represent capacity — deploying tasks via bus or cascade would shift the working/idle ratio and drive thermal activity.

---

*See also:* [[Session 049 — Full Remediation Deployed]] for prior fleet verification baseline | [[ULTRAPLATE Master Index]] for service topology
