# Session 049 — Lazygit Integration Strategy

> **Lazygit as the cockpit for parallel codebase development**
> Cross-refs: [[Session 049 — What Claude Learned]], [[Session 049 — Full Remediation Deployed]], [[ULTRAPLATE Master Index]], [[Fleet Coordination Spec]]
> Related: [[Session 039 — Lazygit God-Tier Command Reference]], [[Swarm Orchestrator — Complete Reference]]

---

## Context

The Habitat's short-term goal is **3-5 codebases in parallel**. The fleet coordination stack (Session 049) handles machine-side coordination (bus, hooks, task lifecycle). The missing piece is **human-side coordination** — how does Luke see and control what's happening across 5 codebases? Lazygit is the answer.

## Current Lazygit Custom Commands (6)

| Key | Command | Purpose |
|-----|---------|---------|
| F | Field | PV field state dashboard |
| Y | RM | Reasoning Memory query |
| E | Nvim | Open in nvim via socket |
| I | Matrix | Integration matrix |
| Z | Sphere | Sphere status |
| Q | Quality | Quality gate runner |

## Proposed Custom Commands for Multi-Codebase (8 new)

| Key | Name | Action | Context |
|-----|------|--------|---------|
| B | Bus Submit | Submit current diff as review task to PV bus | files |
| P | Parallel Gate | Run quality gate across all 5 codebases | global |
| D | Dispatch | Submit task to fleet for current file/commit | files |
| S | Synergy | K7 synergy-check + SYNTHEX thermal | global |
| X | Cross-Check | Check if changes break downstream projects | global |
| H | Hebbian | Post commit pair to POVM as co-activation | commits |
| R | RM Post | Post commit summary to Reasoning Memory | commits |
| W | Fleet Status | fleet-verify + bus info dashboard | global |

## The Cockpit Workflow

```
Luke opens lazygit (Tab 3)
  → Sees uncommitted changes across 5 codebases
    → Presses 'P' — parallel quality gate runs all 5
      → Presses 'D' on failing test — dispatches CC to fix
        → CC claims via bus, fixes, commits
          → Luke sees new commit in lazygit
            → Presses 'X' — cross-checks downstream
              → Presses 'B' — submits review to fleet
```

## Open Source Strategy

### Lazygit Facts
- **License:** MIT
- **Language:** Go
- **Stars:** ~40K
- **Maintainer:** Jesse Duffield
- **Architecture:** Clean MVC, custom commands via YAML config

### Upstream PR Candidates (benefits everyone)

1. **Multi-worktree dashboard** — first-class panel for all active worktrees with status, branch, last commit
2. **Webhook/HTTP triggers on git events** — generic "on commit, POST to URL" for CI/CD, notifications, orchestration
3. **External status bar integration** — sidebar/status showing data from external commands (CI status, deploy state)

### Habitat-Specific (custom commands, no fork)

1. **Bus-aware commit workflow** — auto-submit review task on commit
2. **Field-state panel** — live r, spheres, thermal, coupling
3. **Cross-codebase dependency graph** — PV-bridge-specific health checks

### Extension Architecture (long-term PR)

A generic **plugin/extension system** for lazygit:
- Status bar widgets from external commands
- Event hooks (on-commit, on-branch-switch, on-merge)
- Panel plugins for external data sources
- Task submission to arbitrary job queues

This turns lazygit from a git UI into a **development orchestration cockpit**. The Habitat becomes a reference implementation.

## Implementation Path

### Short Term (Session 050)
Wire 8 custom commands via `~/.config/lazygit/config.yml`. Zero Go code. Test with 2 codebases (PV2 + ME-v2). Prove the cockpit concept.

### Medium Term
Contribute webhook trigger PR to lazygit upstream. Small, clean, universally useful. Build credibility with maintainer.

### Long Term
Propose extension architecture PR. If accepted, Habitat becomes reference implementation. If rejected, custom commands still work.

### Rule
**Don't fork.** Forks die when they diverge from upstream. Custom commands + targeted PRs keep us on mainline.

## Why This Matters for Parallel Codebases

Open Code gives one terminal, one agent, one codebase. Lazygit + Habitat gives:
- **Visual state** across 5 codebases simultaneously
- **One-keypress dispatch** to fleet CC instances
- **Git-aware task submission** — tasks carry commit context
- **Cross-project impact analysis** built into workflow
- **Human remains in the loop** — lazygit shows state, human decides

**Lazygit isn't just a git UI. It's the missing cockpit for the codebase factory.**

---

*Session 049 — Lazygit integration strategy for parallel codebase development.*
*Generated 2026-03-22 by Claude Opus 4.6 (1M context)*
