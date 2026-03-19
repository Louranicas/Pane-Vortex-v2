---
date: 2026-03-17
tags: [zellij, skills, bootstrap, fleet, navigation, mastery, session-035, god-tier]
aliases: [Zellij Master Skill, Bootstrap Reference, God-Tier Bootstrap]
---

# Zellij Master Skill -- Bootstrap Reference

> **One command to god-tier: `/zellij-master`**
> Created Session 035 (auspicious-weasel) | 2026-03-17

---

## The Problem

Each new Claude Code context window starts fresh. Navigation patterns, service maps, plugin keybinds, dispatch protocols, anti-patterns -- all lost. Previous sessions required 10+ minutes of re-reading memory files and re-discovering the environment.

## The Solution

A single skill at `.claude/skills/zellij-master/SKILL.md` that loads the entire synth devenv knowledge base on demand. Type `/zellij-master` and Claude instantly knows:

- 6-tab layout with exact pane positions
- Directional navigation (move-focus, not focus-next-pane)
- Verified fleet dispatch protocol (dump-screen before send)
- Sync-tab broadcast (47ms to all panes)
- All 16 ULTRAPLATE services with ports, health paths, key endpoints
- 8 SAN-K7 nexus commands
- nvim remote socket commands (LSP, buffers, diagnostics)
- 11 Zellij plugin keybinds
- 7 anti-patterns that cause recurring bugs
- Quick start commands for services + PV sphere registration

## Full Skill Inventory (8 skills)

| Skill | Type | Lines | Purpose |
|-------|------|-------|---------|
| /zellij-master | Bootstrap | 140 | Load everything at session start |
| /fleet-dispatch | Task | 74 | Cross-tab verified command delivery |
| /pane-navigate | Background | 51 | Deterministic tab/pane navigation |
| /service-probe | Task | 75 | ULTRAPLATE health + nexus commands |
| /cascade-handoff | Task | 78 | Inter-tab task distribution |
| /nvim-lsp | Task | 76 | Remote neovim + LSP control |
| /field-monitor | Task | 75 | PV Kuramoto field monitoring |
| /plugin-launcher | Task | 78 | Zellij plugin management |

## Key Design Decisions

1. **pane-navigate** uses `user-invocable: false` -- background knowledge only, loads automatically when navigation is needed
2. **zellij-master** is the only skill needed at session start -- it references all others
3. Skills are project-level (`.claude/skills/`) so they auto-discover in pane-vortex
4. Descriptions follow Anthropic's pattern: [What it does] + [When to use it] + [Trigger phrases]
5. Progressive disclosure: descriptions always in context, full body loads on invoke

## Benchmarks (Session 035)

| Operation | Time |
|-----------|------|
| Tab switch | 14ms |
| Pane focus (directional) | 11ms |
| Cross-tab dispatch + return | 75-82ms |
| Sync-tab broadcast | 47ms |
| 9-pane verified dispatch | 760ms |
| Floating pane launch | 21ms |

## Test Results (all PASS)

All 7 original skills + zellij-master tested and verified:
- field-monitor: PV health, decision, nexus metrics
- service-probe: 16-port health (fixed ME/SYNTHEX paths), nexus commands
- pane-navigate: 6-tab circuit 98ms, all 3 pane positions reachable
- fleet-dispatch: 3-pane verified dispatch 276ms
- cascade-handoff: brief write + delivery + RM post
- nvim-lsp: socket, file open, LSP diagnostics, buffer list
- plugin-launcher: harpoon, room, toggle, pipe, 11 plugins

## Links

- [[Session 035 -- Synth DevEnv Mastery and Skills]] -- Full session notes
- [[Zellij Navigation God-Tier -- Session 035]] -- Navigation benchmarks
- [[Zellij Pane Navigation Mastery -- Session 027b]] -- Original benchmarks
- [[Pane-Vortex -- Fleet Coordination Daemon]] -- Project reference

---

*Generated 2026-03-17 by Claude Opus 4.6 (1M context) | Session 035*
