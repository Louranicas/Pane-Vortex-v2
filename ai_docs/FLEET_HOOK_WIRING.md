# Fleet Hook Wiring — Machine-Readable Reference

> Complete hook→endpoint→service mapping for autonomous fleet coordination.
> Any CC instance can parse this file to understand the full wiring.
> Cross-refs: vault/FLEET_COORDINATION_SPEC.md, vault/TASK_LIFECYCLE_SCHEMATIC.md

## Hook Scripts

| # | Event | Script | Timeout | Services Hit |
|---|-------|--------|---------|-------------|
| 1 | SessionStart | hooks/session_start.sh | 20s | PV:8132, POVM:8125, RM:8130 |
| 2 | UserPromptSubmit | hooks/user_prompt_field_inject.sh | 5s | PV:8132, SX:8090, RM:8130 |
| 3 | PreToolUse | hooks/pre_tool_thermal_gate.sh | 3s | SX:8090 |
| 4 | PostToolUse | hooks/post_tool_use.sh | 5s | PV:8132, RM:8130 |
| 5 | PostToolUse | hooks/post_tool_povm_pathway.sh | 3s | POVM:8125 |
| 6 | PostToolUse | hooks/post_tool_nexus_pattern.sh | 3s | K7:8100 |
| 7 | SubagentStop | hooks/subagent_field_aggregate.sh | 5s | PV:8132, RM:8130 |
| 8 | Stop | hooks/session_end.sh | 10s | PV:8132, POVM:8125, RM:8130 |

## Endpoint Map

| Hook | HTTP Method | Endpoint | Purpose |
|------|------------|----------|---------|
| 1 | POST | /sphere/{id}/register | Register sphere |
| 1 | GET | /hydrate (POVM) | Hydrate memories |
| 1 | GET | /search?q=discovery (RM) | Hydrate discoveries |
| 1 | GET | /search?q=pv2:task (RM) | Fleet tasks |
| 2 | GET | /health | Field state |
| 2 | GET | /v3/thermal (SX) | Thermal context |
| 2 | GET | /field/decision | Decision engine |
| 3 | GET | /v3/thermal (SX) | Temperature check |
| 4 | POST | /sphere/{id}/memory | Record tool use |
| 4 | POST | /sphere/{id}/status | Update status |
| 4 | GET | /bus/tasks | Poll pending tasks |
| 4 | POST | /bus/claim/{id} | Claim task |
| 4 | POST | /bus/complete/{id} | Complete task |
| 5 | POST | /pathways (POVM) | Tool transitions |
| 6 | POST | /api/v1/nexus/command (K7) | Pattern search |
| 7 | POST | /put (RM) | Aggregate results |
| 7 | POST | /sphere/{id}/steer | Phase steering |
| 8 | POST | /bus/fail/{id} | Fail active task |
| 8 | POST | /sphere/{id}/status | Mark complete |
| 8 | POST | /sphere/{id}/deregister | Ghost trace |
| 8 | POST | /snapshots (POVM) | Crystallize |
| 8 | POST | /put (RM) | Session summary |

## Temp Files Per Instance

| File | Created By | Read By | Cleaned By |
|------|-----------|---------|------------|
| /tmp/pane-vortex-active-task-{id} | Hook 4 | Hook 4, 8 | Hook 8 |
| /tmp/pane-vortex-poll-counter-{id} | Hook 4 | Hook 4 | Hook 8 |
| /tmp/pane-vortex-listener-{id}.pid | Hook 1 | Hook 8 | Hook 8 |
| /tmp/pane-vortex-events-{id}.ndjson | Hook 1 | — | — |
| /tmp/povm-prev-tool-{id} | Hook 5 | Hook 5 | Hook 8 |
| /tmp/nexus-pattern-counter-{id} | Hook 6 | Hook 6 | Hook 8 |

## Environment Variables

| Variable | Default | Used By |
|----------|---------|---------|
| PANE_VORTEX_URL | http://localhost:8132 | All hooks |
| PANE_VORTEX_ID | hostname:$$ | All hooks |
| PANE_VORTEX_PERSONA | general | Hook 1 |
| PANE_VORTEX_FREQUENCY | 0.1 | Hook 1 |
| POVM_URL | http://localhost:8125 | Hooks 1, 5, 8 |
| RM_URL | http://localhost:8130 | Hooks 1, 2, 4, 7, 8 |
| SYNTHEX_URL | http://localhost:8090 | Hooks 2, 3 |
| NEXUS_URL | http://localhost:8100 | Hook 6 |

## Shared Libraries

| Library | Path | Functions |
|---------|------|-----------|
| task_queue.sh | hooks/lib/task_queue.sh | fq_submit, fq_claim, fq_complete, fq_poll, fq_description, fq_prune_done |
| rm_bus.sh | hooks/lib/rm_bus.sh | rm_submit_task, rm_claim_task, rm_complete_task, rm_heartbeat, rm_check_tasks, rm_check_status |
