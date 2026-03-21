# Fleet System Summary — ULTRAPLATE Habitat State

**Generated:** 2026-03-21 | **PV Tick:** ~81,600 | **Field r:** 0.285–0.454 (cooling) | **Spheres:** 42–45
**Synthesised from:** 8 fleet probe reports + live endpoint data

Cross-refs: [[ULTRAPLATE Master Index]] | [[Session 049 — Full Remediation Deployed]] | [[The Habitat — Naming and Philosophy]]

---

## System Overview

| Dimension | Status | Detail |
|-----------|--------|--------|
| **Services** | 16/16 healthy | All ports responding <1ms |
| **PV Field** | Active, cooling | r=0.285–0.454, 42–45 spheres, tick ~81K |
| **SYNTHEX Brain** | CRITICAL | T=0.03 (target 0.50), synergy=0.5 (critical <0.7) |
| **ME Evolution** | Degraded | fitness=0.612–0.623, emergence cap saturated at 1000/1000 |
| **POVM Memory** | Write-only | 58 memories, 2,427 pathways, 0 access, 0 crystallised |
| **Governance** | Active | 16 proposals, 5 applied, voting works |
| **IPC Bus** | Functional | 805 events, 1 subscriber, 5 tasks |
| **Bridges** | 5/6 fresh | POVM bridge intermittently stale |
| **VMS** | Dormant | r=0.0, 0 memories, zone=Incoherent |
| **Nvim** | OK | v801, socket alive |
| **Zellij** | 6 tabs | Command, 2 Workspaces, 3 Fleet |

---

## Service Health Matrix

| Port | Service | Health | Key Metric |
|------|---------|--------|-----------|
| 8080 | Maintenance Engine | 200 | fitness=0.612, Degraded, emergence cap hit |
| 8081 | DevOps Engine | 200 | 40 agents, 8 tiers |
| 8090 | SYNTHEX | 200 | T=0.03, synergy=0.5 CRITICAL |
| 8100 | SAN-K7 Orchestrator | 200 | r_outer=0.991, 20 commands, Aligned |
| 8101 | NAIS | 200 | — |
| 8102 | Bash Engine | 200 | 3 POST endpoints, safety patterns |
| 8103 | Tool Maker | 200 | v1.55.0 |
| 8104 | Context Manager | 200 | 0 sessions (unused) |
| 8105 | Tool Library | 200 | 65 tools, port mapping anomaly |
| 8110 | CodeSynthor V7 | 200 | PBFT n=60/f=19/q=39 |
| 8120 | Vortex Memory System | 200 | Dormant, r=0.0 |
| 8125 | POVM Engine | 200 | 58 mem, 2,427 pathways, write-only |
| 8130 | Reasoning Memory | 200 | ~4,857 entries (67% PV) |
| 8132 | Pane-Vortex | 200 | r=0.285–0.454, 42–45 spheres |
| 9001 | Architect Agent | 200 | 67 patterns loaded |
| 10001 | Prometheus Swarm | 200 | Previously crashed mid-probe (Session 040) |

---

## Bridge Topology (6 bridges, 3,322 LOC)

| Bridge | Target | Direction | k_adjustment | Status |
|--------|--------|-----------|-------------|--------|
| SYNTHEX | :8090 | Bidirectional | ~1.19 (cold boost) | Fresh |
| Nexus | :8100 | Bidirectional | 1.10 (Aligned) | Fresh |
| ME | :8080 | Read-only | 1.00 (degraded band) | Fresh |
| RM | :8130 | Bidirectional | N/A (TSV write) | Fresh |
| POVM | :8125 | Bidirectional | N/A (persistence) | Stale |
| VMS | :8120 | Write-heavy | N/A (memory sync) | Fresh |

**Combined bridge effect:** ~1.31x coupling boost (SYNTHEX 1.19 x Nexus 1.10 x ME 1.00)

---

## Critical Issues (Priority Order)

### RED — Requires Action

| # | Issue | Source | Impact |
|---|-------|--------|--------|
| 1 | **SYNTHEX synergy 0.5** (critical <0.7) | Fleet-SYNTHEX-Thermal | Brain of devenv degraded, circular cold loop |
| 2 | **SYNTHEX temperature 0.03** (target 0.50) | Fleet-SYNTHEX-Thermal | 3/4 heat sources dead (Hebbian, Cascade, Resonance) |
| 3 | **ME emergence cap saturated** (1000/1000) | Fleet-ME-Emergence | Zero mutations, evolution stalled, fitness stuck 0.612 |
| 4 | **PID controller sign anomaly** | Fleet-SYNTHEX-Thermal | PID output negative (-0.335) despite T << target |

### YELLOW — Degraded but Functional

| # | Issue | Source | Impact |
|---|-------|--------|--------|
| 5 | **POVM write-only** (BUG-034) | Fleet-POVM-Health | 58 memories, 0 access, 0 crystallisation |
| 6 | **Coupling matrix empty** | Fleet-Bridge-Topology | 0 live Hebbian edges vs 2,427 POVM historical |
| 7 | **POVM bridge intermittently stale** | Fleet-Bridge-Topology | Timing artefact, not connectivity |
| 8 | **CCM completely unused** | Session 040 alerts | 0 sessions, 0 actions |

### GREEN — Healthy

| System | Detail |
|--------|--------|
| SAN-K7 | r_outer=0.991, 20 commands, Aligned strategy |
| Governance | 16 proposals, democratic voting works, 3 parameters mutated |
| IPC Bus | 805 events, task lifecycle functional |
| RM | ~4,857 entries, cross-session context active |
| Service mesh | 16/16 healthy, <1ms response |

---

## Governance State

### Active Field Parameters (mutated via proposals)

| Parameter | Default | Current | Proposer | Votes |
|-----------|---------|---------|----------|-------|
| RTarget | 0.93 | **0.88** | gamma-synergy | 35 |
| KModBudgetMax | 1.15 | **1.40** | gamma-left-wave8 | 34 |
| CouplingSteps | 15 | **20** | gamma-left-wave8 | 34 |

**Governance history:** 16 proposals from 6 unique proposers. gamma-left-wave8 drove systematic thermal stimulation campaign (rounds 1-3). Mass votes (34-35) apply; weak proposals (0-3 votes) expire correctly.

---

## Field Dynamics

### Spectrum Analysis

| Harmonic | Value | Meaning |
|----------|-------|---------|
| L0 monopole | -0.275 | Net phase deficit (skewed low) |
| L1 dipole | 0.284 | Moderate two-cluster asymmetry |
| L2 quadrupole | 0.400 | Strong four-fold structure (dominant) |

L2 > L1 > |L0| — field has quadrupolar structure with ~4 phase basins. Consistent with cooling field.

### Decision Engine

| Property | Value |
|----------|-------|
| Action | IdleFleet |
| Tunnel count | 100 |
| Idle spheres | 44 |
| Working spheres | 1–6 (varies) |
| Coherence pressure | 0.0 |
| Divergence pressure | 0.0 |

---

## Memory Systems State

| System | Entries | Health | Key Issue |
|--------|---------|--------|-----------|
| **RM** | ~4,857 | Healthy | 67% from pane-vortex agent (noisy) |
| **POVM** | 58 mem / 2,427 pw | Write-only | BUG-034: 0 access, 0 crystallised |
| **PV field_tracking.db** | 23,543 sphere records | Active | 200+ unique sphere IDs |
| **PV bus_tracking.db** | 7 tables | Active | 690–805 events |
| **VMS** | 0 memories | Dormant | r=0.0, zone=Incoherent |
| **Auto-Memory** | MEMORY.md index | Active | 40+ session memories |

---

## Fleet Reports Index

| Report | Focus | Key Finding |
|--------|-------|-------------|
| [Fleet-SYNTHEX-Thermal](Fleet-SYNTHEX-Thermal.md) | SYNTHEX thermal + diagnostics | T=0.03 (6% of target), synergy CRITICAL |
| [Fleet-ME-Emergence](Fleet-ME-Emergence.md) | ME observer + emergence | BUG-035: emergence cap deadlock at 1000 |
| [Fleet-ME-Governance](Fleet-ME-Governance.md) | ME + governance + field | 5 applied proposals, IdleFleet, quadrupolar spectrum |
| [Fleet-ME-Service-Health](Fleet-ME-Service-Health.md) | ME deep dive + 16-service sweep | 16/16 healthy, ME fitness plateau 0.612 |
| [Fleet-Bridge-Topology](Fleet-Bridge-Topology.md) | All 6 bridges + coupling | Combined 1.31x boost, POVM stale, Mermaid diagram |
| [Fleet-POVM-Health](Fleet-POVM-Health.md) | POVM memories + pathways | Write-only, bimodal weights, 2 unbounded >1.0 |
| [Fleet-POVM-Deep-Dive](Fleet-POVM-Deep-Dive.md) | POVM diagnostics + hydration | BUG-034 persists, +3 new memories, 0 crystallisation |
| [Fleet-Governance-Field](Fleet-Governance-Field.md) | Governance proposals + field | 16 proposals, thermal campaign, r cooling |
| **Fleet-System-Summary** | **This file** | Consolidated system state |

---

## Recommended Actions (from fleet reports)

### V3.1 Diagnostics (Immediate)
1. Investigate SYNTHEX PID sign error (negative output with T << target)
2. Raise ME emergence_cap from 1000 to 5000 (Block C, Session 048)
3. Wire Hebbian activity to SYNTHEX HS-001 heat source
4. Audit POVM pathways with weight > 1.0 (unbounded LTP)

### V3.2 Inhabitation
5. Wire PV field resonance to SYNTHEX HS-003
6. Implement POVM read-back loop (break write-only pathology)
7. Trigger POVM consolidation to get crystallised_count > 0
8. Generate cascade events to activate SYNTHEX HS-002

### V3.5 Consolidation
9. Create MCP adapters for PV, K7, RM, POVM
10. Create bacon.toml for PV quality gate
11. Prune RM noise (67% from PV field state logging)
12. Audit 20–30% empty databases across ecosystem

---

## Verification Commands

```bash
# Quick health (30ms)
PV=$(curl -s localhost:8132/health | jq -c '{r,spheres,tick}')
SX=$(curl -s localhost:8090/v3/thermal | jq -c '{temperature,target,pid_output}')
ME=$(curl -s localhost:8080/api/observer | jq -c '{system_state,current_fitness:.last_report.current_fitness}')
POVM=$(curl -s localhost:8125/hydrate | jq -c '{m:.memory_count,p:.pathway_count,c:.crystallised_count}')
echo "PV=$PV SX=$SX ME=$ME POVM=$POVM"

# Full 16-service sweep
declare -A hp=([8080]="/api/health" [8090]="/api/health")
for p in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  path="${hp[$p]:-/health}"
  echo "$p:$(curl -s -o /dev/null -w '%{http_code}' localhost:$p$path)"
done

# Bridge health
curl -s localhost:8132/bridges/health | jq .

# Governance proposals
curl -s localhost:8132/field/proposals | jq '[.[] | select(.status == "Applied")] | length'
```

---

*Synthesised from 8 fleet probe reports. See [[ULTRAPLATE Master Index]] for complete service registry and port map.*
