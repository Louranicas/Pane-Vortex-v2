# Session 049 — Data Flow Verification (Subagent Analysis)

**Date:** 2026-03-21 | **Task:** 63ae7dd2

## Flow 1: Tool Use → POVM → Hebbian STDP → Coupling Weight

```mermaid
graph TD
    TU["CC Tool Use"] -->|"PostToolUse hook"| H1["post_tool_use.sh<br/>POST /sphere/{id}/status<br/>status=Working"]
    TU -->|"PostToolUse hook"| H2["post_tool_povm_pathway.sh<br/>POST :8125/pathways<br/>{source:PrevTool, target:CurTool, weight:0.5-0.8}"]

    H1 -->|"sphere.status = Working"| STDP["Phase 2.5: apply_stdp()<br/>m19_hebbian_stdp.rs"]
    STDP -->|"Both Working → LTP +0.01<br/>Not co-active → LTD -0.002"| CW["coupling_network.set_weight()<br/>m16_coupling_network.rs"]
    CW -->|"weight scales sin(Δphase)"| KUR["Kuramoto step_inner()<br/>d_phase computation"]

    H2 -->|"tool-pair data"| POVM["POVM :8125<br/>/pathways store"]
    POVM -->|"GET /pathways every 60 ticks"| BRIDGE["m25_povm_bridge.rs<br/>hydrate_pathways()"]
    BRIDGE -->|"cached_pathways"| GAP["⚠ GAP: Cached but<br/>NEVER applied to<br/>coupling weights"]

    style H1 fill:#4CAF50,color:#fff
    style STDP fill:#4CAF50,color:#fff
    style CW fill:#4CAF50,color:#fff
    style KUR fill:#4CAF50,color:#fff
    style H2 fill:#4CAF50,color:#fff
    style POVM fill:#FF9800,color:#fff
    style BRIDGE fill:#FF9800,color:#fff
    style GAP fill:#F44336,color:#fff
```

### Connected (working end-to-end)

| Stage | File | Status |
|-------|------|--------|
| Tool use → sphere status=Working | hooks/post_tool_use.sh | **CONNECTED** |
| Working status → STDP co-activity | m35_tick.rs Phase 2.5 | **CONNECTED** |
| STDP LTP/LTD → connection.weight | m19_hebbian_stdp.rs | **CONNECTED** |
| connection.weight → Kuramoto d_phase | m16_coupling_network.rs | **CONNECTED** |
| Hook → tool-pair → POVM Engine | hooks/post_tool_povm_pathway.sh | **CONNECTED** |
| POVM → bridge hydration | m25_povm_bridge.rs | **CONNECTED** |
| Cached pathways → coupling weights | NOWHERE | **GAP** |

### The Gap

`src/bin/main.rs:501-519` — `hydrate_pathways()` is called, pathways cached, but the `Vec<Pathway>` is logged and discarded. No code translates POVM tool→tool weights into sphere→sphere coupling weights. Semantic mismatch: pathways are `"Read"→"Edit"` but coupling is `"hostname:PID"→"hostname:PID"`.

---

## Flow 2: Prompt → Hook → Field Injection → Task Discovery → Claim

```mermaid
graph TD
    PROMPT["User Prompt"] -->|"UserPromptSubmit"| HOOK["user_prompt_field_inject.sh"]

    HOOK -->|"GET /health"| FIELD["r, tick, spheres"]
    HOOK -->|"GET :8090/v3/thermal"| TEMP["temperature"]
    HOOK -->|"GET /bus/tasks"| TASKS["pending tasks"]

    FIELD --> MSG["systemMessage injection:<br/>[FIELD] r=X tick=Y T=Z<br/>[FLEET TASK] N pending<br/>claim: pane-vortex-client claim ID"]
    TEMP --> MSG
    TASKS --> MSG

    MSG -->|"Claude reads injected context"| CLAUDE["Claude decides to claim"]

    CLAUDE -->|"runs tool"| PTU["PostToolUse fires<br/>post_tool_use.sh"]
    PTU -->|"1-in-5 throttle"| POLL["GET /bus/tasks<br/>filter Pending"]
    POLL -->|"POST /bus/claim/{id}"| CLAIM["Task Claimed"]
    CLAIM -->|"write /tmp/pane-vortex-active-task-{ID}"| TRACK["Active task tracked"]

    TRACK -->|"on TASK_COMPLETE in tool output"| COMPLETE["POST /bus/complete/{id}<br/>rm_complete_task() to RM<br/>delete active task file"]

    style HOOK fill:#2196F3,color:#fff
    style MSG fill:#4CAF50,color:#fff
    style CLAIM fill:#4CAF50,color:#fff
    style COMPLETE fill:#4CAF50,color:#fff
```

### Verified Connected

| Stage | File | Status |
|-------|------|--------|
| Prompt → hook fires | settings.json → user_prompt_field_inject.sh | **CONNECTED** |
| Hook → field + thermal + tasks | 3 curl calls | **CONNECTED** |
| Injection → Claude sees tasks | systemMessage output | **CONNECTED** |
| PostToolUse → auto-claim (1-in-5) | post_tool_use.sh | **CONNECTED** |
| TASK_COMPLETE → auto-complete | post_tool_use.sh grep | **CONNECTED** |
| RM mirroring (claim + complete) | hooks/lib/rm_bus.sh | **CONNECTED** |
| File queue fallback | hooks/lib/task_queue.sh | **CONNECTED** |

### Known Issues

1. **TASK_COMPLETE detection:** Greps `$TOOL_OUTPUT` not Claude's prose — if Claude says TASK_COMPLETE in text only, detection misses until next tool output contains it
2. **Doc mismatch:** FLEET_HOOK_WIRING.md lists `GET /field/decision` for hook 2, but actual script doesn't call it
3. **Project scope guard:** All hooks exit early if `pwd != pane-vortex-v2/` (GAP-G3 by design)

---

## Cross-References

- [[Session 049 - System Architecture]]
- [[Session 049 - Fleet Architecture]]
- [[IPC Bus Architecture Deep Dive]]
- [[ULTRAPLATE Master Index]]
