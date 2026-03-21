# Session 049 — Hook Pipeline Audit

**Date:** 2026-03-21 | **Bus Task:** 242dbd89 | **Method:** 3 parallel Explore subagents

## 1. Safety Audit (All 8 Hook Scripts)

| Hook | LOC | Scope Guard | Timeout | Error Handling | Throttle |
|------|-----|:-----------:|:-------:|:--------------:|:--------:|
| session_start.sh | 60 | Yes | Yes (2s) | Yes | Yes (retry backoff) |
| session_end.sh | 67 | Yes | Yes | Yes | No (fires once) |
| post_tool_use.sh | 100 | Yes | Yes (1-2s) | Yes | Yes (1-in-5) |
| pre_tool_thermal_gate.sh | 29 | Yes | Yes | Yes | No (by design) |
| post_tool_nexus_pattern.sh | 30 | Yes | Yes | Yes | Yes (1-in-10) |
| post_tool_povm_pathway.sh | 40 | Yes | Yes | Yes | No |
| subagent_field_aggregate.sh | 45 | Yes | Yes | Yes | No (single fire) |
| user_prompt_field_inject.sh | 52 | Yes | Yes | Yes | Yes (length guard) |
| lib/task_queue.sh | 67 | **No** | **No** | Yes | No |
| lib/rm_bus.sh | 46 | **No** | Yes (1s) | Yes | No |

**All 8 main hooks have scope guards** (GAP-G3: `[[ "$(pwd)" != "$PV2_DIR"* ]] && exit 0`).
Library scripts lack guards — rely on parent scripts.

### Risk: MEDIUM — `lib/task_queue.sh` has no scope guard or timeout on file ops.

## 2. PostToolUse Task Path (End-to-End Trace)

```
Tool output contains "TASK_COMPLETE"
    │
    ├─ grep detects substring (case-sensitive, no regex)
    ├─ POST /bus/complete/{TASK_ID} (max-time 2s)
    ├─ rm_complete_task() → RM TSV (pv2:done, confidence 0.95)
    └─ rm -f ACTIVE_TASK_FILE
    
No active task? → Throttle check (1-in-5)
    │
    ├─ GET /bus/tasks → filter Pending → first task
    ├─ POST /bus/claim/{TASK_ID} (atomic, first-wins)
    ├─ echo TASK_ID > ACTIVE_TASK_FILE
    ├─ rm_claim_task() → RM TSV (pv2:claim, confidence 0.85)
    └─ Emit systemMessage: "[FLEET TASK] Claimed {id}: {desc}"
    
HTTP empty? → File fallback
    │
    ├─ fq_poll() → ls vault/tasks/pending/ | head -1
    ├─ fq_claim() → mv -n (kernel-atomic, race-safe)
    └─ Emit systemMessage: "[FILE TASK] {desc}"
```

### Key Design Patterns

| Pattern | Implementation |
|---------|---------------|
| Detection | Substring grep for "TASK_COMPLETE" |
| HTTP claim | Single-winner via `BusTask.claim()` status check |
| File claim | Atomic `mv -n` (POSIX race-safe) |
| Throttle | 1-in-5 counter file in /tmp |
| RM broadcast | TSV only: pv2:claim (0.85) / pv2:done (0.95) |
| Dual injection | UserPromptSubmit (proactive) + PostToolUse (reactive) |

## 3. SubagentStop Hook — NOT FIRING

**Script:** `subagent_field_aggregate.sh` (45 LOC)

**What it should do:**
1. Parse subagent type + output from stdin
2. Post `subagent-result` to RM (TSV)
3. Write sphere memory to PV2
4. Steer sphere phase based on subagent type (read→0, write→π/2, test→π, review→3π/2)

**Evidence it's NOT firing:**
- 0 `subagent-result` entries in RM (searched)
- 0 `field_aggregate` entries in RM (searched)
- No phase steering observed matching hook's hardcoded values
- Hook exists on disk but is **not registered in Claude Code's hook system**

**Root cause:** The hook file is scaffolded but not wired into `settings.json` or `~/.claude/hooks/`. It's a Session 047 planned feature awaiting deployment.

## Recommendations

| Priority | Action |
|----------|--------|
| **HIGH** | Wire `subagent_field_aggregate.sh` into settings.json as SubagentStop hook |
| **MEDIUM** | Add scope guard to `lib/task_queue.sh` |
| **LOW** | Document `pre_tool_thermal_gate.sh` intentionally has no throttle |

---
*Cross-refs:* [[Session 049 - Fleet Architecture]], [[Session 049 - System Architecture]], [[Fleet Coordination Spec]]
