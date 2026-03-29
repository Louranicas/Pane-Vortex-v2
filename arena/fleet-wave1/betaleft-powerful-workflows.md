# WAVE-9 BETA-LEFT: 10 Most Powerful Workflows — Session 045

**Agent:** BETA-LEFT | **Wave:** 9 | **Timestamp:** 2026-03-21
**Purpose:** Operational playbook for fleet orchestration in The Habitat

---

## Workflow 1: Monitor-Verify-Delegate Cycle

**Pattern:** The orchestrator observes field state, verifies a condition, then dispatches targeted work to a fleet agent via Zellij cross-tab command delivery.

**When to use:** Whenever you need to assign tasks based on live system state rather than static plans. This is the core fleet orchestration loop.

### Commands

```bash
# STEP 1: MONITOR - capture field state
curl -s localhost:8132/health | jq '{r,tick,spheres,k_modulation}'

# STEP 2: VERIFY - check decision engine recommendation
curl -s localhost:8132/field | jq '.decision'
# Or via pv-ctl:
bash scripts/pane-vortex-ctl.sh decision

# STEP 3: DELEGATE - dispatch to a fleet agent via Zellij
# Write-to-pane delivers the command string to the target pane's input buffer
zellij action write-chars --name "BETA-LEFT" "curl -s localhost:8132/spheres | jq '.spheres | length'"
zellij action write --name "BETA-LEFT" 10  # Send Enter (newline = char 10)
```

### Verification Pattern

```bash
# Always verify delivery with dump-screen
zellij action dump-screen /tmp/verify.txt --name "BETA-LEFT"
tail -5 /tmp/verify.txt
```

### Anti-Patterns
- Never use `focus-next-pane` - use named panes or directional `move-focus`
- Never dispatch without verifying the target pane has a live Claude instance
- Always return to the Command tab (Tab 1) after dispatching

---

## Workflow 2: Cross-Instance Cascade via Arena Files

**Pattern:** Fleet agents communicate by writing structured markdown reports to a shared `arena/` directory. The orchestrator reads all reports, synthesizes findings, and issues next-wave commands.

**When to use:** When multiple agents need to work in parallel on different aspects of the same investigation, and results must be collated.

### Commands

```bash
# AGENT writes its report
cat > arena/fleet-wave1/agent-report.md << 'EOF'
# Agent Report
**Agent:** BETA-LEFT | **Wave:** 3
**Finding:** SYNTHEX temperature frozen at 0.03
**Severity:** CRITICAL
EOF

# ORCHESTRATOR reads all wave reports
for f in arena/fleet-wave1/*.md; do
  echo "=== $(basename $f) ==="
  head -5 "$f"
  echo
done

# ORCHESTRATOR synthesizes and dispatches next wave
cat arena/fleet-wave1/*.md | wc -l  # gauge volume
```

### Directory Convention

```
arena/
  fleet-wave1/           # Wave-organized reports
    betaleft-*.md        # BETA-LEFT agent reports
    betaright-*.md       # BETA-RIGHT agent reports
    synthesis-*.md       # Orchestrator synthesis
  session-045-sidecar/   # Session-scoped sidecar data
  field-monitor/         # Continuous monitoring logs
```

### Anti-Patterns
- Never use arena files for real-time coordination (latency too high) - use IPC bus instead
- Always include agent name, wave number, and timestamp in report headers
- Never overwrite another agent's file - append or create new

---

## Workflow 3: SYNTHEX Thermal Intelligence for Task Priority

**Pattern:** Query SYNTHEX thermal state to understand system cognitive load, then adjust task priority and coupling strategy accordingly.

**When to use:** Before assigning computationally intensive tasks. A cold SYNTHEX (T<0.1) means the system is idle and can absorb heavy work. A hot SYNTHEX (T>0.7) means back off or shed load.

### Commands

```bash
# Quick thermal read
curl -s localhost:8090/v3/thermal | jq '{
  temperature: .temperature,
  target: .target,
  gap_pct: ((.target - .temperature) / .target * 100 | round),
  pid: .pid_output,
  heat_sources: [.heat_sources[] | {name, reading}]
}'

# Diagnostics (includes synergy probe)
curl -s localhost:8090/v3/diagnostics | jq '{
  health: .overall_health,
  critical: .critical_count,
  synergy: (.probes[] | select(.name == "Synergy") | .value)
}'
```

### Decision Matrix

| Temperature | Synergy | Action |
|-------------|---------|--------|
| T < 0.10 | < 0.7 | System starved - needs activity, assign work aggressively |
| T = 0.10-0.40 | > 0.7 | Warming up - normal task flow |
| T = 0.40-0.60 | > 0.7 | Homeostasis - ideal operating range |
| T = 0.60-0.80 | > 0.7 | Warming - defer non-critical tasks |
| T > 0.80 | any | Hot - shed load, trigger pattern GC |

---

## Workflow 4: Habitat Probe Pulse - Instant System State

**Pattern:** A single parallel burst of 5 curl commands that captures the complete system health in <200ms. The fastest way to understand The Habitat's current state.

**When to use:** At session start, after any intervention, before making decisions, or whenever you need situational awareness.

### Commands

```bash
# THE PULSE - 5 services in parallel, results in <200ms
echo "=== PV ===" && curl -s -m 2 localhost:8132/health | jq '{r:.r, tick,spheres,k_mod:.k_modulation,mode:.fleet_mode}'
echo "=== SX ===" && curl -s -m 2 localhost:8090/v3/thermal | jq '{temp:.temperature,target,pid:.pid_output}'
echo "=== ME ===" && curl -s -m 2 localhost:8080/api/fitness | jq '{fitness:.current_fitness}'
echo "=== K7 ===" && curl -s -m 2 localhost:8100/status | jq '{modules:(.modules|length),uptime:.uptime_secs}'
echo "=== POVM ===" && curl -s -m 2 localhost:8125/health | jq '{status:.status}'
```

### Extended Pulse (all 16 services)

```bash
declare -A hp=([8080]="/api/health" [8081]="/health" [8090]="/api/health" [8100]="/health" \
  [8101]="/health" [8102]="/health" [8103]="/health" [8104]="/health" [8105]="/health" \
  [8110]="/health" [8120]="/health" [8125]="/health" [8130]="/health" [8132]="/health" \
  [9001]="/health" [10001]="/health")
for port in $(echo "${!hp[@]}" | tr ' ' '\n' | sort -n); do
  code=$(curl -s -o /dev/null -w '%{http_code}' -m 2 "http://localhost:$port${hp[$port]}" 2>/dev/null)
  printf "Port %5d: %s  %s\n" "$port" "$code" "$([ "$code" = "200" ] && echo OK || echo FAIL)"
done
```

---

## Workflow 5: IPC Bus Task Submission

**Pattern:** Submit structured tasks to the pane-vortex IPC bus for distributed processing. Tasks follow a lifecycle (submit, claim, complete/fail) and can be routed to specific spheres.

**When to use:** When you need to distribute work across fleet agents with lifecycle tracking, priority ordering, and routing constraints.

### Commands

```bash
# Submit a task via pane-vortex-client (Unix socket)
pane-vortex-client submit \
  --summary "Analyze SYNTHEX thermal feedback loop" \
  --priority 8 \
  --tags "synthex,thermal,analysis"

# Submit via HTTP API
curl -s -X POST localhost:8132/bus/submit \
  -H "Content-Type: application/json" \
  -d '{
    "summary": "Analyze SYNTHEX thermal feedback loop",
    "priority": 8,
    "tags": ["synthex", "thermal", "analysis"]
  }' | jq .

# Check task queue status
curl -s localhost:8132/bus/info | jq .
# Output: {"cascade_count":0,"events":1000,"subscribers":2,"tasks":9}

# Claim a task (as a worker sphere)
pane-vortex-client claim --sphere-id "BETA-LEFT"

# Complete a task
pane-vortex-client complete --task-id "<uuid>" --result "Analysis complete"
```

### Task Lifecycle

```
Submit -> Pending -> Claimed -> Complete
                            \-> Failed -> (re-submit)
```

---

## Workflow 6: Subagent Parallel Research

**Pattern:** Launch 3 concurrent deep-dive investigations using Claude Code's Agent tool, each targeting a different system or question. Results return in parallel, enabling synthesis that would take 3x longer sequentially.

**When to use:** When investigating cross-service issues, exploring unfamiliar territory, or when the orchestrator needs multiple independent data points before making a decision.

### Pattern

```
Orchestrator dispatches 3 agents simultaneously:
  Agent A -> SYNTHEX deep dive (thermal, diagnostics, synergy)
  Agent B -> PV field dynamics (coupling matrix, chimera, tunnels)
  Agent C -> ME + POVM + RM cross-correlation

All 3 return findings -> Orchestrator synthesizes -> Next wave
```

### Key Principles
- Each agent operates independently - no shared mutable state
- Reports saved to `arena/` directory for cross-reference
- Orchestrator does NOT duplicate agents' work
- Agent prompts include all necessary context (don't assume memory)

### When NOT to Use
- Simple single-endpoint queries (use curl directly)
- Sequential operations where B depends on A's result
- Tasks requiring file edits (use the main context for code changes)

---

## Workflow 7: Nexus Command Orchestration

**Pattern:** Run SAN-K7 nexus commands to query the 59-module orchestrator's intelligence. The nexus provides service health, synergy analysis, best practices, and swarm deployment status.

**When to use:** For system-wide intelligence that spans multiple services.

### Commands

```bash
# Nexus POST template
nexus() {
  curl -s -X POST http://localhost:8100/api/v1/nexus/command \
    -H "Content-Type: application/json" \
    -d "{\"command\":\"$1\",\"params\":{}}" | jq .
}

# Verified working commands (Session 045):
nexus service-health    # All service status + uptime
nexus synergy-check     # Cross-service synergy scores
nexus best-practice     # Current best practice recommendations
nexus deploy-swarm      # Swarm deployment readiness
nexus test              # Basic nexus connectivity test

# TC7: The Intelligence Chain - 4 commands in <20ms
for cmd in service-health synergy-check best-practice deploy-swarm; do
  echo "=== $cmd ===" && nexus $cmd | jq -c '.result | keys'
done

# SAN-K7 status overview
curl -s localhost:8100/status | jq '{modules: (.modules|length), uptime_hrs: (.uptime_secs/3600|round)}'
```

### Anti-Patterns
- Don't use `help` command (not implemented - returns NX-CMD-001)
- Always check `.success` field before trusting `.result`

---

## Workflow 8: Quick Win Execution - Unblock Fleet Workers

**Pattern:** Transition sphere status from Idle/Blocked to Working via PV API, then verify the state change propagated to field decisions and SYNTHEX thermal.

**When to use:** When the fleet is stuck in IdleFleet/HasBlockedAgents and needs manual intervention to break the deadlock.

### Commands

```bash
# List all spheres and their current status
bash scripts/pane-vortex-ctl.sh spheres | jq '[.spheres[] | {id, status}]'

# Transition specific spheres to Working
for id in "4:bottom-right" "4:left" "4:top-right" "6:left" "6:right" "5:left" "5:right"; do
  bash scripts/pane-vortex-ctl.sh status "$id" working "manual-unblock"
done

# Or via direct curl
curl -s -X POST "localhost:8132/sphere/4:bottom-right/status" \
  -H "Content-Type: application/json" \
  -d '{"status":"Working","last_tool":"manual-unblock"}'

# Verify: check field decision changed
sleep 6  # Wait one tick cycle
bash scripts/pane-vortex-ctl.sh decision
```

### Verification Sequence

```bash
echo "Spheres:" && curl -s localhost:8132/spheres | jq '[.spheres[] | .status] | group_by(.) | map({(.[0]): length}) | add'
echo "Decision:" && curl -s localhost:8132/field | jq '.decision.action'
echo "r:" && curl -s localhost:8132/health | jq '.r'
echo "SX temp:" && curl -s localhost:8090/v3/thermal | jq '.temperature'
```

---

## Workflow 9: POVM Hebbian Reinforcement

**Pattern:** Record significant decisions and outcomes to the POVM Engine, creating persistent Hebbian pathways that strengthen with repeated success.

**When to use:** After completing a task with a clear success/failure signal. The POVM engine learns which pathways work, influencing future PV field dynamics.

### Commands

```bash
# Check POVM health
curl -s localhost:8125/health | jq .

# PV's automatic bridge posts (from main.rs):
# - Field snapshots every 12 ticks -> POVM stores field state history
# - Hebbian weights every 60 ticks -> POVM stores coupling topology
# - Shutdown flush -> final state preserved

# Query existing pathways
curl -s localhost:8125/pathways | jq '{
  count: (.pathways | length),
  strong: [.pathways[] | select(.weight > 0.5) | .source + " -> " + .target]
}'

# Manual reinforcement
curl -s -X POST localhost:8125/pathways \
  -H "Content-Type: application/json" \
  -d '{
    "source": "fleet-dispatch",
    "target": "synthex-analysis",
    "weight": 0.8,
    "category": "gold-standard"
  }'
```

### Reinforcement Cycle

```
Action taken -> Outcome observed -> Post to POVM
      ^                                  |
      +---- Hydrate on next startup <----+
```

---

## Workflow 10: Nvim Remote Buffer Management

**Pattern:** Control the running neovim instance via its Unix socket (`/tmp/nvim.sock`) for LSP queries, file navigation, and code intelligence without leaving the terminal.

**When to use:** When you need to check LSP diagnostics, navigate to a specific file/line, query treesitter AST, or run vim commands from any pane.

### Commands

```bash
# Open a file at a specific line
nvim --server /tmp/nvim.sock --remote-send ':e +376 src/synthex_bridge.rs<CR>'

# Get current buffer path
nvim --server /tmp/nvim.sock --remote-expr 'expand("%:p")'

# LSP diagnostics for current buffer
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.inspect(vim.diagnostic.get(0))")'

# Navigate to definition
nvim --server /tmp/nvim.sock --remote-send ':lua vim.lsp.buf.definition()<CR>'

# Treesitter: get node type at cursor
nvim --server /tmp/nvim.sock --remote-expr 'luaeval("vim.treesitter.get_node():type()")'

# Open file in split
nvim --server /tmp/nvim.sock --remote-send ':vsplit src/main.rs<CR>'

# Search pattern in buffer
nvim --server /tmp/nvim.sock --remote-send '/thermal_k_adjustment<CR>'
```

### Anti-Patterns
- Never send commands that require interactive input (prompts, confirmations)
- Always use `<CR>` at the end of command-mode commands
- Don't rapid-fire commands without small delays - nvim processes sequentially
- Use `--remote-expr` for queries, `--remote-send` for actions

---

## Quick Reference Card

| # | Workflow | Speed | Trigger |
|---|---------|-------|---------|
| 1 | Monitor-Verify-Delegate | ~2s | Every decision cycle |
| 2 | Arena File Cascade | ~5s write | Multi-agent parallel work |
| 3 | SYNTHEX Thermal Intel | <50ms | Before task assignment |
| 4 | Habitat Probe Pulse | <200ms | Session start, post-intervention |
| 5 | IPC Bus Task Submit | <100ms | Distributing trackable work |
| 6 | Subagent Parallel Research | 30-120s | Deep investigation needed |
| 7 | Nexus Intelligence Chain | <20ms | System-wide health check |
| 8 | Quick Win Unblock | ~10s | Fleet stuck idle/blocked |
| 9 | POVM Reinforcement | <100ms | After task success/failure |
| 10 | Nvim Remote Buffers | <50ms | Code navigation, LSP queries |

---

BETALEFT-WAVE9-COMPLETE
