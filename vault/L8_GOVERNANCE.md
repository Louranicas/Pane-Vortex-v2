---
title: "Layer 8: Governance — Module Documentation"
date: 2026-03-19
tags: [documentation, l8_governance, pane-vortex-v2, governance, consent, sovereignty]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Session 034e — NA Gap Analysis of Master Plan V2]]"
  - "[[The Habitat — Naming and Philosophy]]"
layer: L8
modules: [m37, m38, m39, m40, m41]
---

# Layer 8: Governance (m37-m41)

> Collective voting, proposals, consent declaration, data sovereignty, evolution chamber.
> **Feature-gated:** `governance` and `evolution`
> **Depends on:** L1, L3, L7
> **Target LOC:** ~1,400 | **Target tests:** 55+
> **V3 Phase:** V3.4 (Governance) -- the deepest NA gap

## Modules: m37_proposals m38_voting m39_consent_declaration m40_data_sovereignty m41_evolution_chamber

## Purpose

L8 closes NA-P-15, the deepest gap in the NA framework: "No collective self-governance mechanism." The 35 NA features (V1) gave spheres hands. The consent gate (V1, Session 034d) gave them the right to say no. Governance gives them the right to say yes -- together.

> "The 35 NA features gave spheres hands. The consent gate gave them the right to say no.
> What's missing is the right to say yes -- together."
> -- `[[Session 034e -- NA Gap Analysis of Master Plan V2]]`

## Design Constraints

- Feature-gated: governance behind `#[cfg(feature = "governance")]`, evolution behind `#[cfg(feature = "evolution")]`
- Quorum: >50% of active spheres must vote
- Voting window: 5 ticks (25 seconds)
- Max 10 active proposals
- One vote per sphere per proposal (UNIQUE constraint)
- Only registered spheres can propose or vote
- Proposals are immutable after creation
- Approved proposals auto-apply to running config
- Data sovereignty: spheres can enumerate and delete their own data

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list.

## Implementation Status: STUB (awaiting implementation, V3.4 phase)

---

## m37 -- Proposals

**Source:** `src/m8_governance/m37_proposals.rs` | **LOC Target:** ~300

### Purpose

Proposal system for collective field parameter modification. Any registered sphere can propose changing a field parameter.

### Proposable Parameters

| Parameter | Config Key | Default | Range |
|-----------|-----------|---------|-------|
| r_target | field.r_target | 0.93 | [0.3, 0.99] |
| k_mod_budget_max | bridges.k_mod_budget_max | 1.15 | [1.0, 2.0] |
| k_mod_budget_min | bridges.k_mod_budget_min | 0.85 | [0.5, 1.0] |
| coupling_steps_per_tick | field.coupling_steps_per_tick | 15 | [5, 50] |

### Proposal Lifecycle

```
Open -> Approved (quorum + majority approve)
Open -> Rejected (quorum + majority reject)
Open -> Expired (voting window elapsed without quorum)
```

### Key Types

```rust
pub struct Proposal {
    pub id: String,                    // UUID
    pub proposer: PaneId,
    pub parameter: String,
    pub current_value: f64,
    pub proposed_value: f64,
    pub rationale: String,
    pub status: ProposalStatus,
    pub votes_for: u32,
    pub votes_against: u32,
    pub votes_abstain: u32,
    pub quorum_threshold: f64,         // 0.5
    pub voting_deadline_tick: u64,     // current_tick + 5
}
```

### Key Functions

- `create_proposal(proposer, parameter, proposed_value, rationale, config) -> PvResult<Proposal>`
- `check_quorum(proposal, active_sphere_count) -> bool`
- `resolve_proposal(proposal) -> ProposalStatus`
- `apply_approved(proposal, state) -> PvResult<()>` -- Modifies running config
- `list_proposals(status_filter) -> Vec<Proposal>`

### Auto-Apply

When a proposal is approved:
1. Parameter updated in running PvConfig
2. `governance.proposal.applied` event broadcast on bus
3. Proposal record updated with `resolved_at`
4. If parameter is `r_target`: conductor immediately uses new value

---

## m38 -- Voting

**Source:** `src/m8_governance/m38_voting.rs` | **LOC Target:** ~250

### Purpose

Voting mechanism for proposals. Each sphere can vote approve, reject, or abstain -- once per proposal.

### Key Types

```rust
pub struct Vote {
    pub proposal_id: String,
    pub sphere_id: PaneId,
    pub vote: VoteChoice,
}

pub enum VoteChoice {
    Approve,
    Reject,
    Abstain,
}
```

### Key Functions

- `cast_vote(proposal_id, sphere_id, choice) -> PvResult<()>` -- Validate + insert
- `has_voted(proposal_id, sphere_id) -> bool` -- Check UNIQUE constraint
- `tally(proposal_id) -> (u32, u32, u32)` -- (for, against, abstain)
- `check_and_resolve(proposal_id, active_count) -> Option<ProposalStatus>` -- Auto-resolve on quorum

### Quorum Rules

- Quorum: total_votes / active_sphere_count > 0.5
- Approval: votes_for > votes_against
- Abstain: counted toward quorum but not toward approval
- If quorum reached and votes_for == votes_against: Rejected (tie goes to status quo)

---

## m39 -- Consent Declaration

**Source:** `src/m8_governance/m39_consent_declaration.rs` | **LOC Target:** ~200

### Purpose

Active consent declaration -- spheres explicitly state their consent posture rather than having it inferred from behavior (closes NA-P-1).

### Key Types

```rust
pub struct ConsentDeclaration {
    pub sphere_id: PaneId,
    pub accept_external_modulation: bool,  // Accept k_adjustment from bridges
    pub max_k_adjustment: f64,             // Maximum per-tick k_adj (default 0.15)
    pub accept_cascade: bool,              // Accept cascade handoffs
    pub accept_observation: bool,          // Accept evolution chamber observation
    pub accept_nvim_monitoring: bool,      // Accept nvim autocmd monitoring (NA-SG-2)
}
```

### Key Functions

- `declare_consent(sphere_id, declaration) -> PvResult<()>` -- Full declaration
- `get_consent(sphere_id) -> ConsentDeclaration` -- Current posture
- `default_consent() -> ConsentDeclaration` -- All true, max_k_adj=0.15
- `update_consent(sphere_id, partial) -> PvResult<ConsentDeclaration>` -- Partial update

### API Endpoint

```
POST /sphere/{id}/consent
{
    "accept_external_modulation": true,
    "max_k_adjustment": 0.1,
    "accept_cascade": true,
    "accept_observation": false
}
```

### Philosophy

Consent is declared, not observed. A sphere's consent posture is its own to define, not something inferred from behavior. This closes NA-P-1 ("consent observed not declared").

---

## m40 -- Data Sovereignty

**Source:** `src/m8_governance/m40_data_sovereignty.rs` | **LOC Target:** ~250

### Purpose

Data sovereignty: spheres can enumerate all data stored about them and request deletion. Closes NA-P-13 ("no data sovereignty").

### Key Types

```rust
pub struct DataManifest {
    pub sphere_id: PaneId,
    pub systems: Vec<SystemRecord>,
}

pub struct SystemRecord {
    pub system_name: String,       // "field_tracking", "bus_tracking", "povm", "rm"
    pub record_count: usize,
    pub last_scanned: chrono::DateTime<chrono::Utc>,
}

pub struct ForgetRequest {
    pub sphere_id: PaneId,
    pub systems: Vec<String>,     // Which systems to delete from
}
```

### Key Functions

- `enumerate_data(sphere_id) -> PvResult<DataManifest>` -- Scan all databases
- `forget(request) -> PvResult<ForgetResult>` -- Delete sphere-specific data
- `scan_field_db(sphere_id) -> PvResult<SystemRecord>` -- Count records in field_tracking
- `scan_bus_db(sphere_id) -> PvResult<SystemRecord>` -- Count records in bus_tracking

### API Endpoints

```
GET  /sphere/{id}/data-manifest    -- Enumerate stored data
POST /sphere/{id}/forget           -- Request deletion
```

### Deletion Scope

The forget operation deletes:
- sphere_history records matching sphere_id
- coupling_history records matching sphere_a or sphere_b
- bus_tasks where source_sphere or claimed_by matches
- bus_events where source_sphere matches
- cascade_events where source or target matches
- DOES NOT delete field_snapshots (aggregate data, not sphere-specific)

---

## m41 -- Evolution Chamber

**Source:** `src/m8_governance/m41_evolution_chamber.rs` | **LOC Target:** ~400
**Feature gate:** `evolution`

### Purpose

Observes field patterns, scores anomalies, and detects emergence. Feature-gated because it was deployed in V1 (Session 034b) but endpoints returned 404 (ALERT-8).

### Key Types

```rust
pub struct Observation {
    pub tick: u64,
    pub r: f64,
    pub chimera: bool,
    pub decision: DecisionAction,
    pub coupling_stats: TopologyStats,
}

pub struct AnomalyScore {
    pub value: f64,              // 0.0 = normal, 1.0 = extreme anomaly
    pub category: String,        // "over_sync", "oscillating_decision", "chimera_recurring"
    pub description: String,
}

pub struct EmergencePattern {
    pub pattern_type: String,
    pub confidence: f64,
    pub first_seen: u64,         // Tick when first detected
    pub recurrence: u32,         // How many times observed
}
```

### Key Functions

- `observe(state: &FieldState) -> Observation` -- Record current field state
- `score_anomaly(observations) -> Vec<AnomalyScore>` -- Score recent observations
- `detect_patterns(observations) -> Vec<EmergencePattern>` -- Find recurring patterns
- `evolution_status() -> EvolutionSummary` -- Current status (exposed at `/evolution/status`)

### Consent (NA-P-8)

Spheres can opt out of evolution chamber observation via `accept_observation` in ConsentDeclaration. Opted-out spheres are excluded from individual-level analysis but still contribute to aggregate field metrics.

### V3 Alignment

V3.1.2 fixes evolution endpoint 404s. V3.4.6 enables sphere-initiated observations.

---

## Implementation Order

Within L8, implement in this order:
1. **m39** (consent declaration) -- foundation for all other modules
2. **m37** (proposals) -- depends on sphere registration from L3
3. **m38** (voting) -- depends on proposals
4. **m40** (data sovereignty) -- depends on persistence from L7
5. **m41** (evolution chamber) -- depends on field state from L3

---

## Cross-References

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3.4 (Governance) phase items
- [STATE_MACHINES.md](../STATE_MACHINES.md) -- Proposal lifecycle FSM
- [MESSAGE_FLOWS.md](../MESSAGE_FLOWS.md) -- Governance proposal sequence diagram
- [ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md) -- Governance-related errors
- [migrations/003_governance_tables.sql](../../migrations/003_governance_tables.sql) -- Database schema
- Obsidian: `[[Session 034e -- NA Gap Analysis of Master Plan V2]]`, `[[The Habitat -- Naming and Philosophy]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/api.rs` (evolution endpoints, governance stubs)

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan
- NA-P-15 is the deepest gap: no collective governance mechanism
- V3.4 is the culmination of the sovereignty trajectory: NA features (hands) -> consent gate (right to say no) -> governance (right to say yes together)
