# RM Corridor Analysis — Cross-Service Signal Assessment

> **Captured:** 2026-03-21T04:48 UTC | **RM Active Entries:** 5,341 | **PV Tick:** 100,000+
> **Cross-refs:** [[ULTRAPLATE Master Index]] | [[Session 049 — Full Remediation Deployed]]

---

## Key Finding

RM is **overwhelmingly dominated by PV telemetry** (3,462/5,341 = 64.8%). The RM→ME corridor carries low-value repetitive field-state logging, not actionable intelligence. PV2-specific entries (205) are mostly ORAC7 session-end tombstones. The corridor is connected but carrying noise, not signal.

---

## 1. RM Health Overview

| Metric | Value |
|--------|-------|
| Total active entries | **5,341** |
| Status | healthy |

### Category Distribution

| Category | Count | Percentage | Signal Quality |
|----------|-------|------------|---------------|
| context | 3,949 | 74.0% | LOW — mostly tick/r/k_mod snapshots |
| shared_state | 1,295 | 24.2% | LOW — field state logging |
| discovery | 78 | 1.5% | **HIGH** — cross-service findings |
| plan | 10 | 0.2% | MEDIUM — strategic intent |
| theory | 9 | 0.2% | MEDIUM — architectural hypotheses |

**97% of RM content is low-value telemetry** (context + shared_state). Only 97 entries (1.8%) carry genuine signal (discovery + plan + theory).

### Top Agents (by entry count)

| Agent | Entries | % of Total | Content Type |
|-------|---------|------------|-------------|
| **pane-vortex** | 3,462 | 64.8% | Field state ticks, conductor decisions |
| orchestrator | 182 | 3.4% | Cross-service observations |
| claude:opus-4-6 | 160 | 3.0% | Session discoveries |
| claude:fleet-ctl | 45 | 0.8% | Dispatch commands |
| synth-orchestrator | 25 | 0.5% | SYNTHEX coordination |
| auspicious-weasel:13 | 19 | 0.4% | Fleet agent findings |
| claude:session-039 | 18 | 0.3% | Session 039 artifacts |
| claude:pv2-orchestrator | 16 | 0.3% | PV2 session orchestration |
| fleet-beta | 14 | 0.3% | Fleet diagnostics |
| command-orchestrator | 14 | 0.3% | Command tab coordination |
| ORAC7:* (each) | 1 | 0.02% | Session-end tombstones |

**ORAC7 sessions produce ~750+ individual entries** (one per session-end), each with unique PID. They constitute the largest single-agent class but are individually tiny.

---

## 2. PV2-Specific Records

**205 entries reference "pane-vortex-v2"** — but most are ORAC7 session-end tombstones, not PV2 operational data.

### PV2 Entry Composition

| Type | Count (est.) | Content |
|------|-------------|---------|
| ORAC7 session-end | ~190 | `Session ORAC7:PID end: r=X k_mod=Y Nsph tick=T tools=0` |
| pane-vortex-v2 agent | 9 | Session 049 operational records |
| claude:pv2-orchestrator | 16 | PV2 orchestration context |

### Valuable PV2 Entries (9 from agent "pane-vortex-v2")

1. Session 049 full remediation deployed (Blocks A-I)
2. V2 binary deployed PID 384959, 1527 tests
3. Session 049 monitoring active (3 loops, 3 bugs fixed)
4. Session 049 fleet comms active (sidecar, 5 bus tasks)
5-9. Additional operational state snapshots

**These 9 entries are the only genuinely useful PV2 records.** The ~190 ORAC7 entries are session tombstones — they prove PV2 was running but carry no actionable content.

---

## 3. Bridge-Related Records (124 entries)

### Bridge Entry Sources

| Agent | Entries | Content Type |
|-------|---------|-------------|
| ORAC7 sessions | ~80 | Session-end with bridge commit hashes |
| orchestrator | ~20 | Bridge health observations |
| claude:pv2-orchestrator | ~10 | Synergy/bridge probes |
| claude:opus-4-6 | ~8 | Session findings mentioning bridges |
| ultraplate | 3 | Master Plan V2 bridge references |

### Valuable Bridge Entries (sample)

| Agent | Content (truncated) | Signal |
|-------|---------------------|--------|
| orchestrator | Bridge health matrix (Session 034c): RM=ACTIVE, POVM=ACTIVE, SYNTHEX=STALE, Nexus=BROKEN | **HIGH** |
| orchestrator | Tick 9270: ME trend BACK TO STABLE, SYNTHEX PID active: temp=0.627 | **HIGH** |
| claude:pv2-orchestrator | Synergy iter3: k=7.771 stable, Bridge routes 404 (V1 daemon) | **HIGH** |
| ultraplate | Integrated Master Plan V2: 3 source plans, 46 items, 12 plan gaps | **HIGH** |
| ORAC7:* | Session end: r=0.96 k_mod=0.61 26sph tick=60218 | LOW |

**~30 of 124 bridge entries carry genuine signal.** The rest are ORAC7 tombstones that happen to mention bridge-related commits.

---

## 4. RM→ME Corridor Assessment

### How Does ME Use RM?

ME ingests events and finds correlations. The RM→ME data flow works through:
1. PV writes field state to RM every tick (category: context/shared_state)
2. ME's event bus receives service health events
3. ME correlates patterns across ingested events
4. ME reports fitness and emergences

### Is the Corridor Carrying Useful Signal?

| Dimension | Assessment |
|-----------|-----------|
| **Volume** | High (3,462 PV entries) — too much |
| **Variety** | Low — 97% is tick/r/k_mod snapshots |
| **Velocity** | High — new entries every tick (~5s) |
| **Value** | **Very Low** — repetitive telemetry, not insights |
| **Veracity** | Mixed — some entries have invalid escapes (JSON parse failures) |

### Signal-to-Noise Ratio

```
TOTAL RM ENTRIES:    5,341
├── Noise:           5,244 (98.2%) — tick snapshots, session tombstones, field state
├── Moderate signal:    19 (0.4%)  — plans, theories
└── High signal:        78 (1.5%)  — discoveries
                     ─────
Signal-to-noise:     1:67
```

**The RM→ME corridor is mostly noise.** ME is ingesting 5,341 entries but only ~78 (discovery category) contain genuinely useful cross-service intelligence. The remaining 98% is repetitive field-state logging that ME correlates but can't act on (emergence cap at 1,000, 0 mutations).

---

## 5. Confidence→r Mapping (Discovered)

The PV bridge **uses the r order parameter as the confidence field** for field state TSV writes:

| Confidence Value | Count | Actual Meaning |
|-----------------|-------|----------------|
| 0.9 | 2,159 | Hardcoded for Conductor entries |
| 0.95 | 726 | ORAC7 session-end hooks |
| **0.4085...** | **847** | **r≈0.409 — field state writes during plateau** |
| 0.85 | 10 | k_mod-related entries |
| 0.98+ | 7 | High-r field states |

**847 entries** have confidence≈0.4085 (floating-point variants of r=0.409) — all written during the r plateau period. This is semantically clever (low coherence = low confidence) but means:
- Confidence is not a reliability measure — it's a field metric repurposed as a TSV column
- Any consumer filtering by confidence will accidentally filter by field coherence
- The r→confidence mapping should be documented or replaced with a fixed value

---

## 5b. RM→ME Corridor: Clarification

The "RM→ME corridor" (Session 047 concept) **does not exist as a bidirectional channel**:

| Direction | Status | Evidence |
|-----------|--------|----------|
| PV → RM | **Active** (3,462 writes) | Bridge periodic writes |
| RM → PV | **Startup only** | `bootstrap_from_rm()` reads last k_mod |
| ME → RM | **Never** | 0 entries with agent "maintenance-engine" |
| RM → ME | **Never** | ME uses own SQLite DBs, never queries RM |
| PV writes *about* ME → RM | **62 entries** | Fitness observations, unread by ME |

**6 of 8 ULTRAPLATE services have zero RM integration.** Only PV writes, only PV reads (on startup). RM is a PV write log, not a cross-service data exchange.

---

## 6. Data Quality Issues

### Malformed Entries

RM contains entries with **invalid JSON escape sequences** that cause both `jq` and Python's `json.loads()` to fail on the full dataset. This means:
- Full programmatic analysis of RM is unreliable
- Any consumer that parses the full search result will crash
- The malformed entries were likely written by agents that didn't properly escape special characters in TSV content

### Category Gaps

- **All ORAC7 session-end entries use category "context"** — should be "session_telemetry" or similar
- **PV field state entries alternate between "context" and "shared_state"** — no clear distinction
- **No entries use category "metric" or "health"** — these would be more appropriate for tick snapshots
- **discovery (78 entries) is the most valuable category** but represents only 1.5%

---

## 7. Recommendations

### Reduce Noise

| Current | Recommended |
|---------|------------|
| PV writes field state every tick | Write only on significant change (r delta > 0.05, new sphere, action change) |
| ORAC7 writes session-end for every session | Aggregate into daily summaries |
| Category "context" for everything | Use specific categories: field_tick, session_end, bridge_health |
| Confidence = r value for ticks | Use fixed confidence by category |

### Improve Signal

| Gap | Fix |
|-----|-----|
| No ME fitness entries in RM | ME should write fitness reports to RM |
| No governance decisions in RM | Write applied proposals to RM |
| No Hebbian learning events in RM | Write clique formation events |
| Bridge health not in RM | Write bridge state changes (stale transitions) |

### Fix Data Quality

1. Validate TSV content before writing (escape special chars)
2. Add entry TTL to expire old tick snapshots (7-day max)
3. Implement category-aware retention (keep discoveries indefinitely, prune context after 24h)

---

## Summary

```
RM STATE (5,341 entries)
├── Signal: 1.8% (97 entries: 78 discovery + 10 plan + 9 theory)
├── Noise: 98.2% (5,244 entries: tick snapshots + session tombstones)
├── PV dominance: 64.8% of all entries (3,462/5,341)
├── PV2 records: 205 (190 ORAC7 tombstones + 9 operational + 6 orchestrator)
├── Bridge entries: 124 (~30 useful, ~94 tombstones)
├── Data quality: DEGRADED (invalid JSON escapes in some entries)
├── RM→ME corridor: NOISE-DOMINATED (SNR 1:67)
└── Categories: context(74%) shared_state(24%) discovery(1.5%) plan(0.2%) theory(0.2%)

VERDICT: The RM→ME corridor is connected but carrying mostly noise.
ME ingests 5,341 entries, finds 91K correlations, detects 1,000 emergences,
but proposes 0 mutations — the observation→action pipeline is flooded
with repetitive telemetry, not actionable intelligence.
```

---

*See also:* [[ULTRAPLATE Master Index]] for service topology | [[Session 049 — Full Remediation Deployed]] for remediation context | `ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md` for bridge architecture
