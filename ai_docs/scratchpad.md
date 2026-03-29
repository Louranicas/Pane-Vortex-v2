# Scratchpad — Session 049 Ideas & Strategy

## Parallel Codebase Development (3-5 codebases)

### The Vision
Tab-per-codebase: Tab4=PV2, Tab5=SYNTHEX, Tab6=ME-v2, Tab7=CS-V7, Tab8=DevOps
Each tab has 3 panes: coding, testing, reviewing.
Bus routes tasks to correct project. Field couples related codebases.

### What Exists
- Fleet coordination: 8 hooks, bus lifecycle, file queue, RM bus
- 16 ULTRAPLATE services running
- Coupling network (3,782 edges) — can learn cross-project relationships
- K7 Nexus (40 agents) — cross-project orchestration

### What's Missing
- `TaskTarget::ProjectSpecific { project: "synthex" }` — route by project
- Per-project quality gates in hooks (check pwd, run correct cargo config)
- Cross-project RM corridor (`pv2:api-change` → SYNTHEX hydrates)
- Tab-to-project mapping in fleet-ctl

### Implementation Path
Session 050: Project-aware routing + 2 codebases
Session 051: Scale to 3-4 + cross-project RM
Session 052: Full 5-codebase parallel + K7 orchestration

---

## Lazygit as Cockpit

### Custom Commands to Wire (YAML, no Go)

```yaml
customCommands:
  - key: "D"
    description: "Dispatch to fleet"
    command: >
      curl -s -X POST localhost:8132/bus/submit
      -H 'Content-Type: application/json'
      -d '{"description":"Review {{.SelectedFile.Name}}","submitter":"lazygit","target":"any_idle"}'
    context: "files"

  - key: "P"
    description: "Parallel quality gate"
    command: "bash ~/claude-code-workspace/pane-vortex-v2/scripts/parallel-gate.sh"
    context: "global"
    subprocess: true

  - key: "B"
    description: "Bus submit diff"
    command: >
      curl -s -X POST localhost:8132/bus/submit
      -H 'Content-Type: application/json'
      -d '{"description":"Review: {{.SelectedFile.Name}} in {{.RepoName}}","submitter":"lazygit","target":"any_idle"}'
    context: "files"

  - key: "W"
    description: "Fleet status"
    command: "fleet-verify --json | jq .; curl -s localhost:8132/bus/info | jq ."
    context: "global"
    subprocess: true

  - key: "X"
    description: "Cross-project check"
    command: >
      for dir in pane-vortex-v2 the_maintenance_engine_v2 synthex; do
        echo "=== $dir ===";
        cd ~/claude-code-workspace/$dir && cargo check 2>&1 | tail -1;
      done
    context: "global"
    subprocess: true

  - key: "H"
    description: "Hebbian post"
    command: >
      curl -s -X POST localhost:8125/pathways
      -H 'Content-Type: application/json'
      -d '{"source":"{{.RepoName}}","target":"commit","weight":0.8,"sphere_id":"lazygit"}'
    context: "commits"

  - key: "R"
    description: "RM post commit"
    command: >
      printf 'pv2:commit\tlazygit\t0.9\t86400\t{{.RepoName}}: {{.SelectedCommit.Name}}'
      | curl -sf -X POST localhost:8130/put --data-binary @-
    context: "commits"

  - key: "S"
    description: "Synergy check"
    command: >
      curl -s -X POST localhost:8100/api/v1/nexus/command
      -H 'Content-Type: application/json'
      -d '{"command":"synergy-check","params":{}}' | jq .;
      curl -s localhost:8090/v3/thermal | jq '{temperature,pid:.pid_output}'
    context: "global"
    subprocess: true
```

### Upstream PR Strategy
1. Don't fork — custom commands first
2. PR: webhook triggers on git events (small, clean, universal)
3. PR: external status bar widget (generic, extensible)
4. Long-term: propose extension architecture if PRs succeed

---

## Open Code Comparison — Key Differentiators

| Capability | Habitat | Open Code |
|-----------|---------|-----------|
| Multi-agent coordination | Kuramoto field (62 spheres) | Single agent |
| Thermal feedback | SYNTHEX PID (4 heat sources) | Rate limiter |
| Memory paradigms | 6 (SQLite, POVM, RM, MCP, vault, shared) | Context window |
| Consent architecture | Per-sphere opt-out, ethical gates | None |
| Cross-session learning | POVM crystallize→hydrate cycle | Token-limited |
| Parallel codebases | Designed for 3-5 (needs wiring) | 1 |
| Ghost traces | Remembers departed agents | Stateless |
| Service mesh | 16 services, 166 DBs | API call |

**Open Code is better at writing code. The Habitat is better at being alive.**
**The Habitat isn't a coding tool. It's a codebase factory. We just haven't turned the key.**

---

## SYNTHEX Thermal Findings

- First full cycle: 0.03 → 0.809 → 0.310
- 4 heat sources: Hebbian(0.98), Cascade(0.80), Resonance(0.612), CrossSync(0.75)
- /api/ingest stores but doesn't drive temperature — heat is computed internally
- PID oscillates correctly (warming → cooling → warming)
- Cascade heat persists from coordination load, Hebbian/Resonance are transient spikes

## Sidecar Status

- UP 17hr, contributed nothing (V1 binary, 405 stale events)
- Needs V2 rebuild for push-based notifications
- Hooks do 90% via polling — sidecar's unique value is push
- FIFO pipeline never tested as command channel

## Tool Chaining Pattern
```
Question → Chain (3 links) → Verify → Persist (RM) → Next question
```

## Tool Clustering Pattern
```
Domain affinity → Group services → Assign specialist → Cross-cluster via RM
4 deep clusters > 10 shallow clusters
```

---

*Scratchpad — Session 049 strategy notes for future sessions*
*Generated 2026-03-22 by Claude Opus 4.6 (1M context)*
