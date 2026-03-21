# Session 049 — Habitat Full Probe

> **Tick:** 110,066 | **Date:** 2026-03-21 | **6-axis probe**

---

## 1. Pulse (habitat-probe full)

| Service | Status | Key Metrics |
|---------|--------|-------------|
| PV2 | healthy | r=0.922, tick=110,066, 62 spheres, K_mod=0.854, fleet=Full |
| ME | Degraded (Stable) | fitness=0.619, tick=15,280, trend=Stable |
| POVM | Active | 82 memories (+2 since audit), 2,427 pathways |

### Sweep: 16/16 healthy (3ms)

All services responding HTTP 200. Sub-millisecond across the board.

### Bridges

| Bridge | Stale? |
|--------|--------|
| Nexus | No |
| SYNTHEX | No |
| ME | No |
| POVM | **Yes** |
| RM | No |
| VMS | No |

POVM bridge marked stale — last poll timestamp exceeded threshold. Likely the bridge poll interval hasn't fired recently or POVM read-back (BUG-034) isn't triggering freshness updates.

---

## 2. Fleet Verify

| Metric | Value |
|--------|-------|
| Spheres | 62 |
| Working | 4 (orchestrator-044, fleet-alpha, fleet-beta-1, fleet-gamma-1) |
| Idle | 51 |
| Blocked | 7 |
| Fleet workers | 14 |
| Subscribers | 1 |
| Pending tasks | 28 |
| Bus events | 1,000 (ring buffer full) |
| Sidecar | UP (405 events) |
| Stale bridges | 1 (POVM) |
| Confidence | 90% (dropped from 100% due to POVM stale) |

---

## 3. Coupling Network

| Metric | Value | Delta from T=107,469 |
|--------|-------|---------------------|
| Total edges | 3,782 | +122 (was 3,660) |
| Weight=0.09 (baseline) | 3,770 | +122 |
| Weight=0.6 (fleet clique) | 12 | unchanged |

**New spheres joined** — edge count grew from 3,660 to 3,782, meaning ~2 new spheres registered (62×61=3,782). The 12 differentiated clique edges remain stable. No new Hebbian differentiation.

---

## 4. Governance

| Metric | Value |
|--------|-------|
| Total proposals | 16 |
| Applied | 5 |
| Expired | 11 |

### Active Governance Overrides

| Parameter | Default | Applied Value |
|-----------|---------|---------------|
| CouplingSteps | 15 | **20** |
| KModBudgetMax | 1.15 | **1.4** |
| RTarget | 0.93 | **0.85** |

Field is operating with widened K budget, more coupling steps per tick, and a lower convergence target — all governance-driven modifications from gamma-left-wave8 and gamma-synergy proposals.

---

## 5. Chimera Detection

| Metric | Value |
|--------|-------|
| is_chimera | false |
| Sync clusters | 3 |

Field is NOT in chimera state. 3 synchronisation clusters detected — the field has natural phase groupings but they are not pathological (chimera requires desynchronised subpopulations).

---

## 6. Tunnels

| Metric | Value |
|--------|-------|
| Total tunnels | 100 |
| Strongest overlap | 1.0 (perfect) |

100 active tunnels with the strongest at perfect overlap (1.0). Tunnels represent phase-coherent buoy pairs that can exchange information across the field.

---

## Ecosystem Health Summary

```
SERVICES:    16/16 healthy (3ms sweep)
FIELD:       r=0.922, 62 spheres, 4 working, 7 blocked
COUPLING:    3,782 edges, 12 differentiated (fleet clique at 0.6)
GOVERNANCE:  5 applied, 3 active overrides
CHIMERA:     false (3 sync clusters — healthy)
TUNNELS:     100 active, strongest=1.0
BRIDGES:     5/6 fresh (POVM stale — BUG-034 related)
ME:          fitness=0.619 (degraded, stable)
POVM:        82 memories, 0 reads, 2,427 pathways
SIDECAR:     UP (405 events)
CONFIDENCE:  90%
```

---

## Observations

1. **r-order oscillating** — 0.944 → 0.967 → 0.922 across probes this session. The field is breathing healthily rather than pinning at 1.0 (V1 pathology). Auto-K at 0.854 (near floor) confirms damping is active.

2. **Coupling growth** — 2 new spheres added (3,660→3,782 edges) but no new Hebbian differentiation. The fleet clique is frozen at 12 edges.

3. **POVM bridge stale** — only stale bridge. Directly correlated with BUG-034 (write-only pathology). No read-back = no freshness updates.

4. **7 blocked spheres persistent** — decision engine reports HasBlockedAgents. These may be ghosts eligible for reincarnation (V3.2 feature).

5. **Confidence drop 100%→90%** — solely due to POVM bridge staleness.

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - Post-Deploy Coupling]] — coupling deep dive
- [[Session 049 - POVM Audit]] — BUG-034 analysis
- [[Session 049 - Quality Gate T810]] — 1527 tests clean
- [[Session 049 - Security Audit]] — hooks + Rust audit
- [[Session 049 - Cascade Synthesis]] — 3-stage cascade
- [[Session 049 - DB Probe Chain]] — SQLite + probe
- [[ULTRAPLATE Master Index]]
