# PV2-MAIN Final Diagnostics Sweep — Wave 7

> **Instance:** PV2-MAIN | Command tab | 2026-03-21
> **Tick:** 72,582 | **Sweep time:** 4ms

---

## 1. Habitat Probe Sweep

**Result: 16/16 HEALTHY** | Sweep: 4ms

| Port | Service | Status | Probe (ms) |
|------|---------|--------|------------|
| 8080 | Maintenance Engine | 200 | 0 |
| 8081 | DevOps Engine | 200 | 0 |
| 8090 | SYNTHEX | 200 | 0 |
| 8100 | SAN-K7 Orchestrator | 200 | 0 |
| 8101 | NAIS | 200 | 0 |
| 8102 | Bash Engine | 200 | 0 |
| 8103 | Tool Maker | 200 | 0 |
| 8104 | Context Manager | 200 | 0 |
| 8105 | Tool Library | 200 | 0 |
| 8110 | CodeSynthor V7 | 200 | 0 |
| 8120 | Vortex Memory System | 200 | 0 |
| 8125 | POVM Engine | 200 | 0 |
| 8130 | Reasoning Memory | 200 | 1 |
| 8132 | Pane-Vortex | 200 | 0 |
| 9001 | Architect Agent | 200 | 0 |
| 10001 | Prometheus Swarm | 200 | 0 |

**Note:** RM (:8130) is the only service with non-zero probe latency (1ms). All others respond sub-millisecond.

---

## 2. Latency Heatmap — All 16 Services

High-precision curl timing (`time_total` in seconds):

| Port | Service | HTTP | Latency (ms) | Heatmap |
|------|---------|------|-------------|---------|
| 8101 | NAIS | 200 | **0.169** | `█` |
| 8110 | CodeSynthor V7 | 200 | **0.184** | `█` |
| 10001 | Prometheus Swarm | 200 | **0.198** | `█` |
| 8102 | Bash Engine | 200 | **0.200** | `█` |
| 8125 | POVM Engine | 200 | **0.213** | `█` |
| 9001 | Architect Agent | 200 | **0.219** | `█` |
| 8100 | SAN-K7 | 200 | **0.223** | `█` |
| 8103 | Tool Maker | 200 | **0.227** | `█` |
| 8081 | DevOps Engine | 200 | **0.228** | `█` |
| 8105 | Tool Library | 200 | **0.236** | `█` |
| 8104 | Context Manager | 200 | **0.242** | `█` |
| 8090 | SYNTHEX | 404 | **0.279** | `█▌` |
| 8132 | Pane-Vortex | 200 | **0.292** | `█▌` |
| 8080 | ME | 404 | **0.359** | `██` |
| 8120 | VMS | 200 | **0.389** | `██` |
| 8130 | Reasoning Memory | 200 | **1.159** | `██████` |

### Latency Distribution

```
Latency (ms)
  0.0   0.2   0.4   0.6   0.8   1.0   1.2
  |     |     |     |     |     |     |
  NAIS        ▓ 0.169ms
  CS-V7       ▓ 0.184ms
  PromSwarm   ▓ 0.198ms
  BashEng     ▓ 0.200ms
  POVM        ▓ 0.213ms
  Architect   ▓ 0.219ms
  SAN-K7      ▓ 0.223ms
  ToolMaker   ▓ 0.227ms
  DevOps      ▓ 0.228ms
  ToolLib     ▓ 0.236ms
  CCM         ▓ 0.242ms
  SYNTHEX     ▓▒ 0.279ms (404 — /health not at /health)
  PV2         ▓▒ 0.292ms
  ME          ▓▓ 0.359ms (404 — /health at /api/health)
  VMS         ▓▓ 0.389ms
  RM          ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 1.159ms  ← 5x outlier
              |     |     |     |     |     |     |
```

### Latency Tiers

| Tier | Range | Count | Services |
|------|-------|-------|----------|
| **Ultra-fast** | < 0.20ms | 4 | NAIS, CodeSynthor, Prometheus, Bash Engine |
| **Fast** | 0.20-0.25ms | 7 | POVM, Architect, SAN-K7, ToolMaker, DevOps, ToolLib, CCM |
| **Normal** | 0.25-0.40ms | 4 | SYNTHEX, PV2, ME, VMS |
| **Slow** | > 1.0ms | 1 | **Reasoning Memory (1.159ms)** |

### Anomalies

1. **RM is 5x slower** than the median (0.228ms). Its 1.159ms latency is consistent with the habitat-probe 1ms reading. RM manages 3,732 TSV entries — the extra latency likely comes from file I/O or index scanning on health check.

2. **ME and SYNTHEX return 404** on `/health` because their health paths differ:
   - ME: `/api/health` (not `/health`)
   - SYNTHEX: `/api/health` (not `/health`)
   - This is a path mismatch, not a health issue — both respond 200 on their correct paths (confirmed by habitat-probe).

3. **VMS at 0.389ms** is the slowest of the 200-responders (excluding RM). Its health check reads field state which involves an internal r calculation.

---

## 3. POVM Pathway Analysis

### Pathway Statistics

| Metric | Value |
|--------|-------|
| **Total pathways** | 2,427 |
| **Total memories** | 50 |
| **Pathways per memory** | 48.5 avg |
| **Weight min** | 0.15 |
| **Weight max** | 1.0462 |
| **Weight avg** | 0.3026 |
| **Pathways > 1.0 (elevated)** | 2 (0.08%) |
| **Co-activations (all)** | 0 |
| **Last activated (all)** | null |

### Weight Distribution

```
Weight Range     Count   %      Bar
─────────────    ─────   ─────  ────────────────────────────────────────
0.10 - 0.20     ~800    33%    ████████████████████
0.20 - 0.30     ~600    25%    ███████████████
0.30 - 0.40     ~400    16%    ██████████
0.40 - 0.50     ~250    10%    ██████
0.50 - 0.70     ~200    8%     █████
0.70 - 1.00     ~175    7%     ████
1.00 - 1.05     2       0.1%   ▏
```

### Top 5 Strongest Pathways

| # | Pre (source) | Post (target) | Weight | Co-activations |
|---|-------------|---------------|--------|----------------|
| 1 | `nexus-bus:cs-v7` | `synthex` | **1.0462** | 0 |
| 2 | `nexus-bus:devenv-patterns` | `pane-vortex` | **1.0200** | 0 |
| 3 | `operator-028` | `alpha-left` | 1.0000 | 0 |
| 4 | `5:top-right` | `opus-explorer` | 1.0000 | 0 |
| 5 | `13` | `12` | 1.0000 | 0 |

### POVM Health Assessment

| Dimension | Score | Note |
|-----------|-------|------|
| Pathway count | HEALTHY | 2,427 is substantial — rich connectivity graph |
| Weight distribution | DEGRADING | Avg 0.30 (below 1.0 baseline) — mass decay without reinforcement |
| Co-activations | DEAD | All zero — no pathway has been activated this session |
| Crystallisation | ZERO | Consolidation showed 0 crystallised, 50 decayed |
| Strongest pathway | MEANINGFUL | `nexus-bus:cs-v7 → synthex` at 1.0462 — real architectural coupling |

**Diagnosis:** POVM has 2,427 pathways representing the Habitat's learned connectivity, but they're decaying without reinforcement. The average weight of 0.30 means most pathways have lost ~70% of their initial strength. Zero co-activations means PV2's bridge hasn't sent any activation signals (stale bridge confirmed). The top pathway (`nexus-bus:cs-v7 → synthex` at 1.0462) reflects a genuine architectural relationship — CodeSynthor feeds SYNTHEX through Nexus.

**With V2 deployed**, the POVM bridge would:
1. Send activation signals on each tick → co-activations > 0
2. Reinforce active pathways → weight recovery toward 1.0
3. Crystallise frequently-used pathways → permanent storage
4. New pathways created for V2-specific connections (Hebbian, ghost, governance)

---

## 4. Summary

### Infrastructure Health: EXCELLENT

- **16/16 services responding** — zero downtime across 65+ hour uptime
- **Median latency: 0.228ms** — sub-millisecond for 14/16 services
- **Single outlier: RM at 1.159ms** — 5x median but still sub-2ms
- **Total sweep: 4ms** — habitat-probe can health-check all 16 in 4ms

### Application Health: CRITICAL (unchanged)

| System | Status | Note |
|--------|--------|------|
| PV Field | r=0.678, IdleFleet | Stable post-QW1 but below R_TARGET |
| Bridges | 3/6 stale | POVM, RM, VMS still dark |
| SYNTHEX | Synergy 0.5 CRITICAL | Thermally frozen |
| ME | Deadlocked | Emergence cap saturated |
| POVM | 2,427 pathways decaying | Avg weight 0.30, zero activations |
| Bus | Improved post-QW1 | No longer monotone HasBlockedAgents |

### Bottom Line

The infrastructure is rock-solid. Every service is up, fast, and stable. The problems are all at the **application coordination layer** — bridges, coupling, thermal dynamics, evolution — and they all trace back to the V1 binary limitation. V2 is built, tested (1,527 tests), and ready. Deploy is the single remaining action.

---

PV2MAIN-WAVE7-COMPLETE
