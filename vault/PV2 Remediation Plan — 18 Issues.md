# PV2 Remediation Plan — 18 Issues

> From Session 044 fleet orchestration (17 analyses, 219KB, 7 surfaces)
> Addresses: 7 consent/governance gaps + 4 ghost gaps + 3 untested handlers + 4 architecture issues

## Gap Analysis (5 Critical Gaps Found & Mitigated)

| Gap | Severity | Issue | Mitigation |
|-----|----------|-------|------------|
| GAP-A | P0 | 8 V3.1 MASTERPLAN items missing (BUG-008, evolution 404s, etc.) | Added Phase 0 |
| GAP-B | P0 | Lock ordering deadlock: tick_governance → ConsentGate while AppState held | Extract-then-apply pattern |
| GAP-C | P1 | New AppState fields break snapshot deserialization | `#[serde(skip)]` for ProposalManager, `#[serde(default)]` for r_target_override |
| GAP-D | P1 | Enabling governance feature may cause cross-module import errors | All governance refs outside L8 must be `#[cfg(feature = "governance")]` |
| GAP-E | P1 | NexusBus mid-tick lock window (API can mutate state between coupling and Hebbian) | Documented as tech debt; V2 passes `&mut AppState` (no lock release) |

## Execution Order

```
Phase 0 (V3.1 Diagnostics — MUST come first):
  0.1 BUG-008 ME EventBus (highest impact)
  0.2-0.8 Evolution 404s, SYNTHEX synergy, Prometheus, CCM, ports, VMS

Phase 1 (V3.1 — parallel, no dependencies):
  1.1 Add 10+ missing API handler tests (phase, steer, bus_suggestions)
  1.2 Fix K sensitivity cliff (IQR spread + 25% rate limiter)
  1.3 Enrich ghost trace at deregister (all neighbors, not top-3)
  1.4 Fix conductor/bridge composition (no last-writer-wins)

Phase 2 (V3.2 — depends on 1.3):
  2.1 Add accept_ghost to AppState (consume ghost, exact ID match)
  2.2 Wire ghost phase restoration into registration (G1 fix)

Phase 3 (V3.3/V3.4 — depends on 1.4):
  3.1 Enable governance feature in Cargo.toml defaults
  3.2 Add ProposalManager to AppState
  3.3 Add Phase 3.5 tick_governance (GAP-1 — CRITICAL)
  3.4 Add runtime budget variable (GAP-2)
  3.5 Wire divergence_requested exemption (GAP-4)
  3.6 Extend apply_combined for 6 bridges (GAP-3)
  3.7 Fix voting window 5→24 ticks (GAP-7)
  3.8 Add SphereOverride + OptOutPolicy variants (GAP-5, GAP-6)

Phase 4 (V3.4 — depends on 3.2, 3.3):
  4.1 Add 7 governance API routes (propose, vote, consent, data manifest)

Phase 5 (Integration — depends on all):
  5.1 Governance E2E test
  5.2 Ghost reincarnation E2E test
  5.3 Consent gate integration test
```

## Test Budget: 1,379 → 1,456 (+77 tests)

## Critical Path: Phase 1 → Phase 2 → Phase 3.1-3.3 → Phase 4 → Phase 5

## Links
- [[Session 044 — Deep Synthesis]]
- [[Consent Flow Analysis]]
- [[GAP-1 Fix — Governance Actuator]]
- [[IPC Bus Architecture Deep Dive]]
- [[The Habitat — Integrated Master Plan V3]]
