# Fleet POVM Health Report

**Date:** 2026-03-21
**Source:** Fleet agent probe (POVM Engine, port 8125)
**Cross-ref:** [[MASTER INDEX]], [[Session 049 — Fleet SYNTHEX Report]], [[Session 049 — Bridge Diagnostics and Schematics]]

---

## Hydration Summary

| Metric | Value | Assessment |
|--------|-------|------------|
| Memories | 58 | Moderate store |
| Pathways | 2,427 | Large network |
| Crystallised | **0** | NO CONSOLIDATION |
| Sessions | **0** | NO SESSION TRACKING |
| Latest r | 0.408 | Mid-coherence (from PV field) |

---

## Memory Access Analysis

| Metric | Value | Assessment |
|--------|-------|------------|
| Total memories | 58 | — |
| Zero-access memories | **58 (100%)** | CRITICAL — none ever recalled |
| Max access count | 0 | — |
| Avg access count | 0.0 | — |

**All 58 memories have never been accessed.** POVM is a write-only store — memories are deposited but never retrieved by any service. The `hydrate` endpoint exists and returns data, but no service calls it during normal operation (PV calls `hydrate_pathways()` + `hydrate_summary()` on startup only).

### Memory Sample (first 3)

| ID | Content (truncated) |
|----|---------------------|
| 32e9b820 | Session 027: Zellij synthetic devenv fully deployed and expl... |
| 5052d714 | Session 027b: Pane navigation mastery. 9 fleet panes deploye... |
| 8f6cec1d | Session 027c: Complete system schematics created. 13-section... |

All memories are session-level summaries (Session 027+). No fine-grained tool-level or sphere-level memories — these go to PV's own sphere memory system and RM instead.

---

## Pathway Weight Distribution

| Range | Count | Percentage | Interpretation |
|-------|-------|------------|----------------|
| < 0.3 | 2,314 | **95.3%** | Weak/default connections |
| 0.3 – 0.9 | 62 | 2.6% | Intermediate (transitioning) |
| > 0.9 | 51 | 2.1% | Strong (consolidated) |
| > 1.0 | 2+ | — | **UNBOUNDED — exceeds theoretical max** |

| Stat | Value |
|------|-------|
| Min weight | 0.15 |
| Max weight | **1.0462** |
| Avg weight | 0.303 |

### Bimodal Distribution Confirmed

The weight distribution is strongly bimodal — clustering at <0.3 (weak) and >0.9 (strong) with very few in between. This confirms the **phase-transition learning** pattern identified in Session 039: pathways either consolidate strongly or decay toward baseline. There is no gradual middle ground.

### Unbounded Weight Anomaly

At least 2 pathways exceed weight 1.0:
- Weight 1.0462 (source: null, target: null)
- Weight 1.020 (source: null, target: null)

Both have **null source and null target** — these are orphaned pathways with no memory anchors. The weight exceeding 1.0 suggests Hebbian LTP is applied without an upper clamp. The null source/target means these pathways were created or corrupted without proper memory linkage.

---

## Health Assessment

### Working Well
- **Pathway formation is active** — 2,427 pathways indicates Hebbian learning is functional
- **Bimodal distribution is healthy** — phase-transition learning is the expected behavior for STDP
- **Bridge connectivity OK** — PV reports `povm_stale: false`, bridge is alive
- **Hydration endpoint responsive** — <50ms response time

### Critical Issues

| # | Issue | Severity | Impact |
|---|-------|----------|--------|
| 1 | **100% zero-access memories** | CRITICAL | POVM is write-only; no recall = no learning loop |
| 2 | **0 crystallised memories** | HIGH | Consolidation pipeline never runs |
| 3 | **0 sessions** | HIGH | Session tracking non-functional |
| 4 | **Unbounded weights >1.0** | MEDIUM | LTP has no upper clamp; could drift further |
| 5 | **Null source/target on strong pathways** | MEDIUM | Orphaned pathways — structural integrity gap |

---

## Relationship to SYNTHEX Thermal State

SYNTHEX heat source HS-001 (Hebbian) reads **0.0** despite POVM having 2,427 active pathways. This disconnect confirms the finding from [[Session 049 — Fleet SYNTHEX Report]]: **Hebbian learning activity in POVM is not being bridged to SYNTHEX as thermal signal.** The fuel exists (pathways) but the pipe is disconnected.

The combined effect:
- POVM accumulates pathways but never recalls them → no feedback loop
- SYNTHEX sees no Hebbian heat → temperature stays at 0.03
- PV's `povm_bridge.rs` writes snapshots every 12 ticks and weights every 60 ticks, but SYNTHEX doesn't consume POVM data as a heat source

---

## Recommendations

### Immediate
1. **Add weight clamp to POVM** — Cap pathway weights at 1.0 to prevent drift
2. **Audit null source/target pathways** — Identify creation path for orphaned pathways
3. **Trigger consolidation** — POST to `/consolidate` to test if crystallisation works at all

### Medium-term (V3.2 Inhabitation)
4. **Wire POVM pathway events to SYNTHEX HS-001** — Bridge pathway weight changes as Hebbian heat signals
5. **Implement periodic recall** — Have PV or another service call `/memories` with context to drive access counts above zero
6. **Add session registration** — PV's `povm_bridge.rs` should register sessions on startup so `session_count > 0`

### Verification Commands
```bash
# Test consolidation
curl -s -X POST localhost:8125/consolidate | jq '.'

# Check if any memory gets accessed after hydration
curl -s localhost:8125/hydrate | jq '.memory_count'
curl -s localhost:8125/memories | jq '[.[] | select(.access_count > 0)] | length'

# Monitor pathway weight drift
curl -s localhost:8125/pathways | jq '[.[] | select(.weight > 1.0)] | length'
```

---

## ULTRAPLATE Master Index Cross-Reference

| Service | Port | Role in This Analysis |
|---------|------|-----------------------|
| POVM Engine | 8125 | Subject — memory/pathway store probed |
| Pane-Vortex | 8132 | Bridge partner — writes snapshots + weights every 12/60 ticks |
| SYNTHEX | 8090 | Thermal consumer — Hebbian heat source reads 0.0 despite active POVM |
| RM | 8130 | Parallel memory store — 2,169 PV entries (vs POVM's 58) |
| VMS | 8120 | Dormant — r=0.0, 0 memories, zone=Incoherent |

See [[MASTER INDEX]] for full service registry.
See [[Session 049 — Fleet SYNTHEX Report]] for thermal starvation analysis.
See [[Session 040 — Deep Exploration]] for original POVM bimodal discovery.
