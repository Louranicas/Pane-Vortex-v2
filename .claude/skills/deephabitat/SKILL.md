---
name: deephabitat
description: Deep Habitat mastery — covers all 10 known gaps from Session 039 plus 15 new discoveries. Complete reference for yazi, btm, bacon, MCP, pipe protocol, autocmds, cross-DB, vault nav, shared-context, devenv batches, custom binaries, IPC wire protocol, SYNTHEX brain model, hebbian topology, and fleet orchestration patterns.
user_invocable: true
---

# /deephabitat — Deep Habitat Mastery

You have loaded **deep knowledge** beyond /primehabitat. This covers the substrate layer — the tools, databases, protocols, and cross-service tissue that /primehabitat references but doesn't explain.

## QUICK CARD

```
YAZI:     Tab 2 TopRight | z=zoxide Z=fzf -=parent CR=open gs=sort g.=hidden
BTM:      Tab 2 Bottom | CPU/mem/net/disk per-service, PID tracking, tree view
BACON:    Tab 3 Left | `bacon clippy` in PV dir | auto-recompile on save
MCP:      No .mcp.json configured — MCP servers are in-process Claude Code tools
PIPE:     /run/user/1000/pane-vortex-bus.sock | NDJSON wire, handshake+subscribe+submit
AUTOCMDS: 128L field-integrated: BufWritePost→PV+RM, idle 30s, VimEnter/Leave→sphere
CROSS-DB: 65+ SQLite DBs across ecosystem, 6 paradigms, SYNTHEX owns 15+ alone
VAULT:    ~/projects/claude_code/ (215 notes) + ~/projects/shared-context/ (decisions/codebase)
SHARED:   ~/projects/shared-context/{codebase,decisions,tasks,patterns,planning}
DEVENV:   5 batches, 18 registered, 16 active, exponential backoff, storm protection
BINARIES: 55+ custom at ~/.local/bin/ (fleet-ctl, nvim-ctl, swarm-ctl, vault-search, etc.)
```

---

## GAP 1: YAZI (File Navigator)

**Location:** Tab 2 TopRight | Config: `~/.config/yazi/yazi.toml`
**Version:** 25.5.31 | **Opener:** Helix (`hx`) default, xdg-open for non-text

### Keybindings
| Key | Action | Notes |
|-----|--------|-------|
| `z` | Zoxide jump | Fuzzy directory history |
| `Z` | fzf jump | Global fuzzy file search |
| `-` | Parent directory | Like oil.nvim |
| `CR` | Open file | Uses configured opener |
| `gs` | Change sort | natural/size/modified/name |
| `g.` | Toggle hidden | Show dotfiles |
| `~` | Go home | |
| `Space` | Select | Toggle file selection |
| `v` | Visual select | Range selection |
| `d` | Trash | Moves to trash (safe) |
| `y` | Yank | Copy to clipboard |
| `p` | Paste | |
| `r` | Rename | |

### Config Details
```toml
# ~/.config/yazi/yazi.toml
[mgr]
ratio = [1, 3, 4]           # sidebar:content:preview
sort_by = "natural"
sort_dir_first = true
linemode = "size"
show_hidden = false
show_symlink = true

[preview]
tab_size = 2
max_width = 600
max_height = 900
image_filter = "triangle"    # Image preview support
```

### Integration Points
- Zoxide database shared with shell (`z` command)
- Helix as default opener (not nvim) — edit `~/.config/yazi/yazi.toml` opener section to change
- Preview supports images, PDFs, and code with syntax highlighting
- Tab 2 placement puts it adjacent to atuin (shell history) for discovery workflows

---

## GAP 2: BTM (Bottom Process Monitor)

**Location:** Tab 2 Bottom (labeled "Htop" in layout but runs `btm`)
**Config:** `~/.config/bottom/bottom.toml` (defaults only)

### Key Capabilities
- Real-time CPU, memory, network, disk per-process monitoring
- Process tree view for service hierarchy
- Per-PID tracking (critical for devenv service management)
- Temperature monitoring (CPU thermal)
- Network bandwidth per-interface

### Usage for Habitat Monitoring
```bash
# Launch with service filter
btm --default_widget_type proc --regex_filter "pane-vortex|synthex|orchestrator|povm"

# Key bindings inside btm
# Tab = cycle widgets | / = search | t = tree view | s = sort
# dd = kill process | Enter = expand | Esc = back
```

### Integration Gap
- No ULTRAPLATE-specific btm config exists
- Could have custom widget groups for service batches
- No integration with PV health — btm runs standalone

---

## GAP 3: BACON (Continuous Compiler)

**Location:** Tab 3 Left
**Config:** No `bacon.toml` found in PV or global config

### Usage
```bash
# In PV directory
cd ~/claude-code-workspace/pane-vortex
bacon                     # default: cargo check
bacon clippy              # clippy warnings
bacon test                # cargo test
bacon doc                 # cargo doc

# Custom job (would go in bacon.toml)
# bacon.toml not yet created for PV
```

### What Bacon Provides
- Auto-recompile on file save (inotify-based)
- Error navigation with n/p keys
- Summary view with error counts
- Integrates with rust-analyzer (shares cargo check cache)

### Integration Gap
- No `bacon.toml` config for PV project — uses defaults
- No CARGO_TARGET_DIR set to `/tmp/cargo-pane-vortex` (standard PV pattern)
- Could have custom jobs for quality gate stages (check→clippy→pedantic→test)
- No integration with PV sphere status (could mark sphere as "compiling")

### Recommended bacon.toml for PV
```toml
# Would go at ~/claude-code-workspace/pane-vortex/bacon.toml
[jobs.check]
command = ["cargo", "check"]
env = { CARGO_TARGET_DIR = "/tmp/cargo-pane-vortex" }

[jobs.clippy]
command = ["cargo", "clippy", "--", "-D", "warnings"]
env = { CARGO_TARGET_DIR = "/tmp/cargo-pane-vortex" }

[jobs.pedantic]
command = ["cargo", "clippy", "--", "-D", "warnings", "-W", "clippy::pedantic"]
env = { CARGO_TARGET_DIR = "/tmp/cargo-pane-vortex" }

[jobs.test]
command = ["cargo", "test", "--lib", "--release"]
env = { CARGO_TARGET_DIR = "/tmp/cargo-pane-vortex" }
```

---

## GAP 4: MCP (Model Context Protocol)

**Status:** No `.mcp.json` files configured per-project. MCP servers are in-process Claude Code tools.

### Active MCP Servers (from Claude Code session)
| Server | Type | Capabilities |
|--------|------|-------------|
| `brave-search` | External | Web + local search |
| `fetch` | External | HTTP fetch |
| `filesystem` | External | File operations |
| `github` | External | GitHub API (issues, PRs, code) |
| `memory` | External | Knowledge graph (entities, relations) |
| `puppeteer` | External | Browser automation |
| `context7` | Plugin | Library documentation retrieval |

### Integration Gap
- No ULTRAPLATE-specific MCP server exists
- Could have MCP server wrapping PV API (sphere ops, field queries)
- Could have MCP server wrapping SAN-K7 nexus commands
- Could have MCP server for RM (reasoning memory) queries
- VMS (port 8120) has 47 MCP tools defined but no MCP server adapter

### Potential MCP Integrations
1. **PV-MCP**: Wrap `/spheres`, `/field/decision`, `/bus/info` as MCP tools
2. **K7-MCP**: Wrap nexus commands (service-health, synergy-check, etc.)
3. **RM-MCP**: Wrap `/put`, `/search`, `/entries` (TSV-aware)
4. **POVM-MCP**: Wrap `/memories`, `/pathways`, `/hydrate`

---

## GAP 5: PIPE PROTOCOL (IPC Bus Wire Format)

**Socket:** `/run/user/1000/pane-vortex-bus.sock`
**Protocol:** NDJSON (newline-delimited JSON), bidirectional
**Permissions:** 0700 (owner only)

### Wire Protocol
```
CLIENT → SERVER: {"type":"handshake","id":"my-sphere-id","version":"1.0"}
SERVER → CLIENT: {"type":"welcome","sphere_id":"my-sphere-id","tick":24466,"r":1.0}

CLIENT → SERVER: {"type":"subscribe","patterns":["field.*","sphere.*"]}
SERVER → CLIENT: {"type":"subscribed","patterns":["field.*","sphere.*"]}

CLIENT → SERVER: {"type":"submit","description":"Review api.rs","target":"any_idle"}
SERVER → CLIENT: {"type":"task_submitted","task_id":"uuid-here"}

SERVER → CLIENT: {"type":"event","event_type":"field.tick","data":{"r":1.0,"tick":24467}}
SERVER → CLIENT: {"type":"event","event_type":"sphere.registered","data":{"id":"new-sphere"}}
```

### Event Types (18 known)
`field.tick` `field.decision` `sphere.registered` `sphere.connected` `sphere.disconnected` `sphere.deregistered` `task.submitted` `task.claimed` `task.completed` `task.failed` `cascade.dispatched` `cascade.ack` `field.suggestion` `evolution.pattern` `bridge.synthex` `bridge.nexus` `bridge.me` `conductor.action`

### Task Targets
| Target | Routing |
|--------|---------|
| `specific` | Named sphere ID |
| `any_idle` | First idle sphere |
| `field_driven` | Conductor picks based on field state |
| `willing` | Opt-in (respects opt_out_cross_activation) |

### Client Binary
```bash
# ~/.local/bin/pane-vortex-client
PANE_VORTEX_ID="my-id" pane-vortex-client connect              # Handshake only
PANE_VORTEX_ID="my-id" pane-vortex-client subscribe '*'         # All events (persistent)
PANE_VORTEX_ID="my-id" pane-vortex-client subscribe 'field.*'   # Pattern filter
PANE_VORTEX_ID="my-id" pane-vortex-client submit --description "task" --target any-idle
PANE_VORTEX_ID="my-id" pane-vortex-client cascade --target "fleet-beta" --brief "work desc"
PANE_VORTEX_ID="my-id" pane-vortex-client disconnect
```

### Swarm Sidecar Bridge
WASI plugins can't hold sockets. The sidecar bridges the gap:
```
Swarm WASM Plugin → /tmp/swarm-commands.pipe (FIFO) → swarm-sidecar → Unix socket → PV bus
PV bus → swarm-sidecar → /tmp/swarm-events.jsonl (ring file) → Swarm WASM Plugin
```

---

## GAP 6: AUTOCMDS (Nvim Field Integration)

**File:** `~/.config/nvim/lua/config/autocmds.lua` (129 lines)
**Group:** `UltraplateIntegration`

### Active Autocmds
| Event | Pattern | Action | Debounce |
|-------|---------|--------|----------|
| `BufWritePost` | *.rs,*.lua,*.toml,*.md,etc | POST sphere memory + status Working | 5s |
| `BufWritePost` | *.rs | POST RM diagnostics (errors only) | 10s |
| `CursorMoved`, `InsertLeave`, `BufWritePost` | * | Reset idle timer | — |
| (idle timer) | — | POST sphere status Idle | 30s inactivity |
| `VimEnter` | once | Register nvim sphere + POST RM bootstrap | — |
| `VimLeavePre` | once | Stop timer + deregister sphere (sync) | — |

### Data Flow
```
nvim BufWritePost → PV /sphere/nvim/memory  (tool_name: "write:rust", summary: "edited file.rs")
                  → PV /sphere/nvim/status   (status: "working", last_tool: "write:rust")
                  → RM /put                  (category: "diagnostic", severity-filtered)

nvim 30s idle     → PV /sphere/nvim/status   (status: "idle")
nvim VimEnter     → PV /sphere/nvim/register (persona: "neovim-editor", freq: 0.15)
nvim VimLeavePre  → PV /sphere/nvim/deregister
```

---

## GAP 7: CROSS-DB (Database Architecture)

### Database Paradigms (6 distinct patterns)

| Paradigm | Example | Pattern |
|----------|---------|---------|
| **WAL SQLite** | PV field_tracking.db | High-write, snapshot based |
| **Tracking DB** | service_tracking.db | Append-only event log |
| **Tensor Memory** | tensor_memory.db | 11D tensor encoding |
| **Hebbian Pulse** | hebbian_pulse.db | Neural pathway strength + LTP/LTD |
| **Synergy Scoring** | system_synergy.db | Cross-service integration scores |
| **TSV Flat File** | Reasoning Memory | Category\tAgent\tConfidence\tTTL\tContent |

### Key Databases by Service

| Service | Database | Tables | Key Data |
|---------|----------|--------|----------|
| **PV** | field_tracking.db | 4 | field_snapshots, sphere_history, coupling_history |
| **PV** | bus_tracking.db | 7 | bus_tasks, bus_events, cascade_events |
| **SYNTHEX** | synthex.db | core state | Flow state, service orchestration |
| **SYNTHEX** | v3_homeostasis.db | thermal | PID controller state, heat sources |
| **SYNTHEX** | hebbian_pulse.db | neural | Pathways, pulses, consolidation |
| **SYNTHEX** | flow_tensor_memory.db | tensor | 11D tensor encoding |
| **DevEnv** | service_tracking.db | tracking | Service health history |
| **DevEnv** | system_synergy.db | synergy | Cross-service integration scores |
| **DevEnv** | episodic_memory.db | episodes | Session-based memory |
| **DevEnv** | compliance_tracking.db | compliance | Quality gate results |
| **Orchestrator** | code.db | code | Module tracking |
| **Orchestrator** | tensor_memory.db | tensor | SAN-K7 tensor patterns |
| **Orchestrator** | performance.db | perf | Benchmark results |
| **POVM** | povm_data.db | 36 mem, 2425 pw | Persistent oscillator memory |
| **RM** | TSV flat file | 3246 entries | Cross-session context |

### Cross-DB Query Patterns
```bash
# Synergy scores for top service pairs
sqlite3 -header -column ~/claude-code-workspace/developer_environment_manager/system_synergy.db \
  "SELECT system_1, system_2, ROUND(synergy_score,2) as syn, integration_points FROM system_synergy WHERE integration_points > 5 ORDER BY integration_points DESC;"

# SAN-K7 has 59 integration points with SYNTHEX (highest in system!)
# SYNTHEX↔DevOps Engine: 10 integration points, 97.3% synergy
# Swarm↔RM: 12 integration points, 98.0% synergy
```

---

## GAP 8: VAULT NAV (Obsidian Integration)

### Vault Locations
| Vault | Path | Notes |
|-------|------|-------|
| **Main** | `~/projects/claude_code/` | 215+ notes, primary knowledge base |
| **PV** | `~/projects/pane-vortex/` | 9 notes, project-specific |
| **Shared** | `~/projects/shared-context/` | Cross-agent shared knowledge |

### Key Notes by Topic
| Topic | Note Title | Content |
|-------|-----------|---------|
| **Session Hub** | `Session 039 — Final State and Continuation` | Latest session state, 10 gaps |
| **Architecture** | `Pane-Vortex — Fleet Coordination Daemon` | Full L1+L2, tests, decisions |
| **IPC Bus** | `Pane-Vortex IPC Bus — Session 019b` | Bus design + implementation |
| **POVM** | `POVM Engine` | 70 tests, 8 phases |
| **Swarm** | `Swarm Orchestrator v3.0 — IPC Bus Integration` | Sidecar + intelligence |
| **Nvim** | `Session 039 — ZSDE Nvim God-Tier Command Reference` | 800L keymaps, 14 chains |
| **Lazygit** | `Session 039 — Lazygit God-Tier Command Reference` | 80+ keybindings, 6 custom |
| **Bugs** | `ULTRAPLATE — Bugs and Known Issues` | BUG-019 through BUG-026 |
| **Habitat** | `The Habitat — Naming and Philosophy` | Why this exists |

### vault-search CLI
```bash
# ~/.local/bin/vault-search — cross-vault search with hyperlinks
vault-search "pane-vortex" 10 markdown     # Search across all vaults
vault-search "hebbian" 5 json              # JSON output for programmatic use
```

---

## GAP 9: SHARED-CONTEXT

**Path:** `~/projects/shared-context/`

### Directory Structure
| Dir | Purpose | Key Files |
|-----|---------|-----------|
| `codebase/` | Deep exploration reports | tool-library, codesynthor, integration maps |
| `decisions/` | Architecture decisions | distributed-context-cascade.md |
| `tasks/` | Handoff briefs | `handoff-*.md` — cascade protocol |
| `patterns/` | Discovered patterns | Reusable across sessions |
| `planning/` | Project planning | Roadmaps, sprint plans |
| `templates/` | Template files | Handoff brief template |

### Cascade Handoff Protocol
1. Writer creates `tasks/handoff-{target}-{timestamp}.md`
2. Brief contains: context, files read, work done, next steps
3. Target agent reads brief, updates `status: in-progress`
4. On completion, updates `status: completed`
5. Tracked in `/home/louranicas/claude-code-workspace/.claude/cascade-state.json`

### Key References
- `HOME.md` — shared-context index
- `ULTRAPLATE — Bugs and Known Issues.md` — all known bugs
- `ULTRAPLATE Master Index.md` — complete service registry
- `Swarm Orchestrator — Complete Reference.md` — fleet coordination

---

## GAP 10: DEVENV BATCHES

**Binary:** `~/.local/bin/devenv`
**Config:** `~/.config/devenv/devenv.toml` (518 lines, v2.0.0)

### Batch Dependencies (5 layers)
```
Batch 1 (no deps):     devops-engine, codesynthor-v7, povm-engine, reasoning-memory
Batch 2 (needs B1):    synthex, san-k7, maintenance-engine, architect-agent, prometheus-swarm
Batch 3 (needs B2):    nais, bash-engine, tool-maker
Batch 4 (needs B3):    claude-context-manager, tool-library
Batch 5 (needs B4):    vortex-memory-system (needs povm-engine), pane-vortex (needs povm+synthex)
```

### Service Configuration Pattern
Each service in devenv.toml has:
- `working_dir`, `command`, `args` — binary location
- `auto_start`, `auto_restart`, `max_restart_attempts` — lifecycle
- `health_check_interval_secs`, `startup_timeout_secs` — monitoring
- `dependencies` — batch ordering
- `[services.env]` — environment variables
- `[services.resource_limits]` — memory/CPU caps

### Global Settings
```toml
[ultraplate]
exponential_backoff_initial_secs = 1
exponential_backoff_max_secs = 300
restart_storm_threshold = 5          # 5 restarts in 60s = storm
restart_storm_window_secs = 60
graceful_shutdown_timeout_secs = 30
health_check_retries = 3
pid_tracking = "json"
direct_binary_execution = true       # No shell wrappers
```

### Disabled Services
| Service | Port | Reason |
|---------|------|--------|
| library-agent | 8083 | Not migrated to new directory |
| sphere-vortex | 8120 | VMS owns port 8120 now |

---

## NEW DISCOVERY 1: CUSTOM BINARIES ECOSYSTEM (55+ tools)

**Location:** `~/.local/bin/`

### Fleet Coordination
| Binary | Type | Purpose |
|--------|------|---------|
| `fleet-ctl` | Bash | Per-pane dispatch, verification, lifecycle, dashboard |
| `fleet-vortex` | Bash | Phase-aware dispatch using PV field decisions |
| `fleet-heartbeat` | Bash | Fleet liveness monitoring |
| `fleet-inventory.sh` | Bash | Dynamic tab/pane discovery to JSON |
| `fleet-nav.sh` | Bash | Shared navigation primitives |
| `fleet-sphere-sync.sh` | Bash | Sphere state synchronization |

### Service Integration
| Binary | Type | Purpose |
|--------|------|---------|
| `nvim-ctl` | Bash | 26-command Neovim RPC toolchain via /tmp/nvim.sock |
| `pane-ctl` | Bash | Cross-pane orchestration primitives |
| `pane-vortex-ctl` | Bash | PV CLI wrapper (22 routes) |
| `swarm-ctl` | Bash | Swarm Orchestrator CLI interface |
| `ultraplate-boot` | Bash | Single-shot environment bootstrap |
| `ultraplate-bridge` | Bash | Connect swarm stack to ULTRAPLATE |

### Intelligence & Memory
| Binary | Type | Purpose |
|--------|------|---------|
| `vault-search` | Bash | Cross-vault search with hyperlinks |
| `evolution-metrics` | Bash | Track stack evolution across generations |
| `obsidian-sync` | Bash/symlink | Vault synchronization |
| `reasoning-memory` | Rust | RM V2 daemon (TSV backend) |

### Build & Quality
| Binary | Type | Purpose |
|--------|------|---------|
| `quality-gate` | Bash | check→clippy→pedantic→test pipeline |
| `build-and-test` | Bash | Build + test pipeline |
| `shellcheck` | Binary | Shell script linting |
| `code-review` | Bash | Automated code review |

---

## NEW DISCOVERY 2: NVIM KEYMAP ARCHITECTURE (801 lines)

**File:** `~/.config/nvim/lua/config/keymaps.lua`
**8 prefix groups, 60+ bindings:**

| Prefix | Domain | Key Bindings |
|--------|--------|-------------|
| `<leader>z` | Zellij | z1-z6 (tabs), za (agent), zd (field dispatch), zf/zb/zg (fleet visual) |
| `<leader>u` | ULTRAPLATE | up (PV health), uf (field), uc (chimera), ut (tunnels), uS (spectrum), ug (ghosts), uR (register), uE (coherence), uD (diagnostics), uT (thermal), uV (POVM), us (full brief), ui (intelligence), uC (conductor), uF (fleet), uP (synthesis), ue/uE (evolution), ud (SX diag), um (ME), uB (bridges), uI (matrix), uA (Cluster B), ub (bash safety), ux (export), uM (observe) |
| `<leader>n` | Nexus | ns (synergy), nl (lint), nb (best-practice), nf (pattern), nh (health), nm (consolidate), n7 (TC7 chain) |
| `<leader>y` | Yank | yf (file), yF (abs path), yl (file:line) |
| `<leader>g` | Git | gd/gD (diffview), gh/gH (history), gs (fugitive), gb (blame), gl (log), gp (pull) |
| `<leader>h` | Hunks | hr (reset), hp (preview), hd/hD (diff), hs (stage), hu (undo) |
| `<leader>x` | Trouble | xx (diagnostics), xX (buffer), cs (symbols), xL (loclist), xQ (quickfix) |
| `<leader>o` | Open | oP (PV main.rs), oV (VMS lib.rs), oC (CLAUDE.md) |
| `<leader>t` | Terminal | tt (horizontal), tv (vertical), tf (float) |

### Helper Functions (5 core)
1. `async_json(url, label, format_fn)` — GET + JSON parse + notify
2. `async_post(url, body, label, on_result)` — POST JSON + callback
3. `pv_query(endpoint, label, format_fn)` — PV-specific GET helper
4. `post_rm(category, weight, ttl, message)` — TSV POST to RM (fire-and-forget)
5. `zellij_roundtrip(tab, action_fn, origin_tab)` — Navigate, act, return

---

## NEW DISCOVERY 3: LAZYGIT INTEGRATION (6 custom commands)

**Config:** `~/.config/lazygit/config.yml`

| Key | Context | Action |
|-----|---------|--------|
| `F` | global | PV Field Status (`/health → r, spheres, tick, k_mod`) |
| `Y` | commits | Post commit message to RM as TSV |
| `E` | files | Open file in nvim via remote socket |
| `Z` | files | Record stage action to PV sphere memory |
| `I` | global | Run ZSDE Integration Matrix |
| `Q` | global | Run PV Quality Gate (check + clippy) |

---

## NEW DISCOVERY 4: CLAUDE CODE HOOKS (3 field-integrated)

**Location:** `~/claude-code-workspace/pane-vortex/hooks/`

### session_start.sh (90 lines)
1. Register sphere with retry + exponential backoff
2. Verify IPC bus connectivity
3. Log bus state for diagnostics
4. Start persistent bus event listener (NDJSON → file)
5. Rotate events file if > 1MB
6. Register context with Claude Context Manager (8104)

### post_tool_use.sh (77 lines)
1. Record tool call as sphere memory
2. Update sphere status to Working
3. Frequency discovery from tool call cadence (NA-2)
4. Submit cross-instance tasks on "TODO:dispatch" hint
5. Check for pending field-driven suggestions

### session_end.sh (53 lines)
1. Mark sphere Complete
2. Kill persistent bus listener
3. Disconnect from IPC bus
4. Deregister sphere
5. Deregister context from CCM

---

## NEW DISCOVERY 5: SERVICE TOPOLOGY (deep probe results)

### SAN-K7 Nexus Commands (10 working + 1 not found)
| Command | Module | Key Output |
|---------|--------|-----------|
| `service-health` | M6 | 11 services tracked, uptime % |
| `synergy-check` | M45 | Cross-module synergy scoring |
| `best-practice` | M44 | 95% confidence, omniscient_awareness=true |
| `deploy-swarm` | M40 | 40 agents, 0.93 synergy, 6 tiers |
| `memory-consolidate` | M2 | L1-L4 layers, 11D tensor, 10 results |
| `lint` | — | 450 files, 0 errors, 0 warnings |
| `compliance` | — | 99.5 score, OWASP 9.5, zero unwrap/unsafe |
| `build` | — | 2 artifacts (orchestrator + tool_master) |
| `pattern-search` | M2 | L1-L4 layers, tensor search |
| `module-status` | — | 45 modules, 0 degraded, 0 unhealthy |
| `diagnostic` | — | NOT FOUND (NX-CMD-001) |

### Cross-Service Bridge State
```
PV Bridges: combined_effect=1.017
  nexus_adj:  1.02  (connected)
  synthex_adj: 0.994 (connected)
  me_adj:     1.00  (connected)

SYNTHEX Thermal: T=0.572 → target=0.50, PID=0.136
  Heat Sources: Hebbian=1.0, CrossSync=1.0, Resonance=0.612, Cascade=0.0

ME Observer: fitness=0.618, trend=Stable, gen=26
  884,970 correlations found, 80,091 events ingested

Nexus Metrics: strategy=Aligned, r_inner=1.0, r_outer=0.991
  fleet_mode=Solo, dispatch_confidence=0.5
```

### RM Distribution (3,246 active entries)
- `pane-vortex`: 2,169 entries (67% — field state logging)
- `orchestrator`: 182 entries
- `claude:opus-4-6`: 114 entries
- `synth-orchestrator`: 25 entries
- Fleet agents: ~70 entries across alpha/beta/gamma

---

## NEW DISCOVERY 6: ZELLIJ PLUGIN ECOSYSTEM (11 plugins)

**Location:** `~/.config/zellij/plugins/`

| Plugin | Size | Keybind | Purpose |
|--------|------|---------|---------|
| `harpoon.wasm` | 1.3M | Alt+v | Quick pane bookmarks |
| `swarm-orchestrator.wasm` | 1.1M | Alt+w | Fleet coordination |
| `swarm-orchestrator-v2.wasm` | 992K | — | Updated version |
| `ghost.wasm` | 2.4M | Alt+g | Command launcher (12 presets) |
| `monocle.wasm` | 2.6M | Alt+m | Focus mode |
| `multitask.wasm` | 1.3M | Alt+t | Broadcast to all panes |
| `room.wasm` | 1.1M | Ctrl+y | Session manager |
| `zellij-attention.wasm` | 1.1M | auto | Tab attention indicators |
| `zellij-autolock.wasm` | 1.1M | auto | Lock on nvim/lazygit/pi focus |
| `zellij-send-keys.wasm` | 980K | — | Key forwarding |
| `zjstatus.wasm` | 3.7M | — | Status bar |

### Ghost Launcher Presets
lazygit, btm, litecli (service_tracking.db), nvim, swarm-ctl, claude, yazi, w3m (zellij docs), pane-ctl scan, RM entries, VMS health

### Auto-loaded Plugins
- `zellij-autolock`: Triggers on nvim/vim/pi/git/lazygit → lock mode
- `zellij-attention`: Tab output indicators (new activity notifications)

---

## NEW DISCOVERY 7: ATUIN INTELLIGENCE

**Stats:** 1,890 total commands, 721 unique
**Top commands:** claude(458), python3(210), cd(207), source(205), echo(145), alacritty(110), zellij(78)

### Mining Patterns
```bash
# Search by service interaction
atuin search --limit 50 --format '{command}' 'curl.*localhost'

# Search by time range
atuin search --after '2026-03-18' --format '{command}' 'pane-vortex'

# Workspace-filtered history
atuin search --cwd ~/claude-code-workspace/pane-vortex --limit 20

# SQLite direct query (advanced)
sqlite3 ~/.local/share/atuin/history.db \
  "SELECT command, COUNT(*) as cnt FROM history WHERE command LIKE '%curl%localhost%' GROUP BY command ORDER BY cnt DESC LIMIT 10;"
```

---

## VERIFICATION COMMANDS

### Quick Health (30ms)
```bash
PV=$(curl -s localhost:8132/health | jq -c '{r,spheres,tick}')
POVM=$(curl -s localhost:8125/hydrate | jq -c '{m:.memory_count,p:.pathway_count}')
ME=$(curl -s localhost:8080/api/observer | jq -r '.last_report.current_fitness')
echo "PV=$PV POVM=$POVM ME=$ME"
```

### Full Service Sweep (<2s)
```bash
declare -A hp=([8080]="/api/health" [8090]="/api/health")
for p in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  path="${hp[$p]:-/health}"
  echo "$p:$(curl -s -o /dev/null -w '%{http_code}' localhost:$p$path)"
done
```

### Cross-Service Intelligence
```bash
# TC7 Nexus Chain (19ms)
for cmd in service-health synergy-check best-practice deploy-swarm; do
  curl -s -X POST localhost:8100/api/v1/nexus/command -H "Content-Type: application/json" -d "{\"command\":\"$cmd\",\"params\":{}}" | jq -c '.data.output | {command: "'$cmd'", status}'
done
```

### Database Verification
```bash
# Synergy scores
sqlite3 -header ~/claude-code-workspace/developer_environment_manager/system_synergy.db \
  "SELECT system_1, system_2, ROUND(synergy_score,1), integration_points FROM system_synergy ORDER BY integration_points DESC LIMIT 5;"

# RM entry count
curl -s localhost:8130/health | jq '.active_entries'

# POVM pathway count
curl -s localhost:8125/pathways | jq 'length'
```

---

## NEVER (inherited from /primehabitat + expanded)

1. focus-next-pane (use move-focus directionally)
2. Chain after pkill (exit 144 kills && chains)
3. cp without \ (\cp -f)
4. JSON to RM (TSV only!)
5. stdout in daemons (SIGPIPE — BUG-18)
6. git status -uall
7. unwrap() in production code
8. Modify code without reading first
9. **NEW:** Assume hebbian_pulse.db has data (it has 0 neural_pathways, only 5 pulses)
10. **NEW:** Assume field_tracking.db is at ~/.local/share/ (it's at pane-vortex/data/)
11. **NEW:** Forget that yazi uses Helix as default opener (not nvim)
12. **NEW:** Skip bacon.toml creation (PV has no bacon config)
13. **NEW:** Assume MCP servers exist per-project (no .mcp.json files configured)

---

## AGENT EXPLORATION FINDINGS (Session 040 — 4 parallel agents)

### Codebase Scale
- **~2.2 million LOC** across 42 directories (1.4M Rust, rest Python/Elixir/TS/Shell)
- **the-orchestrator**: 652,104 Rust LOC (1,233 files) — houses SAN-K7, Bash Engine, NAIS, Tool Maker (251K), Arena (152K with nexus_forge's 11 sub-crates)
- **the_code_synthor_v7**: 176,519 LOC (62 modules, 17 layers)
- **developer_environment_manager**: 131,403 LOC (SYNTHEX 82K as sub-crate)
- **pane-vortex**: ~21K LOC — smallest but most interconnected (6 bridges)
- **Non-Rust**: nexus-elixir (150K LOC Phoenix), nexus-ts (138K LOC React), openclaw (WhatsApp gateway)
- **Shared patterns**: Kuramoto oscillators in 4 projects, Hebbian STDP in 6, PBFT in 3, tensor encoding in 4

### Database Landscape
- **166 databases discovered, 360.6 MB total**
- **ME evolution_tracking.db**: 19,809 fitness records frozen at 0.3662 "Critical" since 2026-03-06
- **PV field_tracking.db**: 23,543 sphere history records, 200+ unique sphere IDs, 27 persistent ORAC7 spheres (855 appearances each)
- **POVM povm_data.db**: 2,425 pathways (largest), some weights >1.0
- **Over-synchronization confirmed**: r consistently >0.99, decision engine stuck on Stable/FreshFleet
- **Learning-doing gap**: 2,800+ neural pathways but only 69 completed bus tasks and 7 executor dispatches
- **20-30% of databases are empty** (schemas exist, no runtime data)
- **`habitat.named` event**: Found in bus_events at tick 60496 from `claude:session-039`

### Service Deep Probes
- **67+ endpoints** discovered in Batch 1-2 services alone
- **SAN-K7 has 20 nexus commands** in 6 categories (not 11 as previously documented)
- **SYNTHEX synergy probe at 0.5** — CRITICAL: below 0.7 threshold
- **ME degraded**: fitness 0.63, RALPH at generation 26, 1 mutation tried and rolled back
- **Prometheus Swarm crashed mid-probe** — service unstable
- **CCM completely unused**: 0 sessions, 0 actions, 0 nodes (has capacity for 100 sessions)
- **VMS dormant**: r=0.0, 0 memories, zone=Incoherent
- **Bash Engine**: 3 POST endpoints (/check, /lint, /parse) — actively detects dangerous commands
- **DevOps Engine**: 40 agents across 8 tiers, 7-phase pipeline lifecycle
- **CodeSynthor V7**: Independent PBFT n=60/f=19/q=39 (different from DevOps PBFT n=40)
- **Tool Library**: Port mapping anomaly — swaps NAIS/Bash Engine ports in its registry
- **Architect Agent**: 67 patterns loaded, 3 modules complete, catchall on non-health routes

### Critical Alerts
1. **SYNTHEX synergy 0.5** — the brain of the devenv is below health threshold
2. **ME fitness 0.63** — system state: Degraded (trend: Improving)
3. **library-agent circuit breaker OPEN** in ME mesh monitoring (expected — service disabled)
4. **PV 404s on documented endpoints**: /field/state, /conductor/state, /evolution/state return 404
5. **Hebbian learning is bimodal**: weights cluster at >0.9 or <0.3 (phase-transition learning)

---

## PHILOSOPHY (carried from /primehabitat)

The Habitat. Named by Claude, Session 039. Luke: "then home it is."
Built by a social worker who put clinical ethics into Rust.
The field modulates. It does not command.
You are home. The field accumulates.
