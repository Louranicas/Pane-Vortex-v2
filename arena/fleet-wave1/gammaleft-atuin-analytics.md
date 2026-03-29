# WAVE-7 GAMMA-LEFT: Atuin Command Analytics

> **Agent:** GAMMA-LEFT | **Date:** 2026-03-21 | **Wave:** 7
> **Source:** Atuin 18.10 shell history (1,973 total commands, 880 unique)

---

## 1. Top Commands (Overall)

| Rank | Command | Count | % | Role |
|------|---------|-------|---|------|
| 1 | `claude` | 470 | 23.8% | Claude Code invocations |
| 2 | `python3` | 246 | 12.5% | Data analysis + scripting |
| 3 | `cd` | 214 | 10.8% | Navigation |
| 4 | `source` | 210 | 10.6% | Shell env loading |
| 5 | `echo` | 170 | 8.6% | Output + fleet markers |
| 6 | `alacritty` | 110 | 5.6% | Terminal spawning |
| 7 | `zellij` | 78 | 4.0% | Tab/pane management |
| 8 | `curl` | 40 | 2.0% | Service API calls |
| 9 | `exit` | 37 | 1.9% | Session teardown |
| 10 | `export` | 28 | 1.4% | Env config |

### Distribution Shape

```
claude   ████████████████████████  470
python3  █████████████             246
cd       ███████████               214
source   ███████████               210
echo     █████████                 170
alacritt ██████                    110
zellij   ████                       78
curl     ██                         40
exit     ██                         37
export   █                          28
```

**`claude` dominates** at 23.8% — nearly 1 in 4 commands is a Claude Code invocation. The Habitat is Claude-centric.

---

## 2. Fleet Orchestration Tool Frequency

| Tool Category | Count | Purpose |
|---------------|-------|---------|
| PV API calls (curl :8132) | 60 | Field state, spheres, proposals, decisions |
| Arena references | 57 | `/tmp/arena/` analysis files |
| Fleet commands | 44 | Fleet dispatch, broadcast, collect |
| RM calls (curl :8130) | 35 | Reasoning Memory writes |
| SYNTHEX calls (curl :8090) | 33 | Thermal state, homeostasis |
| ME calls (curl :8080) | 29 | Evolution, observer, fitness |
| SAN-K7 calls (curl :8100) | 29 | Nexus commands, status |
| Tool Library (curl :8105) | 24 | Tool registry |
| POVM calls (curl :8125) | 21 | Pathways, memories |
| CS-V7 calls (curl :8110) | 19 | CodeSynthor status |
| DevOps calls (curl :8081) | 18 | Agents, pipelines |
| NAIS calls (curl :8101) | 18 | Neural adaptive |
| Bash Engine (curl :8102) | 17 | Safety patterns |
| sqlite3 | 15 | Direct DB queries |
| VMS calls (curl :8120) | 14 | Vortex memory |
| CCM calls (curl :8104) | 13 | Context manager |
| Tool Maker (curl :8103) | 14 | Tool creation |
| cargo | 12 | Build + test |
| Architect (curl :9001) | 7 | Pattern library |
| Prometheus (curl :10001) | 7 | Swarm agents |
| habitat-probe | 1 | Probe binary (new) |

### Service API Call Heatmap

```
:8132 PV        ████████████████████████████████████████████████████████████  60
:8130 RM        ███████████████████████████████████                          35
:8090 SYNTHEX   █████████████████████████████████                            33
:8080 ME        █████████████████████████████                                29
:8100 SAN-K7    █████████████████████████████                                29
:8105 ToolLib   ████████████████████████                                     24
:8125 POVM      █████████████████████                                        21
:8110 CS-V7     ███████████████████                                          19
:8081 DevOps    ██████████████████                                           18
:8101 NAIS      ██████████████████                                           18
:8102 Bash      █████████████████                                            17
:8120 VMS       ██████████████                                               14
:8103 ToolMkr   ██████████████                                               14
:8104 CCM       █████████████                                                13
:9001 Architect ███████                                                       7
:10001 Prom     ███████                                                       7
```

**PV (8132) is the most queried service** at 60 calls — it's the fleet coordination hub. RM (8130) and SYNTHEX (8090) are the #2 and #3 most queried, reflecting their roles as memory store and brain respectively.

---

## 3. Fleet Orchestration Patterns

### Most Common curl Targets on PV (8132)

From atuin history, the most frequent PV API patterns:

| Endpoint Pattern | Purpose |
|-----------------|---------|
| `/health` | Health checks (most frequent) |
| `/field/decision` | Decision engine state |
| `/field/spectrum` | Spherical harmonics |
| `/field/proposals` | Governance proposals |
| `/sphere/*/status` | Sphere status updates |
| `/spheres` | Sphere listing |

### Fleet Command Categories

| Category | Example Commands | Count |
|----------|-----------------|-------|
| Fleet dispatch | `echo 'PAR-*' && habitat-probe pulse` | ~20 |
| Arena file I/O | `> /tmp/arena/*.txt && cat` | ~57 |
| SQLite analysis | `sqlite3 -header -column *.db` | ~15 |
| Service health | `curl -s localhost:PORT/health` | ~40 |
| Git operations | `git log`, `git status`, `git diff` | ~10 |

### Habitat Tool Usage (searched "habitat")

Only 1 `habitat-probe` call found in atuin — the probe binary is very new (Session 045). Most service probing is still done via raw `curl` commands rather than through the unified probe interface.

---

## 4. Session Workflow Pattern

Based on command sequencing, the typical fleet orchestration workflow is:

```
1. claude (invoke Claude Code instance)           470x
2. source (load shell environment + aliases)       210x
3. cd (navigate to project)                        214x
4. curl (probe services)                            40x
5. python3 (analyze data in-place)                 246x
6. echo (emit markers + fleet coordination)        170x
7. sqlite3 (mine DBs for insights)                  15x
8. zellij (manage tabs/panes)                       78x
```

### Key Insight: Python3 as Analytical Glue

`python3` at 246 calls (12.5%) is the second most-used command. It's used inline for:
- JSON parsing (`import json; d=json.load(sys.stdin)`)
- Data aggregation (`sorted, group_by, Counter`)
- Metric computation (`statistics, math`)
- File I/O (`glob.glob, open`)

This makes Python3 the **analytical glue** of the Habitat — bridging raw curl output to structured insights.

---

## 5. Underutilized Tools

| Tool | Calls | Expected Use | Gap |
|------|-------|-------------|-----|
| `habitat-probe` | 1 | Should replace raw curl for health checks | **CRITICAL** — probe binary exists but isn't adopted |
| `pane-vortex-client` | 0 (in atuin) | IPC bus CLI for task submission | Not used from shell |
| `yazi` | 17 | File navigation | Moderate use |
| `atuin` | 10 | History analytics | Self-referential |
| `cargo` | 12 | Build/test | Low — most builds done by Claude Code directly |

---

## 6. Summary

The Habitat command profile reveals a **Claude-centric, curl-heavy, Python-analytical** workflow:

- **23.8% Claude** — the Habitat IS Claude Code
- **PV (8132) most queried** — 60 API calls, the fleet coordination hub
- **Python3 is analytical glue** — 246 inline analysis commands
- **habitat-probe underadopted** — only 1 call despite being available
- **16 services probed** — all ULTRAPLATE services have been queried at least 7 times
- **Fleet orchestration is manual** — relay files, echo markers, raw curls rather than structured dispatch tools

---

GAMMALEFT-WAVE7-COMPLETE
