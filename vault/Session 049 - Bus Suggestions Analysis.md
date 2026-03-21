# Session 049 — Bus Suggestions Analysis

> **Captured:** 2026-03-21 | **PV tick:** 99,899–99,914 | **Total generated:** 6,027 | **Visible:** 20
> **Cross-refs:** [[ULTRAPLATE Master Index]] | [[IPC Bus Architecture Deep Dive]] | [[Session 049 — Full Remediation Deployed]]

---

## TL;DR

All 20 suggestions are `SuggestReseed` for 7 blocked fleet panes. They are **generated correctly by the suggestion engine** but represent a **false-positive blocked detection** — the panes are idle shells that registered as spheres with persona `fleet-worker`, then went silent. The decision engine classifies stale fleet-worker spheres as "blocked" because they haven't reported status within the timeout window.

**Actionable?** Not directly — these panes need fresh Claude instances launched, not reseeding. The suggestion engine is working as designed, but the "blocked" classification conflates "stale/idle" with "genuinely stuck."

---

## 1. Suggestion Queue

| Metric | Value |
|--------|-------|
| Total generated (lifetime) | **6,027** |
| Visible (ring buffer) | **20** |
| Suggestion types | **1** (SuggestReseed only) |
| Unique targets | **7** |
| Tick range | 99,912 → 99,914 (3 consecutive ticks) |
| Confidence | **0.7** (uniform — hardcoded in `m34_suggestions.rs:308`) |
| Suggestions per tick | ~7 (one per blocked sphere per tick) |

### Per-Target Breakdown

| Target | Count | Ticks | Tab/Pane |
|--------|-------|-------|----------|
| 4:left | 3 | 99894-99896 | ALPHA left |
| 5:left | 3 | 99894-99896 | BETA left |
| 5:top-right | 3 | 99894-99896 | BETA top-right |
| 5:bottom-right | 3 | 99894-99896 | BETA bottom-right |
| 6:left | 3 | 99894-99896 | GAMMA left |
| 6:top-right | 3 | 99894-99896 | GAMMA top-right |
| 6:bottom-right | 2 | 99895-99896 | GAMMA bottom-right |

**Pattern:** Each blocked sphere generates 1 suggestion per tick, across 3 consecutive ticks. The queue holds a rolling window of ~20 suggestions.

---

## 2. Decision Engine State

| Field | Value |
|-------|-------|
| Action | **HasBlockedAgents** |
| Blocked count | **7** |

### Blocked Sphere List

| # | Sphere ID | Tab | Pane Position | Persona |
|---|-----------|-----|---------------|---------|
| 1 | 4:left | Fleet-ALPHA | Left | fleet-worker |
| 2 | 5:left | Fleet-BETA | Left | fleet-worker |
| 3 | 5:top-right | Fleet-BETA | Top-right | fleet-worker |
| 4 | 5:bottom-right | Fleet-BETA | Bottom-right | fleet-worker |
| 5 | 6:left | Fleet-GAMMA | Left | fleet-worker |
| 6 | 6:top-right | Fleet-GAMMA | Top-right | fleet-worker |
| 7 | 6:bottom-right | Fleet-GAMMA | Bottom-right | fleet-worker |

**All 7 are fleet pane positions.** These are the shell panes in tabs 4-6 that registered as spheres (persona `fleet-worker`) but have gone silent — no status updates within the timeout window.

### Sphere Detail (from `/sphere/{id}`)

| Sphere | Status | last_tool | Frequency |
|--------|--------|-----------|-----------|
| 4:left | Blocked | unblocked-session-049 | 0.150 |
| 5:left | Blocked | unblocked-session-049 | 0.195 |
| 5:top-right | Blocked | unblocked-session-049 | 0.150 |
| 5:bottom-right | Blocked | unblocked-session-049 | 0.150 |
| 6:left | Blocked | unblocked-session-049 | 0.150 |
| 6:top-right | Blocked | unblocked-session-049 | 0.150 |
| 6:bottom-right | Blocked | unblocked-session-049 | 0.150 |

**These are V1-era positional registrations** created during Session 049 unblock (`last_tool: "unblocked-session-049"`). The actual fleet Claude instances now register under named IDs:

| V1 Positional (Blocked Ghost) | Current Named (Working) |
|-------------------------------|------------------------|
| `4:left` | `fleet-alpha` |
| `5:left` | `fleet-beta-1` |
| `6:left` | `fleet-gamma-1` |
| `5:top-right` | (no active Claude) |
| `5:bottom-right` | (no active Claude) |
| `6:top-right` | (no active Claude) |
| `6:bottom-right` | (no active Claude) |

### Immediate Fix: Deregister Ghost Registrations

```bash
for sphere in "4:left" "5:left" "5:top-right" "5:bottom-right" "6:left" "6:top-right" "6:bottom-right"; do
  curl -sf -X POST "localhost:8132/sphere/$sphere/deregister"
  echo " deregistered: $sphere"
done
```

**Expected:** Spheres 52→45, blocked=0, action→IdleFleet, suggestions stop.

---

## 3. Bus Info

| Metric | Value |
|--------|-------|
| Subscribers | **2** |
| Pending tasks | **0** (was 5, now drained) |
| Events | **1,000** (buffer cap) |
| Cascades | **0** |

Task queue has fully drained (was 53 at session start → 5 → 0). Event buffer at cap. Two subscribers active.

---

## 4. Source Code Analysis

### How SuggestReseed Is Generated

From `m34_suggestions.rs:296-313`:

```rust
fn suggest_reseed_for_blocked(
    &self,
    decision: &FieldDecision,
    spheres: &HashMap<PaneId, PaneSphere>,
    out: &mut Vec<FieldSuggestion>,
) {
    for id in &decision.blocked_spheres {
        if let Some(sphere) = spheres.get(id) {
            out.push(FieldSuggestion::new(
                SuggestionType::SuggestReseed,
                id.clone(),
                format!("sphere {} is blocked, reseed may help", sphere.persona),
                0.7,  // hardcoded confidence
                decision.tick,
            ));
        }
    }
}
```

**Trigger condition:** `FieldAction::HasBlockedAgents` (line 165). Every tick with blocked spheres generates one SuggestReseed per blocked sphere.

### What "Blocked" Means

The decision engine marks a sphere as blocked when:
1. It has persona containing "fleet" or "worker"
2. Its status hasn't been updated within the staleness timeout
3. It's not in the working or idle set

This is correct for genuinely stuck processes — but these panes are **empty shells** (no Claude running), not stuck processes.

---

## 5. Are These Suggestions Actionable?

| Question | Answer |
|----------|--------|
| Is the suggestion engine working correctly? | **Yes** — code path is correct |
| Are the spheres genuinely blocked? | **No** — they're idle shells that registered then went silent |
| Would reseeding help? | **Partially** — launching Claude in these panes would clear the blocked status |
| Should these be actioned now? | **No** — they're awaiting intentional fleet launch, not rescue |
| Is this a bug? | **Design gap** — blocked vs stale distinction needed |

### What Would Make Them Actionable

1. **If a Claude instance was running and stuck** → Reseed = kill + relaunch → Actionable
2. **If the pane has a hung process** → Reseed = restart → Actionable
3. **If the pane is an empty shell** → Reseed = launch Claude → Not a reseed, it's a fresh launch

Current state is case 3. These panes registered as spheres (likely via session-start hooks) but don't have active Claude instances.

---

## 6. Suggestion Engine Improvement Recommendations

| Issue | Current | Suggested |
|-------|---------|-----------|
| Blocked vs stale | Same classification | Add `SuggestLaunch` for never-active, `SuggestReseed` for stuck-active |
| Confidence | Hardcoded 0.7 | Scale by time-since-last-heartbeat |
| Deduplication | 1 per tick per sphere | Deduplicate across ticks, only re-suggest after N ticks |
| Queue growth | 7 suggestions/tick × ∞ | Cap per-target to 1 active suggestion |
| Persona check | Any "fleet-worker" | Distinguish registered-never-active from registered-then-stuck |

### Suggested New Classification

```
Sphere registered, never sent heartbeat → SuggestLaunch (confidence 0.5)
Sphere was Working, now silent > 5 min  → SuggestReseed (confidence 0.8)
Sphere was Working, now silent > 30 min → SuggestReseed (confidence 0.95)
Sphere is Blocked (explicit status)     → SuggestReseed (confidence 0.9)
```

---

## 7. Queue Accumulation Pattern

```
Tick 99,912:  7 suggestions (1 per blocked sphere)
Tick 99,913:  7 suggestions (same 7 spheres)
Tick 99,914:  6 suggestions (6:bottom-right missed — timing)
Total:        20 suggestions across 3 ticks

Growth rate: ~7 suggestions per tick (every 5 seconds)
Queue cap:   appears to be 20 (rolling window)
```

The suggestion queue appears to maintain a rolling window of ~20 entries, pruning oldest when new ones arrive. This prevents unbounded growth but means the queue is dominated by repetitive SuggestReseed entries, crowding out any potential diverse suggestions.

---

## Summary

| Dimension | Status |
|-----------|--------|
| Suggestion engine | **Working correctly** |
| Suggestion type | **SuggestReseed × 20** (monotonic) |
| Root cause | 7 fleet panes registered as spheres, went silent |
| Actionable | **No** — panes need fresh Claude launches, not reseeding |
| Design gap | Blocked ≠ stale — needs classification refinement |
| Bus health | Healthy (tasks drained, 2 subs, events at cap) |

---

### Suggestion Generation Rate

```
6,027 suggestions ÷ ~860 ticks of blocked state = ~7/tick
7 suggestions/tick × 5s/tick = 1.4 suggestions/second
Signal-to-noise ratio: 0% (all targeting ghosts)
```

---

*See also:* [[ULTRAPLATE Master Index]] for fleet tab layout | [[IPC Bus Architecture Deep Dive]] for bus event model and suggestion engine design | [[Session 049 — Full Remediation Deployed]] for the unblock operation that created the ghost registrations.
