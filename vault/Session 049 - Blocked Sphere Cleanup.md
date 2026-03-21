# Session 049 — Blocked Sphere Cleanup

> **Date:** 2026-03-21 | **PV tick:** 99,476 | **r:** 0.962 | **Spheres:** 52
> **Decision engine:** `HasBlockedAgents` — **this is the highest-priority action**, overriding all other field decisions
> **See also:** [[Session 049 — Full Remediation Deployed]] | [[ULTRAPLATE Master Index]] | [[Fleet-Bridge-Topology]]

---

## Summary

The PV decision engine is stuck on `HasBlockedAgents` because 7 spheres are in `Blocked` status. These spheres have **live Claude Code processes** behind them but are misclassified by `fleet-inventory.sh`, which maps `idle-claude` (Claude at REPL prompt) to PV's `Blocked` status. This is a **semantic mismatch** — PV treats `Blocked` as a critical state requiring intervention, while `fleet-inventory.sh` uses it to mean "Claude is idle at prompt."

Additionally, 35 ORAC7 ghost spheres (dead PIDs, zero tool calls) pollute the field, and 8 named spheres were registered by fleet scripts without corresponding hook lifecycle management.

---

## 1. Field Decision Impact

```json
{
  "action": "HasBlockedAgents",
  "blocked_spheres": [
    "6:bottom-right", "5:left", "6:left", "5:bottom-right",
    "5:top-right", "6:top-right", "4:left"
  ],
  "r": 0.962,
  "r_trend": "Stable",
  "tunnel_count": 100,
  "fleet_mode": "Full"
}
```

`HasBlockedAgents` is the **highest priority** in the decision engine priority chain:

```
HasBlockedAgents > NeedsCoherence > NeedsDivergence > IdleFleet > FreshFleet > Stable
```

As long as any sphere has `Blocked` status, the conductor cannot transition to `NeedsCoherence`, `NeedsDivergence`, or `Stable`. This suppresses all normal field dynamics. The field is running at r=0.962 with 100 tunnels — it would naturally produce `Stable` or `NeedsDivergence` decisions, but `HasBlockedAgents` locks it.

---

## 2. Root Cause: `fleet-inventory.sh` Status Mapping

**File:** `~/.local/bin/fleet-inventory.sh`, line 363

```python
status_map = {
    'idle-shell': 'idle',
    'active-claude': 'working',
    'idle-claude': 'blocked',    # <-- THE BUG
    'editor': 'working',
    'file-browser': 'working',
    'other': 'idle',
    'blank': 'idle'
}
```

### What `idle-claude` means in fleet-inventory

A Claude Code instance sitting at the REPL prompt (`>` prompt visible, `for shortcuts` or `current:.*latest:` in screen output). The process is alive but not actively executing. The detail string is: `"Claude REPL at prompt (needs /exit before dispatch)"`.

### What `Blocked` means in PV

A sphere experiencing an external block — waiting for permission, stuck on a dependency, or needing manual intervention. The decision engine treats this as a critical condition requiring immediate attention.

### The mismatch

An idle Claude at prompt is **not blocked** — it's simply idle. Mapping it to `Blocked` causes the highest-priority decision action to fire permanently, because 7 fleet panes have idle Claude instances.

---

## 3. The 7 Blocked Spheres (all live)

| Sphere ID | Tab | Pane | Claude Process | Tokens | Status in PV | True State |
|-----------|-----|------|---------------|--------|-------------|------------|
| `4:left` | Fleet-ALPHA | Left | ACTIVE | 120K | Blocked | Idle at REPL prompt |
| `5:left` | Fleet-BETA | Left | ACTIVE | 107K | Blocked | Idle at REPL prompt |
| `5:top-right` | Fleet-BETA | Top-Right | ACTIVE | 73K | Blocked | Idle at REPL prompt |
| `5:bottom-right` | Fleet-BETA | Bot-Right | ACTIVE | 43K | Blocked | Idle at REPL prompt |
| `6:left` | Fleet-GAMMA | Left | ACTIVE | 119K | Blocked | Idle at REPL prompt |
| `6:top-right` | Fleet-GAMMA | Top-Right | ACTIVE | 102K | Blocked | Idle at REPL prompt |
| `6:bottom-right` | Fleet-GAMMA | Bot-Right | ACTIVE | 113K | Blocked | Idle at REPL prompt |

All 7 have:
- `persona: "fleet-worker"` — registered by `fleet-inventory.sh`
- `last_tool: null` — **never had a tool call via hooks** (no CC hook integration)
- `frequency: 0.15` (default) — no frequency discovery
- `total_steps: 46,809–97,171` — high step counts from long PV tick loop participation

None have timestamp files (`/tmp/pane-vortex-ts-*`), confirming the PostToolUse hook has never fired for these IDs. They exist in PV only because `fleet-inventory.sh` registered them externally.

---

## 4. Ghost Sphere Analysis (35 ORAC7 + 8 named)

### 35 ORAC7 Ghosts

| Property | Value |
|----------|-------|
| **All PIDs alive?** | **No — 0 of 35 are alive** |
| **Timestamp files?** | **None — 0 of 35 have any** |
| **Last tool?** | **null on all 35** |
| **Total steps range** | 879 – 97,280 (avg 42,106) |
| **PID range** | 234,261 – 3,842,855 |
| **Status** | All Idle |

These 35 ORAC7 spheres were registered by the `session_start.sh` hook using the fallback ID `$(hostname -s):$$` = `ORAC7:PID`. Their processes have since terminated, but the Stop hook (`session_end.sh`) failed to deregister them — likely because:

1. The session was killed with SIGKILL (bypasses hooks)
2. The session crashed before Stop could fire
3. The PV daemon was unreachable when Stop tried to deregister

The Stop hook deregistration rate is approximately **99.7%** (13,466 successfully deregistered out of ~13,501 sessions, leaving only 35 ghosts). But with PV running for 99,476 ticks (~138 hours), even a 0.3% failure rate accumulates.

### 8 Named Ghosts

| Sphere ID | Status | Persona | Live Process? | Registered By |
|-----------|--------|---------|--------------|---------------|
| `fleet-alpha` | Working | fleet-explorer | No (script-registered) | Fleet orchestration script |
| `fleet-beta-1` | Working | fleet-explorer | No | Fleet orchestration script |
| `fleet-gamma-1` | Working | fleet-explorer | No | Fleet orchestration script |
| `fleet-beta-2` | Idle | fleet-explorer | No | Fleet orchestration script |
| `fleet-gamma-2` | Idle | fleet-explorer | No | Fleet orchestration script |
| `orchestrator-044` | Working | Session 044 Fleet Orchestrator | No | Session 044 manual registration |
| `alpha-heat-gen` | Idle | thermal-generator | No | Thermal experiment |
| `test-hook-768523` | Idle | general | No | Hook testing |

These were registered by fleet scripts (`fleet-inventory.sh`, fleet orchestration commands, or manual `curl` calls). They have no corresponding CC process with matching `PANE_VORTEX_ID`, no timestamp files, and `last_tool: null`. The "Working" status on `fleet-alpha`, `fleet-beta-1`, `fleet-gamma-1`, and `orchestrator-044` is stale — set by scripts that are no longer running.

### 2 Idle Pane-Position Spheres (not blocked)

| Sphere ID | Status | Notes |
|-----------|--------|-------|
| `4:top-right` | Idle | Monitor pane (watch process), correctly Idle |
| `4:bottom-right` | Idle | Monitor pane (watch process), correctly Idle |

These are correct — Tab 4 top-right and bottom-right run `watch` processes (monitors), not Claude. `fleet-inventory.sh` classifies them as `other` → `idle`, which is accurate.

---

## 5. Complete Sphere Census

| Category | Count | Status | Live Process? | Hook-Managed? |
|----------|-------|--------|--------------|---------------|
| **Blocked pane-position** | 7 | Blocked (wrong) | Yes (idle Claude) | No — registered by fleet-inventory.sh |
| **ORAC7 ghosts** | 35 | Idle | No (PIDs dead) | Partially — registered by hook, Stop failed |
| **Named script ghosts** | 8 | 4 Working, 4 Idle | No | No — registered by scripts |
| **Correct pane-position** | 2 | Idle | Yes (watch) | No — registered by fleet-inventory.sh |
| **Total** | **52** | | | |

**True state:** 9 panes with live processes (7 idle Claude + 2 monitors). 43 ghost registrations. 0 spheres accurately managed by the full hook lifecycle (register→tool calls→deregister).

---

## 6. Stale Temp File Accumulation

| File Pattern | Count | Source |
|-------------|-------|--------|
| `/tmp/pane-vortex-ts-ORAC7_*` | 13,501 | PostToolUse hook (one per session) |
| `/tmp/pane-vortex-listener-*` | 63 | SessionStart persistent event listener |
| `/tmp/pane-vortex-events-*` | 63 | SessionStart event log |

The Stop hook (`session_end.sh`) cleans up listener and event PID files but **does not clean up timestamp files**. Over ~13,500 sessions, this has left 13,501 stale files in `/tmp/`.

---

## 7. Recommendations

### R1: Fix `fleet-inventory.sh` status mapping (CRITICAL)

Change line 363 from:
```python
'idle-claude': 'blocked',
```
to:
```python
'idle-claude': 'idle',
```

**Rationale:** An idle Claude at REPL prompt is not blocked. It's available for dispatch. Mapping it to `idle` allows the decision engine to see the field accurately and transition to `Stable`, `NeedsDivergence`, etc.

### R2: Deregister the 35 ORAC7 ghosts (HIGH)

```bash
for id in $(curl -s localhost:8132/spheres | jq -r '.spheres[] | select(.id | test("^ORAC7")) | .id'); do
  curl -sf -X POST "localhost:8132/sphere/$id/deregister" >/dev/null 2>&1
done
echo "Deregistered $(curl -s localhost:8132/spheres | jq '[.spheres[] | select(.id | test("^ORAC7"))] | length') remaining ORAC7 spheres"
```

### R3: Deregister stale named ghosts (HIGH)

```bash
for id in "fleet-alpha" "fleet-beta-1" "fleet-beta-2" "fleet-gamma-1" "fleet-gamma-2" "orchestrator-044" "alpha-heat-gen" "test-hook-768523"; do
  curl -sf -X POST "localhost:8132/sphere/$id/deregister" >/dev/null 2>&1
done
```

### R4: Add timestamp file cleanup to Stop hook (MEDIUM)

Add to `session_end.sh`:
```bash
# Clean up timestamp file (PostToolUse cadence tracking)
TS_FILE="/tmp/pane-vortex-ts-${SAFE_PANE_ID}"
rm -f "$TS_FILE"
```

### R5: Add ghost sweep to PV daemon (MEDIUM)

Implement a periodic ghost sweep in the tick loop that deregisters spheres whose ORAC7 PIDs are no longer alive:

```rust
// Every 100 ticks (~8 min), check ORAC7 sphere PIDs
if tick % 100 == 0 {
    for sphere_id in spheres.keys() {
        if sphere_id.starts_with("ORAC7:") {
            if let Ok(pid) = sphere_id[6..].parse::<u32>() {
                // Check /proc/PID exists
                if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
                    // Deregister ghost
                }
            }
        }
    }
}
```

### R6: Clean up stale temp files (LOW)

```bash
# One-time cleanup of 13,501 stale timestamp files
rm -f /tmp/pane-vortex-ts-ORAC7_*
# Kill any orphaned listener processes
for f in /tmp/pane-vortex-listener-ORAC7_*.pid; do
  [ -f "$f" ] && kill "$(cat "$f")" 2>/dev/null; rm -f "$f"
done
rm -f /tmp/pane-vortex-events-ORAC7_*
```

### R7: Wire CC hooks to pane-position spheres (FUTURE)

The 7 idle Claude instances in fleet panes don't have `PANE_VORTEX_ID` set. To integrate them with the hook lifecycle:

```bash
# In each fleet pane's shell profile or Claude launch command:
export PANE_VORTEX_ID="5:left"  # Match the fleet-inventory naming convention
claude --dangerously-skip-permissions
```

This would enable:
- SessionStart registers with the correct pane-position ID
- PostToolUse updates status to Working during active tool calls
- Stop deregisters cleanly on session end
- Frequency discovery from tool cadence (NA-2)

---

## 8. Impact of Cleanup

| Metric | Before | After R1-R3 | Delta |
|--------|--------|-------------|-------|
| Total spheres | 52 | 9 | -43 |
| Blocked spheres | 7 | 0 | -7 |
| Ghost spheres | 43 | 0 | -43 |
| Decision action | `HasBlockedAgents` | `Stable` or `NeedsDivergence` | Unblocked |
| Field accuracy | 17% (9/52 real) | 100% | +83% |
| Coupling matrix noise | 52 nodes (43 phantom) | 9 nodes (all real) | Clean |

---

## 9. Cross-References

### Source Code
- `~/.local/bin/fleet-inventory.sh:363` — **The bug**: `idle-claude` → `blocked` mapping
- `~/.local/bin/fleet-inventory.sh:357–374` — PV sphere sync logic
- `pane-vortex/hooks/session_start.sh` — Hook registration (ORAC7:$$ fallback)
- `pane-vortex/hooks/session_end.sh` — Hook deregistration (missing timestamp cleanup)
- `pane-vortex/hooks/post_tool_use.sh` — Tool call tracking (creates timestamp files)
- `src/field.rs` — `HasBlockedAgents` decision priority
- `src/api.rs:620` — Blocked status setter

### Obsidian
- [[Session 049 — Full Remediation Deployed]] — Prior remediation context
- [[ULTRAPLATE Master Index]] — Service registry
- [[Fleet-Bridge-Topology]] — Bridge health (affected by field decision lock)
- [[Session 049 - POVM Hydration Analysis]] — Related: ORAC7 ID namespace mismatch

### Related Issues
- **ALERT-5 (Session 040):** Over-synchronisation — related to ghost sphere inflation of coupling calculations
- **BUG-034:** POVM write-only pathology — ORAC7 ID rotation prevents hydration (same root cause)
- **ALERT-6:** Learning-doing gap — 0 Hebbian edges despite 2,427 POVM pathways (ghost spheres never co-activate)

---

*Captured at tick 99,476 | 52 spheres (9 real, 43 ghost) | decision locked on HasBlockedAgents | 2026-03-21*
