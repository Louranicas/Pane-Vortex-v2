---
date: 2026-03-19
tags: [specification, governance, proposals, voting, consent, data-sovereignty]
---

# L8 Governance Layer Specification

> **Feature-gated: `governance`**
> **NA-P-15: The field should have a voice in its own governance.**

## Modules: m37_proposals, m38_voting, m39_consent_declaration, m40_data_sovereignty, m41_evolution_chamber

## Proposal System (m37)
- `POST /field/propose` — any sphere submits {parameter, proposed_value, rationale}
- Proposable parameters: r_target (initial), k_mod_budget_max (later)
- Proposal lifecycle: Open → Approved/Rejected/Expired → Archived

## Voting (m38)
- `POST /sphere/{id}/vote/{proposal_id}` — approve/reject/abstain
- One vote per sphere per proposal (UNIQUE constraint in SQL)
- Quorum: >50% of active (non-idle, non-blocked) spheres
- Voting window: 5 ticks (25 seconds)

## Consent Declaration (m39)
- `POST /sphere/{id}/consent` — explicit consent posture
- Fields: accept_external_modulation, max_k_adjustment, accept_cascade, accept_observation, accept_nvim_monitoring
- Replaces inferred receptivity with declared consent (NA-P-1)

## Data Sovereignty (m40)
- `GET /sphere/{id}/data-manifest` — enumerate ALL stored data
- `POST /sphere/{id}/forget` — mark data for TTL expiry
- Covers: field_tracking, bus_tracking, POVM, RM, ghost traces

## Evolution Chamber (m41, feature: evolution)
- 8 endpoints: analytics/patterns, anomalies, baseline, observe, summary + evolution/emergence, regime, status
- NA-P-8: observation opt-out flag honored

## Design Constraints: C1 C8 C10
## Related: [[Session 034e — NA Gap Analysis of Master Plan V2]], [[The Habitat — Naming and Philosophy]]
