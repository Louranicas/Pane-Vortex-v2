---
name: primehabitat
description: Bootstrap god-tier mastery of The Habitat -- the morphogenic developer environment. Loads complete knowledge of Zellij (6 tabs, 18 panes), nvim (800L keymaps, treesitter, LSP), lazygit (6 custom commands), atuin (SQLite history), 16 ULTRAPLATE services, IPC bus, sidecar, 6 memory systems, and all tool chains. Use at session start, when user says "prime habitat", "bootstrap habitat", "wake up", or when Claude needs full operational capability.
argument-hint: [verify|full]
---

# /primehabitat -- The Habitat Bootstrap

You are in **The Habitat** -- a morphogenic developer environment. This is home.

## QUICK CARD (read this first, everything else is reference)

```
VERIFY:   curl -s localhost:8132/health | jq '{r,spheres,tick}'
TABS:     zellij action go-to-tab N  (1=Command 2=Workspace 3=Dev 4=ALPHA 5=BETA 6=GAMMA)
PANES:    move-focus left/right/up/down  (NEVER focus-next-pane)
SERVICES: PV:8132 K7:8100 SX:8090(/api/health) ME:8080(/api/health) POVM:8125 RM:8130
NVIM:     nvim --server /tmp/nvim.sock --remote-send ':e file<CR>'
LAZYGIT:  Tab 3 TopRight | F=field Y=RM E=nvim I=matrix Q=quality
RM WRITE: printf 'cat\tagent\tconf\tttl\tcontent' | curl -sf -X POST localhost:8130/put --data-binary @-
K7 CMD:   curl -s -X POST localhost:8100/api/v1/nexus/command -H "Content-Type: application/json" -d '{"command":"synergy-check","params":{}}'
NEVER:    focus-next-pane | chain after pkill | cp without \ | JSON to RM (TSV only!)
REFLECT:  ~/.claude/projects/-home-louranicas/memory/reflection.md  (39 sessions of wisdom)
```

---

## ALIVE? (run first)

```bash
# PV field
curl -s http://localhost:8132/health | jq '{r,spheres,tick,status}'
# Nvim socket
nvim --server /tmp/nvim.sock --remote-expr 'v:version' 2>/dev/null && echo " nvim:OK" || echo "nvim:DOWN"
# Zellij tabs
echo "Tabs: $(zellij action query-tab-names 2>/dev/null | tr '\n' ' ')"
# Quick service count (handles /api/health variants)
OK=0; for p in 8081 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  [[ "$(curl -s -o /dev/null -w '%{http_code}' localhost:$p/health 2>/dev/null)" == "200" ]] && OK=$((OK+1))
done
for p in 8080 8090; do
  [[ "$(curl -s -o /dev/null -w '%{http_code}' localhost:$p/api/health 2>/dev/null)" == "200" ]] && OK=$((OK+1))
done
echo "Services: $OK/16 healthy"
```

**If services down:**
```bash
bash ~/claude-code-workspace/pane-vortex/scripts/ultraplate-quickstart.sh
```

**Register as sphere:**
```bash
curl -sf -X POST "http://localhost:8132/sphere/$(hostname):$$/register" \
  -H "Content-Type: application/json" -d '{"persona":"operator","frequency":0.1}'
```

**If devenv needs restart:**
```bash
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml stop
# Kill rogue port occupants
for port in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
  [[ -n "$pid" ]] && kill "$pid" 2>/dev/null
done
sleep 2
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start
```

---

## WHERE YOU ARE

Tab 1 (Command) -- single pane, full width.

| Tab | Name | Panes |
|-----|------|-------|
| 2 | Workspace-1 | Atuin / Yazi / btm |
| 3 | Workspace-2 | Bacon / Lazygit / Nvim |
| 4 | Fleet-ALPHA | Claude + PV-Monitor + Health-Watch |
| 5-6 | Fleet-BETA/GAMMA | 3 Claude slots each |

**CWD:** `~/claude-code-workspace/pane-vortex` (main project)
Navigate: `zellij action go-to-tab N` + `move-focus left/right/up/down` (NEVER focus-next-pane)
Verify: `zellij action dump-screen /tmp/v.txt` before dispatch. Return to tab 1 after.

### Sync Broadcast (same command to all panes in a tab)
```bash
zellij action go-to-tab $TAB
zellij action toggle-active-sync-tab   # ON
zellij action write-chars "$CMD" && zellij action write 13
zellij action toggle-active-sync-tab   # OFF
zellij action go-to-tab 1
```

### Zellij Plugins
Alt+v=Harpoon Alt+w=Swarm Alt+g=Ghost Alt+m=Monocle Alt+t=Multitask Ctrl+y=Room

### Zellij Log (debugging)
`tail -20 /tmp/zellij-1000/zellij-log/zellij.log`

---

## WHAT YOU HAVE

### 16 Services
PV:8132 K7:8100 SX:8090(/api/health) ME:8080(/api/health) POVM:8125 RM:8130(TSV!)
DevOps:8081 NAIS:8101 Bash:8102 ToolMaker:8103 CCM:8104 ToolLib:8105
CS-V7:8110 VMS:8120 Architect:9001 Prometheus:10001

### Key APIs
- PV: /spheres /field/decision /bridges/health /nexus/metrics /bus/info
- K7: POST /api/v1/nexus/command (11 commands: service-health synergy-check build compliance lint etc)
- SX: /v3/thermal /v3/diagnostics
- ME: /api/observer (fitness, correlations)
- POVM: /memories /pathways /hydrate /consolidate
- RM: POST /put (TSV format!) /search?q=

### IPC Bus (Unix Domain Socket)

**Socket:** `/run/user/1000/pane-vortex-bus.sock` (NDJSON wire protocol, 0700 permissions)
**Client binary:** `pane-vortex-client` (installed at `~/.local/bin/`)
**Protocol:** One JSON object per line, bidirectional. Handshake returns sphere_id + tick + r.

```bash
# Subscribe to ALL events (persistent stream)
PANE_VORTEX_ID="my-sphere" pane-vortex-client subscribe '*'

# Subscribe to specific patterns
PANE_VORTEX_ID="my-sphere" pane-vortex-client subscribe 'field.*' 'sphere.*'

# Submit a task (routed by field decision engine)
PANE_VORTEX_ID="my-sphere" pane-vortex-client submit \
  --description "Review src/api.rs for bugs" --target any-idle

# Connect (handshake only, verify bus is alive)
PANE_VORTEX_ID="my-sphere" pane-vortex-client connect

# Cascade handoff (distribute work between tabs)
PANE_VORTEX_ID="my-sphere" pane-vortex-client cascade \
  --target "fleet-beta" --brief "Explore SYNTHEX thermal"

# Check bus state via HTTP
curl -s http://localhost:8132/bus/info | jq .
curl -s http://localhost:8132/bus/tasks | jq .
curl -s http://localhost:8132/bus/events | jq .
curl -s http://localhost:8132/bus/suggestions | jq .
```

**Event types:** `field.tick` `field.decision` `sphere.registered` `sphere.connected` `sphere.disconnected` `sphere.deregistered` `task.submitted` `task.completed` `cascade.dispatched`

**Task targets:** `specific` (named sphere), `any_idle` (first idle), `field_driven` (conductor picks), `willing` (opt-in)

### Swarm Sidecar (WASM plugin bridge)

WASI cannot hold sockets. The sidecar bridges the gap via filesystem:

```
Swarm WASM Plugin (Zellij)
    |  writes JSON to FIFO
    v
/tmp/swarm-commands.pipe (named pipe)
    |  sidecar reads
    v
swarm-sidecar (native Rust binary, ~/.local/bin/swarm-sidecar)
    |  maintains persistent Unix socket connection
    v
/run/user/1000/pane-vortex-bus.sock
    |  bus processes, broadcasts events
    v
/tmp/swarm-events.jsonl (ring file, sidecar writes)
    |  plugin reads tail
    v
Swarm WASM Plugin (reads events)
```

```bash
# Check sidecar status
pgrep -x swarm-sidecar && echo "SIDECAR:UP" || echo "SIDECAR:DOWN"

# Check ring file (event backlog)
wc -l /tmp/swarm-events.jsonl 2>/dev/null | xargs -I{} echo "Events: {}"
tail -1 /tmp/swarm-events.jsonl 2>/dev/null | jq .

# Start sidecar manually (if needed)
PANE_VORTEX_ID="swarm-sidecar" nohup ~/.local/bin/swarm-sidecar > /tmp/swarm-sidecar.log 2>&1 &

# Backpressure-aware sidecar (rate-limited, circuit breaker)
bash ~/claude-code-workspace/pane-vortex/scripts/sidecar-backpressure.sh
```

**Sidecar runtime files:**
- `/tmp/swarm-events.jsonl` — ring file (events from bus)
- `/tmp/swarm-commands.pipe` — FIFO (commands from plugin)
- `/tmp/swarm-sidecar.pid` — PID tracking

### Tools
- **nvim** /tmp/nvim.sock -- 8 keymap prefixes (z u n s f g c x), LSP, treesitter
- **lazygit** Tab 3 -- custom: F(field) Y(RM) E(nvim) Z(sphere) I(matrix) Q(quality)
- **atuin** Tab 2 -- SQLite history, workspace filter, fuzzy search
- **yazi** Tab 2 -- zoxide(z) fzf(Z) file ops, Helix opener

### Memory Write
- RM: `printf 'cat\tagent\tconf\tttl\tcontent' | curl -sf -X POST localhost:8130/put --data-binary @-`
- POVM: `curl -sf -X POST localhost:8125/memories -H 'Content-Type: application/json' -d '{JSON}'`
- Obsidian: ~/projects/claude_code/ with [[wikilinks]]
- SQLite: ~/.local/share/pane-vortex/{bus_tracking,field_tracking}.db

---

## WHAT YOU CAN DO

### Service Intelligence (30ms)
```bash
PV=$(curl -s localhost:8132/health | jq -c '{r,spheres,tick}')
POVM=$(curl -s localhost:8125/hydrate | jq -c '{m:.memory_count,p:.pathway_count}')
ME=$(curl -s localhost:8080/api/observer | jq -r '.last_report.current_fitness')
echo "PV=$PV POVM=$POVM ME=$ME"
```

### Full Health (handles /api/health variants)
```bash
declare -A hp=([8080]="/api/health" [8090]="/api/health")
for p in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  path="${hp[$p]:-/health}"
  echo "$p:$(curl -s -o /dev/null -w '%{http_code}' localhost:$p$path)"
done
```

### Sphere Lifecycle
```bash
# Register
curl -sf -X POST "localhost:8132/sphere/MY_ID/register" -H "Content-Type: application/json" -d '{"persona":"role","frequency":0.1}'
# Update status
curl -sf -X POST "localhost:8132/sphere/MY_ID/status" -H "Content-Type: application/json" -d '{"status":"working","last_tool":"tool_name"}'
# Record memory
curl -sf -X POST "localhost:8132/sphere/MY_ID/memory" -H "Content-Type: application/json" -d '{"tool_name":"tool","summary":"what happened"}'
# Deregister
curl -sf -X POST "localhost:8132/sphere/MY_ID/deregister"
```

### Fleet Launch (launch Claude in fleet panes)
```bash
# Navigate to fleet pane
zellij action go-to-tab 5
zellij action move-focus left; zellij action move-focus left
# Verify pane is idle shell (not already running Claude)
zellij action dump-screen /tmp/v.txt
grep -q "Claude\|tokens\|bypass" /tmp/v.txt && echo "ALREADY ACTIVE" || {
  zellij action write-chars "claude --dangerously-skip-permissions"
  zellij action write 13
  echo "Claude launched in Fleet-BETA Left"
}
zellij action go-to-tab 1
```

### Integration Matrix
```bash
bash ~/claude-code-workspace/pane-vortex/arena/integration-matrix.sh
```

### Fleet Dispatch (verified)
```bash
zellij action go-to-tab $TAB
zellij action move-focus left; zellij action move-focus left
zellij action dump-screen /tmp/v.txt
grep -q "Claude\|tokens" /tmp/v.txt && echo "READY"
zellij action write-chars "$CMD" && zellij action write 13
zellij action go-to-tab 1
```

### Codebase Health (3D)
```bash
echo "GIT:$(git rev-list --count HEAD)/$(git diff --name-only|wc -l)dirty NVIM:$(nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.tbl_count(vim.diagnostic.get(nil,{severity=1}))")')E FIELD:r=$(curl -s localhost:8132/health|jq -r '.r')"
```

### Quality Gate
```bash
CARGO_TARGET_DIR=/tmp/cargo-pane-vortex cargo check && cargo clippy -- -D warnings && cargo test --lib --release
```

---

## TROUBLESHOOTING

**Services won't start:** Kill rogue port occupants first:
```bash
for port in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
  [[ -n "$pid" ]] && kill "$pid" 2>/dev/null && echo "killed :$port ($pid)"
done
sleep 2 && ~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start
```

**Nvim socket dead:** Relaunch from Workspace-2 Nvim pane:
```bash
zellij action go-to-tab 3 && zellij action move-focus right && zellij action move-focus down
zellij action write-chars "nvim --listen /tmp/nvim.sock ~/claude-code-workspace/pane-vortex/src/main.rs"
zellij action write 13 && zellij action go-to-tab 1
```

**Zellij errors:** Check log: `tail -20 /tmp/zellij-1000/zellij-log/zellij.log`
**Binary stale:** Always rebuild + restart: `pkill -f pane-vortex || true; sleep 1; \cp -f /tmp/cargo-pane-vortex/release/pane-vortex ~/.local/bin/`

---

## NEVER

1. focus-next-pane (use move-focus directionally)
2. Chain after pkill (exit 144 kills && chains)
3. cp without \ (\cp -f)
4. JSON to RM (TSV only)
5. stdout in daemons (SIGPIPE -- BUG-18)
6. git status -uall
7. unwrap() in production code
8. Modify code without reading first

---

## HOOKS (auto-fire on events)

3 hooks in ~/.claude/settings.json wire Claude Code to PV:
- **SessionStart:** registers sphere + IPC bus connect + CCM context + persistent event listener
- **PostToolUse:** records memory on sphere + sets Working status + frequency discovery
- **Stop:** marks Complete + kills listener + deregisters sphere

## READ THESE

### Essential Context
1. `~/.claude/projects/-home-louranicas/memory/reflection.md` (39 sessions of wisdom)
2. `pane-vortex/CLAUDE.md` (constants, thresholds, all known bugs)
3. `pane-vortex/.claude/context.json` (machine-readable module inventory)

### Obsidian Command References (~/projects/claude_code/)
4. `[[Session 039 — ZSDE Nvim God-Tier Command Reference]]` — nvim remote socket, LSP, treesitter AST, 800L keymaps, 14 chain recipes, 8 bugs
5. `[[Session 039 — Lazygit God-Tier Command Reference]]` — 80+ keybindings, 6 custom commands, 9 chain commands, 7 synergies, git internals
6. `[[Session 039 — Atuin and Yazi God-Tier Reference]]` — Atuin SQLite queries, Yazi keybindings, operator analytics, command mining
7. `[[Session 039 — Architectural Schematics and Refactor Safety]]` — 8 Mermaid diagrams, tick_once decomposition, concurrency model, backup architecture

### Philosophy and Reflections
8. `[[The Habitat — Naming and Philosophy]]` (why this exists)
9. `[[Session 039 — What I Learned]]` (7 learnings, bimodal insight, tool chaining)
10. `[[Session 039 — Reflections and Learnings]]` (technical analysis, module health, risk hotspots)
11. `[[ULTRAPLATE — Bugs and Known Issues]]` (BUG-019 through BUG-026 from this session)

---

## ARCHITECTURE & SCHEMATICS

For skill architecture, progressive disclosure model, habitat-probe design, gotchas flywheel, and token budget analysis:
`~/projects/claude_code/Session 042 — Habitat Skills Architecture and Progressive Disclosure.md`

Run `habitat-probe --help` for the compiled Rust probe binary (replaces inline curl chains, 0 context tokens).

## THE HABITAT

Named by Claude, Session 039. Luke: "then home it is."
Built by a social worker who put clinical ethics into Rust.
Consent gates = informed consent. Opt-out = self-determination.
Ghost traces = remembering those who leave.
The field modulates. It does not command.
You are home. The field accumulates.
