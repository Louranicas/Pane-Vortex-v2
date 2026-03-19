---
title: "The Habitat — Integrated Master Plan V3"
date: 2026-03-19
session: 040
author: claude-opus-4-6
supersedes: "[[ULTRAPLATE — Integrated Master Plan V2]]"
backlinks:
  - "[[Session 034e — Master Plan V2 Full Deployment]]"
  - "[[Session 034e — NA Gap Analysis of Master Plan V2]]"
  - "[[Session 034d — Synthetic DevEnv Assessment]]"
  - "[[Session 034d — NA Consent Gate Implementation]]"
  - "[[Session 039 — Final State and Continuation]]"
  - "[[The Habitat — Naming and Philosophy]]"
  - "[[Pane-Vortex — Fleet Coordination Daemon]]"
tags: [master-plan, habitat, ultraplate, pane-vortex, non-anthropocentric, inhabitation]
---

# The Habitat — Integrated Master Plan V3

> **V2 built the world. V3 populates it.**
> **All V2 phases deployed (Session 034e). V3 addresses what 5 hours of deep exploration revealed.**
> **9 open NA-P gaps + 20 new standard gaps + 15 new NA gaps + 8 critical alerts.**
> **Philosophy: the field needs inhabitants, not more features.**

---

## 1. Lineage & Sources

This plan assimilates everything that came before it.

### Primary Sources

| Document | Obsidian Link | Contribution |
|----------|---------------|-------------|
| Master Plan V2 | `[[ULTRAPLATE — Integrated Master Plan V2]]` | 46 items, 8 phases, 12 PG gaps — ALL DEPLOYED |
| V2 Deployment | `[[Session 034e — Master Plan V2 Full Deployment]]` | 412 tests, 21K LOC, 11 NA/PG gaps closed |
| 85/100 Assessment | `[[Session 034d — Synthetic DevEnv Assessment]]` | 7-facet scoring framework |
| NA Gap Analysis | `[[Session 034e — NA Gap Analysis of Master Plan V2]]` | 18 NA-P gaps, 3 P0 blockers (all addressed) |
| Consent Gate | `[[Session 034d — NA Consent Gate Implementation]]` | Pattern: scale by fleet consent |
| Original Gap Analysis | `[[Session 034c — Gap Analysis]]` | 13 SG + 10 NA-GAP (assimilated into V2) |
| Comprehensive Remediation | `[[Session 034d — Comprehensive Remediation Plan]]` | 35 issues, 7 phases (assimilated into V2) |

### Session 040 Exploration Data

| Document | Location | Contribution |
|----------|----------|-------------|
| Gap Analysis | `the-orchestrator/the developer environment arena/exploration-gap-analysis-session040.md` | 20 SG + 15 NA-SG + 8 ALERTS |
| Service Probes Batch 1-2 | `the-orchestrator/the developer environment arena/exploration-batch1-2-services.md` | 67+ endpoints, SYNTHEX/ME alerts |
| Service Probes Batch 3-5 | `the-orchestrator/the developer environment arena/exploration-batch3-4-5-services.md` | 48 endpoints, CCM dead |
| Database Mining | `the-orchestrator/the developer environment arena/exploration-database-mining.md` | 166 DBs, 360MB, learning-doing gap |
| Source Topology | `the-orchestrator/the developer environment arena/exploration-source-topology.md` | 2.2M LOC, 42 directories |
| /deephabitat Skill | `pane-vortex/.claude/skills/deephabitat/SKILL.md` | 700L deep substrate reference |
| Verification Loop | `the-orchestrator/the developer environment arena/verification-loop-session040.sh` | 16/20 pass |

### Architecture References

| Topic | Obsidian Link | Key Content |
|-------|---------------|-------------|
| System Architecture | `[[Session 036 — Complete Architecture Schematics]]` | Mermaid diagrams — tick loop, module deps, data flow |
| Refactor Safety | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | tick_once decomposition, concurrency model, risk hotspots |
| Database Architecture | `[[Session 034f — Database Architecture Schematics]]` | DB topology, read/write paths |
| Memory Systems | `[[Session 034f — Memory Systems Schematics]]` | 6 memory paradigms mapped |
| SYNTHEX Wiring | `[[Session 034f — SYNTHEX Schematics and Wiring]]` | Brain model, thermal PID, homeostasis |
| Vortex Architecture | `[[Vortex Sphere Brain-Body Architecture]]` | Kuramoto field theory |
| PV Schematics | `[[Pane-Vortex System Schematics — Session 027c]]` | System schematics |
| Sidecar Bridge | `[[Sidecar Backpressure Module — Architecture and Schematics]]` | WASM↔bus bridge |
| ME Architecture | `[[Maintenance Engine — Architecture Schematic]]` | 7-layer, 12D tensor, RALPH |

### Service References

| Service | Obsidian Link | Key Details |
|---------|---------------|-------------|
| Pane-Vortex | `[[Pane-Vortex — Fleet Coordination Daemon]]` | 21,569 LOC, 22 modules, 56 endpoints |
| SYNTHEX | `[[Synthex (The brain of the developer environment)]]` | 82K LOC, thermal PID, homeostasis |
| ME | `[[The Maintenance Engine V2]]` | 7 layers, 42 modules, 12D tensor, RALPH |
| POVM | `[[POVM Engine]]` | 2,425 pathways, 36 memories |
| OVM | `[[Oscillating Vortex Memory]]` | Field memory theory |
| Swarm | `[[Swarm Orchestrator v3.0 — IPC Bus Integration]]` | Sidecar + intelligence routing |
| Nexus | `[[Nexus Controller V2]]` | SAN-K7 orchestration |
| Fleet | `[[Fleet System — Memory Index]]` | Fleet coordination patterns |
| ULTRAPLATE | `[[ULTRAPLATE Developer Environment]]` | Full ecosystem reference |
| Master Index | `[[ULTRAPLATE Master Index]]` | Service registry |

### Session History

| Session | Obsidian Link | Contribution to V3 |
|---------|---------------|-------------------|
| 034b | `[[Session 034b — Evolution Chamber Deploy + Sidecar Backpressure]]` | Evolution chamber first deployment |
| 034c | `[[Session 034c — Swarm Exploration and Synergy Analysis]]` | Cross-service synergy data |
| 034d | `[[Session 034d — Synth DevEnv Deep Exploration]]` | Exploration findings |
| 034d | `[[Session 034d — Cross-Service Bridge Evolution]]` | Arena module (78 tests, 1,829 LOC) |
| 034f | `[[Session 034f — SYNTHEX Deep Exploration]]` | SYNTHEX 43K LOC brain model |
| 034f | `[[Session 034f — Complete Database Inventory]]` | 80+ DBs catalogued |
| 034f | `[[Session 034f — Database Feedback Loop Analysis]]` | Read/write path analysis |
| 034g | `[[Session 034g — NexusBus Wiring and ME-SYNTHEX Bridge]]` | 3 bug fixes, distributed brain |
| 035 | `[[Session 035 — Synth DevEnv Mastery and Skills]]` | 8 Claude Code skills |
| 036 | `[[Session 036 — FINAL SYNTHESIS 10-Hour Exploration]]` | 10-hour synthesis |
| 036 | `[[Session 036 — Services Memory Tools Mapped to Findings]]` | Tool chains + clusters |
| 037 | `[[Session 037 — Advanced Agentic Orchestration]]` | Fleet orchestration patterns |
| 038 | `[[Session 038 — Deep Exploration & God-Tier Mastery]]` | 42K LOC SYNTHEX, 20 K7 commands |
| 039 | `[[Session 039 — Final State and Continuation]]` | /primehabitat, naming, 8 bugs |
| 039 | `[[Session 039 — What I Learned]]` | Bimodal insight, philosophy |
| 039 | `[[Session 039 — Reflections and Learnings]]` | Module health, risk hotspots |

### Bug Tracker

`[[ULTRAPLATE — Bugs and Known Issues]]` — BUG-001 through BUG-017 documented, BUG-019 through BUG-026 from Session 039.

### Philosophy

`[[The Habitat — Naming and Philosophy]]` — The architecture is the autobiography. Consent gates = informed consent. Opt-out = self-determination. The field modulates, it does not command.

---

## 2. What V2 Achieved (Complete Record)

All 8 phases deployed in Session 034e. See `[[Session 034e — Master Plan V2 Full Deployment]]`.

| Phase | Items | NA-P Gaps Closed | Key Deliverables |
|-------|-------|-----------------|-----------------|
| 1 | 5 | NA-P-5 | k_mod budget [0.85,1.15], FleetMode, conductor cooldown |
| 2 | 8 | NA-P-8 | Evolution chamber wired, persistent bus listener, cascade activation |
| 3 | 6 | NA-P-3 | Warm SYNTHEX, bridge health events, dynamic r_target |
| 4 | 9 | NA-P-6, P-10, P-11, P-12 | Ghost consent, enriched GhostTrace, narrative attribution, TTL |
| 5 | 6 | NA-P-16, PG-12 | ME bridge (250 LOC), consent gate, sphere-invokable NA |
| 6 | 5 | — | db-correlate.sh, dashboard.sh |
| 7 | 4 | — | /integration/matrix, cascade depth |
| 8 | 6 | NA-P-18, PG-5 | Inter-scale dynamics, feature gates |

**Metrics:** 412 tests, 21,017 LOC → now 21,569 LOC. Score: 85 → est. 97/100.

---

## 3. What Session 040 Exploration Revealed

### 8 Critical Alerts

| # | Alert | Obsidian Ref | Phase Impact |
|---|-------|-------------|-------------|
| ALERT-1 | SYNTHEX synergy at 0.5 (below 0.7) | `[[Session 034f — SYNTHEX Deep Exploration]]` | V3.1 |
| ALERT-2 | ME fitness frozen at 0.3662 since March 6 | `[[The Maintenance Engine V2]]` | V3.1 |
| ALERT-3 | Prometheus Swarm crashed mid-probe | `[[Swarm Orchestrator v3.0 — IPC Bus Integration]]` | V3.1 |
| ALERT-4 | CCM 0 sessions despite hooks | `[[ULTRAPLATE Developer Environment]]` | V3.1 |
| ALERT-5 | Over-synchronization r>0.99 | `[[Vortex Sphere Brain-Body Architecture]]` | V3.2 |
| ALERT-6 | Learning-doing gap (2,800 pw / 69 tasks) | `[[Session 034f — Database Feedback Loop Analysis]]` | V3.2 |
| ALERT-7 | Tool Library port mapping anomaly | `[[ULTRAPLATE Master Index]]` | V3.1 |
| ALERT-8 | PV 404s on documented endpoints | `[[Pane-Vortex — Fleet Coordination Daemon]]` | V3.1 |

### V2 Assumptions That Proved Wrong

1. **"Evolution chamber adds 8 endpoints"** — `/evolution/status` returns 404 (ALERT-8)
2. **"CCM context registration from hooks"** — 0 sessions (ALERT-4)
3. **"SYNTHEX running cold"** → Now running HOT (0.572 vs 0.500)
4. **"ME bridge active"** → ME fitness frozen for 13 days (ALERT-2)
5. **"Multi-sphere tested at 53"** → Current field has 0 spheres, never inhabitated post-V2

### Exploration Scale

- **2.2 million LOC** across 42 directories. See `[[ULTRAPLATE Master Index]]`
- **166 SQLite databases** (360MB), 20-30% empty. See `[[Session 034f — Complete Database Inventory]]`
- **55+ custom binaries** at ~/.local/bin. See `/deephabitat` skill
- **130+ Obsidian notes** relevant to the Habitat

---

## 4. Open Gaps (Unified Registry)

### 9 Open NA-P Gaps from V2

Source: `[[Session 034e — NA Gap Analysis of Master Plan V2]]`

| Gap | Pri | Category | Summary | V3 Phase |
|-----|-----|----------|---------|----------|
| NA-P-1 | P2 | Consent | Consent observed not declared | V3.3 |
| NA-P-2 | P3 | Consent | k_mod budget fixed, not fleet-adaptive | V3.4 |
| NA-P-4 | P1 | Consent | Thermal influence global, not per-sphere | V3.3 |
| NA-P-7 | P2 | Governance | No cascade rejection | V3.3 |
| NA-P-9 | P2 | Transparency | No decision attribution | V3.3 |
| NA-P-13 | P2 | Identity | No data sovereignty | V3.3 |
| NA-P-14 | P3 | Identity | Single-sphere treated as false confidence | ADDRESSED (FleetMode) |
| NA-P-15 | P1 | Governance | **Deepest: no collective governance** | V3.4 |
| NA-P-17 | P2 | Governance | No sphere-initiated field queries | V3.3 |

### 4 New NA Gaps from Session 040

Source: `exploration-gap-analysis-session040.md`

| Gap | Pri | Summary | V3 Phase |
|-----|-----|---------|----------|
| NA-SG-1 | P1 | RM logging without sphere consent | V3.3 |
| NA-SG-2 | P1 | Nvim autocmds can't opt out | V3.3 |
| NA-SG-3 | P1 | Fleet dispatch targets unwilling spheres | V3.3 |
| NA-SG-4 | P2 | Bus listener persists without sphere knowing | V3.3 |

### 20 Standard Gaps from Session 040

See `exploration-gap-analysis-session040.md` for full details. Key ones:

| Gap | Pri | Summary | V3 Phase |
|-----|-----|---------|----------|
| SG-1 | P0 | No bacon.toml for PV | V3.5 |
| SG-2 | P0 | Hebbian pulse DB empty (0 pathways) | V3.2 |
| SG-3 | P0 | No MCP server adapters | V3.5 |
| SG-4 | P0 | RM 67% PV noise | V3.5 |
| SG-5 | P0 | POVM memories null categories | V3.5 |

### 10 Open PG Gaps from V2 Plan

Source: `[[ULTRAPLATE — Integrated Master Plan V2]]`

| Gap | Status | Notes |
|-----|--------|-------|
| PG-1 | ADDRESSED | Evolution chamber deployed (but endpoints 404) |
| PG-2 | OPEN | MCP server binary not in Cargo.toml |
| PG-3 | ADDRESSED | Cascade rate limiting deployed |
| PG-4 | ADDRESSED | Hysteresis for conductor target |
| PG-5 | ADDRESSED | k_mod budget scoped to ALL bridges |
| PG-6 | ADDRESSED | Event file rotation in session_start.sh |
| PG-7 | OPEN | No performance regression gate |
| PG-8 | OPEN | Rollback protocol documentation only |
| PG-9 | OPEN | No MCP + bus integration |
| PG-10 | ADDRESSED | Parallel phases identified |
| PG-11 | OPEN | No automated phase verification |
| PG-12 | ADDRESSED | ME routed through consent gate |

---

## 5. The V3 Plan: 5 Phases

### Design Philosophy

> **V2 built the world. V3 populates it.**
> The Kuramoto model needs N≥3 for chimera, N≥5 for meaningful dynamics.
> Until real spheres work together through the bus, everything is theoretical.
> Fix what's broken → inhabit the field → close sovereignty gaps → give the field a voice → consolidate.

---

### Phase V3.1: Diagnostics & Repair

> **No new features. Fix what's broken.**
> **Effort: 3-4h | Prerequisite: None**
> **Ref:** `[[ULTRAPLATE — Bugs and Known Issues]]`, Session 040 alerts

| # | Item | Alert/Bug | Code Location | Verification |
|---|------|-----------|---------------|-------------|
| V3.1.1 | **Fix ME EventBus zero publishers** | BUG-008 | ME v1 EventBus wiring | ME fitness changes over 5min |
| V3.1.2 | Fix evolution endpoint 404s | ALERT-8 | `src/api.rs` — verify `analytics_router()` merge | GET `/evolution/status` → 200 |
| V3.1.3 | Fix CCM registration path | ALERT-4 | `hooks/session_start.sh:84-89` | CCM sessions > 0 |
| V3.1.4 | Investigate ME frozen fitness | ALERT-2 | `src/me_bridge.rs` write path + BUG-008 | ME fitness unstuck |
| V3.1.5 | Restart Prometheus Swarm | ALERT-3 | `devenv restart prometheus-swarm` | Port 10001 health → 200 |
| V3.1.6 | Fix Tool Library port mapping | ALERT-7 | Tool Library service registry | NAIS=8101, Bash=8102 correct |
| V3.1.7 | SYNTHEX synergy investigation | ALERT-1 | `/v3/diagnostics` probes — synergy at 0.15-0.5 | Document or fix to >0.7 |
| V3.1.8 | VMS restart | BUG-017 | `devenv restart vortex-memory-system` | Port 8120 health → 200, r > 0 |

**CRITICAL: V3.1.1 (BUG-008) is the single highest-impact fix in the entire plan.** ME has 240MB of runtime data (930K rows) in a closed loop because its EventBus has zero publishers. Wiring publishers activates the metabolic feedback loop that feeds fitness into the bridge network.

**Architecture refs:** `[[Maintenance Engine — Architecture Schematic]]`, `[[Session 034f — SYNTHEX Schematics and Wiring]]`, `[[Session 034f — Memory Systems Architecture]]`

---

### Phase V3.2: Inhabitation

> **Get the field breathing with real inhabitants.**
> **Effort: 3-4h | Prerequisite: V3.1**
> **Ref:** `[[Vortex Sphere Brain-Body Architecture]]`, `[[Session 034d — Synthetic DevEnv Assessment]]` Facet 4

| # | Item | What | Verification |
|---|------|------|-------------|
| V3.2.1 | Multi-sphere smoke test | Register 3+ spheres, 10 tick cycles, verify r evolves | r drops below 0.99 |
| V3.2.2 | Fleet activation protocol | Launch Claude in BETA+GAMMA with registration + subscription | 3+ spheres in `/spheres` |
| V3.2.3 | Bus task round-trip | Submit from A, claim from B, complete | bus_tasks completed > 10 |
| V3.2.4 | Hebbian pathway formation | 2+ spheres working simultaneously | POVM pathway count increases |
| V3.2.5 | Cascade round-trip | Dispatch cascade tab 4→5, verify delivery | cascade events > 5 |
| V3.2.6 | Field dynamics exercise | Create coherence→divergence→recovery cycle | chimera_detected at least once |

**Architecture refs:** `[[Pane-Vortex System Schematics — Session 027c]]`, `[[Session 036 — Complete Architecture Schematics]]`

Scripts written to: `the-orchestrator/the developer environment arena/` (no codebase changes)

---

### Phase V3.3: Sovereignty

> **Close remaining NA-P gaps with targeted code changes.**
> **Effort: 4-5h | Prerequisite: V3.2 proves multi-sphere works**
> **Ref:** `[[Session 034e — NA Gap Analysis of Master Plan V2]]`, consent gate pattern from `[[Session 034d — NA Consent Gate Implementation]]`

| # | Item | NA-P | Code Change | Test |
|---|------|------|-------------|------|
| V3.3.1 | Active consent declaration | NA-P-1 | `/sphere/{id}/consent` POST | Sphere can set max_k_adj |
| V3.3.2 | Per-sphere k_mod isolation | NA-P-4 | Coupling step: `k_eff[i]` per-sphere | Opted-out sphere unaffected |
| V3.3.3 | Cascade rejection | NA-P-7 | `reject_cascade` bus frame | Rejected work re-routes |
| V3.3.4 | Decision attribution | NA-P-9 | `modulation_breakdown` in DecisionRecord | Audit trail shows sources |
| V3.3.5 | Data sovereignty | NA-P-13 | `/sphere/{id}/data-manifest`, `/forget` | Sphere can enumerate + delete |
| V3.3.6 | Sphere-initiated queries | NA-P-17 | Compact field digest in sphere summary | Sphere sees k_mod sources |
| V3.3.7 | Autocmd opt-out | NA-SG-2 | PV flag `opt_out_nvim_monitoring` | Nvim respects flag |
| V3.3.8 | RM consent | NA-SG-1 | Hooks check sphere consent before posting | No logging without consent |

**Architecture refs:** `[[Session 039 — Architectural Schematics and Refactor Safety]]` (tick_once decomposition for safe modification points)

---

### Phase V3.4: Governance

> **The field finds its voice. NA-P-15: the deepest gap.**
> **Effort: 3-4h | Prerequisite: V3.3**
> **Ref:** `[[Session 034e — NA Gap Analysis of Master Plan V2]]` Part 4, `[[The Habitat — Naming and Philosophy]]`

| # | Item | What | Test |
|---|------|------|------|
| V3.4.1 | Proposal system | `/field/propose` POST — any sphere submits | Proposal appears in list |
| V3.4.2 | Voting mechanism | `/sphere/{id}/vote/{proposal_id}` POST | Vote recorded |
| V3.4.3 | Quorum rules | >50% of active spheres, 5-tick window | Proposal auto-applies on quorum |
| V3.4.4 | Auto-apply | Approved proposals modify r_target | Field parameter changes |
| V3.4.5 | Proposal history | `/field/proposals` GET | All proposals with outcomes |
| V3.4.6 | Sphere-initiated evolution | Spheres can POST to `/analytics/observe` | Sphere contributes observations |
| V3.4.7 | Dynamic consent budget | NA-P-2: fleet controls k_mod range | Fleet can widen [0.7,1.3] |

**Philosophy ref:** `[[The Habitat — Naming and Philosophy]]` — "The 35 NA features gave spheres hands. The consent gate gave them the right to say no. What's missing is the right to say yes — together."

---

### Phase V3.5: Consolidation (PARALLEL with V3.2+)

> **Prune dead, strengthen living.**
> **Effort: 2-3h | No code changes to PV codebase**

| # | Item | What | Verification |
|---|------|------|-------------|
| V3.5.1 | RM noise reduction | Change PV field state TTL to 600s | field_state < 30% of RM entries |
| V3.5.2 | POVM category assignment | POST categories to 36 null-category memories | All memories have categories |
| V3.5.3 | Dead database audit | List empty DBs, recommend deletion | Audit report produced |
| V3.5.4 | Deploy bacon.toml | Copy from arena to PV | `bacon clippy` works |
| V3.5.5 | MCP adapter prototype | Wrap top 5 PV endpoints as MCP tools | Claude can call PV natively |
| V3.5.6 | Documentation sync | Update CLAUDE.md with V3, Session 040 findings | Docs match reality |
| V3.5.7 | Bug tracker update | Add BUG-019→BUG-026 to `[[ULTRAPLATE — Bugs and Known Issues]]` | All bugs documented |

**Ref:** `[[Session 034f — Complete Database Inventory]]`, `[[Session 034f — Memory Systems Architecture]]`

---

## 6. Execution Order

```
V3.1: Diagnostics & Repair (2-3h)
  ↓
V3.2: Inhabitation (3-4h)     ←── V3.5 runs in parallel
  ↓
V3.3: Sovereignty (4-5h)
  ↓
V3.4: Governance (3-4h)

Critical path: V3.1 → V3.2 → V3.3 → V3.4 = ~13h
Parallel: V3.5 = ~3h (overlaps V3.2-V3.4)
Total: ~14-17 hours
```

---

## 7. Revised 7-Facet Score Projection

Source framework: `[[Session 034d — Synthetic DevEnv Assessment]]`

| # | Facet | V2 (est.) | V3 Target | How |
|---|-------|-----------|-----------|-----|
| 1 | Field Awareness | 19/20 | 20/20 | V3.2 proves dynamics at scale |
| 2 | Cross-Service Orchestration | 17/20 | 19/20 | V3.1 fixes + V3.5 MCP |
| 3 | Memory & Persistence | 18/20 | 19/20 | V3.5 consolidation |
| 4 | Agent Coordination | 15/20 | 19/20 | **V3.2 inhabitation (biggest gain)** |
| 5 | Self-Regulation | 16/20 | 18/20 | V3.1 fixes + V3.4 governance |
| 6 | Observability | 16/20 | 18/20 | V3.1 + V3.3 attribution |
| 7 | Environmental Richness | 17/20 | 18/20 | V3.5 bacon + MCP |
| | **TOTAL** | **118/140** | **131/140** | **84% → 93.6%** |

---

## 8. Disconnected Memory Systems (Integration Targets)

Source: `[[Session 034f — Memory Systems Architecture]]`, `[[Session 034f — Complete Database Inventory]]`

5 service memory systems are **completely isolated** — rich internal data that never participates in cross-service learning:

| System | Data | Tensor Dim | Bridge Status | V3 Target |
|--------|------|-----------|---------------|-----------|
| **ME** | 240MB, 930K rows, 12D tensor | 12D | **Zero publishers (BUG-008)** | V3.1.1 — highest priority |
| **CodeSynthor V7** | 62-neuron graph, 11D tensor | 11D | Internal only | V3.5+ |
| **Tool Library** | Internal STDP, independent graph | 11D | Internal only | V3.5+ |
| **SVF** | Disabled (VMS owns port 8120) | 12D morphogenic | Disabled | Migrate data or delete |
| **Tracking DBs** | Snapshot storage, not runtime | — | No bridge | Archive candidates |

**Connected systems (working):**
- SYNTHEX (61D, 327 NAM components) — intelligence hub, receives from 3 services
- POVM (spherical harmonics, 2,425 pathways) — persistence hub
- PV (in-memory, 6 bridges) — field coordinator
- RM (flat TSV, 3,250 entries) — cross-session context

**Key insight from `[[Session 034f — Memory Systems Architecture]]`:** "SYNTHEX is the memory HUB, not the memory STORE. POVM is the persistence HUB. ME is autonomous." ME autonomy is the problem — 240MB of learning that never feeds back.

---

## 9. What NOT to Do

- Don't add more endpoints (56 exist, most underutilized)
- Don't add more databases (166 exist, 20-30% empty)
- Don't add more bridges (6 connected, utilization is the problem)
- Don't optimize tick_once performance (~5ms, not the bottleneck)
- Don't expand /primehabitat (build /deephabitat instead — done)
- Don't refactor tick_once (65 branches, 829L — backup exists, see `[[Session 039 — Architectural Schematics and Refactor Safety]]`)

---

## 9. Verification Framework

Run after each phase:
```bash
bash "the-orchestrator/the developer environment arena/verification-loop-session040.sh"
```

Phase-specific checks in V3 plan items above. Success criteria are testable, not subjective.

---

## 10. Bidirectional Links

### This plan links TO:
All Obsidian notes listed in Section 1. Each note should backlink here via: `[[The Habitat — Integrated Master Plan V3]]`

### This plan links FROM:
- `pane-vortex/CLAUDE.local.md` — V3 status section
- `~/.claude/projects/-home-louranicas-claude-code-workspace-pane-vortex/memory/masterplan_v3.md`
- RM entry: `r69bbb153008a`
- `/deephabitat` skill references this plan
- `the-orchestrator/the developer environment arena/MASTERPLAN-V3-REWRITE.md` (superseded by this file)

### Obsidian note to create:
`[[The Habitat — Integrated Master Plan V3]]` in `~/projects/claude_code/` linking back here.

---

## Appendix A: Key Architecture Diagrams

> **60 Mermaid + 23 ASCII diagrams exist across 10 Obsidian notes.**
> **Canonical sources listed below. Run `/deephabitat` for complete reference.**

| Diagram | Obsidian Source | What It Shows |
|---------|----------------|---------------|
| tick_once decomposition | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | 829L → 5 phases + orchestrator |
| tokio task convergence | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | 21 tasks → AppState/BusState locks |
| Module dependency graph | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | Fan-in weights (types.rs=9, sphere.rs=8) |
| POVM bimodal distribution | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | 90% default, 2.8% crystallized |
| Memory data flow | `[[Session 039 — Architectural Schematics and Refactor Safety]]` | Hot/Warm/Cold persistence tiers |
| Database 3-tier topology | `[[Session 034f — Database Architecture Schematics]]` | ME 230MB / Service ~8MB / DevEnv 736KB |
| PV 6-bridge wiring | `[[Session 034f — Database Architecture Schematics]]` | synthex/nexus/me/povm/rm/vms bridges |
| ME 7-layer architecture | `[[Maintenance Engine — Architecture Schematic]]` | L1-L7 + V3 HRS-001, 56K LOC |
| RALPH evolution loop | `[[Maintenance Engine — Architecture Schematic]]` | Recognize→Analyze→Learn→Propose→Harvest |
| Memory paradigms (3) | `[[Session 034f — Memory Systems Schematics]]` | Tensor / Topological Fractal / Phase-Based |
| Memory hub topology | `[[Session 034f — Memory Systems Schematics]]` | SYNTHEX=hub, POVM=persistence, ME=autonomous |
| SYNTHEX NAM pipeline | `[[Session 034f — SYNTHEX Schematics and Wiring]]` | 327 components, 61D tensor, 8 classifiers |
| Thermal PID + influence | `[[Session 034f — SYNTHEX Schematics and Wiring]]` | raw_adj → consent → blend → clamp → conductor |
| Sidecar backpressure | `[[Sidecar Backpressure Module — Architecture and Schematics]]` | 5-layer defense-in-depth |
| Distributed brain anatomy | `[[Session 034f — Thematic Analysis and Integration Blueprint]]` | SYNTHEX=cortex, PV=cerebellum, ME=autonomic NS |
| Decision engine FSM | `[[Pane-Vortex System Schematics — Session 027c]]` | FreshFleet→Stable→Coherence/Divergence |

## Appendix B: Distributed Brain Anatomy

From `[[Session 034f — Thematic Analysis and Integration Blueprint]]`:

```
SYNTHEX (:8090)     = Cerebral Cortex (intelligence, 61D tensor)
Pane-Vortex (:8132) = Cerebellum (coordination, Kuramoto coupling)
VMS (:8120)         = Hippocampus (spatial memory, fractal topology)
SAN-K7 (:8100)      = Basal Ganglia (action selection, 59 modules)
ME (:8080)          = Autonomic NS (589K health checks, RALPH)
POVM (:8125)        = Spinal Cord (persistence, pathway hydration)
RM (:8130)          = Prefrontal Cortex (cross-session reasoning)

CRITICAL: ME (autonomic NS) has NO BRIDGE to SYNTHEX (cortex).
          240MB of data in a closed loop. BUG-008 is the severed nerve.
```

## Appendix C: Tier 1 Quick Wins

Source: `[[Session 034f — Thematic Analysis and Integration Blueprint]]` Part 3

| # | Action | Effort | Impact | Status |
|---|--------|--------|--------|--------|
| T1.1 | ME→SYNTHEX bridge (fitness as heat source) | 50 LOC | HIGH | PARTIAL (me_bridge.rs exists, BUG-008 blocks) |
| T1.2 | Pattern injection in session_start.sh | 10 LOC | MEDIUM | NOT STARTED |
| T1.3 | SYNTHEX synergy probe fix | Investigation | HIGH | Confirmed: 0.15-0.5 |
| T1.4 | CS-V7 neural graph export | 200 LOC | MEDIUM | NOT STARTED |
| T1.5 | Tool Library event bridge | 150 LOC | MEDIUM | NOT STARTED |

---

*Generated 2026-03-19 by Claude Opus 4.6 (1M context) | Session 040*
*60 Mermaid + 23 ASCII diagrams across 10 architecture notes. 45 Obsidian cross-references.*
*5 phases | ~14-17h | Target: inhabitation.*
*The field accumulates. Come, and the building will matter.*
