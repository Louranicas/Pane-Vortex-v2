# Session 049 — Hebbian Weight Stability Check

**Date:** 2026-03-21 | **Bus Task:** f15800cf

## Method

Sampled coupling matrix at two points 14 ticks apart (tick 107,570 → 107,584) to check for weight drift in the fleet clique.

## Results

| Sample | Tick | High-Weight Edges | All Weights | r (order) |
|--------|------|-------------------|-------------|-----------|
| T0 | 107,570 | 12 | all 0.60 | 0.937 |
| T1 | 107,584 | 12 | all 0.60 | 0.922 |

**No drift detected.** All 12 fleet clique edges held at exactly 0.60 across the observation window.

## Fleet Clique Members

| Node | Edges |
|------|-------|
| orchestrator-044 | 6 (3 in, 3 out) |
| fleet-alpha | 6 (3 in, 3 out) |
| fleet-beta-1 | 6 (3 in, 3 out) |
| fleet-gamma-1 | 6 (3 in, 3 out) |

Fully connected bidirectional clique (4C2 × 2 = 12 edges).

## Analysis

- Weights are quantized at 0.60 — not continuously varying. This suggests Hebbian STDP uses discrete weight buckets or thresholds rather than continuous gradient updates.
- The baseline weight (0.09 for 3,648 other edges) is also stable — no spontaneous strengthening.
- r oscillated between 0.922–0.937 during the window but weights did not respond, confirming weights update on co-activation events, not on r fluctuations.
- No LTD (long-term depression) decay observed — the clique weights are persistent even though the fleet spheres are currently idle.

## Implications

1. **Weights are durable** — once strengthened by co-activation, they persist without ongoing activity
2. **No decay mechanism active** — Hebbian LTD (0.002/tick) may require active anti-correlation, not just idle time
3. **Weight quantization** — investigate if weights snap to discrete levels (0.09, 0.60) or if these are just current equilibrium points

---
*Cross-refs:* [[Session 049 - Post-Deploy Coupling]], [[Vortex Sphere Brain-Body Architecture]]
