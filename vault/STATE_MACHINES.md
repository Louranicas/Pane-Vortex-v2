---
title: "Pane-Vortex V2 — State Machines"
date: 2026-03-19
tags: [fsm, state-machines, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Pane-Vortex System Schematics — Session 027c]]"
---

# Pane-Vortex V2 — State Machines

> Formal FSM definitions for all lifecycle and decision state machines in PV V2.
> See [SCHEMATICS.md](SCHEMATICS.md) for Mermaid visual representations.

---

## 1. Sphere Lifecycle FSM

**Module:** m11_sphere, m15_app_state
**States:** 6 status states + 3 maturity states

### Status States

| State | Description | Transitions To |
|-------|-------------|---------------|
| `Registered` | Initial state after POST /register | `Idle` (immediate) |
| `Idle` | Sphere exists but not working | `Working`, `Deregistered` |
| `Working` | Sphere actively processing a task | `Idle`, `Complete`, `Blocked`, `Deregistered` |
| `Complete` | Sphere finished a task | `Idle`, `Working`, `Deregistered` |
| `Blocked` | Sphere waiting on external dependency | `Working`, `Idle`, `Deregistered` |
| `Deregistered` | Sphere removed from field | `Ghost` -> `Registered` (re-registration) |

### Status Transition Rules

```
Registered -> Idle         : always (automatic on creation)
Idle       -> Working      : POST /sphere/{id}/status {status: "working"}
Working    -> Idle         : POST /sphere/{id}/status {status: "idle"}
Working    -> Complete     : POST /sphere/{id}/status {status: "complete"}
Working    -> Blocked      : POST /sphere/{id}/status {status: "blocked"}
Blocked    -> Working      : POST /sphere/{id}/status {status: "working"}
Blocked    -> Idle         : POST /sphere/{id}/status {status: "idle"}
Complete   -> Idle         : POST /sphere/{id}/status {status: "idle"}
Complete   -> Working      : POST /sphere/{id}/status {status: "working"}
Any        -> Deregistered : POST /deregister {id}
Deregistered -> Ghost      : automatic (GhostTrace created)
Ghost      -> Registered   : POST /register {id} (weight inheritance)
```

### Maturity States

Maturity is a derived property based on `step_count`:

| Maturity | Condition | Effect |
|----------|-----------|--------|
| `Newcomer` | step_count < 50 | 2x Hebbian LTP boost |
| `Established` | 50 <= step_count < 200 | Normal LTP rate |
| `Senior` | step_count >= 200 | Normal LTP rate, trusted for governance |

### Ghost Trace

When a sphere deregisters:
1. A `GhostTrace` is created with: id, persona, Hebbian weights, top memories, departure timestamp, consent status
2. Ghost is pushed to a `VecDeque<GhostTrace>` (max 20 entries, FIFO eviction)
3. On re-registration with matching ID: weights are inherited from the ghost (consent-gated, NA-P-11)

---

## 2. Bus Task Lifecycle FSM

**Module:** m30_bus_types
**States:** 5

| State | Description | Transitions To |
|-------|-------------|---------------|
| `Submitted` | Task created, waiting for a claimer | `Claimed`, `Expired` |
| `Claimed` | Task assigned to a sphere | `Completed`, `Failed`, `Expired` |
| `Completed` | Task finished successfully | (terminal) |
| `Failed` | Task execution failed | (terminal) |
| `Expired` | Task TTL exceeded | (terminal) |

### Transition Rules

```
Submitted -> Claimed    : sphere sends Claim frame with task_id
Submitted -> Expired    : TTL elapsed (default 3600s)
Claimed   -> Completed  : sphere sends Complete frame with result
Claimed   -> Failed     : sphere sends Fail frame with error
Claimed   -> Expired    : TTL elapsed after claim
```

### Task Routing Types

| Target Type | Routing Strategy |
|-------------|-----------------|
| `Specific(PaneId)` | Route to named sphere only |
| `AnyIdle` | Route to first idle sphere |
| `FieldDriven` | Route based on field state (suggestions) |
| `Willing` | Route to sphere with matching consent + capacity |

### Constraints

- A task can only be claimed once (first-come-first-served)
- Claiming a non-Submitted task returns `TaskAlreadyClaimed` error
- Completed/Failed/Expired are terminal — no further transitions
- Task tags and dependencies are set at creation and immutable
- Task dependencies: a task cannot be claimed until all depends_on tasks are Completed

---

## 3. Proposal Lifecycle FSM (V3.4, Governance Feature)

**Module:** m37_proposals, m38_voting
**Feature gate:** `governance`
**States:** 4

| State | Description | Transitions To |
|-------|-------------|---------------|
| `Open` | Accepting votes, within voting window | `Approved`, `Rejected`, `Expired` |
| `Approved` | Quorum reached, majority approve | (terminal, auto-apply) |
| `Rejected` | Quorum reached, majority reject or abstain | (terminal) |
| `Expired` | Voting window elapsed without quorum | (terminal) |

### Transition Rules

```
Open     -> Approved  : votes_for / (votes_for + votes_against + votes_abstain) > 0.5
                        AND total_votes / active_spheres > quorum_threshold (0.5)
Open     -> Rejected  : quorum reached AND votes_against >= votes_for
Open     -> Expired   : current_tick > voting_deadline_tick
```

### Auto-Apply

When a proposal transitions to `Approved`:
1. The targeted parameter is updated in the running config
2. A `governance.proposal.applied` bus event is broadcast
3. The proposal record is updated with `resolved_at` timestamp
4. If the parameter is `r_target`, the conductor immediately uses the new value

### Proposable Parameters

| Parameter | Current Default | Range | Notes |
|-----------|----------------|-------|-------|
| `r_target` | 0.93 | [0.3, 0.99] | Dynamic target for order parameter |
| `k_mod_budget_max` | 1.15 | [1.0, 2.0] | Upper bound on external influence |
| `k_mod_budget_min` | 0.85 | [0.5, 1.0] | Lower bound on external influence |
| `coupling_steps_per_tick` | 15 | [5, 50] | Coupling integration granularity |

### Constraints

- Max 10 active (Open) proposals at a time
- Each sphere can vote once per proposal (UNIQUE constraint on proposal_id + sphere_id)
- Voting window: 5 ticks (25 seconds at default tick interval)
- Only registered spheres can propose or vote
- Proposals cannot be amended after creation — submit a new one

---

## 4. Cascade Lifecycle FSM

**Module:** m33_cascade
**States:** 3

| State | Description | Transitions To |
|-------|-------------|---------------|
| `Dispatched` | Cascade handoff sent to target | `Acked`, `Rejected` |
| `Acked` | Target acknowledged and accepted | (terminal) |
| `Rejected` | Target rejected the cascade | (terminal, re-route) |

### Transition Rules

```
Dispatched -> Acked    : target sends CascadeAck frame with status=acked
Dispatched -> Rejected : target sends CascadeAck frame with status=rejected
                         OR target has accept_cascade=false in consent
```

### Cascade Handoff Process

1. Source sphere sends `CascadeHandoff` frame: `{ source, target, brief, depth }`
2. Bus validates rate limit (max 10 cascades per minute per source)
3. Bus checks target consent: `consent_declarations.accept_cascade`
4. If consented: forward to target sphere's connection
5. If not consented: immediately return `Rejected`
6. Target responds with `CascadeAck` (accept or reject)
7. If rejected: bus can re-route to next eligible sphere

### Fallback Brief

If no bus connection is available for the target:
1. Write a markdown brief to `~/projects/shared-context/tasks/cascade-{uuid}.md`
2. Set file permissions to 0600
3. Record cascade event in persistence layer

### Depth Tracking

- Each cascade carries a `depth` counter starting at 0
- When a cascaded task triggers a sub-cascade, depth increments
- Max depth is not currently enforced (documented in V1 gap analysis as a future constraint)

---

## 5. Conductor Decision FSM

**Module:** m31_conductor
**States:** 7 (one per decision action)

### Decision Priority Chain

The conductor evaluates conditions in strict priority order. The first matching condition produces the decision:

```
1. HasBlockedAgents    : blocked_count > 0
2. NeedsCoherence      : r > r_low (0.3) AND r_trend < r_falling (-0.03) AND sphere_count >= 2
3. NeedsDivergence     : r > r_high (0.8) AND idle_ratio > 0.6 AND sphere_count >= 2
4. IdleFleet           : all spheres idle (idle_count == sphere_count AND sphere_count > 0)
5. FreshFleet          : sphere_count > 0 AND no sphere has has_worked == true
6. Recovering          : warmup_remaining > 0
7. Stable              : default (none of the above)
```

### State Actions

| Decision | K Adjustment | Side Effects |
|----------|-------------|-------------|
| `HasBlockedAgents` | None (emergency steer instead) | EmergencyCoherence message to blocked spheres |
| `NeedsCoherence` | +gain * (r_target - r) | PI controller increases coupling |
| `NeedsDivergence` | -gain * (r - r_target) | PI controller decreases coupling |
| `IdleFleet` | None | No action, log only |
| `FreshFleet` | None | No action, log only |
| `Recovering` | None | Suppress all decisions during warmup |
| `Stable` | Breathing blend | Minor oscillation from emergent breathing |

### PI Controller

The proportional controller adjusts K:
```
k_adj = gain * (r_target - r_current)
```
Where `gain = 0.15` and `r_target = 0.93` (dynamic, fleet-negotiable via governance).

### Divergence Cooldown

After a NeedsDivergence decision:
- Coherence decisions are suppressed for `divergence_cooldown_ticks = 3`
- This prevents oscillation between coherence and divergence

### Per-Status K Modulation (from NA-22)

Working pairs get 1.2x coupling. Mixed pairs get 0.5x. Blocked pairs get 0.0x:

| Sphere A Status | Sphere B Status | K Multiplier |
|----------------|----------------|-------------|
| Working | Working | 1.2 |
| Working | Idle | 0.5 |
| Working | Blocked | 0.0 |
| Idle | Idle | 0.3 |
| Blocked | Blocked | 0.0 |

---

## State Invariants

These invariants must hold across all FSMs:

1. **Phase in [0, TAU)** — after every phase update, wrap with `.rem_euclid(TAU)`
2. **Frequency in [0.001, 10.0]** — clamp on every frequency change
3. **Sphere count <= SPHERE_CAP (200)** — reject registration above cap
4. **Ghost count <= 20** — FIFO eviction of oldest ghosts
5. **Message log is VecDeque with cap** — never unbounded Vec
6. **Lock ordering: AppState before BusState** — prevents deadlock
7. **k_mod in [K_MOD_BUDGET_MIN, K_MOD_BUDGET_MAX]** — clamped after all adjustments
8. **Only terminal states are immutable** — Completed, Failed, Expired, Approved, Rejected cannot transition

---

## Cross-References

- **[SCHEMATICS.md](SCHEMATICS.md)** — Mermaid diagrams of these FSMs
- **[MESSAGE_FLOWS.md](MESSAGE_FLOWS.md)** — Sequence diagrams showing state transitions in context
- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Module-level type definitions
- **[ERROR_TAXONOMY.md](ERROR_TAXONOMY.md)** — Errors triggered by invalid transitions
- **[config/default.toml](../config/default.toml)** — Threshold values referenced above
- **Obsidian:** `[[Pane-Vortex System Schematics — Session 027c]]` (V1 FSM diagrams)
