# Session 049 — Subagent Exploration Report

**Date:** 2026-03-21 | **Method:** 2 parallel Explore subagents

## Agent 1: TODO/FIXME Scan

**Total: 5 TODO, 0 FIXME**

All TODOs are in `src/bin/scaffold.rs` — template placeholders for generated documentation:

| Line | TODO Text |
|------|----------|
| 955 | `TODO: Document {doc} during implementation.` |
| 991 | `TODO: Specify during implementation.` |
| 1053 | `TODO: Document during implementation.` |
| 1120 | `TODO: Expand with full route table during implementation.` |
| 1125 | `TODO: Add Mermaid diagrams during implementation.` |

**Verdict:** Clean codebase. All 5 TODOs are meta-template strings inside the scaffold generator, not actionable items in PV2 implementation code.

## Agent 2: API Route Map

**Total: 38 base + 5 governance = 43 routes**

| Group | Count | Methods |
|-------|-------|---------|
| Core | 3 | GET (health, spheres, ghosts) |
| Field | 8 | GET (field, r, decision, decisions, chimera, tunnels, k, spectrum) |
| Sphere CRUD | 7 | GET/POST (detail, register, deregister, accept-ghost, memory, status, heartbeat) |
| Sphere Advanced | 4 | GET/POST (neighbors, inbox, inbox/send, inbox/ack) |
| Gap Fix | 3 | POST/GET (phase, steer, suggestions) |
| Coupling | 2 | GET/POST (matrix, weight) |
| Bus | 9 | GET/POST (info, tasks, events, submit, claim, complete, fail, cascade, cascades) |
| Bridges | 1 | GET (health) |
| **Governance** (feature-gated) | **5** | POST/GET (propose, proposals, vote, consent, data-manifest) |

**Path parameters:** `{pane_id}`, `{task_id}`, `{proposal_id}`
**Router:** Axum `Router::new()` with `CorsLayer::permissive()`, lines 1581–1648

---
*Cross-refs:* [[Session 049 — Master Index]], [[Session 049 - Codebase Discovery]]
