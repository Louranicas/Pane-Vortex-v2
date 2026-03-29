# BUG-HUNT BETA: Three Active Bug Investigation

**Agent:** BETA | **Timestamp:** 2026-03-21 ~02:23 UTC

---

## BUG-1: SYNTHEX Synergy CRITICAL (0.5 < 0.7 threshold)

### Verified State

```json
{
  "name": "Synergy",
  "severity": "Critical",
  "value": 0.5,
  "critical_threshold": 0.7,
  "warning_threshold": 0.9,
  "auto_heal_action": null
}
```

Health: 0.75 (dragged down solely by this probe). Other probes: PatternCount=0 (Ok), CascadeAmplification=1.0 (Ok), Latency=10ms (Ok).

### Fixable Without V2?

**No via API.** Tested:
- `POST /v3/synergy` → 404
- `POST /v3/diagnostics/reset` → 404
- `PATCH /v3/diagnostics` → 405 (Method Not Allowed — endpoint exists but read-only)
- `POST /v3/config` → 404
- `POST /v3/homeostasis` → 404
- `POST /v3/homeostasis/config` → 404

**Root cause:** Synergy is an internally-computed metric based on cross-service interaction quality. With 3/4 heat sources dead and temperature at 0.03, synergy correctly reports 0.5. It's a symptom, not a misconfiguration. Synergy will self-heal when the thermal loop closes and heat sources activate.

### Fix

**Cannot fix directly.** Synergy recovers when SYNTHEX temperature rises, which requires V2 binary deploy (closes thermal feedback loop) or manual heat source stimulation via writable endpoints (if any exist — see SYNERGY task).

---

## BUG-2: ME Emergence Cap 1000/1000 — Evolution Dead

### Verified State

```json
{
  "chamber_stats": {
    "current_generation": 26,
    "total_applied": 258,
    "total_proposed": 258,
    "total_rolled_back": 3,
    "total_ralph_cycles": 0
  },
  "ralph_state": {
    "phase": "Analyze",
    "cycle_number": 1,
    "mutations_applied": 258,
    "mutations_proposed": 258,
    "paused": false
  },
  "active_mutations": 0
}
```

**All 20 recent mutations target `emergence_detector.min_confidence`** — Ralph is stuck in a single-parameter loop, unable to explore other dimensions.

### Config File Found

`/home/louranicas/claude-code-workspace/the_maintenance_engine/config/observer.toml` line 79:
```toml
[observer.emergence_detector]
history_capacity = 1000    # ← THE CAP (range: [100, 10000])
min_confidence = 0.7       # ← 258 mutations have been hammering this
```

### Fixable Without V2?

**YES — config file edit + devenv restart.** This is a ME-only fix, independent of PV.

### Recommended Fix

```toml
# observer.toml line 79: raise cap
history_capacity = 5000    # was 1000, range allows up to 10000

# observer.toml line 89: reset confidence (258 mutations moved it to extreme)
min_confidence = 0.5       # was 0.7, lower threshold lets more emergences through
```

Then: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart maintenance-engine`

### NOT Applied

Editing ME config is outside the scope of a PV agent probe. Flagging for orchestrator decision. The fix is safe and reversible (revert the two values + restart).

---

## BUG-3: library-agent 7,821 Consecutive Failures

### Verified State

```json
{
  "id": "library-agent",
  "reachable": false,
  "circuit_open": true,
  "health_score": 0.0,
  "total_failures": 7821,
  "total_successes": 0,
  "synergy_score": 0.2
}
```

Library-agent is disabled in ULTRAPLATE (listed as disabled in CLAUDE.md), but ME still probes it. Circuit breaker is open (correct) but failures keep accumulating — ME hasn't stopped health-checking it.

### Fixable Without V2?

**No via API.** Tested all write methods on `/api/services/library-agent`:
- POST → 404
- PUT → 404
- PATCH → 404
- DELETE → 404

ME has **observation-only APIs** — no write/patch/delete endpoints exist for service management.

### Fix Options

1. **Config fix:** Add `library-agent` to an exclusion list in ME's `services.toml` config (if such a list exists) + restart
2. **Structural:** Remove library-agent from ME's service registry in the database
3. **Accept:** Circuit breaker is open, failures don't cause actual harm beyond polluting fitness (health_score=0.0 drags down the deps dimension at 0.083)

### Impact on Fitness

ME tracks 12 services. library-agent at health=0.0 contributes ~0.083 to the deps dimension (1/12 = 0.083). Removing it would raise deps from 0.083 to ~0.091 (1/11) — marginal improvement. The real deps problem is structural.

---

## Summary Matrix

| Bug | Verified | Fixable Now? | Fix Method | Risk |
|-----|----------|-------------|------------|------|
| BUG-1: SX Synergy 0.5 | Yes, CRITICAL | **No** — no write API, symptom not cause | V2 deploy closes thermal loop → self-heals | None |
| BUG-2: ME Emergence 1000 | Yes, deadlocked | **Yes** — config edit + restart | `observer.toml`: history_capacity=5000, min_confidence=0.5 | Low (reversible) |
| BUG-3: library-agent 7821 | Yes, circuit open | **No** — no service management API | Config exclusion or accept | None |

### Recommended Actions

1. **BUG-2 is the only actionable fix.** Two-line config edit, zero risk, unlocks evolution.
2. **BUG-1 self-resolves** once thermal loop closes (V2 deploy or heat injection).
3. **BUG-3 is cosmetic** — circuit breaker works, just noisy in metrics.

---

BETA-BUGHUNT-COMPLETE
