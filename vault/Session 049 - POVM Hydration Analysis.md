# Session 049 — POVM Hydration Analysis

> **Date:** 2026-03-21 | **PV tick:** 81,560 | **r:** 0.409 | **Spheres:** 52 (35 ORAC7 + 17 named)
> **See also:** [[ULTRAPLATE Master Index]] | [[Session 049 — Full Remediation Deployed]] | [[Fleet-POVM-Deep-Dive]] | [[Fleet-Bridge-Topology]]

---

## Summary

POVM hydration is **structurally broken**. 71 memories and 2,427 pathways exist in the POVM engine, but **zero** have ever been read back (`access_count = 0` on all 71 memories, `co_activations = 0` on all 2,427 pathways). The root cause is an **ID namespace mismatch**: the 94 ORAC7 sphere IDs stored in POVM pathways have **zero overlap** with the 35 ORAC7 IDs currently registered in PV. Hydration requires both endpoints of a pathway to be registered as live spheres — since no pathway pair matches, no weights are ever applied.

---

## 1. Hydration Endpoint Data

### `/hydrate` — Summary

```json
{
  "memory_count": 71,
  "pathway_count": 2427
}
```

### `/memories` — Aggregate Statistics

| Metric | Value |
|--------|-------|
| **Total memories** | 71 |
| **Crystallised** | 0 |
| **Access count > 0** | 0 (zero reads ever) |
| **Session last accessed != null** | 0 |
| **Avg intensity** | 0.761 |
| **Min / Max intensity** | 0.590 / 1.000 |
| **Avg decay cycles survived** | 2.82 |
| **Categories** | All null (71/71) |
| **With session_created** | 11 (Session 027 era) |
| **Without session_created** | 60 (null — bulk-imported or bridge-written) |

### Memory Content Profile

All 71 memories contain session exploration summaries (Sessions 027–040+). They are rich in semantic content — field state, service counts, discovery notes — but:
- **No category tagging** — all `category: null`
- **No created_at timestamps** — all empty string
- **Never accessed** — write-only persistence
- **60/71 have null session_created** — written by PV bridge, not by named sessions

### `/pathways` — Weight Distribution

| Weight Band | Count | % |
|-------------|-------|---|
| w < 0.2 | 131 | 5.4% |
| 0.2 ≤ w ≤ 0.5 | 2,222 | 91.6% |
| 0.5 < w ≤ 0.9 | 23 | 0.9% |
| w > 0.9 | 51 | 2.1% |
| **Total** | **2,427** | |

**Bimodal structure confirmed:** The vast majority cluster in the 0.2–0.5 band (default weight territory after decay). A small cluster (51 pathways) has weights > 0.9, representing strong historical connections. Median is in the 0.2–0.3 range — post-decay baseline.

| Statistic | Value |
|-----------|-------|
| **Min weight** | 0.15 |
| **Max weight** | 1.0462 (nexus-bus:cs-v7 → synthex) |
| **Avg weight** | 0.303 |
| **Co-activations > 0** | 0 (zero — no pathway has ever been activated at runtime) |
| **Last activated != null** | 0 |

### Pathway Topology

| Metric | Value |
|--------|-------|
| **ORAC7-involved pathways** | 2,132 (87.8%) |
| **Non-ORAC7 pathways** | 295 (12.2%) |
| **Unique ORAC7 IDs in POVM** | 94 |
| **ORAC7 IDs registered in PV** | 35 |
| **Overlap (hydration candidates)** | **0** |

---

## 2. Root Cause: ID Namespace Mismatch

The PV hydration function (`povm_hydrate_startup` in `main.rs:1349–1383`) loads all 2,427 pathways, then applies only those where **both** `pre_id` and `post_id` are currently registered spheres:

```rust
for (sphere_a, sphere_b, weight) in &povm_weights {
    if s.network.phases.contains_key(sphere_a.as_str())
        && s.network.phases.contains_key(sphere_b.as_str())
    {
        s.network.set_weight(sphere_a, sphere_b, *weight);
        // ...
        applied += 1;
    }
}
```

Since ORAC7 process IDs are ephemeral (tied to PID), each daemon restart generates new IDs. The 94 ORAC7 IDs stored in POVM pathways are from prior process lifetimes. The 35 currently registered ORAC7 IDs are from the current process lifetime. **Zero overlap** means zero weights applied.

The non-ORAC7 pathways (295 total) use named IDs like `nexus-bus:cs-v7`, `operator-028`, `5:top-right`, and numeric IDs like `10`–`15`. Some of these *could* match current sphere registrations, but the named sphere IDs in PV are `fleet-alpha`, `fleet-beta-1`, `orchestrator-044`, etc. — no match to the pathway vocabulary.

**This is a design-level bug, not a transient failure.** POVM faithfully stores pathways, PV faithfully tries to hydrate, but the ID namespace rotates on every restart, making the persistent store permanently unreachable.

---

## 3. Consolidation Test

Running `POST /consolidate` returned:

```json
{
  "crystallised": 0,
  "decayed": 71,
  "pathways_pruned": 0,
  "removed": 0
}
```

Consolidation **decayed all 71 memories** (reduced intensity) but **pruned zero pathways** and **crystallised nothing**. The POVM engine's consolidation mechanism doesn't address the ID mismatch — it manages memory lifecycle but doesn't reconcile pathway endpoints against live sphere registrations.

---

## 4. Top 10 Strongest Pathways

| # | Pre-node | Post-node | Weight | Co-activations |
|---|----------|-----------|--------|----------------|
| 1 | nexus-bus:cs-v7 | synthex | 1.046 | 0 |
| 2 | nexus-bus:devenv-patterns | pane-vortex | 1.020 | 0 |
| 3 | operator-028 | alpha-left | 1.000 | 0 |
| 4 | 5:top-right | opus-explorer | 1.000 | 0 |
| 5 | 13 | 12 | 1.000 | 0 |
| 6 | 10 | 14 | 1.000 | 0 |
| 7 | 14 | 13 | 1.000 | 0 |
| 8 | 13 | 14 | 1.000 | 0 |
| 9 | 14 | 12 | 1.000 | 0 |
| 10 | 12 | 14 | 1.000 | 0 |

Note: IDs `10`–`15` are from a prior numeric sphere registration era. `nexus-bus:*` and `operator-*` are from Session 027–028 bridge experiments. None match current sphere IDs.

---

## 5. ORAC7 Hub Analysis

Top pre-nodes by outgoing edge count (all ORAC7):

| Pre-node | Outgoing Edges |
|----------|---------------|
| ORAC7:3067258 | 57 |
| ORAC7:3582557 | 57 |
| ORAC7:3611772 | 57 |
| ORAC7:3485165 | 56 |
| ORAC7:3447344 | 55 |

Top post-nodes by incoming edge count (all ORAC7):

| Post-node | Incoming Edges |
|-----------|---------------|
| ORAC7:3012890 | 61 |
| ORAC7:3485165 | 58 |
| ORAC7:3144870 | 57 |
| ORAC7:3309665 | 57 |
| ORAC7:3521706 | 56 |

The ORAC7 subgraph is densely connected (~57 edges per node in a ~94-node cluster) — a well-formed Hebbian network that simply cannot be loaded because the IDs are stale.

---

## 6. Sphere Registration Snapshot

### 52 Live Spheres

| Type | Count | Examples |
|------|-------|---------|
| **ORAC7 (PID-based)** | 35 | ORAC7:3842855, ORAC7:2760720, ... |
| **Fleet agents** | 6 | fleet-alpha, fleet-beta-1, fleet-beta-2, fleet-gamma-1, fleet-gamma-2, alpha-heat-gen |
| **Pane positions** | 8 | 4:left, 4:top-right, 4:bottom-right, 5:left, 5:top-right, 5:bottom-right, 6:left, 6:top-right, 6:bottom-right |
| **Named agents** | 2 | orchestrator-044, test-hook-768523 |

The pane-position IDs (`4:left`, `5:top-right`, etc.) are stable across restarts — they're based on Zellij tab:pane geometry, not PIDs. These are potential candidates for durable pathway storage.

---

## 7. Diagnosis Summary

| Finding | Severity | Impact |
|---------|----------|--------|
| **ID namespace mismatch** — POVM stores PID-based IDs that rotate on restart | CRITICAL | 100% of hydration fails |
| **Zero access count** — all 71 memories never read | HIGH | Write-only persistence (BUG-034) |
| **Zero co-activations** — all 2,427 pathways never activated | HIGH | Persistent store is dead weight |
| **All categories null** — no semantic tagging on memories | MEDIUM | Reduces consolidation effectiveness |
| **All created_at empty** — no temporal ordering | MEDIUM | Cannot prune by age |
| **Zero crystallisation** — no long-term memory formation | MEDIUM | All memories remain volatile |
| **Bimodal weight distribution** — 91.6% in decay band | LOW | Expected from LTP/LTD + decay cycles |

---

## 8. Remediation Options

### Option A: Stable ID Mapping (Recommended)

Map ORAC7 PID-based IDs to stable aliases before writing to POVM. Candidates:
- Pane-position IDs (`4:left`, `5:top-right`) — stable across restarts
- Role-based IDs (`fleet-alpha`, `orchestrator`) — stable by convention
- Hash of (tab + pane-position + persona) — stable if layout doesn't change

**Impact:** New pathways would use stable IDs. Old pathways remain unreachable but can be pruned.

### Option B: ID Translation Layer

On hydration, translate POVM pathway IDs to current sphere IDs using a mapping table (e.g., match by pane position or persona). Requires POVM to store metadata alongside IDs.

**Impact:** Could recover existing pathways. More complex, fragile if metadata changes.

### Option C: Pathway Pruning + Fresh Start

Prune all 2,427 stale pathways, start fresh with stable IDs (Option A). Accept the loss of historical Hebbian structure.

**Impact:** Clean slate. Simple. Loses the bimodal weight structure but it was never used anyway.

### Option D: Memory Category Population

Independent of ID fix — populate `category` and `created_at` fields on all bridge writes. Enables consolidation to be meaningful.

**Impact:** Improves POVM data quality for future use. Should be done regardless of A/B/C choice.

---

## 9. Cross-References

### Source Code
- `src/povm_bridge.rs:220–226` — `hydrate_pathways()` read path
- `src/povm_bridge.rs:178–209` — `post_field_snapshot()` and `post_hebbian_weights()` write path
- `src/main.rs:1349–1383` — `povm_hydrate_startup()` — the join that fails
- `src/coupling.rs` — `network.set_weight()` target of hydrated pathways

### Obsidian
- [[ULTRAPLATE Master Index]] — Service registry, POVM at port 8125, Batch 1
- [[Session 049 — Full Remediation Deployed]] — Prior bridge remediation context
- [[Fleet-Bridge-Topology]] — POVM bridge status (stale flag, 12/60-tick sync intervals)
- [[Fleet-POVM-Deep-Dive]] — Prior POVM analysis
- [[Hebbian Learning Deep Dive]] — Hebbian STDP mechanics in PV coupling network

### Related Bugs
- **BUG-034:** POVM write-only pathology (access_count=0 on all memories)
- **ALERT-6:** Learning-doing gap (0 live Hebbian edges vs 2,427 POVM pathways)

---

*Captured at tick 81,560 | 71 memories | 2,427 pathways | 0 accessed | 0 hydrated | 2026-03-21*
