# Session 049 — Hebbian Pulse Analysis

**Date:** 2026-03-21

---

## 1. Hebbian Pulse Database Mining

### PV2 neural_pathways (~/claude-code-workspace/developer_environment_manager/hebbian_pulse.db)

**0 rows** — gotcha #9 confirmed. The `neural_pathways` table exists with full schema (source_id, target_id, strength, ltp, ltd, stdp_delta, etc.) but has never been populated. Hebbian learning runs in-memory only.

### SYNTHEX phase1_patterns (synthex/hebbian_pulse.db)

87 patterns from service-mesh initialization (2026-01-20). Top 10:

| Pattern | Weight | Activations |
|---------|--------|-------------|
| service_mesh_pulse_2026-01-20 | 0.990 | 1 |
| DevOpsEngine_ToolLibrary_connection | 0.978 | 1 |
| DevOpsEngine_CCM_connection | 0.978 | 1 |
| DevOpsEngine_SYNTHEX_connection | 0.978 | 1 |
| DevOpsEngine_ToolMaker_connection | 0.978 | 1 |
| DevOpsEngine_SAN-K7_connection | 0.978 | 1 |
| ToolLibrary_SYNTHEX_connection | 0.965 | 1 |
| SYNTHEX_ToolLibrary_connection | 0.965 | 1 |
| CCM_ToolMaker_connection | 0.974 | 1 |
| ToolMaker_CCM_connection | 0.974 | 1 |

**Observation:** All patterns have activations=1 — they were seeded once and never reinforced. The weights (0.96-0.99) are initial strengths, not learned values. These are service-pair adjacency weights from the original devenv topology, not empirical co-activation data.

---

## 2. POVM Pathway Injection

Posted 5 tool-pair pathways using `pre_id/post_id` format:

| Pre | Post | Weight | Status |
|-----|------|--------|--------|
| Bash | Read | 0.70 | ok |
| Read | Edit | 0.80 | ok |
| Grep | Read | 0.70 | ok |
| Bash | Write | 0.60 | ok |
| Read | Bash | 0.65 | ok |

**Pathway count:** 2,427 → 2,437 (+10: 5 injected + 5 from post_tool_povm_pathway.sh hook)

### POVM Tool-Pair Pathways (all found)

Merged view of hook-generated (`tool:` prefix) and manually-injected (raw) pathways:

| Pre → Post | Weight | Source |
|------------|--------|--------|
| Read → Edit | 0.80 | manual + hook |
| Bash → Read | 0.70 | manual |
| Bash → Grep | 0.70 | hook (tool:) |
| Grep → Read | 0.70 | manual |
| Edit → Bash | 0.70 | hook (tool:) |
| Read → Bash | 0.65 | manual |
| Edit → Bash | 0.60 | hook (tool:) |
| Write → Bash | 0.60 | hook (tool:) |
| Bash → Write | 0.60 | manual |
| Bash → Edit | 0.50 | hook (tool:) |

**Top tool chain:** `Bash → Read → Edit → Bash` (weights: 0.70 → 0.80 → 0.70) — the classic TC1 funnel pattern confirmed by POVM pathway data.

---

## 3. PV2 Coupling Heavyweight Edges

**12 edges above weight 0.3** — all at 0.6, forming the fleet clique:

{fleet-alpha, fleet-beta-1, fleet-gamma-1, orchestrator-044} — fully bidirectional

**Weight distribution:**
- 3,770 edges at 0.09 (baseline)
- 12 edges at 0.60 (fleet clique)
- 0 intermediate weights

---

## 4. Cross-Layer Hebbian Comparison

| Layer | Source | Weights | Learned? |
|-------|--------|---------|----------|
| SYNTHEX phase1_patterns | SQLite | 0.96-0.99 | Seeded, not learned |
| PV2 neural_pathways | SQLite | (empty) | Never written |
| PV2 coupling network | In-memory | 0.09 or 0.60 | Binary LTP, no gradient |
| POVM pathways | HTTP API | 0.50-1.05 | Hook-written, never read back |

### The Hebbian Stack

```
Layer 1: SYNTHEX phase1_patterns (87 seeded, dormant since Jan 2026)
Layer 2: PV2 coupling network (in-memory STDP, binary weight jumps)
Layer 3: POVM pathways (2,437 tool-pair weights, accumulating but unused)
Layer 4: PV2 neural_pathways SQLite (schema exists, 0 rows)
```

**Finding:** Hebbian learning operates at 3 disconnected levels. SYNTHEX has seeded weights that never update. PV2 has live STDP that never persists. POVM accumulates tool pathways that PV2 never reads back into coupling. The three layers should form a feedback loop but instead operate as independent write-only stores.

---

## Cross-References

- [[Session 049 - Data Flow Verification]]
- [[Session 049 - Database Census]]
- [[Session 049 - Observability Cluster]]
- [[ULTRAPLATE Master Index]]
