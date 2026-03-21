# Consent Flow Analysis

> Discovered Session 044 by Claude-Main instance analyzing M28 + M37 source files

## Summary

M28 ([[m28_consent_gate]]) and M37 ([[m37_proposals]]) operate on **separate consent planes** with no wiring between them.

- **M28 ConsentGate (L6):** Guards EXTERNAL bridge adjustments (SYNTHEX, Nexus, ME) using per-sphere SphereConsent records
- **M37 Proposals (L8):** Guards INTERNAL parameter changes via democratic voting (quorum 50% + simple majority)

## What Gets Consent-Gated

### ConsentGate (M28)
- SYNTHEX thermal k_adjustment (every 6 ticks via M22)
- Nexus strategy coupling multiplier (every 60 ticks via M23)
- ME health-derived adjustment (via M24)
- Combined multiplicative product via `apply_combined()`
- Per-sphere overrides (V3.3 placeholder)

### ProposalManager (M37)
- `RTarget`: target order parameter [0.5, 0.99]
- `KModBudgetMax`: maximum external influence budget [1.0, 1.5]
- `CouplingSteps`: coupling iterations per tick [1, 50]

## 7 Gaps Identified

| Gap | Description | Severity | Fix Phase |
|-----|-------------|----------|-----------|
| GAP-1 | No actuator — `approved_unapplied()` has results but nothing executes | Critical | V3.4 |
| GAP-2 | Proposal→Gate feedback missing — can't widen K_MOD_BUDGET dynamically | High | V3.4 |
| GAP-3 | POVM/RM/VMS bridges bypass consent entirely | High | V3.3 |
| GAP-4 | `divergence_requested` flag never checked in `apply()` | Medium | V3.3 |
| GAP-5 | Per-sphere override not connected to proposals | Medium | V3.4 |
| GAP-6 | No proposable opt-out variant | Low | V3.4 |
| GAP-7 | 5-tick voting window too short for 60-tick Nexus intervals | Medium | V3.4 |

## Fix Strategy

GAP-1 is the critical path. The executor (M33) or tick orchestrator (M35) needs a `check_approved_proposals()` call that:
1. Reads `approved_unapplied()` from ProposalManager
2. Applies changes to the relevant constants/config
3. Calls `mark_applied()` on each executed proposal
4. Notifies ConsentGate if K_MOD_BUDGET changed

## Links
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[CONSENT_SPEC]]
- [[The Habitat — Integrated Master Plan V3]]
