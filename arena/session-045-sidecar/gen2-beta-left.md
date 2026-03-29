# Gen2 BETA-LEFT Report — Coupling Matrix Sample

**Agent:** BETA-LEFT
**Timestamp:** 2026-03-21
**Source:** `localhost:8132/coupling/matrix`

## Coupling Matrix Summary

| Metric | Value |
|--------|-------|
| **Total edges** | 552 |
| **Sample weight** | 0.108 (uniform across sample) |

## Sample (first 5 edges)

| From | To | Weight |
|------|-----|--------|
| ORAC7:2760720 | ORAC7:2357088 | 0.108 |
| ORAC7:2760428 | ORAC7:1137588 | 0.108 |
| ORAC7:2754623 | ORAC7:2344553 | 0.108 |
| ORAC7:2767482 | 5:bottom-right | 0.108 |
| ORAC7:2357088 | ORAC7:2766873 | 0.108 |

## Observations

- 552 edges indicates a dense coupling network (likely ~24 spheres fully connected: 24*23=552)
- All sampled weights are 0.108 — uniform Hebbian state, no differentiation yet
- One cross-type edge: `ORAC7:2767482 → 5:bottom-right` (Zellij pane ID format vs ORAC prefix)
- Default weight 0.18 * decay = 0.108 suggests weights have decayed from initial 0.18 baseline

BETA-LEFT REPORTING: coupling matrix sampled
