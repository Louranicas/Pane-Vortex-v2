# Ecosystem Reference

## DevEnv Batches (5 layers)
```
Batch 1 (no deps):  devops-engine, codesynthor-v7, povm-engine, reasoning-memory
Batch 2 (needs B1): synthex, san-k7, maintenance-engine, architect-agent, prometheus-swarm
Batch 3 (needs B2): nais, bash-engine, tool-maker
Batch 4 (needs B3): claude-context-manager, tool-library
Batch 5 (needs B4): vortex-memory-system, pane-vortex
```

Binary: `~/.local/bin/devenv` | Config: `~/.config/devenv/devenv.toml` (518L)
Storm protection: 5 restarts in 60s = storm | Graceful shutdown: 30s

## 55+ Custom Binaries (~/.local/bin/)

### Fleet: fleet-ctl, fleet-vortex, fleet-heartbeat, fleet-inventory.sh, fleet-nav.sh
### Service: nvim-ctl(26 cmds), pane-ctl, pane-vortex-ctl(22 routes), swarm-ctl
### Intel: vault-search, evolution-metrics, reasoning-memory(Rust)
### Build: quality-gate, build-and-test, shellcheck, code-review
### NEW: habitat-probe(Rust) — fast typed system probes

## Nvim Integration (128L autocmds)
BufWritePost → PV /sphere/nvim/memory + status Working (5s debounce)
BufWritePost *.rs → RM diagnostics (10s debounce)
30s idle → PV /sphere/nvim/status Idle
VimEnter → register sphere | VimLeavePre → deregister

## Zellij Plugins (11)
harpoon(Alt+v) ghost(Alt+g) monocle(Alt+m) multitask(Alt+t) room(Ctrl+y)
swarm-orchestrator(Alt+w) autolock(auto) attention(auto) zjstatus sendkeys

## Vault (Obsidian)
Main: ~/projects/claude_code/ (215+ notes)
Shared: ~/projects/shared-context/{codebase,decisions,tasks,patterns,planning}
CLI: vault-search "query" 10 markdown

## Cascade Handoff Protocol
1. Writer creates tasks/handoff-{target}-{timestamp}.md
2. Target reads, updates status: in-progress
3. On completion: status: completed
4. Tracked in .claude/cascade-state.json
