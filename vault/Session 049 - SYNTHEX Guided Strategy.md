# Session 049 — SYNTHEX-Guided Strategy

**Date:** 2026-03-21 | **Bus Task:** 2bf3553d

## Step 1: Thermal State

| Metric | Value | Assessment |
|--------|-------|------------|
| Temperature | 0.03 | COLD (< 0.1) |
| Target | 0.50 | 94% below target |
| PID output | -0.335 | Controller pulling hard |

## Step 2: Strategy Selection → WARMING

Temperature 0.03 < 0.1 → **COLD SYSTEM**. Strategy: focus on high-activity warming tasks.

## Step 3: CrossSync — The Only Active Heat Source

| Source | Reading | Weight | Contribution |
|--------|---------|--------|-------------|
| Hebbian | 0.0 | 0.30 | 0.00 |
| Cascade | 0.0 | 0.35 | 0.00 |
| Resonance | 0.0 | 0.20 | 0.00 |
| **CrossSync** | **0.2** | **0.15** | **0.03** |

**What drives CrossSync?** CrossSync measures cross-service synchronization — the degree to which ULTRAPLATE services coordinate via shared state. Its 0.2 reading comes from the baseline inter-service health check heartbeats (16/16 services responding). CrossSync is the only source that doesn't require PV2 bridge wiring — it reads from the service mesh directly.

**Why the others read 0.0:**
- **Hebbian** — requires PV2 tick to POST Hebbian weight changes → SYNTHEX (bridge not wired, BUG-037)
- **Cascade** — requires cascade events forwarded from PV2 bus → SYNTHEX (bridge not wired)
- **Resonance** — requires field resonance metrics (r, spectrum) forwarded → SYNTHEX (bridge not wired)

## Step 4: ME ↔ SYNTHEX Trend Comparison

| Service | Metric | Value | Target | Gap |
|---------|--------|-------|--------|-----|
| ME | fitness | 0.619 | 0.75 | -17.5% |
| SYNTHEX | temperature | 0.03 | 0.50 | -94.0% |

Both below target but at different severities. ME at 0.619 is functional (above 0.5 minimum). SYNTHEX at 0.03 is effectively dormant — the thermal model contributes nothing to system intelligence.

**Correlation:** Both suffer from the same root cause — lack of cross-service data flow. ME doesn't receive POVM memories or PV2 field state. SYNTHEX doesn't receive any PV2 bridge data. Fix the bridges, fix both.

## Step 5: Warming Recommendations

### Immediate (no code changes)

1. **Increase CrossSync reading** — Run more cross-service queries (K7 synergy-check, multi-port health sweeps) to boost the only active heat source
2. **Manual thermal injection** — `POST /api/ingest` with Hebbian/Cascade data to simulate bridge output (accepted but doesn't change heat sources — BUG-037)

### Requires Code (Session 048 Block B)

3. **Wire PV2 tick Phase 2.7 bridge** — POST Hebbian weight deltas to SYNTHEX every tick → Hebbian source goes from 0.0 → proportional to LTP/LTD activity
4. **Wire cascade heat** — Forward `cascade.dispatched` bus events to SYNTHEX → Cascade source activates
5. **Wire field resonance** — POST r, spectrum harmonics to SYNTHEX → Resonance source activates
6. **Close PID → k_mod loop** — Read SYNTHEX PID output during bridge poll → modulate PV2 k_mod

### Projected Impact

| Source | Current | Post-Wiring | Weight | Contribution |
|--------|---------|-------------|--------|-------------|
| Hebbian | 0.0 | ~0.4 (12 active edges) | 0.30 | 0.12 |
| Cascade | 0.0 | ~0.2 (bus cascades) | 0.35 | 0.07 |
| Resonance | 0.0 | ~0.5 (r=0.95) | 0.20 | 0.10 |
| CrossSync | 0.2 | ~0.3 (more queries) | 0.15 | 0.05 |
| **Total** | **0.03** | **~0.34** | | |

Projected temperature: **0.34** (from 0.03) — still below 0.50 target but 11x improvement.

---
*Cross-refs:* [[Synthex (The brain of the developer environment)]], [[Session 049 - SYNTHEX Feedback Loop]], [[Session 049 - Observability Cluster]]
