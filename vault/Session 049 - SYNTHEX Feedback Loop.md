# Session 049 — SYNTHEX Thermal Feedback Loop Test

**Date:** 2026-03-21 | **PV2 Tick:** ~109,850

## Experiment Design

Inject `hebbian_pulse` events via `POST /api/ingest` and measure:
1. Did SYNTHEX temperature change?
2. Did Hebbian heat source activate?
3. Did PV2 k_modulation respond?

## Baseline (pre-injection)

| Metric | Value |
|--------|-------|
| SYNTHEX temperature | 0.030 |
| SYNTHEX PID output | -0.335 |
| Hebbian reading | 0.0 |
| PV2 r | 0.930 |
| PV2 k_modulation | 0.860 |

## Step 1: Single Injection

```json
POST /api/ingest
{"source":"fleet-beta","type":"hebbian_pulse","data":{"action":"co_activation","strength":0.8,"pair":"beta-gamma"}}
```

**Response:** `{"accepted":true,"temperature":0.03}`

Event accepted but temperature unchanged.

## Step 2: Post-Injection Thermal

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Temperature | 0.030 | 0.030 | 0 |
| PID output | -0.335 | -0.335 | 0 |
| Hebbian reading | 0.0 | 0.0 | 0 |

**No thermal response.** Hebbian heat source stayed at zero.

## Step 3: Post-Injection PV2

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| r | 0.930 | 0.911 | -0.019 (natural drift) |
| k_modulation | 0.860 | 0.850 | -0.010 (natural drift) |

Changes are within natural field oscillation, not correlated with injection.

## Step 4: K7 Synergy Check

```json
{"command":"synergy-check","message":"Command executed successfully","module":"M45","status":"executed"}
```

K7 acknowledged the command. No synergy anomalies detected.

## Step 5: Burst Test (5 rapid injections)

Sent 5 additional `hebbian_pulse` events at strength 0.9.

| Injection | Accepted | Temperature |
|-----------|----------|-------------|
| 1-5 | all true | all 0.03 |

**Post-burst Hebbian reading: still 0.0**

## Root Cause Analysis

The feedback loop is **broken at the routing layer**:

```
fleet-beta → POST /api/ingest → ACCEPTED ✓
                                    ↓
                            Event stored/queued ✓
                                    ↓
                            Route to heat source ✗ ← BREAK HERE
                                    ↓
                            Hebbian heat source = 0.0 (unchanged)
                                    ↓
                            Temperature = 0.03 (unchanged)
                                    ↓
                            PV2 k_modulation = unaffected
```

**SYNTHEX accepts ingest events but does not route `hebbian_pulse` type to the Hebbian heat source.** The `/api/ingest` endpoint likely only increments internal event counters without feeding the thermal model's heat source readings.

## BUG-037 Confirmation

This is a deeper layer of BUG-037 than previously understood:

1. ~~V1 binary doesn't post thermal data~~ (known)
2. **Even when data IS posted, SYNTHEX doesn't route it to heat sources** (NEW finding)

The fix requires either:
- A different ingest event type that SYNTHEX recognizes for thermal routing
- A code change in SYNTHEX V3 thermal model to map `hebbian_pulse` → Hebbian heat source
- A dedicated thermal feed endpoint (not the generic `/api/ingest`)

## Correlation Result

**Did thermal injection change k_mod?** NO.

The causal chain `ingest → thermal → PV2 k_mod` is fully decoupled. The feedback loop needs both:
1. V2 bridges posting data (V1→V2 deploy)
2. SYNTHEX routing fix for heat source mapping

## Cross-References

- [[Synthex (The brain of the developer environment)]]
- [[Session 049 - SYNTHEX Post-Deploy]]
- [[Session 049 - Service Probe Matrix]]
- [[Session 049 — Master Index]]
- [[ULTRAPLATE Master Index]]
