# Fleet Coordination Specification

> Autonomous CC fleet coordination via hooks, file queue, and RM bus.
> Deployed Session 049. Plan: `.claude/plans/abstract-percolating-sutherland.md`
> Cross-refs: [[Session 049 — Full Remediation Deployed]], [[IPC Bus Architecture Deep Dive]], [[ULTRAPLATE Master Index]]

## Task Polling Protocol

1. PostToolUse fires on every tool call
2. Project scope guard exits if `pwd` is not PV2 (GAP-G3)
3. Status + memory updates fire-and-forget (background curl)
4. Active task TASK_COMPLETE detection (GAP-G4)
5. 1-in-5 throttle on polling (GAP-G5)
6. HTTP primary: `GET /bus/tasks` → filter Pending → `POST /bus/claim/{id}`
7. File fallback: `ls vault/tasks/pending/ | head -1` → `mv -n` atomic claim
8. RM hydration on SessionStart (cross-session tasks)

## API Endpoints (3 new)

| Method | Route | Purpose |
|--------|-------|---------|
| POST | `/bus/claim/{task_id}` | Atomic claim of pending task |
| POST | `/bus/complete/{task_id}` | Mark claimed task done |
| POST | `/bus/fail/{task_id}` | Mark claimed task failed |

## File Queue

```
vault/tasks/
  pending/   — task files (oldest first)
  claimed/   — mv -n atomic claim
  done/      — completed (pruned after 7 days)
```

Format: Markdown with YAML frontmatter (id, description, submitted_by, claimed_by, target).

## RM Message Bus

| Category | TTL | Purpose |
|----------|-----|---------|
| `pv2:task` | 3600s | Cross-session task |
| `pv2:claim` | 1800s | Claim broadcast |
| `pv2:done` | 7200s | Completion report |
| `pv2:status` | 300s | Heartbeat |

## Hook Scripts (8)

| Hook | File | Timeout | Purpose |
|------|------|---------|---------|
| SessionStart | session_start.sh | 20s | Register, hydrate, RM announce |
| UserPromptSubmit | user_prompt_field_inject.sh | 5s | Field state injection |
| PreToolUse | pre_tool_thermal_gate.sh | 3s | SYNTHEX thermal gate |
| PostToolUse | post_tool_use.sh | 5s | Task polling + status |
| PostToolUse | post_tool_povm_pathway.sh | 3s | Hebbian transitions |
| PostToolUse | post_tool_nexus_pattern.sh | 3s | SAN-K7 patterns |
| SubagentStop | subagent_field_aggregate.sh | 5s | RM aggregate + phase steering |
| Stop | session_end.sh | 10s | Fail task, crystallize, deregister |

## GAP Mitigations

- **G1** claimed_at + stale requeue (300s timeout)
- **G2** prune_completed_tasks in tick loop (3600s)
- **G3** Project scope guard on all hooks
- **G4** TASK_COMPLETE pattern detection
- **G5** 1-in-5 PostToolUse throttle
- **G6** Submitter validation
- **G7** pv2: RM category prefix
- **G8** done/ prune + .gitignore
- **G9** HTTP > file > RM dedup hierarchy
- **G10** V1→V2 transition procedure

## Rollback

```bash
\cp -f ~/.claude/settings.json.bak ~/.claude/settings.json
```

See also: [[Consent Flow Analysis]], [[The Habitat — Naming and Philosophy]]
