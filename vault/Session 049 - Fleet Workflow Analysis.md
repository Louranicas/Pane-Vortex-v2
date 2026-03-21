# Session 049 — Fleet Workflow Analysis

> **2 workflows mapped: Autonomous Task Discovery + Cascade Dispatch**
> **Task discovery: FULLY WIRED. Cascade: 6 gaps identified.**
> **Captured:** 2026-03-21

---

## Workflow 1: Autonomous Task Discovery

```mermaid
sequenceDiagram
    participant Orch as Orchestrator
    participant Bus as PV2 Bus :8132
    participant UPS as UserPromptSubmit Hook
    participant Claude as Claude Instance
    participant PTU as PostToolUse Hook
    participant RM as RM :8130

    Orch->>Bus: POST /bus/submit {description, target}
    Note over Bus: Task status: Pending

    UPS->>Bus: GET /bus/tasks (on every prompt)
    Bus-->>UPS: [{id, description, status:"Pending"}]
    UPS-->>Claude: "[FLEET TASK] 1 pending: <description>"

    Claude->>Claude: Works, uses tools...
    PTU->>PTU: 1-in-5 throttle check
    PTU->>Bus: GET /bus/tasks
    Bus-->>PTU: [pending tasks]
    PTU->>Bus: POST /bus/claim/{id} (ATOMIC)
    Bus-->>PTU: {"status":"Claimed"}
    PTU->>PTU: Write /tmp/pane-vortex-active-task-{id}
    PTU-->>Claude: "[FLEET TASK] Claimed: <description>"

    Claude->>Claude: Works on task...
    Note over Claude: Tool output contains "TASK_COMPLETE"

    PTU->>PTU: grep -q "TASK_COMPLETE" in output
    PTU->>Bus: POST /bus/complete/{id}
    PTU->>RM: PUT /put (pv2:done, TSV)
    PTU->>PTU: rm /tmp/pane-vortex-active-task-{id}
    Note over Bus: Task status: Completed
```

### Status: FULLY WIRED AND VERIFIED

| Step | Code | Tested |
|------|------|--------|
| Task submission | m10_api_server.rs:1071-1090 | Yes |
| Pending filter | user_prompt_field_inject.sh:33 | Yes |
| System message injection | user_prompt_field_inject.sh:43-47 | Yes |
| Atomic claim | m30_bus_types.rs:140-148 | Yes |
| TASK_COMPLETE detection | post_tool_use.sh:42 | Yes |
| Task completion | m10_api_server.rs:1199-1230 | Yes |
| File queue fallback | hooks/lib/task_queue.sh | Yes |
| Session-end failure recovery | session_end.sh:18-24 | Yes |

**Known limitations:** 1-in-5 polling throttle (4 tool uses delay), TASK_COMPLETE must be in stdout, no task prioritization.

---

## Workflow 2: Cascade Dispatch

```mermaid
sequenceDiagram
    participant Orch as Orchestrator
    participant API as API Server
    participant CT as CascadeTracker (M33)
    participant Bus as IPC Bus (M29)
    participant Target as Target Pane
    participant Hook as SubagentStop Hook
    participant RM as RM :8130

    Orch->>API: POST /bus/cascade {source, target, brief}
    API->>CT: initiate(source, target, brief)
    Note over CT: Rate limit: 10/min, max 50 pending
    CT->>Bus: publish "cascade.initiated"
    Bus-->>Target: BusFrame::Cascade via IPC socket

    Target->>Target: Executes work from brief
    Target->>Target: Spawns subagents if needed

    Hook->>RM: PUT /put (subagent-result, TSV)
    Hook->>API: POST /sphere/{id}/memory
    Hook->>API: POST /sphere/{id}/steer (phase)
    Note over Hook: Phase map: read→0, write→π/2, test→π, review→3π/2

    Target->>API: POST /bus/complete/{task_id}
    API->>Bus: publish "task.completed"
    Bus-->>Orch: Event subscription notification
```

### Status: PARTIALLY WIRED — 6 Gaps

| Gap | Issue | Impact |
|-----|-------|--------|
| GAP-A1 | Cascade ACK/REJECT endpoint missing from API routes | Target can't formally acknowledge receipt |
| GAP-A2 | Executor doesn't dispatch to Zellij | Rust daemon selects target but external agent must do zellij write-chars |
| GAP-A3 | SubagentStop hook assumes pane-vortex-client binary | No fallback if CLI missing |
| GAP-A4 | V1 sidecar bidirectional compat incomplete | V2 responses may confuse V1 clients |
| GAP-A5 | No auto-re-cascade on rejection | Manual operator intervention required |
| GAP-A6 | No callback URL in cascade request | Target doesn't know where to post results |

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Fleet Coordination Spec]] — task protocol and hook wiring
- [[Session 049 - Hook Pipeline Audit]] — hook safety analysis
- [[ULTRAPLATE Master Index]]
