# Task Lifecycle Schematic

> State machine, edge cases, and prune timing for IPC bus tasks.
> Cross-refs: [[Fleet Coordination Spec]], [[IPC Bus Architecture Deep Dive]], [[Session 049 — Full Remediation Deployed]]

## State Machine

```
PENDING ──claim──► CLAIMED ──complete──► COMPLETED ──prune(3600s)──► [REMOVED]
   ▲                  │
   └──requeue(300s)───┘      ──fail──► FAILED ──prune(3600s)──► [REMOVED]
```

## Fields

| Field | Type | Set When |
|-------|------|----------|
| id | TaskId | submit |
| description | String | submit |
| target | TaskTarget | submit |
| status | TaskStatus | transitions |
| submitted_by | PaneId | submit |
| claimed_by | Option | claim |
| submitted_at | f64 | submit |
| claimed_at | Option | claim (GAP-G1) |
| completed_at | Option | complete/fail |

## Prune Timing (tick loop)

- `prune_completed_tasks(3600.0)` — remove terminal tasks older than 1 hour
- `prune_stale_claims(300.0)` — requeue claimed tasks stuck longer than 5 min

## Edge Cases

| Scenario | Handling |
|----------|----------|
| Claimed sphere crashes | Stale claim requeued after 300s |
| Double claim attempt | Second claim returns 409 (not pending) |
| Complete without claim | Returns 400 (not claimed) |
| 256 pending cap | Submit returns error |
| Unknown task_id | Returns 404 |
| Submitter validation | PaneId regex enforced (GAP-G6) |

## Integration Test

```bash
TASK_ID=$(curl -s -X POST localhost:8132/bus/submit \
    -H 'Content-Type: application/json' \
    -d '{"description":"test","submitter":"test-sphere","target":"any_idle"}' | jq -r '.task_id')
curl -s -X POST "localhost:8132/bus/claim/$TASK_ID" \
    -H 'Content-Type: application/json' -d '{"claimer":"test-sphere"}'
curl -s -X POST "localhost:8132/bus/complete/$TASK_ID" \
    -H 'Content-Type: application/json' -d '{}'
```
