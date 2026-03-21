# Fleet ME & Governance State — Diagnostic Snapshot

> **Captured:** 2026-03-21T04:28 UTC | **ME:** 8080 | **PV:** 8132
> **Cross-refs:** [[ULTRAPLATE Master Index]] | [[Session 049 — Full Remediation Deployed]] | [SCHEMATICS_FIELD_AND_GOVERNANCE](SCHEMATICS_FIELD_AND_GOVERNANCE.md)

---

## 1. Maintenance Engine Observer (`/api/observer`)

| Metric | Value |
|--------|-------|
| System State | **Degraded** |
| Fitness | **0.6232** |
| Fitness Trend | Stable |
| Generation | 27 |
| Tick Count | 14,791 |
| Uptime | 2,121 seconds (~35 min) |
| RALPH Cycles | 7 |
| Reports Generated | 35 |

### Cumulative Metrics

| Counter | Value |
|---------|-------|
| Events Ingested | 3,679 |
| Correlations Found | 40,730 |
| Emergences Detected | **1,000** (cap hit) |
| Mutations Proposed | 0 |
| Mutations Applied | 0 |
| Mutations Rolled Back | 0 |
| Observer Errors | 0 |

**Analysis:** ME is **Degraded** at fitness 0.6232 (improved from 0.37 in Session 040). Emergences capped at 1,000. Zero mutations proposed/applied — evolutionary loop is observing but not mutating. 7 RALPH cycles, 35 reports. Zero errors — engine is internally healthy.

---

## 2. Governance Proposals (`/field/proposals`)

**Total proposals:** 16 (5 Applied, 11 Expired)

### Applied Proposals (Active)

| Parameter | From → To | Proposer | Votes | Tick |
|-----------|-----------|----------|-------|------|
| RTarget | 0.93 → **0.88** | gamma-synergy | 35 | 74,332 |
| RTarget | 0.93 → **0.85** | gamma-left-wave8 | 34 | 73,126 |
| KModBudgetMax | 1.15 → **1.25** | pioneer-1 | 20 | 69,359 |
| KModBudgetMax | 1.15 → **1.40** | gamma-left-wave8 | 34 | 73,136 |
| CouplingSteps | 15 → **20** | gamma-left-wave8 | 34 | 73,134 |

**Effective values:** RTarget=0.88 (latest applied), KModBudgetMax=1.40, CouplingSteps=20

### Expired Proposals

| Parameter | Proposed | Proposer | Votes | Reason |
|-----------|----------|----------|-------|--------|
| RTarget → 0.88 | 4:left | 2 | test governance vote pipeline |
| RTarget → 0.85 | gamma-left-wave8 | 0 | Thermal stimulation |
| RTarget → 0.88 | gamma-orchestrator | 0 | Post-unblock convergence test |
| RTarget → 0.85 | explore-test-2 | 0 | BUG-032 verification |
| RTarget → 0.75 | alpha-heat-gen | 3 | Lower for thermal activity |
| RTarget → 0.85 | gamma-left-wave8 | 5 | Thermal stimulation round 2 |
| KModBudgetMax → 1.30 | alpha-heat-gen | 3 | Widen coupling range |
| KModBudgetMax → 1.35 | gamma-left-wave8 | 1 | Widen budget ceiling |
| KModBudgetMax → 1.40 | gamma-left-wave8 | 9 | Round 2 widen ceiling |
| CouplingSteps → 20 | gamma-left-wave8 | 0 | Thermal stimulation |
| CouplingSteps → 20 | gamma-left-wave8 | 8 | Round 2 coupling iterations |

**Analysis:** Governance is active. 3 parameters modified via democratic vote. `gamma-left-wave8` drove thermal stimulation campaign (rounds 1-3). Mass votes (34-35) apply; small tests (0-3) expire correctly.

---

## 3. Field Spectrum (`/field/spectrum`)

| Harmonic | Value | Interpretation |
|----------|-------|----------------|
| L0 Monopole | **-0.374** | Net negative phase bias |
| L1 Dipole | **0.409** | Moderate bipolarity |
| L2 Quadrupole | **0.472** | Four-fold cluster structure |

**Analysis:** L2 > L1 indicates **chimera-like structure** — local clusters more coherent than global field. L0 negative means more spheres in [π, 2π] half. L1=0.41 matches POVM latest_r.

---

## 4. Field Decision Engine (`/field/decision`)

| Field | Value |
|-------|-------|
| Action | **IdleFleet** |
| Tunnel Count | **100** |
| Idle Spheres | **44** |
| Working Spheres | **1** |

**Analysis:** 44/45 spheres idle. 100 tunnels active (10.1% of possible pairs). Decision engine correctly identifies IdleFleet — no longer stuck on Stable.

---

## 5. IPC Bus State (`/bus/info`)

| Metric | Value |
|--------|-------|
| Events | **690** |
| Subscribers | **1** |
| Tasks | **5** |
| Cascades | **0** |

---

## Governance Timeline

```
Tick 68,841  explore-test-2: RTarget→0.85 (expired, 0 votes)
Tick 68,881  4:left: RTarget→0.88 (expired, 2 votes)
Tick 69,359  pioneer-1: KModBudgetMax→1.25 (APPLIED, 20 votes)
Tick 72,637  gamma-orchestrator: RTarget→0.88 (expired, 0 votes)
Tick 73,003+ gamma-left-wave8: thermal stimulation rounds 1-3
Tick 73,126  RTarget→0.85 APPLIED (34 votes)
Tick 73,134  CouplingSteps→20 APPLIED (34 votes)
Tick 73,136  KModBudgetMax→1.40 APPLIED (34 votes)
Tick 73,811  alpha-heat-gen proposals (expired, 3 votes each)
Tick 74,332  gamma-synergy: RTarget→0.88 APPLIED (35 votes)
```

## Summary

| System | Status | Key Finding |
|--------|--------|-------------|
| ME Observer | **Degraded** (0.623) | Emergence cap at 1,000, zero mutations |
| Governance | **Active** | 5 applied proposals, voting works |
| Field Spectrum | Multi-modal | Quadrupole > Dipole > Monopole |
| Decision Engine | IdleFleet | 44 idle, 1 working, 100 tunnels |
| IPC Bus | Alive | 690 events, 1 subscriber |

---

*See also:* [SCHEMATICS_FIELD_AND_GOVERNANCE](SCHEMATICS_FIELD_AND_GOVERNANCE.md) for field architecture and governance flow diagrams.
