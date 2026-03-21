# Session 049 — Codebase Discovery

**Date:** 2026-03-21

## Rust Source Files

**55 .rs files** in `src/`

## Hook Scripts (10 total)

| Script | Location |
|--------|----------|
| rm_bus.sh | hooks/lib/ |
| task_queue.sh | hooks/lib/ |
| post_tool_nexus_pattern.sh | hooks/ |
| post_tool_povm_pathway.sh | hooks/ |
| post_tool_use.sh | hooks/ |
| pre_tool_thermal_gate.sh | hooks/ |
| session_end.sh | hooks/ |
| session_start.sh | hooks/ |
| subagent_field_aggregate.sh | hooks/ |
| user_prompt_field_inject.sh | hooks/ |

## TASK_COMPLETE Detection

Found in 3 hook files:

- **user_prompt_field_inject.sh** — Injects "[FLEET TASK AVAILABLE]" with claim instructions and "Include TASK_COMPLETE when done"
- **post_tool_use.sh** — GAP-G4: Detects TASK_COMPLETE in tool output, auto-completes claimed tasks. Also injects TASK_COMPLETE instructions for new claims
- Pattern: `grep -q "TASK_COMPLETE"` in $TOOL_OUTPUT triggers completion flow

## RM Bus Categories (`pv2:` prefix)

All in `hooks/lib/rm_bus.sh`:

| Category | Purpose | Confidence | TTL |
|----------|---------|-----------|-----|
| pv2:task | Task submission | 0.9 | 3600s |
| pv2:claim | Task claim | 0.85 | 1800s |
| pv2:done | Task completion | 0.95 | 7200s |
| pv2:status | Status heartbeat | 0.9 | 300s |

Search endpoints: `/search?q=pv2:task` and `/search?q=pv2:status`

## Architecture Summary

- **55 Rust files** across 8 layers, 41 modules
- **10 hook scripts** (8 event hooks + 2 library modules)
- **TASK_COMPLETE** wired into prompt injection + post-tool detection
- **RM bus** uses 4 TSV categories with `pv2:` prefix

## Cross-References

- [[Session 049 — Master Index]]
- [[ULTRAPLATE Master Index]]
