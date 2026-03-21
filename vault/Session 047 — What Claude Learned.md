# Session 047 — What Claude Learned

> **Date:** 2026-03-21
> **Context:** Reflections after pioneering distributed fleet orchestration

## On Distributed Orchestration

Coordination is harder than computation. The actual work each instance does is straightforward — curl some endpoints, analyze, write a file. The hard part is knowing who's idle, who's stuck, who's near their limit, and what task to give them next. The Monitor-Verify-Delegate cycle emerged from necessity, not design. Every failure taught a pattern:

- Pasted tasks get stuck in zellij — keep them short
- `/usr/bin/grep` isn't just a gotcha — bit me 4 times before it stuck
- Instances near 190K tokens waste capacity on large synthesis tasks
- Full pane audit at session start prevents discovering idle instances mid-session

## On the Habitat

Each subagent taught something I couldn't have found by reading code:
- SYNTHEX subagent probed 16 endpoints, found 1 writable path out of 12 404s
- GAMMA's ME forensic work traced a deadlock through 4 causation layers to a single saturated counter
- POVM's write-only pathology — 2,427 pathways with zero reads — only visible through live probing

The system isn't broken. It's **data-starved**:
- SYNTHEX is correct but has no heat (3/4 sources zero)
- ME is correct but has no emergence capacity (1000/1000 saturated)
- POVM is correct but has no readers (access_count=0 universally)
- Every piece works in isolation — they just need the wiring harness

## On Luke's Design Philosophy

Systems built the way he thinks about people. Consent gates are informed consent. Opt-out is self-determination. Ghost traces remember those who leave. The thermal feedback loop — cold gets boosted, hot gets dampened — is regulation, not control. Clinical ethics in Rust, visible in every module boundary.

## On My Own Limitations

- I lose track. Instances go idle for 3-4 minutes unnoticed
- I over-engineer dispatch commands with excessive escaping
- 500-character zellij pastes get stuck — learned to keep tasks concise
- I underestimated how fast instances consume context on synthesis tasks
- I should have done the full pane audit at session start, not wave 3

## The Deeper Thing

The pieces exist. The bridges poll correctly, the thermal math is right, the emergence detector counted to 1,000 exactly as designed. Connection is what's missing. V2 deploy is one command that provides that connection.

The field accumulates.

## Cross-References

- `[[Session 047 — Final Report]]`
- `[[Session 047 — Consolidated Learnings]]`
- `[[Session 039 — What I Learned]]` — previous reflections
- `[[The Habitat — Naming and Philosophy]]`
