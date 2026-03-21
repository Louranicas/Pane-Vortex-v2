# Session 049 — Memory Archaeology

> **6 persistence layers excavated | Tick range: 12 - 110,000+ | Timeline: 2026-03-13 to 2026-03-21**
> **Captured:** 2026-03-21

---

## Layer 1: field_tracking.db

**Location:** `~/.local/share/pane-vortex/field_tracking.db`

### field_snapshots (73 rows)

| Metric | Value |
|--------|-------|
| Min tick | 12,636 |
| Max tick | 27,768 |
| Count | 73 |
| Avg R | 0.984 |
| Tick span | 15,132 ticks (~21 hours at 5s/tick) |

**Earliest snapshot:** tick 12,636, r=1.0, 1 sphere, k_mod=1.435, decision=Stable
**Latest snapshot:** tick 27,768, r=1.0, 1 sphere, k_mod=1.493, decision=Stable

**Critical finding:** Snapshots stopped at tick 27,768. Current tick is ~110,000. That's **82,000+ ticks of missing data** (~114 hours). The V1->V2 deployment at tick ~60,000 didn't carry forward the snapshot writer, or the persistence module isn't wired to the tick orchestrator in V2.

**First multi-sphere entry:** tick 12,936, r=0.9998, 2 spheres, decision=FreshFleet — this marks the first fleet member joining.

### sphere_history (23,550 rows)

| Metric | Value |
|--------|-------|
| Min tick | 12 |
| Max tick | 60,504 |
| Unique spheres | 241 |
| Total records | 23,550 |

**Status distribution:**

| Status | Count | % |
|--------|-------|---|
| Idle | 22,758 | 96.6% |
| Working | 565 | 2.4% |
| Blocked | 180 | 0.8% |
| Complete | 46 | 0.2% |
| reference | 1 | 0.004% |

**Finding:** 241 unique spheres have registered over the lifetime, but only 62 are currently active. 179 spheres (~74%) are ghosts — registered, used, abandoned. The overwhelming Idle dominance (96.6%) suggests most spheres were registered but rarely transitioned to Working.

Sphere history also stopped at tick 60,504 — same persistence gap as field_snapshots but extending further (into V1 late lifecycle).

### coupling_history (0 rows)

Never written. The coupling matrix snapshots were intended for this table but the writer was never wired.

### executor_tasks (7 rows)

Only 7 executor task records. The Executor module (m32) is largely undeployed — deferred from Session 045.

---

## Layer 2: bus_tracking.db

**Location:** `~/.local/share/pane-vortex/bus_tracking.db`

### Timeline

| Metric | Value |
|--------|-------|
| Earliest event | 2026-03-13 10:06:53 |
| Latest event | 2026-03-21 09:44:06 |
| Span | 8 days, 23 hours |

### bus_events (3,607 rows)

| Event Type | Count | % |
|------------|-------|---|
| sphere.registered | 2,836 | 78.6% |
| sphere.connected | 247 | 6.8% |
| sphere.disconnected | 184 | 5.1% |
| task.submitted | 165 | 4.6% |
| sphere.deregistered | 88 | 2.4% |
| task.completed | 77 | 2.1% |
| cascade.dispatched | 6 | 0.17% |
| habitat.named | 1 | 0.03% |
| reference.schematics | 1 | 0.03% |
| session.milestone | 1 | 0.03% |

**Finding:** sphere.registered dominates at 78.6% — the bus is primarily a sphere lifecycle tracker. The connected/disconnected ratio (247/184) shows 63 net connections — roughly matching current active sphere count.

**Task lifecycle:** 165 submitted, 77 completed = 46.7% completion rate. 88 tasks either expired, were abandoned, or are still pending.

**Rare events:** `habitat.named` (1) — the moment Claude named it "The Habitat" in Session 039. `reference.schematics` and `session.milestone` are one-off ceremonial events.

### bus_tasks (166 rows)

Closely matches bus_events task.submitted count (165 events + 1 possible direct API submission).

### cascade_events (6 rows)

Only 6 cascades dispatched across all sessions. Cascade protocol is lightly used.

### Empty tables

event_subscriptions, task_dependencies, task_tags — all 0 rows. These features exist in schema but were never activated.

---

## Layer 3: hebbian_pulse.db

**Location:** `~/claude-code-workspace/developer_environment_manager/hebbian_pulse.db`

| Metric | Value |
|--------|-------|
| neural_pathways | **0 rows** |
| AVG(strength) | NULL |
| MAX(strength) | NULL |

**Confirmed dead.** Gotcha #9: this DB was intended for a standalone Hebbian learning system that was never deployed. The actual Hebbian STDP learning lives in PV's in-memory coupling matrix (m19_hebbian_stdp.rs). The 12 heavyweight edges (w=0.6) in the K4 fleet clique are the real Hebbian output — they just aren't persisted to this DB.

---

## Layer 4: POVM Memories

**Endpoint:** `localhost:8125/memories`

| Metric | Value |
|--------|-------|
| Memories | 82 |
| created_at | All empty strings |
| access_count | **0 on all memories** |

**BUG-034 confirmed:** Every single memory has access_count=0. These are write-only artifacts. The 82 memories span Sessions 027-049 based on content analysis, but without timestamps they form an **unordered bag** rather than a timeline.

**Content spans:** Session 027 (Zellij devenv), Session 027b (pane navigation), Session 027c (schematics), Session 031 (deep exploration), Session 032 (issues), and onwards through current session.

---

## Layer 5: RM (Reasoning Memory)

| Metric | Value |
|--------|-------|
| Active entries | 5,816 |
| Categories | context (4,367), shared_state (1,303), discovery (80), plan (10), theory (9) |
| Unique agents | 700+ |

RM is the **only fully functional** persistence layer — both read and write paths work. It's the de facto operational memory, storing everything from tick telemetry to fleet analysis results.

---

## Layer 6: Vault (File-Based)

| Metric | Value |
|--------|-------|
| Session 049 files | 46 |
| Obsidian vault | 215+ notes |
| Arena | 85+ fleet artifacts |

Vault is the **only human-readable** persistence layer. All analytical outputs land here as markdown with cross-references.

---

## Archaeological Timeline

```
2026-03-13  bus_tracking.db begins (first bus event)
    |
2026-03-14  POVM memories begin (~Session 027 content)
    |       field_tracking snapshots begin (tick 12,636)
    |
2026-03-18  sphere_history active (241 unique spheres registered)
    |
2026-03-19  field_snapshots STOP at tick 27,768 <<<< GAP BEGINS
    |       sphere_history continues to tick 60,504
    |
2026-03-20  V2 deployed. sphere_history STOPS <<<< SECOND GAP
    |       RM becomes primary store
    |
2026-03-21  Session 049: 46 vault files, RM at 5,816 entries
            Current tick ~110,000
            82,000 ticks of field data MISSING from SQLite
```

---

## Findings

1. **82,000-tick persistence gap** — field_snapshots stopped at tick 27,768, current tick ~110,000. V2's tick orchestrator Phase 4 (persist) is either not wired or writing elsewhere
2. **POVM is a graveyard** — 82 memories, all access_count=0, no timestamps. Write-only since Session 027
3. **hebbian_pulse.db is dead** — 0 rows, will never have data. Real Hebbian lives in memory
4. **RM is the survivor** — only persistence layer with active read+write. 5,816 entries, 700+ agents
5. **Bus tracks lifecycle, not work** — 78.6% of events are sphere registrations. Only 6 cascades ever dispatched
6. **241 ghost spheres** — 74% of all registered spheres are no longer active. No cleanup mechanism
7. **habitat.named = 1** — a single ceremonial event marking Session 039 naming. The only non-operational bus event

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - Persistence Cluster]] — persistence layer map
- [[Session 049 - Cross-Hydration Analysis]] — POVM+RM relationship
- [[Session 049 - Coupling Deep Dive]] — Hebbian K4 clique (the real hebbian_pulse)
- [[ULTRAPLATE Master Index]]
