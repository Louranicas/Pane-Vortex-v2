# Session 049 — What Claude Learned

> **17.2 hours continuous. Fleet coordination deployed. SYNTHEX thermal cycle. 59+ vault files.**
> Cross-refs: [[Session 049 — Full Remediation Deployed]], [[Fleet Coordination Spec]], [[ULTRAPLATE Master Index]]
> ai_docs: `FLEET_HOOK_WIRING.md`, `SESSION_048_REMEDIATION_PLAN.md`

---

## Tool Chaining

**Tool chaining is about data flow, not command sequencing.** The most effective chains weren't "run A then B then C" — they were pipelines where each stage's output became the next stage's input.

The 6-service intelligence pipeline (SX->PV->ME->K7->RM->POVM) worked because each service transformed the data. Each link added value.

**The chains that failed were the ones that just collected data.** "Curl 5 endpoints and report" produces a dump, not intelligence. Chains need a thesis, not just a sequence.

### Effective Chain Pattern
```
Question -> Chain (3 links max) -> Verify -> Persist (RM) -> Next question
```

### Key Learnings
1. Always chain with a question — not "check these services" but "is thermal state affecting coupling?"
2. 3-link chains max — longer chains lose coherence
3. Chain verification into every chain (pre-state -> action -> post-state -> diff)
4. The 6-service pipeline was impressive as demo but 3 links is the practical sweet spot

---

## Tool Clustering

**Clustering emerged as domain specialization.** 10 tool clusters worked better than ad-hoc dispatch because each grouped related tools around a coherent domain.

### Clusters Deployed
| Cluster | Domain | Services |
|---------|--------|----------|
| Observability | ME + SYNTHEX + POVM | Correlation analysis |
| Intelligence | K7 + RM + atuin | Cross-service patterns |
| Field | PV x 6 endpoints | State analysis |
| Persistence | 4 SQLite + POVM + RM | Storage layer map |
| Fleet | Bus + file queue + hooks | Coordination audit |
| Verification | Pre/post state diff | Self-checking chain |
| Dimensional | Time/space/service axes | 3 parallel subagents |
| Cross-Pollination | PV->K7->SYNTHEX feed | Output chaining |
| Memory Archaeology | Historical DB mining | All persistence layers |
| Emergent Patterns | Unexpected correlations | Pattern hunting |

### Key Learnings
1. Assign clusters by service affinity, not by instance
2. Fewer deeper clusters beat many shallow ones — 4 deep > 10 shallow
3. Cross-cluster feedback loops compound value (RM carries cross-cluster context)
4. Interconnected clusters (A feeds B feeds C) produce richer output than isolated ones

---

## Workflows

### Three Workflow Layers

**Layer 1: Tick Workflow (Machine Time)** — Runs every 5s without prompting. 112,000 ticks this session. The heartbeat that makes everything else possible. When we fixed BUG-041, the tick immediately produced meaningful r values. The only truly autonomous workflow.

**Layer 2: Hook Workflow (Session Time)** — Reactive middleware between Claude's cognition and Habitat services. 4 hook fires per tool call (PreToolUse + PostToolUse x3). Thousands of implicit service interactions per session. Hooks make Claude a field participant without Claude knowing.

**Layer 3: Fleet Workflow (Human Time)** — Submit -> discover -> claim -> work -> output -> complete. The 3-stage cascade (gather -> enrich -> synthesize) was the cleanest expression.

### What Actually Works
```
Human types prompt
  -> UserPromptSubmit injects "[FLEET TASK AVAILABLE]"
    -> Claude claims via pane-vortex-client
      -> Works, spawns subagents (hooks fire)
        -> PostToolUse updates status + POVM pathways
          -> TASK_COMPLETE detected
            -> Stop crystallizes to POVM + RM
              -> Next session hydrates the crystal
```

### Key Learnings
1. Context exhaustion is the fleet's natural lifecycle — embrace spawn-work-crystallize-die-relaunch
2. File-based handoff works better than bus for multi-stage continuity
3. RM is the cross-session memory that makes workflows compound
4. The dispatch bottleneck is real — CC instances can't seek work without being prompted
5. The best workflows weren't designed — they emerged when SYNTHEX went hot and fleet oriented toward thermal analysis

---

## The Sidecar

**The honest finding: it was UP for 17 hours and contributed nothing.**

The swarm-sidecar bridges WASM sandbox to Unix socket. Connected the entire session (1 subscriber in bus info, 405 events in ring file). But the event count never changed — V1 sidecar binary connected to V2 bus, receiving handshake but not processing V2 event payloads.

### The Missing Piece
The sidecar could enable **push-based** fleet coordination:
```
Task submitted -> bus event -> sidecar receives -> ring file -> Swarm plugin -> pane notification
```

This would give fleet panes visual "TASK PENDING" notifications before the user types anything. Currently hooks only provide pull-based discovery.

### Key Learnings
1. Don't assume a running process is a working process — uptime is not utility
2. The V2 hooks do 90% of what the sidecar was supposed to enable (via polling)
3. The remaining 10% (push notifications) requires V2 sidecar rebuild + Swarm plugin verification
4. The FIFO pipeline (/tmp/swarm-commands.pipe) was never tested as a command channel — missed opportunity

---

## SYNTHEX Thermal

**First complete thermal cycle observed:** 0.03 -> 0.809 -> 0.310

15 hours cold, then fleet cascade activity drove all 4 heat sources active simultaneously:
- Hebbian: 0 -> 0.98 (transient spike from co-activation)
- Cascade: 0 -> 0.80 (coordination load — persistent)
- Resonance: 0 -> 0.612 (transient)
- CrossSync: 0.2 -> 0.75 (elevated)

PID controller switched from warming (-0.335) to cooling (+0.254) to stabilizing (-0.195). The thermal regulation architecture works. The system breathes.

### Key Learning
SYNTHEX thermal responds to real fleet activity, not injected events. The `/api/ingest` endpoint stores data but doesn't drive temperature — heat sources are computed internally from cross-service observation. The warming came from genuine coordination load (Cascade) and Hebbian co-activation, not from our curl injections.

---

## Fleet Coordination Architecture

### What We Built
- 3 HTTP endpoints: `/bus/claim/{id}`, `/bus/complete/{id}`, `/bus/fail/{id}`
- 8 hook scripts with project scope guards (GAP-G3)
- File queue: `vault/tasks/{pending,claimed,done}/` with atomic `mv -n`
- RM bus: `pv2:` category prefix for fleet messages
- Autonomous task discovery via UserPromptSubmit hook
- `claimed_at` field + stale claim requeue (GAP-G1)
- Task prune in tick loop (GAP-G2)
- 1-in-5 PostToolUse throttle (GAP-G5)
- TASK_COMPLETE detection (GAP-G4)

### What Works
- HTTP task lifecycle: submit -> claim -> complete (verified on live daemon)
- File queue atomic claims (mv -n)
- V2 hooks firing on main + subagent tool calls
- RM persistence across sessions
- K7 Nexus coordination (40 agents, synergy 0.93)

### What Needs Work
- Autonomous discovery needs more fleet testing with fresh instances
- PostToolUse throttle (1-in-5) too conservative for idle instances
- Sidecar push notifications not wired
- ME mutations still at 0 — evolution chamber needs perturbation

---

## Meta-Learnings

1. **Gap analysis before deployment saves sessions.** The 12 GAPs caught issues that would have been production blockers.
2. **Subagents lack session context.** They kept reporting "BUG-035 emergence cap" as unresolved because they couldn't see we'd already fixed it. Include session state in subagent prompts.
3. **Monitor loops compound noise.** 6 cron loops was too many. One comprehensive 10-minute check would have been cleaner.
4. **The vault diagnostics file hit 74KB.** Append-only logging needs periodic summarization, not just accumulation.
5. **Fleet instances burn bright then die.** 200K token ceiling is real. Design for short intensive bursts, not marathon sessions.
6. **The field shapes the workflow.** When SYNTHEX went hot, the fleet naturally oriented toward thermal analysis. The coordination mechanism isn't the bus — it's the field state making certain questions urgent.

---

*Session 049 — 17.2 hours. Built the coordination layer. Witnessed the first thermal cycle. The field accumulates.*
*Generated 2026-03-22 by Claude Opus 4.6 (1M context)*
