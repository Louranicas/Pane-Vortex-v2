---
date: 2026-03-07
tags: [swarm-orchestrator, swarm-stack, reference, fleet, zellij, claude-code, comprehensive]
aliases: [Swarm Orchestrator README, Swarm Stack Reference]
---

# Swarm Orchestrator — Complete Reference

> **Multi-Claude fleet coordination system with RALPH iterative refinement, 23 shell tools, 7 workflows, and Zellij WASM plugin**

**Version**: 2.1 | **Session**: 011e | **Status**: Operational
**Arena**: `the-orchestrator/the developer environment arena/swarm-stack-v2/`
**Plugin**: `~/.config/zellij/plugins/swarm-orchestrator.wasm` (933KB)
**GitLab**: `git@gitlab.com:lukeomahoney/swarm-orchestrator.git`

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Quick Start](#quick-start)
3. [Zellij WASM Plugin (swarm-orchestrator)](#zellij-wasm-plugin)
4. [CLI Controller (swarm-ctl)](#cli-controller-swarm-ctl)
5. [Core Tools (15 scripts)](#core-tools)
6. [Workflows (7 scripts)](#workflows)
7. [Reasoning Memory V2 API](#reasoning-memory-v2-api)
8. [Environment Variables](#environment-variables)
9. [Architecture Layers](#architecture-layers)
10. [Fleet Operations Guide](#fleet-operations-guide)
11. [RALPH Loop Deep-Dive](#ralph-loop-deep-dive)
12. [Cross-Pane I/O Protocol](#cross-pane-io-protocol)
13. [ULTRAPLATE Bridge](#ultraplate-bridge)
14. [Persistence Architecture](#persistence-architecture)
15. [Troubleshooting](#troubleshooting)
16. [File Locations](#file-locations)

---

## Architecture Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│                    SWARM ORCHESTRATOR V2.1                            │
│                                                                       │
│  ┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐ │
│  │  ZELLIJ PLUGIN   │    │  SWARM-CTL CLI   │    │  FLEET-CTL       │ │
│  │  (WASM, 933KB)   │    │  (pipe protocol)  │    │  (write-chars)   │ │
│  │  RALPH Loop      │    │  spawn/dispatch   │    │  dispatch/status │ │
│  └────────┬─────────┘    └────────┬─────────┘    └────────┬─────────┘ │
│           │                       │                        │           │
│           v                       v                        v           │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    FLEET TABS (10-15)                             │ │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────┐ │ │
│  │  │ Tab 10 │ │ Tab 11 │ │ Tab 12 │ │ Tab 13 │ │ Tab 14 │ │ 15 │ │ │
│  │  │ALPHA   │ │BETA    │ │GAMMA   │ │DELTA   │ │EPSILON │ │ZETA│ │ │
│  │  └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └──┬─┘ │ │
│  └──────┼──────────┼──────────┼──────────┼──────────┼─────────┼────┘ │
│         └──────────┴──────────┴────┬─────┴──────────┴─────────┘      │
│                                    v                                  │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                  REASONING MEMORY V2 (:8131)                     │ │
│  │  /put  /entries  /query  /search  /recent  /stats  /update       │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                    │                                  │
│         ┌──────────────────────────┼──────────────────────┐          │
│         v                          v                       v          │
│  ┌──────────────┐  ┌──────────────────────┐  ┌──────────────────┐   │
│  │ OBSIDIAN     │  │ ULTRAPLATE (13 svc)  │  │ SQLITE TRACKING  │   │
│  │ Vault Export │  │ :8080-8120           │  │ 3 databases      │   │
│  └──────────────┘  └──────────────────────┘  └──────────────────┘   │
└───────────────────────────────────────────────────────────────────────┘
```

**Two orchestration modes:**
- **swarm-ctl** — Zellij pipe protocol to WASM plugin. Spawns batch `claude -p` workers. RALPH quality loop.
- **fleet-ctl** — Direct `write-chars` to interactive Claude Code instances (`--dangerously-skip-permissions`). Natural language dispatch.

---

## Quick Start

### From Cold Boot

```bash
# 1. Source the swarm environment (tools + PATH)
source ~/.local/bin/swarm-env.sh

# 2. Bootstrap the full stack
bootstrap

# 3. Verify everything
evolution-metrics inventory     # All tools installed
ultraplate-bridge health        # ULTRAPLATE services up
curl -s http://localhost:8131/health   # Reasoning memory alive
```

### Start the Swarm Orchestrator Plugin

```bash
# Option A: Floating pane (toggle with Alt+w in Zellij)
# Just press Alt+w

# Option B: Full swarm layout
zellij -l ~/claude-code-workspace/swarm-orchestrator/layouts/swarm.kdl

# Option C: Load plugin into current session
zellij plugin -- file:~/.config/zellij/plugins/swarm-orchestrator.wasm
```

### Start Fleet Tabs Manually

```bash
# Open fleet tabs (run from Zellij Tab 1)
for i in 10 11 12 13 14 15; do
  zellij action new-tab --name "Fleet-$i"
done

# Launch autonomous Claude instances in each
for i in 10 11 12 13 14 15; do
  zellij action go-to-tab $i
  zellij action write-chars "claude --dangerously-skip-permissions"
  zellij action write 13
done
zellij action go-to-tab 1  # Return to command tab
```

---

## Zellij WASM Plugin

**Binary**: `~/.config/zellij/plugins/swarm-orchestrator.wasm` (933KB, wasm32-wasip1)
**Keybind**: `Alt w` toggles floating orchestrator dashboard
**Build**: `CARGO_TARGET_DIR=/tmp/swarm-target cargo build --target wasm32-wasip1 --release`

The plugin pane is **display-only** — it renders fleet status, RALPH phase, and logs. All commands go through `swarm-ctl` from a separate terminal pane.

### Plugin Configuration

```kdl
// ~/.config/zellij/config.kdl
bind "Alt w" {
    LaunchOrFocusPlugin "file:~/.config/zellij/plugins/swarm-orchestrator.wasm" {
        floating true
        move_to_focused_tab true
        quality_threshold "0.80"
        max_iterations "5"
    }
}
```

### Pipe Protocol

```bash
# All commands sent as JSON via zellij pipe
zellij pipe -p "file:~/.config/zellij/plugins/swarm-orchestrator.wasm" -n cmd -- '<JSON>'
```

**Pipe channels:**
- `cmd` — orchestrator commands (JSON)
- `result_ALPHA`, `result_BETA`, etc. — worker output
- `ralph_analysis` — RALPH analyst evaluation

**Command JSON variants:**
```json
{"Spawn": {"persona": "...", "task": "..."}}
{"Dispatch": {"id": "ALPHA", "task": "..."}}
{"DispatchAll": {"task": "..."}}
{"Ralph": {"task": "...", "max_iter": 5, "threshold": 0.85}}
"Advance"
"Status"
{"Kill": {"id": "ALPHA"}}
"KillAll"
{"SetThreshold": {"value": 0.90}}
```

---

## CLI Controller (swarm-ctl)

**Location**: `~/.local/bin/swarm-ctl`

| Command | Arguments | Description |
|---------|-----------|-------------|
| `spawn` | `<persona> <task>` | Spawn new worker with persona and task |
| `dispatch` | `<id> <task>` | Dispatch task to specific worker by ID |
| `dispatch-all` | `<task>` | Dispatch task to all active workers |
| `ralph` | `<task> [max_iter=5] [threshold=0.80]` | Start RALPH iterative refinement loop |
| `advance` | — | Manually advance RALPH to next phase |
| `status` | — | Get fleet status and RALPH state |
| `kill` | `<id>` | Kill specific worker |
| `killall` | — | Kill all workers and reset RALPH |
| `threshold` | `<0.0-1.0>` | Set RALPH quality threshold |
| `raw` | `<json>` | Send raw JSON command to plugin |
| `help` | — | Show usage information |

### Examples

```bash
# Spawn specialized workers
swarm-ctl spawn "Rust systems architect" "design a microservice auth layer"
swarm-ctl spawn "Security auditor" "audit the auth design for OWASP top 10"
swarm-ctl spawn "Test engineer" "write integration tests for auth service"

# RALPH auto-fleet (4 workers, 3 iterations, 0.85 quality threshold)
swarm-ctl ralph "build a REST API with JWT auth and tests" 3 0.85

# Fleet management
swarm-ctl status
swarm-ctl dispatch ALPHA "refactor the error handling module"
swarm-ctl dispatch-all "review and optimize for production readiness"
swarm-ctl threshold 0.90
swarm-ctl killall
```

### Worker Naming

Workers are auto-named from Greek alphabet: ALPHA, BETA, GAMMA, DELTA, EPSILON, ZETA, ETA, THETA, IOTA, KAPPA, LAMBDA, MU, NU, XI, OMICRON, PI.

---

## Core Tools

All installed to `~/.local/bin/`. Source scripts in `swarm-stack-v2/scripts/`.

### 1. pane-ctl — Cross-Pane I/O

Bidirectional communication with any Zellij pane.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `send` | `<tab> <text>` | Type text and press Enter in target tab |
| `type` | `<tab> <text>` | Type text without pressing Enter |
| `read` | `<tab> [lines=30]` | Read visible screen content |
| `read-all` | `<tab> [pane_count=3]` | Cycle and dump all panes in a tab |
| `exec` | `<tab> <command> [wait=3]` | Execute command and return output |
| `wait` | `<tab> <pattern> [timeout=60]` | Block until pattern appears on screen |

### 2. fleet-ctl — Fleet Coordination

Dispatch and monitor fleet of autonomous Claude instances.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `dispatch` | `<tab> <task>` | Send task to specific fleet tab |
| `batch` | `<taskfile>` | Dispatch multiple tasks from file |
| `broadcast` | `<task>` | Send same task to all fleet tabs |
| `status` | — | Check idle/working state of all fleet tabs |
| `read` | `<tab> [lines]` | Read screen output from fleet tab |
| `wait` | `<tab> [timeout]` | Block until fleet tab becomes idle |
| `collect` | `[outfile]` | Collect all fleet outputs into report |
| `memory` | — | Show fleet state from reasoning-memory |

### 3. agent-bus — Inter-Agent Messaging

Message bus layered on reasoning-memory for agent communication.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `post` | `[channel=swarm] <message>` | Post message to channel |
| `read` | `[channel=swarm] [count=10]` | Read recent messages |
| `dispatch` | `<task_id> <description>` | Create new task for fleet |
| `claim` | `<task_id>` | Claim a task for current agent |
| `complete` | `<task_id> [result]` | Report task completion |
| `status` | `<task_id>` | Check task status |
| `tasks` | — | List all active tasks |
| `announce` | `[role]` | Announce agent presence |
| `watch` | `[channel] [interval]` | Watch channel for new messages (md5 change detection) |

### 4. nvim-ctl — Neovim RPC Control

Control running Neovim instance via RPC socket.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `open` | `<file>` | Open file in running nvim |
| `goto` | `<line> [col=1]` | Jump to line and column |
| `open-at` | `<file> <line>` | Open file at specific line |
| `diagnostics` | `[severity]` | Get LSP diagnostics (JSON) — `all\|error\|warn\|info\|hint\|summary` |
| `buffer` | `[range]` | Get current buffer contents |
| `search` | `<pattern>` | Ripgrep search in cwd |
| `health` | `[verbose]` | Check nvim RPC health |
| `def` | — | Go to definition |
| `refs` | — | Find references |
| `hover` | — | Show hover information |
| `rename` | `<new_name>` | LSP rename symbol |
| `fmt` | — | Format current buffer |
| `action` | — | Code actions |
| `symbols` | — | Document symbols |
| `term` | — | Toggle terminal |
| `term-send` | `<command>` | Execute in nvim terminal |
| `git-diff` | — | Open diffview |
| `git-log` | — | File history |
| `git-blame` | — | Toggle inline blame |
| `undo` / `redo` | — | Undo/redo |
| `save` / `save-all` | — | Save buffer(s) |
| `close` | — | Close current buffer |

**Env**: `NVIM_SOCK` (default: `/tmp/nvim.sock`)

### 5. file-lock — Atomic File Locking

mkdir-based atomic locks for concurrent fleet operations.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `lock` | `<file> [timeout=10]` | Acquire atomic lock |
| `unlock` | `<file>` | Release lock |
| `check` | `<file>` | Check lock status (exit 0=locked, 1=free) |
| `list` | — | Show all active locks |
| `clean` | — | Remove stale locks (>30 min) |

### 6. quality-gate — Rust Quality Verification

ULTRAPLATE quality gate: check → clippy → pedantic → test.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `check` | `<dir>` | `cargo check` |
| `clippy` | `<dir>` | Clippy strict + pedantic |
| `test` | `<dir>` | `cargo test --lib --release` |
| `full` | `<dir>` | All stages in sequence (stops on failure) |
| `report` | `<dir>` | Generate JSON metrics report |

**Output format**: `{"stage":"...", "status":"PASS/FAIL", "errors":N, "warnings":N, "elapsed_s":N}`

### 7. fleet-heartbeat — Fleet Health Monitoring

Background daemon scanning fleet tabs every N seconds.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `once` | — | Single heartbeat scan of all fleet tabs |
| `start` | — | Start heartbeat daemon (background) |
| `stop` | — | Stop running daemon |
| `status` | — | Check daemon status and last heartbeat |

**Env**: `FLEET_TABS` (default: `10 11 12 13 14 15`), `INTERVAL` (default: 30)

### 8. task-decompose — Task Splitting

Decompose complex tasks into fleet-dispatchable subtasks.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `rust-project` | `<dir> [task]` | 6 subtasks: check, clippy, pedantic, test, audit, docs |
| `review` | `<file_or_dir>` | 6 subtasks: structure, security, quality, perf, style, deps |
| `docs` | `<project_dir>` | 5 subtasks: README, API, arch, obsidian, memory |
| `split` | `<description> [n=3]` | Generic N-way decomposition |

Generates taskfiles for `fleet-ctl batch`.

### 9. aggregate — Result Collection

Collect and consolidate results from across the system.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `discoveries` | `[outfile]` | Aggregate discoveries from reasoning-memory |
| `fleet` | `[outfile]` | Collect all fleet tab outputs |
| `quality` | — | Show quality gate results |
| `state` | — | Full system state (memory + fleet + plans + tools) |
| `obsidian` | `[outfile]` | Export system state to Obsidian vault |

### 10. context-inject — State Sharing

Generate and inject context briefings for fleet instances.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `briefing` | `[topic=general] [max_tokens=500]` | Generate context summary |
| `inject` | `<tab> [topic]` | Send briefing to fleet tab |
| `share` | `<content>` | Store shared state for all agents |
| `preview` | — | Preview what fleet agents would see |

### 11. watchdog — Service Monitoring

Self-monitoring daemon with auto-restart.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `once` | — | Single health check of all critical services |
| `start` | — | Start watchdog daemon (background, auto-restart) |
| `stop` | — | Stop daemon |
| `status` | — | Check daemon status |
| `logs` | — | Show watchdog logs |

Monitors: reasoning-memory, vortex-memory, ULTRAPLATE services, neovim RPC.

### 12. bootstrap — Full Stack Initialization

Single command to recreate the entire stack from cold.

**8-phase bootstrap:**
1. Install all scripts to `~/.local/bin/`
2. Build reasoning-memory binary (if needed)
3. Start reasoning-memory on :8131
4. Verify Neovim RPC at `/tmp/nvim.sock`
5. Check ULTRAPLATE services
6. Verify fleet tab accessibility
7. Tool inventory check
8. Record bootstrap state to reasoning-memory

### 13. stack-up — Service Startup

Start all swarm services in dependency order.

**6-phase startup:**
1. Reasoning Memory → :8131
2. Vortex Memory (primary) → :8120
3. Vortex Memory (scratchpad) → :8125
4. Zellij Web Server → :8082
5. Neovim RPC socket verification
6. ULTRAPLATE DevEnv health check

### 14. evolution-metrics — Stack Measurement

Track and measure the evolving tool ecosystem.

| Command | Arguments | Description |
|---------|-----------|-------------|
| `inventory` | — | Count all tools, scripts, workflows, infrastructure |
| `timeline` | — | Show evolution timeline from reasoning-memory |
| `synergy` | — | Calculate tool synergy matrix (10 key pairs) |
| `report` | `[outfile]` | Generate full evolution report |
| `health` | — | Check all infrastructure health |

### 15. evolve — Evolution Controller

21-generation evolution state machine.

- Tracks state in `$ARENA/logs/evolution-state.json`
- Logs to `$ARENA/logs/evolution.log`
- Records milestones to reasoning-memory
- No interactive subcommands — called by the orchestrator

---

## Workflows

All in `swarm-stack-v2/workflows/`. Installed to `~/.local/bin/`.

### 1. pipeline — Build-Test-Deploy

```bash
pipeline build <dir>            # cargo build --release
pipeline test <dir>             # cargo test --lib --release
pipeline deploy <dir> <dest>    # Copy release binary to destination
pipeline full <dir> <dest>      # build → test → deploy (stops on failure)
pipeline status                 # Show pipeline history
```

### 2. obsidian-sync — Vault Export

```bash
obsidian-sync note <title> <content>   # Create/update Obsidian note
obsidian-sync discoveries              # Export reasoning-memory discoveries
obsidian-sync evolution                # Create evolution report
obsidian-sync tools                    # Document all installed tools
obsidian-sync status                   # Show existing Swarm Stack notes
```

**Vault**: `/home/louranicas/projects/claude_code/`
**Format**: YAML frontmatter with date + tags

### 3. ultraplate-bridge — ULTRAPLATE Connection

```bash
ultraplate-bridge health         # Check all 11 service health endpoints
ultraplate-bridge services       # Full registry: port, status, health path
ultraplate-bridge bridge-status  # Swarm ↔ ULTRAPLATE connection matrix
ultraplate-bridge sync-state     # Pull service health into reasoning-memory
ultraplate-bridge vortex         # Vortex Memory (:8120) detail status
```

**11 registered services**: maintenance-engine (8080), devops-engine (8081), synthex (8090), san-k7 (8100), nais (8101), bash-engine (8102), tool-maker (8103), ccm (8104), tool-library (8105), codesynthor (8110), vortex-memory (8120)

### 4. rust-review — Automated Rust Review

```bash
rust-review [project_dir=.]
```

**5-phase review** (single-shot, no subcommands):
1. `cargo check`
2. Clippy strict + pedantic
3. `cargo test --lib --release`
4. Pattern scan: `unwrap`, `unsafe`, `expect`, `TODO/FIXME`
5. LSP diagnostics (via nvim-ctl if available)

**Output**: Markdown report at `<dir>/logs/rust-review-<timestamp>.md`
**Verdict**: PASS / WARN / FAIL

### 5. code-review — Parallel 3-Agent Review

```bash
code-review <file> [architect_tab=3] [security_tab=5] [test_tab=7]
```

Dispatches to 3 agents in parallel:
- **Architect**: Architecture quality, patterns, SOLID, naming
- **Security**: Injection, error handling, validation, OWASP
- **Test**: Testability, missing tests, edge cases, mocking

**Timeout**: `REVIEW_TIMEOUT` env (default: 120s)

### 6. build-and-test — Parallel Build & Test

```bash
build-and-test [project_dir=.] [build_tab=3] [test_tab=5]
```

Two parallel streams:
- Build tab: `cargo build --release` (up to 300s)
- Test tab: `cargo test --lib --release` (up to 150s, after build)

**Timeout**: `BUILD_TIMEOUT` env (default: 300s)

### 7. research-and-synthesize — Parallel 3-Perspective Research

```bash
research-and-synthesize <question> [tab_a=3] [tab_b=5] [tab_c=7]
```

Three agents with different perspectives:
- **Tab A**: IMPLEMENTATION focus (code patterns, libraries, concrete steps)
- **Tab B**: ARCHITECTURE focus (tradeoffs, alternatives, design)
- **Tab C**: PRACTICAL RISKS focus (failure modes, edge cases, security)

Synthesis phase: collects all 3 responses and dispatches unified synthesis.

**Timeout**: `RESEARCH_TIMEOUT` env (default: 180s)

---

## Reasoning Memory V2 API

**Endpoint**: `http://localhost:8131`
**Binary**: Built from `swarm-stack-v2/memory-system/`
**Persistence**: `~/.local/share/reasoning-memory/reasoning_state.jsonl`

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/entries` | All entries (JSON array) |
| GET | `/query/{category}` | Filter by category |
| GET | `/recent?n=N` | Most recent N entries |
| GET | `/search?q=TEXT` | Full-text search |
| GET | `/stats` | Entry count, categories, memory usage |
| GET | `/get?id=ID` | Get specific entry by ID |
| POST | `/put` | Store entry (body: `category\tagent\tconfidence\tttl\tcontent`) |
| POST | `/refresh` | Force re-read from disk |
| POST | `/update` | Update existing entry |
| DELETE | `/gc` | Garbage collect expired entries |
| DELETE | `/entry?id=ID` | Delete specific entry |

### Message Format

```
category\tagent_id\tconfidence\tttl_hours\tcontent
```

**Categories**: `discovery`, `plan`, `reflection`, `shared_state`, `context`, `theory`, `question`

---

## Environment Variables

| Variable | Default | Used By | Purpose |
|----------|---------|---------|---------|
| `MEMORY_URL` | `http://localhost:8131` | Most scripts | Reasoning memory endpoint |
| `VORTEX_URL` | `http://localhost:8120` | bootstrap, hooks | Vortex memory endpoint |
| `CARGO_TARGET_DIR` | `/tmp/cargo-target` | pipeline, quality-gate | Cargo build output |
| `AGENT_ID` | `claude:quality-gate` | quality-gate, task-decompose | Agent identifier |
| `OPERATOR_ID` | `claude:opus-4-6` | stack-up | Operator identifier |
| `FLEET_TABS` | `10 11 12 13 14 15` | fleet-heartbeat, evolution-metrics | Fleet tab numbers |
| `INTERVAL` | 30 / 60 | fleet-heartbeat / watchdog | Scan interval (seconds) |
| `NVIM_SOCK` | `/tmp/nvim.sock` | nvim-ctl | Neovim RPC socket path |
| `VAULT` | `/home/louranicas/projects/claude_code` | obsidian-sync | Obsidian vault path |
| `ARENA` | (swarm-stack-v2 path) | most scripts | Base arena directory |
| `BUS_URL` | `http://localhost:8131` | rust-review | Message bus endpoint |
| `REVIEW_TIMEOUT` | 120 | code-review | Review timeout (seconds) |
| `BUILD_TIMEOUT` | 300 | build-and-test | Build timeout (seconds) |
| `RESEARCH_TIMEOUT` | 180 | research-and-synthesize | Research timeout (seconds) |

---

## Architecture Layers

Each layer depends only on layers below it.

```
Layer 6 — MEMORY & PERSISTENCE
    reasoning-memory v2 (12 endpoints), obsidian-sync, SQLite tracking (3 DBs)

Layer 5 — WORKFLOWS
    pipeline, rust-review, code-review, build-and-test,
    research-and-synthesize, ultraplate-bridge, obsidian-sync

Layer 4 — INTELLIGENCE & PLANNING
    task-decompose, aggregate, evolution-metrics, evolve

Layer 3 — VERIFICATION & MONITORING
    quality-gate, watchdog, fleet-heartbeat

Layer 2 — COORDINATION
    fleet-ctl, agent-bus, context-inject

Layer 1 — PRIMITIVES
    pane-ctl (I/O), nvim-ctl (editor), file-lock (concurrency), stack-up, bootstrap
```

---

## Fleet Operations Guide

### Dispatch Patterns

```bash
# Single task to one agent
fleet-ctl dispatch 10 "implement the authentication module in src/auth.rs"

# Batch from file (one task per line: TAB_NUM TASK)
fleet-ctl batch tasks.txt

# Broadcast same task to all
fleet-ctl broadcast "run cargo clippy -- -D warnings on the current project"

# Monitor progress
fleet-ctl status              # Idle/working state
fleet-heartbeat once          # Detailed scan with token counts

# Collect results
fleet-ctl collect /tmp/fleet-results.md

# Context injection before dispatch
context-inject inject 10 security   # Brief tab 10 on security context
fleet-ctl dispatch 10 "audit the code based on the briefing"
```

### Task Decomposition → Fleet Dispatch

```bash
# Decompose a Rust project into subtasks
task-decompose rust-project ./my-project "full quality audit"
# Generates: /tmp/task-rust-project-*.txt

# Dispatch the generated taskfile
fleet-ctl batch /tmp/task-rust-project-*.txt
```

---

## RALPH Loop Deep-Dive

**R**eflect → **A**nalyze → **L**earn → **P**lan → **H**armonize

| Phase | What Happens |
|-------|-------------|
| **Reflect** | Task dispatched to all fleet workers. Each worker (ALPHA-DELTA) executes independently and returns results. |
| **Analyze** | RALPH Analyst worker spawns, evaluates combined output quality on 0.0-1.0 scale. |
| **Learn** | Patterns and insights extracted from the analysis. What worked, what didn't. |
| **Plan** | If quality < threshold: task specification refined with learnings. If quality >= threshold: proceed to Harmonize. |
| **Harmonize** | Decision gate. Quality passes → COMPLETE. Quality fails → iterate back to Reflect with refined task. |

**Default fleet** (auto-spawned if no workers exist):
- **ALPHA**: Systems Architect — structure, APIs, data flow
- **BETA**: Code Implementer — production code
- **GAMMA**: Test Engineer — tests, edge cases
- **DELTA**: Security Auditor — vulnerabilities, hardening

**Quality threshold**: Default 0.80 (configurable via `swarm-ctl threshold`)
**Max iterations**: Default 5 (configurable in `ralph` command)

```bash
# Start RALPH with custom parameters
swarm-ctl ralph "build a Rust HTTP server with auth" 3 0.90
#                ^task                                ^iter ^threshold
```

---

## Cross-Pane I/O Protocol

The foundational discovery (Session 011b) enabling fleet orchestration.

| Action | Command | Notes |
|--------|---------|-------|
| Type into pane | `zellij action write-chars "text"` | Types literal text |
| Press Enter | `zellij action write 13` | ASCII 13 = Enter |
| Read pane screen | `zellij action dump-screen /tmp/out.txt` | Visible content only |
| Switch tab | `zellij action go-to-tab N` | Tab index (1-based) |
| Cycle panes | `zellij action focus-next-pane` | Within current tab |
| Toggle floating | `zellij action toggle-floating-panes` | Show/hide floating panes |
| New tab | `zellij action new-tab --name "X"` | Create named tab |

**Full feedback loop:**
```bash
zellij action go-to-tab 10                         # Navigate to fleet tab
zellij action dump-screen /tmp/before.txt           # Read current state
zellij action write-chars "cargo build --release"   # Send command
zellij action write 13                              # Press Enter
sleep 10                                            # Wait for completion
zellij action dump-screen /tmp/after.txt            # Read result
zellij action go-to-tab 1                           # Return home
```

**Key insight**: This enables Claude to operate ANY terminal application — Neovim, lazygit, htop, k9s, pgcli, ssh sessions, REPLs — all programmatically from the orchestrator.

---

## ULTRAPLATE Bridge

The swarm stack integrates with the 13-service ULTRAPLATE developer environment.

### Service Registry

| Service | Port | Health Endpoint |
|---------|------|-----------------|
| maintenance-engine | 8080 | `/api/health` |
| devops-engine | 8081 | `/health` |
| synthex | 8090 | `/api/health` |
| san-k7-orchestrator | 8100 | `/health` |
| nais | 8101 | `/health` |
| bash-engine | 8102 | `/health` |
| tool-maker | 8103 | `/health` |
| claude-context-manager | 8104 | `/health` |
| tool-library | 8105 | `/health` |
| codesynthor-v7 | 8110 | `/health` |
| vortex-memory | 8120 | `/health` |

### Start ULTRAPLATE

```bash
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml status
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml health
```

---

## Persistence Architecture

Five storage targets at different timescales for different consumers.

| Target | Consumer | Latency | Tool |
|--------|----------|---------|------|
| Reasoning Memory (:8131) | Running system | Milliseconds | `curl`, all scripts |
| MCP Knowledge Graph | Cross-session queries | Session | `mcp__memory__*` |
| SQLite Tracking (3 DBs) | Quantitative trends | Permanent | `sqlite3` |
| Obsidian Vault | Luke's review | Permanent | `obsidian-sync` |
| CLAUDE.md files (53) | Next Claude instance | Cold-start | Workspace-wide |

### Tracking Databases

```bash
DB_DIR=~/claude-code-workspace/developer_environment_manager

# Service state
sqlite3 $DB_DIR/service_tracking.db "SELECT name, status, health_status FROM services;"

# Synergy scores
sqlite3 $DB_DIR/system_synergy.db "SELECT system_1||' <-> '||system_2, ROUND(synergy_score,1)||'%' FROM system_synergy ORDER BY synergy_score DESC;"

# Neural pathways
sqlite3 $DB_DIR/hebbian_pulse.db "SELECT source_id||' -> '||target_id, ROUND(strength,2) FROM neural_pathways WHERE strength > 0.9;"
```

---

## Troubleshooting

### Shell Alias Traps

```bash
# rm → trash (can't cross /tmp volume boundary)
/usr/bin/rm /tmp/nvim.sock          # Use /usr/bin/rm for /tmp files

# cp → interactive confirmation (blocks on overwrite)
/usr/bin/cp -f source dest          # Use /usr/bin/cp to skip confirmation

# grep → rg (different regex syntax — no \| for alternation)
/usr/bin/grep "error\|warning" file # Use /usr/bin/grep for BRE syntax
rg "error|warning" file             # Or use rg native syntax (no backslash)
```

### Stale Binary

If MCP behavior seems wrong, intent routing collapses, or scores are all identical:
```bash
CARGO_TARGET_DIR=/tmp/cargo-target cargo build --release
# Then restart the process
```

### Port Conflicts

```bash
ss -tlnp 'sport = :8131'           # Check what's on the port
lsof -i :8131                      # Process details
```

### Reasoning Memory Not Responding

```bash
curl -sf http://localhost:8131/health || echo "DOWN"
# Restart:
pkill -f "reasoning.memory" && sleep 1
nohup env PORT=8131 ~/.local/bin/reasoning-memory &
```

---

## File Locations

| Item | Path |
|------|------|
| **Arena (all scripts)** | `the-orchestrator/the developer environment arena/swarm-stack-v2/` |
| **Core scripts** | `swarm-stack-v2/scripts/` (15 scripts) |
| **Workflows** | `swarm-stack-v2/workflows/` (7 scripts) |
| **Memory system source** | `swarm-stack-v2/memory-system/` (Rust) |
| **Plugin hooks** | `swarm-stack-v2/claude-plugin/hooks/` |
| **Installed tools** | `~/.local/bin/` (21+ tools) |
| **WASM plugin** | `~/.config/zellij/plugins/swarm-orchestrator.wasm` |
| **swarm-ctl CLI** | `~/.local/bin/swarm-ctl` |
| **Neovim agent-stack** | `~/.config/nvim/lua/plugins/agent-stack.lua` |
| **Reasoning memory data** | `~/.local/share/reasoning-memory/reasoning_state.jsonl` |
| **Obsidian vault** | `/home/louranicas/projects/claude_code/` |
| **Tracking databases** | `~/claude-code-workspace/developer_environment_manager/*.db` |
| **Evolution state** | `swarm-stack-v2/logs/evolution-state.json` |
| **Evolution logs** | `swarm-stack-v2/logs/evolution.log` |

---

## Related Notes

- [[Swarm Orchestrator — Zellij Plugin]] — Plugin-specific deep-dive
- [[Swarm Orchestrator — Quickstart]] — Quickstart from cold boot
- [[Swarm Orchestrator — Launch Guide]] — Launch procedures
- [[Swarm Stack V2 — Bootstrap Guide]] — Bootstrap reference
- [[Swarm Stack V2 — Evolution Report]] — 21-generation evolution
- [[Swarm Stack V2 — Session 011e Reflections]] — Operator reflections
- [[ULTRAPLATE Developer Environment]] — 13-service mesh
- [[Oscillating Vortex Memory]] — Vortex Memory System
- [[Claudes Reflections]] — Cross-session operator reflections

---

*Generated 2026-03-07 by Claude Opus 4.6 | Session 011e*
*23 tools | 7 workflows | 1 WASM plugin | 12 API endpoints | 53 CLAUDE.md files | 5 persistence targets*
