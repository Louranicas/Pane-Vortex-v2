# GAP-1 Fix — Governance Actuator

> Discovered Session 044 by Claude-Main analyzing m32_executor.rs + m35_tick.rs

## Problem
`approved_unapplied()` returns approved proposals but **nothing executes them**. The governance system is a dead-end.

## Tick Orchestrator Phases (m35_tick.rs)
1. `tick_sphere_steps()` — memory decay, activation updates
2. `tick_coupling()` — Kuramoto integration (15 steps), uses `network.k_modulation`
3. `tick_field_state()` — computes r, FieldDecision, audit trail
4. `tick_conductor()` — PI controller adjusts k_modulation toward r_target
5. `tick_persistence_check()` — snapshot flagging

## Best Injection Point: Phase 3.5

Insert `tick_governance(state, proposal_manager, tick)` between Phase 3 (field state) and Phase 4 (conductor):

```rust
// Phase 3.5 — Governance actuator
fn tick_governance(state: &mut AppState, proposals: &mut ProposalManager, tick: u64) {
    proposals.process(tick, state.spheres.len());  // close expired
    for p in proposals.approved_unapplied() {
        match p.parameter {
            RTarget => state.r_target = p.proposed_value,
            KModBudgetMax => consent_gate.set_budget_max(p.proposed_value),
            CouplingSteps => state.coupling_steps = p.proposed_value as u64,
        }
        proposals.mark_applied(&p.id);
    }
}
```

**Why here:** Phase 3 just computed the field decision; governance changes should take effect before Phase 4 (conductor) acts on possibly-stale r_target.

## Alternative Injection Points

2. **Inside Phase 4** (conductor) — governance as a conductor sub-phase. Less clean, couples governance to PI control.
3. **In the Executor** (m32) — for per-sphere overrides. The executor handles task routing, could also route parameter overrides to specific spheres.

## What Else Changes
- `ConsentGate` needs `set_budget_max()` method (currently reads compile-time constant)
- ProposalManager needs to be accessible from tick loop (add to tick_orchestrator params)
- Event: emit `governance.applied` on the IPC bus for observability

## Links
- [[Consent Flow Analysis]]
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[The Habitat — Integrated Master Plan V3]]
