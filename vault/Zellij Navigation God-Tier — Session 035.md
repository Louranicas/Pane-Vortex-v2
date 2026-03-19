---
date: 2026-03-16
tags: [zellij, navigation, fleet, dispatch, plugins, nvim, sync-tab, benchmarks, mastery, session-035]
aliases: [Zellij God-Tier, Fleet Dispatch Mastery, Session 035 Navigation]
---

# Zellij Navigation God-Tier — Session 035

> **Extends [[Zellij Pane Navigation Mastery — Session 027b]] with directional targeting, sync-tab broadcast, plugin pipes, verified dispatch, and nvim/LSP integration.**
> **Session**: auspicious-weasel | **Date**: 2026-03-16

---

## Evolution from Session 027b

| Capability | Session 027b | Session 035 |
|-----------|-------------|-------------|
| Pane targeting | focus-next-pane cycle | **Directional move-focus** (deterministic) |
| Broadcast | Manual 9-pane loop (725ms) | **toggle-active-sync-tab** (47ms) |
| Verification | PANE_ID grep | **dump-screen + Claude process grep** |
| Plugin comms | Not explored | **pipe --name to plugins** |
| nvim integration | Not explored | **--remote-send/expr via socket** |
| Dispatch time | 725ms/9 panes | **760ms with verification** |

---

## Pane Layout (unchanged)

```
+------------+----------------+
|            |   TOP-RIGHT    |
|   LEFT     +----------------+
|            |  BOTTOM-RIGHT  |
+------------+----------------+
```

---

## Tab Navigation (all ~14ms)

| Method | Command | Best for |
|--------|---------|----------|
| By index | `zellij action go-to-tab N` | Known tab number |
| By name | `zellij action go-to-tab-name "NAME"` | Readable scripts |
| Next/prev | `go-to-next-tab` / `go-to-previous-tab` | Sequential traversal |
| Full circuit | Loop `go-to-tab 1..6` | Verification sweeps |

**Benchmark**: 6-tab circuit = 89ms (14.8ms/tab)

---

## Directional Pane Targeting (KEY IMPROVEMENT)

**Replaces focus-next-pane cycling.** Directional moves are deterministic regardless of current focus.

| Target | Command | Speed |
|--------|---------|-------|
| LEFT | `move-focus left` (2x if unsure) | ~11ms |
| TOP-RIGHT | `move-focus right; move-focus up` | ~22ms |
| BOT-RIGHT | `move-focus right; move-focus down` | ~22ms |

### Why directional beats cycling

- `focus-next-pane` order: BOT-RIGHT -> LEFT -> TOP-RIGHT (must know current position)
- `move-focus` directional: idempotent, no state dependency, composable
- `move-focus left; move-focus left` always lands on LEFT pane regardless of start

---

## Cross-Tab Dispatch (Verified)

```bash
dispatch_to_pane() {
    local tab=$1 pane_dir=$2 prompt=$3

    # Navigate
    zellij action go-to-tab "$tab"
    case "$pane_dir" in
        left)     zellij action move-focus left; zellij action move-focus left ;;
        topright) zellij action move-focus right; zellij action move-focus up ;;
        botright) zellij action move-focus right; zellij action move-focus down ;;
    esac

    # Verify Claude is running
    zellij action dump-screen /tmp/dispatch-verify.txt
    if /usr/bin/grep -q "tokens\|Claude\|bypass" /tmp/dispatch-verify.txt; then
        zellij action write-chars "$prompt"
        zellij action write 13
        echo "dispatched"
    else
        echo "no Claude instance"
    fi

    # Return home
    zellij action go-to-tab 1
}
```

**Benchmark**: 9-pane verified dispatch = **760ms** (84ms/pane including verification)

---

## Broadcast Methods

### 1. Sync-Tab Broadcast (FASTEST — 47ms)

Sends same input to ALL panes in current tab simultaneously:

```bash
zellij action go-to-tab 4
zellij action toggle-active-sync-tab    # Enable sync
zellij action write-chars "echo hello"
zellij action write 13
zellij action toggle-active-sync-tab    # Disable sync
zellij action go-to-tab 1
```

**Use case**: Same command to all panes in one tab (e.g., health check)

### 2. Per-Pane Broadcast (458ms for 9 panes)

Navigate + write to each pane individually. Allows different commands per pane.

### 3. Plugin Pipe (async)

```bash
zellij action pipe --name "fleet-status" -- "PV r=0.929 spheres=12"
```

Sends data to all listening plugins asynchronously.

---

## Reading Pane Content Remotely

```bash
# Current screen only
zellij action dump-screen /tmp/pane-output.txt

# Full scrollback
zellij action dump-screen --full /tmp/pane-full.txt

# Pattern: go to pane, dump, return, read
zellij action go-to-tab 4
zellij action move-focus right; zellij action move-focus up
zellij action dump-screen /tmp/alpha-topright.txt
zellij action go-to-tab 1
tail -10 /tmp/alpha-topright.txt
```

---

## nvim + Zellij Integration

### zellij-nav.nvim (seamless pane traversal)

- `C-h` -> ZellijNavigateLeft (crosses nvim/zellij boundary)
- `C-j` -> ZellijNavigateDown
- `C-k` -> ZellijNavigateUp
- `C-l` -> ZellijNavigateRight

### Remote Control via Socket

```bash
# Open file
nvim --server /tmp/nvim.sock --remote-send ':e /path/to/file<CR>'

# Query LSP diagnostics count
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.tbl_count(vim.diagnostic.get(0))")'

# Get current buffer name
nvim --server /tmp/nvim.sock --remote-expr 'bufname("%")'

# List buffers
nvim --server /tmp/nvim.sock --remote-expr 'execute("ls")'

# Trigger LSP actions
nvim --server /tmp/nvim.sock --remote-send ':lua vim.lsp.buf.hover()<CR>'
nvim --server /tmp/nvim.sock --remote-send ':lua vim.diagnostic.setloclist()<CR>'
```

### nvim Keymaps (from RM)

580 custom keymaps: `<leader>z*` (zellij dispatch), `<leader>u*` (ULTRAPLATE services), `<leader>n*` (nexus), `<leader>y*` (yank), `<leader>o*` (quick-open), `<leader>g*` (git), `<leader>h*` (hunks)

---

## Zellij Plugin Reference

### Auto-loaded (background)

| Plugin | Function |
|--------|----------|
| zellij-autolock.wasm | Locks input when nvim/vim/lazygit/git focused |
| zellij-attention.wasm | Tab indicators when panes have new output |

### Keybind-activated (floating)

| Plugin | Keybind | Function |
|--------|---------|----------|
| harpoon.wasm | `Alt+v` | Terminal bookmarks |
| room.wasm | `Ctrl+y` | Session/pane quick-jump |
| ghost.wasm | `Alt+g` | Command launcher (bash -ic) |
| monocle.wasm | `Alt+m` | Fuzzy finder |
| multitask.wasm | `Alt+t` | Multi-pane broadcaster |
| swarm-orchestrator.wasm | `Alt+w` | Swarm orchestrator (quality_threshold=0.80) |

### Other available

| Plugin | Notes |
|--------|-------|
| swarm-orchestrator-v2.wasm | v2 loaded alongside v1 |
| zellij-send-keys.wasm | Key forwarding for autolock |
| zjstatus.wasm | Custom status bar (conflicts with default tab-bar) |

### Plugin Communication

```bash
# Pipe to all listening plugins
zellij action pipe --name "event-name" -- "payload data"

# Pipe to specific plugin
zellij action pipe --name "broadcast" \
    --plugin "file:~/.config/zellij/plugins/multitask.wasm" \
    -- "message"
```

---

## Complete Command Reference

| Command | Purpose | Speed |
|---------|---------|-------|
| `go-to-tab N` | Switch to tab by index | 14ms |
| `go-to-tab-name "NAME"` | Switch to tab by name | 14ms |
| `go-to-next-tab` / `go-to-previous-tab` | Sequential tab nav | 14ms |
| `move-focus left/right/up/down` | Directional pane focus | 11ms |
| `move-focus-or-tab left/right` | Cross pane/tab boundary | 15ms |
| `focus-next-pane` / `focus-previous-pane` | Cycle panes | 11ms |
| `write-chars "text"` | Type text into pane | instant |
| `write 13` | Send Enter key | instant |
| `dump-screen PATH` | Capture pane content | ~5ms |
| `dump-screen --full PATH` | Capture with scrollback | ~10ms |
| `toggle-active-sync-tab` | Broadcast mode on/off | instant |
| `toggle-floating-panes` | Show/hide floating panes | instant |
| `new-pane --floating --name X` | Quick floating pane | 21ms |
| `new-pane --direction right/down` | Split pane | ~15ms |
| `pipe --name X -- DATA` | Send data to plugins | async |
| `rename-tab NAME` | Label tab | instant |
| `rename-pane NAME` | Label pane | instant |
| `close-pane` / `close-tab` | Remove pane/tab | instant |
| `toggle-fullscreen` | Fullscreen focused pane | instant |
| `toggle-pane-embed-or-floating` | Float/embed toggle | instant |
| `query-tab-names` | List all tab names | ~5ms |
| `dump-layout` | Full layout structure | ~10ms |
| `launch-plugin --floating URL` | Launch plugin | ~20ms |

---

## Benchmarks Summary (Session 035)

| Operation | Time |
|-----------|------|
| Single tab switch | 14ms |
| 6-tab full circuit | 89ms |
| Single pane focus (directional) | 11ms |
| Cross-tab pane dispatch + return | 75-82ms |
| 9-pane manual broadcast | 458ms |
| Sync-tab broadcast (all panes) | 47ms |
| 9-pane VERIFIED dispatch | 760ms |
| Floating pane launch | 21ms |

---

## Anti-Patterns

1. **Never use focus-next-pane for targeting** — use directional move-focus instead
2. **Never send commands without verifying** — dump-screen + grep for Claude first
3. **Never chain after pkill** — exit 144 kills the chain
4. **Always return to tab 1** after cross-tab dispatch (home base)
5. **Never use sleeps between zellij commands** — they are synchronous, no sleep needed
6. **Never launch Claude -p one-shot for fleet agents** — launch interactive instances first, then dispatch prompts

---

## Synth DevEnv Tab Layout

| Tab | Name | Panes | Tools |
|-----|------|-------|-------|
| 1 | Orchestrator | Command, PV-Monitor, Health-Watch | field-monitor.sh, dashboard.sh |
| 2 | Workspace-1 | Atuin, Yazi, btm | Exploration deck |
| 3 | Workspace-2 | Bacon, Lazygit, Nvim | Development deck |
| 4 | Fleet-ALPHA [IDLE] | Left, TopRight, BotRight | Claude Code x3 |
| 5 | Fleet-BETA [IDLE] | Left, TopRight, BotRight | Claude Code x3 |
| 6 | Fleet-GAMMA [IDLE] | Left, TopRight, BotRight | Claude Code x3 |

---

## Service Discovery (Session 035)

### SAN-K7 Nexus Commands (POST /api/v1/nexus/command)

| Command | Params | Response |
|---------|--------|----------|
| service-health | {} | 11 services healthy, uptime 99.5-99.9% |
| synergy-check | {target:"all"} | Module M45, status executed |
| module-status | {} | 45 modules healthy, 0 degraded |
| build | {target:"pane-vortex"} | 2 artifacts, 0 errors, 4500ms |
| compliance | {} | OWASP 9.5, zero unsafe/unwrap |
| pattern-search | {pattern:"kuramoto"} | 10 results, L1-L4, 11 tensor dims |
| memory-consolidate | {} | Module M2, 10 results |
| best-practice | {topic:"concurrency"} | M44, confidence 0.95 |

### SYNTHEX V3 Thermal State

- Temperature: 0.569 (target 0.5) — WARM
- Heat sources: Hebbian=0.984, Cascade=0.0, Resonance=0.619, CrossSync=1.0
- PID output: 0.134 (slight cooling needed)
- Diagnostics: overall_health=0.75, Synergy probe CRITICAL (0.15 vs 0.7 threshold)

### PV Nexus Bridge

- Strategy: Aligned, combined coherence: 0.990
- r_inner=0.989, r_outer=0.991 (nested Kuramoto in sync)
- Fleet mode: Medium, dispatch confidence: 0.990

---

## Links

- [[Zellij Pane Navigation Mastery — Session 027b]] — Original benchmarks (725ms broadcast)
- [[Zellij Synthetic DevEnv — Session 027]] — Initial environment setup
- [[Session 031 — Zellij Synthetic DevEnv Exploration]] — Plugin discovery
- [[Swarm Orchestrator — Zellij Plugin]] — Custom WASM plugin
- [[Pane-Vortex — Fleet Coordination Daemon]] — Kuramoto field coordination
- [[Harpoon Plugin]] — Terminal bookmarks
- [[Session 019 — Zellij Environment Exploration]] — Early zellij work

---

*Generated 2026-03-16 by Claude Opus 4.6 (1M context) | Session 035 — auspicious-weasel*
*10 drills | 9-pane verified dispatch 760ms | sync-tab 47ms | directional targeting | nvim LSP integration | 10 plugins | 16 services*
