# Fleet Governance & Field State — Live Diagnostics

**Generated:** 2026-03-21T04:29Z | **Session:** 050 | **Field tick:** 81,623

Cross-refs: [[ULTRAPLATE Master Index]] | [[Session 049 — Full Remediation Deployed]] | [[SCHEMATICS_FIELD_AND_GOVERNANCE]]

---

## 1. Governance Proposals (`/field/proposals`)

**Total proposals:** 16 | **Applied:** 4 | **Expired:** 12

### Applied Proposals (Active Field Parameters)

| Parameter | Current → Proposed | Proposer | Votes | Tick |
|-----------|--------------------|----------|-------|------|
| RTarget | 0.93 → **0.88** | gamma-synergy | 35 | 74,332 |
| RTarget | 0.93 → **0.85** | gamma-left-wave8 | 34 | 73,126 |
| KModBudgetMax | 1.15 → **1.25** | pioneer-1 | 20 | 69,359 |
| KModBudgetMax | 1.15 → **1.40** | gamma-left-wave8 | 34 | 73,136 |
| CouplingSteps | 15 → **20** | gamma-left-wave8 | 34 | 73,134 |

**Note:** RTarget has two applied proposals (0.88 and 0.85). The later proposal (tick 74,332 for 0.88) appears to have overridden the earlier 0.85. Similarly, KModBudgetMax was widened twice: 1.15→1.25→1.40.

### Expired Proposals (Did Not Reach Quorum)

| Parameter | Proposed | Proposer | Votes | Reason |
|-----------|----------|----------|-------|--------|
| RTarget | 0.88 | 4:left | 2 | Test governance pipeline |
| RTarget | 0.85 | gamma-left-wave8 | 0 | Thermal stimulation |
| RTarget | 0.88 | gamma-orchestrator | 0 | Post-unblock convergence |
| RTarget | 0.85 | explore-test-2 | 0 | BUG-032 verification |
| RTarget | 0.75 | alpha-heat-gen | 3 | Natural divergence |
| RTarget | 0.85 | gamma-left-wave8 | 5 | Thermal stimulation round 2 |
| KModBudgetMax | 1.30 | alpha-heat-gen | 3 | Widen k_mod budget |
| KModBudgetMax | 1.35 | gamma-left-wave8 | 1 | Thermal stimulation |
| KModBudgetMax | 1.25 | pioneer-1 | — | (actually applied) |
| KModBudgetMax | 1.40 | gamma-left-wave8 | 9 | Round 2 |
| CouplingSteps | 20 | gamma-left-wave8 | 0 | First attempt |
| CouplingSteps | 20 | gamma-left-wave8 | 8 | Round 2 |

### Governance Patterns Observed

1. **Thermal stimulation campaign:** gamma-left-wave8 drove 8 of 16 proposals, systematically targeting RTarget, KModBudgetMax, and CouplingSteps
2. **Mass vote mechanism:** Round 3 proposals (tick ~73,130) achieved 34 votes each — suggests coordinated fleet voting
3. **Iterative escalation:** KModBudgetMax went 1.15→1.25→1.35→1.40 across rounds
4. **Exploration probes:** pioneer-1, explore-test-2, 4:left all submitted test proposals with minimal votes
5. **Current effective parameters:** RTarget=0.88, KModBudgetMax=1.40, CouplingSteps=20

---

## 2. Field Spectrum (`/field/spectrum`)

```
L0 monopole:    -0.275 (negative = net phase deficit)
L1 dipole:       0.284 (moderate asymmetry)
L2 quadrupole:   0.400 (strong quadrupolar structure)
```

### Harmonic Analysis

| Mode | Value | Interpretation |
|------|-------|----------------|
| **L0 (monopole)** | -0.275 | Negative monopole — phases are net-negative relative to π. Field slightly skewed toward low-phase region (0→π). |
| **L1 (dipole)** | 0.284 | Moderate dipole — two distinct phase clusters exist with asymmetric distribution. |
| **L2 (quadrupole)** | 0.400 | Strong quadrupolar — four-fold symmetry in phase space. Spheres clustered in ~4 phase groups. |

**Interpretation:** L2 > L1 > |L0| indicates the field has more quadrupolar structure than dipolar. This is typical of a cooling field with multiple phase basins — spheres settle into ~4 distinct clusters rather than a single synchronized state. The negative L0 confirms the field is phase-cold (consistent with SX T=0.03).

---

## 3. Field Decision (`/field/decision`)

| Property | Value |
|----------|-------|
| **Action** | IdleFleet |
| **r** | 0.285 |
| **r_trend** | Stable |
| **Tunnel count** | 100 |
| **Fleet mode** | Full |
| **Idle spheres** | 44 |
| **Working spheres** | 6 |
| **Blocked spheres** | 0 |
| **Coherence pressure** | 0.0 |
| **Divergence pressure** | 0.0 |

### Working Spheres (6)

| Sphere | Type |
|--------|------|
| fleet-beta-1 | Fleet Claude instance |
| fleet-beta-2 | Fleet Claude instance |
| fleet-alpha | Fleet Claude instance |
| fleet-gamma-1 | Fleet Claude instance |
| fleet-gamma-2 | Fleet Claude instance |
| orchestrator-044 | Session orchestrator |

### Routing

- **Focused pool:** 50 spheres (all spheres route to focused — no exploratory pool)
- **Exploratory pool:** 0 spheres

### Strongest Tunnel

```
Sphere A:  ORAC7:372707  (semantic: secondary)
Sphere B:  4:left         (semantic: secondary)
Overlap:   1.0 (perfect)
```

**Analysis:** r dropped from 0.409 (earlier pulse) to 0.285 (now). Field is cooling — consistent with 44/45 spheres idle at pulse time, now 6 working. Zero coherence/divergence pressure means the decision engine is in equilibrium. All routing goes to "focused" — no exploratory assignments happening.

---

## 4. IPC Bus State (`/bus/info`)

| Metric | Value |
|--------|-------|
| Tasks | 5 |
| Events | 805 |
| Subscribers | 1 |
| Cascades | 0 |

**Note:** Bus task count dropped from 53 (initial probe) to 5. Either tasks expired or were cleaned. Event buffer at 805 (was 1000). Single subscriber active. No cascades pending.

---

## Governance Health Assessment

| Dimension | Status | Detail |
|-----------|--------|--------|
| Proposal submission | **Healthy** | 16 proposals from 6 unique proposers |
| Voting mechanism | **Healthy** | Mass votes (34) achieved quorum |
| Parameter mutation | **Active** | 4 proposals applied, 3 parameters changed |
| Quorum threshold | **Effective** | 12 expired = system rejects weak proposals |
| Campaign coordination | **Observed** | gamma-left-wave8 drove systematic thermal campaign |
| Field spectrum | **Quadrupolar** | L2=0.40 dominant, 4 phase clusters |
| Decision routing | **Idle** | All focused, no exploratory, zero pressure |
| Bus health | **Functional** | 1 subscriber, 5 tasks, 805 events |

---

*See also:* [[SCHEMATICS_FIELD_AND_GOVERNANCE]] for governance flow diagrams | [[Session 049 — Full Remediation Deployed]] for prior governance fixes | [[ULTRAPLATE Master Index]] for service topology | `ai_docs/SCHEMATICS_FIELD_AND_GOVERNANCE.md` for Mermaid diagrams
