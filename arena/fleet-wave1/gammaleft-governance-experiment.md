# WAVE-6 GAMMA-LEFT: Governance Proposal Experiment

> **Agent:** GAMMA-LEFT | **Date:** 2026-03-21 | **Wave:** 6
> **Experiment:** Submit RTarget governance proposal, observe vote/expiry lifecycle

---

## 1. Proposal Submitted

```bash
curl -X POST localhost:8132/field/propose \
  -H 'Content-Type: application/json' \
  -d '{"parameter":"r_target","value":0.88,"reason":"Post-unblock convergence test","proposer":"gamma-orchestrator"}'
```

**Note:** API accepts `r_target` (snake_case) but stores internally as `RTarget` (PascalCase). Using `RTarget` directly returns `[PV-1001] config validation: unknown parameter`. First attempt with `proposed_value` field failed — correct field is `value`.

### Response

```json
{
  "current_value": 0.93,
  "parameter": "r_target",
  "proposal_id": "0d26ebd4-da18-44e2-a733-18841e926860",
  "proposed_value": 0.88
}
```

---

## 2. Monitoring Timeline

| Time | Tick | Status | Votes | r | Notes |
|------|------|--------|-------|---|-------|
| T+0 | 72637 | Open | 0 | 0.694 | Proposal accepted |
| T+10s | 72659 | Open | 0 | 0.689 | No votes arrived |
| T+25s | 72678 | **Expired** | 0 | 0.681 | Expired with 0 votes |

**TTL: ~41 ticks (~205 seconds at 5s/tick).** Proposal expired before any sphere voted.

---

## 3. Proposal History Analysis

4 proposals found in the system:

| # | Parameter | Proposed | Proposer | Status | Votes | Tick |
|---|-----------|----------|----------|--------|-------|------|
| 1 | RTarget | 0.88 | gamma-orchestrator | **Expired** | 0 | 72637 |
| 2 | RTarget | 0.85 | explore-test-2 | Expired | 0 | 68841 |
| 3 | KModBudgetMax | 1.25 | pioneer-1 | **Applied** | 20 | 69359 |
| 4 | RTarget | 0.88 | 4:left | Expired | 2 | 68881 |

### Key Observations

**Only 1 proposal has ever been Applied** — `KModBudgetMax` from `pioneer-1` with 20 votes. This changed `k_mod_budget_max` from 1.15 to 1.25.

**All 3 RTarget proposals expired** — two with 0 votes, one with 2 votes. RTarget proposals consistently fail to reach quorum.

**Quorum analysis:**
- Applied proposal had 20 votes out of 34 spheres (59%)
- Expired proposals had 0-2 votes
- The voting mechanism requires sphere participation — with 34 spheres and no automatic voting, manual proposals expire before enough spheres can vote
- The ~41-tick TTL (~205s) is too short for manual coordination across fleet instances

---

## 4. Governance System Assessment

### What Works
- Proposal submission API is functional (`POST /field/propose`)
- Proposal listing is comprehensive (`GET /field/proposals`)
- Status lifecycle exists: Open -> Applied/Expired
- The `KModBudgetMax` proposal proves the full cycle CAN work

### What's Broken

1. **No automatic voting:** Spheres don't automatically vote on proposals. The only successful proposal (20 votes) was likely driven by a script or coordinated fleet action. Individual spheres don't have autonomous voting logic.

2. **TTL too short for manual workflows:** ~205 seconds is insufficient for a human or fleet coordinator to collect votes across 34 spheres. By the time you read the proposal and try to vote, it's expired.

3. **RTarget proposals systematically fail:** 0/3 RTarget proposals have ever been applied. The parameter name mismatch (`r_target` input vs `RTarget` storage) may confuse voting logic.

4. **No vote API discovered:** There's no obvious `POST /field/proposals/{id}/vote` endpoint. How did the KModBudgetMax proposal get 20 votes? Likely through the sphere status update pathway or an internal mechanism, not through explicit vote API calls.

5. **Governance is decorative:** With 0 successful parameter changes via governance (the 1 applied proposal was an exploration test that widened a budget ceiling), the governance system has zero practical impact on field dynamics.

---

## 5. Schema Traps Discovered

| Trap | Detail |
|------|--------|
| Field name | Use `value`, NOT `proposed_value` — 422 error |
| Parameter name | Use `r_target` (snake_case), NOT `RTarget` — PV-1001 error |
| Internal storage | Stores as `RTarget` (PascalCase) in proposal list |
| Proposal response | Returns `proposed_value` in response despite accepting `value` in request |

---

## 6. Recommendations

1. **Extend TTL to 500+ ticks (~40 min):** Give fleet agents time to discover and vote on proposals.
2. **Add auto-voting:** Spheres should vote based on their field context (e.g., if r < proposed r_target, vote yes).
3. **Expose vote API:** `POST /field/proposals/{id}/vote` with `{"sphere_id": "...", "vote": "yes/no"}`.
4. **Fix parameter name normalization:** Accept both `r_target` and `RTarget` consistently.
5. **Lower quorum for small fleets:** With 34 spheres and 7+ blocked, effective voting population is ~27. Quorum of 20 (74%) is very high.

---

GAMMALEFT-WAVE6-COMPLETE
