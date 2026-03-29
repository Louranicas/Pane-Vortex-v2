# Top 10 Hook Points (Subagent Analysis)

1. UserPromptSubmit — field state injection (20-30% recon reduction)
2. SessionStart — sphere registration + IPC bus (40-60s saved per agent)
3. PostToolUse — POVM pathway recording (100% coverage vs 30%)
4. PreToolUse — safety gate (blocks 7 anti-patterns)
5. Stop — sphere deregister + session crystallization
6. SubagentStop — cascade result aggregation (2-3x faster)
7. PreCompact — handoff context serialization
8. PostToolUse Extended — auto arena file generation (3x volume)
9. UserPromptSubmit Extended — consensus check before destructive ops
10. PostToolUse Extended — cross-service correlation recording

Expected: 40-50% workflow automation.
