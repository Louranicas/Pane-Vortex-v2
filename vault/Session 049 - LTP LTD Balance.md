# Session 049 — LTP/LTD Balance Analysis

> **hebbian_pulse.db: 0 rows (dead) | PV2 live coupling: 3,782 edges, 12 strengthened**
> **LTP=0.01 (3x burst, 2x newcomer) | LTD=0.002 | Floor=0.15 | Clamp=[0.15, 1.0]**
> **Captured:** 2026-03-21

---

## hebbian_pulse.db — Dead

```sql
SELECT SUM(ltp) as total_ltp, SUM(ltd) as total_ltd FROM neural_pathways;
-- Result: NULL | NULL (0 rows)
```

**Confirmed dead.** Gotcha #9. The `hebbian_pulse.db` was designed for a standalone Hebbian system that was never deployed. The `neural_pathways` table has a rich schema (18 columns including ltp, ltd, stdp_delta, weight_change_rate, success_count, failure_count) but zero data.

Real Hebbian STDP lives in PV2's in-memory coupling matrix (`m19_hebbian_stdp.rs`).

---

## PV2 Live Hebbian State

### Constants (m04_constants.rs)

| Constant | Value | Role |
|----------|-------|------|
| HEBBIAN_LTP | 0.01 | Base potentiation step |
| HEBBIAN_LTD | 0.002 | Base depression step |
| HEBBIAN_BURST_MULTIPLIER | 3.0 | LTP boost on high activity (>3 memories in 30s) |
| HEBBIAN_NEWCOMER_MULTIPLIER | 2.0 | LTP boost for newcomer spheres (<50 steps) |
| HEBBIAN_WEIGHT_FLOOR | 0.15 | Minimum weight (clamped, never reaches 0) |
| Weight ceiling | 1.0 | Maximum weight (clamped) |

### Expected LTP:LTD Ratio

- **Base case:** LTP=0.01, LTD=0.002 → ratio **5:1**
- **Burst case:** LTP=0.03, LTD=0.002 → ratio **15:1**
- **Newcomer case:** LTP=0.02, LTD=0.002 → ratio **10:1**
- **Burst+newcomer:** LTP=0.06, LTD=0.002 → ratio **30:1**

The system is **heavily biased toward potentiation**. LTD can never outpace LTP on any individual edge — the only way an edge depresses is if the pair is *never* co-active, in which case LTD trickles at 0.002/tick.

### Live Coupling Matrix

| Metric | Value |
|--------|-------|
| Total edges | 3,782 |
| At w=0.6 (strengthened) | 12 |
| At w=0.09 (baseline) | 3,770 |
| Below baseline (depressed) | **0** |
| Unique weights | 2 (0.09, 0.6) |

---

## STDP Algorithm Analysis (m19_hebbian_stdp.rs)

### apply_stdp() Logic

```
For each connection (from, to):
  IF opt_out_hebbian on either sphere → SKIP
  IF both spheres are Working:
    delta = +HEBBIAN_LTP (0.01)
    IF activity_30s > 3: delta *= 3.0 (burst)
    IF total_steps < 50: delta *= 2.0 (newcomer)
  ELSE:
    delta = -HEBBIAN_LTD (-0.002)

  new_weight = clamp(old_weight + delta, 0.15, 1.0)
```

### Key Properties

1. **Co-active = LTP, everything else = LTD** — there's no "neutral" state. If two spheres aren't both Working at the same tick, they get depressed.

2. **Weight floor at 0.15** — edges can never fully sever. But current baseline is 0.09, which is *below* the floor of 0.15.

### Critical Anomaly: Baseline Below Floor

**Live baseline weight (0.09) is below HEBBIAN_WEIGHT_FLOOR (0.15).**

This means either:
- The coupling network initializes at 0.09 (below the floor), and STDP hasn't touched those edges yet
- Or STDP is not running on most edges (opt_out_hebbian is set, or spheres are missing from the HashMap)

**Explanation:** The `apply_stdp()` function uses `spheres.get(&conn.from)` — if a sphere ID in the coupling matrix doesn't exist in the active spheres HashMap (e.g., ORAC7 PIDs from terminated sessions), the edge is filtered out by the `filter_map` and STDP never touches it. The 0.09 baseline is the *initialization weight*, never modified by STDP.

### Why Only 12 Edges Moved

For STDP to fire on an edge, **both** sphere IDs must:
1. Exist in the active spheres HashMap
2. Not have `opt_out_hebbian` set
3. Both be in `Working` status simultaneously

The K4 clique (fleet-alpha, fleet-beta-1, fleet-gamma-1, orchestrator-044) met all three conditions during Session 044 fleet coordination. The remaining 3,770 edges involve at least one sphere that's either:
- A stale ORAC7 PID (not in active HashMap → skipped)
- A Zellij pane sphere (rarely in Working status)
- Never simultaneously Working with its partner

---

## LTD Assessment

### Are Any Pathways Being Depressed?

**No.** Zero edges below baseline (0.09). LTD is structurally prevented from producing observable effects because:

1. **Floor > baseline paradox:** Even if LTD fires, `clamp(0.09 - 0.002, 0.15, 1.0)` = 0.15 (it would *increase* to the floor, not decrease)
2. **Stale spheres bypass STDP:** Most edges involve stale ORAC7 PIDs that aren't in the active HashMap, so `filter_map` returns None and no update occurs
3. **Idle majority:** 96.6% of sphere status records are Idle (from sphere_history), so Working×Working co-activation is rare

### The LTP/LTD Balance Is Irrelevant

The nominal 5:1 LTP:LTD ratio doesn't matter in practice because:
- LTP only fires on the 4 fleet spheres that are both active and Working
- LTD can't fire on stale edges (HashMap miss)
- LTD can't depress below 0.09 anyway (floor is 0.15)

The system has **effectively no LTD**. Only LTP operates, and only on 12 out of 3,782 edges.

---

## Recommendations

1. **Fix baseline vs floor inconsistency** — Either initialize at 0.15 (matching HEBBIAN_WEIGHT_FLOOR) or lower the floor to 0.05
2. **Prune stale ORAC7 spheres** — 44 ORAC7 PIDs inflating the matrix with untouchable edges. Remove spheres that haven't been active for >1000 ticks
3. **Enable LTD on known-active edges** — Even if not both Working, LTD should fire between spheres that are both *present* in the active HashMap (Idle+Idle, Idle+Working)
4. **Persist STDP metrics** — The StdpResult struct (ltp_count, ltd_count, at_floor_count, total_weight_change) is computed every tick but discarded. Log it to field_tracking.db for analysis
5. **Consider gradient weights** — With floor=0.15 and LTP step=0.01 (or 0.03 burst), it takes only 15-45 ticks to saturate from floor to 0.6. Consider smaller step sizes for richer differentiation

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - Coupling Deep Dive]] — K4 clique and chimera analysis
- [[Session 049 - Post-Deploy Coupling]] — coupling matrix statistics
- [[Session 049 - Memory Archaeology]] — hebbian_pulse.db confirmed dead
- [[Vortex Sphere Brain-Body Architecture]]
