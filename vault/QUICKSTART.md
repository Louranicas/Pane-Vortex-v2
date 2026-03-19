---
title: "Pane-Vortex V2 — Comprehensive Quickstart & Deployment Guide"
date: 2026-03-19
tags: [quickstart, build, deploy, end-to-end, pane-vortex-v2, habitat]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
claude_md: "../CLAUDE.md"
claude_local_md: "../CLAUDE.local.md"
---

# Pane-Vortex V2 — Comprehensive Quickstart

> **Full end-to-end deployment guide. Nothing missed.**
> **Bootstrap:** [CLAUDE.md](../CLAUDE.md) | **Implementation phases:** [CLAUDE.local.md](../CLAUDE.local.md)
> **Master plan:** [MASTERPLAN.md](../MASTERPLAN.md) | **Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`

---

## 0. Orientation (Read First)

| Document | Path | What It Tells You |
|----------|------|-------------------|
| **[CLAUDE.md](../CLAUDE.md)** | Project root | Architecture (8 layers, 41 modules), quality gate, rules, constants, anti-patterns |
| **[CLAUDE.local.md](../CLAUDE.local.md)** | Project root | V3 plan status, implementation order (L1→L3→L4→L5→L2→L6→L7→L8), traps to avoid |
| **[MASTERPLAN.md](../MASTERPLAN.md)** | Project root | V3 plan (5 phases, 499 lines, 99 Obsidian cross-refs), 9 critical alerts, 13 NA gaps |
| **[ONBOARDING.md](ONBOARDING.md)** | ai_docs/ | 5-level progressive reading order for new Claude instances |
| **[DESIGN_CONSTRAINTS.md](../ai_specs/DESIGN_CONSTRAINTS.md)** | ai_specs/ | C1-C14 constraint definitions — the rules that govern all code |

**New to this project?** Read in this order:
1. This file (QUICKSTART)
2. [CLAUDE.md](../CLAUDE.md)
3. [CLAUDE.local.md](../CLAUDE.local.md) — especially "Implementation Order"
4. [ONBOARDING.md](ONBOARDING.md) — then follow its 5-level reading path

---

## 1. Prerequisites

### Required

| Tool | Version | Check | Notes |
|------|---------|-------|-------|
| Rust | 1.75+ | `rustc --version` | Stable toolchain |
| Cargo | Latest | `cargo --version` | Comes with Rust |
| curl | Any | `curl --version` | Health checks |
| jq | 1.6+ | `jq --version` | JSON formatting |

### Optional but Recommended

| Tool | Purpose | Check |
|------|---------|-------|
| bacon | Continuous compiler | `bacon --version` (config at `bacon.toml`) |
| sqlite3 | DB inspection | `sqlite3 --version` |
| lazygit | Git UI with PV integration | `lazygit --version` |
| nvim | Editor with 801L keymaps | `nvim --version` |
| zellij | Multiplexer (The Habitat) | `zellij --version` |

### System Dependencies

The project uses `rusqlite` with `bundled` feature — **no system SQLite headers needed**. All 152 Rust crate dependencies are already downloaded to `/tmp/cargo-pv2/`.

---

## 2. Project Structure

```
pane-vortex-v2/
├── CLAUDE.md                    # Bootstrap context (READ FIRST)
├── CLAUDE.local.md              # Implementation phases + V3 status
├── MASTERPLAN.md                # V3 plan (499L, 99 Obsidian refs)
├── Cargo.toml                   # 15 deps, 5 feature gates, clippy deny rules
├── bacon.toml                   # 5-job continuous compiler config
├── README.md                    # Project overview
├── CHANGELOG.md                 # Version history
├── .gitignore                   # Rust + IDE + runtime exclusions
│
├── src/                         # 8 layers, 41 modules, 59 Rust files
│   ├── lib.rs                   # Crate root — #![forbid(unsafe_code)]
│   ├── bin/main.rs              # Daemon entry point
│   ├── bin/client.rs            # IPC bus CLI client
│   ├── m1_foundation/           # L1: types, errors, config, constants, traits, validation
│   ├── m2_services/             # L2: registry, health, lifecycle, API server
│   ├── m3_field/                # L3: sphere, field state, chimera, messaging, app state
│   ├── m4_coupling/             # L4: Kuramoto network, auto-K, topology
│   ├── m5_learning/             # L5: Hebbian STDP, buoy network, memory
│   ├── m6_bridges/              # L6: 6 bridges + consent gate
│   ├── m7_coordination/         # L7: IPC bus, conductor, executor, tick, persistence
│   └── m8_governance/           # L8: proposals, voting, consent, data sovereignty
│
├── ai_docs/                     # 20 documentation files (5,714 lines)
│   ├── INDEX.md                 # Navigation hub
│   ├── ONBOARDING.md            # 5-level reading order
│   ├── ARCHITECTURE_DEEP_DIVE.md
│   ├── CODE_MODULE_MAP.md       # All 41 modules with signatures
│   ├── SCHEMATICS.md            # 6 Mermaid diagrams
│   └── modules/                 # L1-L8 per-layer docs
│
├── ai_specs/                    # 28 specification files (10,477 lines)
│   ├── INDEX.md                 # Spec navigation
│   ├── KURAMOTO_FIELD_SPEC.md   # The math
│   ├── IPC_BUS_SPEC.md          # Unix socket architecture
│   ├── API_SPEC.md              # 76 HTTP endpoints
│   ├── CONSENT_SPEC.md          # NA sovereignty framework
│   ├── DESIGN_CONSTRAINTS.md    # C1-C14 rules
│   ├── layers/                  # L1-L8 per-layer specs
│   └── patterns/                # 79 patterns + 42 anti-patterns
│
├── config/                      # Configuration
│   ├── default.toml             # 10 sections, all defaults documented
│   └── production.toml          # Production overrides
│
├── migrations/                  # SQLite schema
│   ├── 001_field_tables.sql     # 3 tables: field_snapshots, sphere_history, coupling_history
│   ├── 002_bus_tables.sql       # 6 tables: bus_tasks, bus_events, subscriptions, cascade, tags, deps
│   └── 003_governance_tables.sql # 4 tables: proposals, votes, consent_declarations, data_manifests
│
├── .claude/                     # Operational context
│   ├── context.json             # Machine-readable module inventory
│   ├── status.json              # Heartbeat
│   ├── patterns.json            # 15 cached patterns
│   ├── anti_patterns.json       # 15 anti-patterns
│   ├── queries/                 # 3 SQL template files (field, bus, governance)
│   ├── schemas/                 # 2 JSON schemas (bus_frame, bus_event)
│   └── skills/deephabitat/      # /deephabitat skill
│
├── hooks/                       # Claude Code session hooks
│   ├── session_start.sh         # Register sphere + start bus listener
│   ├── post_tool_use.sh         # Record memory + frequency discovery
│   └── session_end.sh           # Deregister sphere + cleanup
│
├── tests/                       # 6 integration test stubs
├── benches/                     # tick_loop.rs benchmark stub
├── scripts/                     # verify-scaffold.sh
├── data/                        # Runtime SQLite databases (created at startup)
└── vault/                       # Obsidian vault (60 notes)
```

---

## 3. Build

### First Build (downloads all dependencies)

```bash
cd ~/claude-code-workspace/pane-vortex-v2
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release 2>&1 | tail -20
```

First build takes ~30-60s (compiling 152 crates). Subsequent builds are incremental (~1-5s).

### Feature Flags

| Feature | Default | What It Enables |
|---------|---------|-----------------|
| `api` | YES | Axum HTTP server (m10) |
| `persistence` | YES | SQLite WAL persistence (m36) |
| `bridges` | YES | External service bridges (m22-m28) |
| `evolution` | NO | Evolution chamber (m41) |
| `governance` | NO | Collective governance (m37-m40) |
| `full` | NO | All features enabled |

```bash
# Default build (api + persistence + bridges)
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release

# Full build (includes governance + evolution)
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release --features full

# Minimal build (no optional features)
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release --no-default-features
```

### Binary Locations

| Binary | Path | Size (approx) |
|--------|------|---------------|
| `pane-vortex` | `/tmp/cargo-pv2/release/pane-vortex` | ~6MB |
| `pane-vortex-client` | `/tmp/cargo-pv2/release/pane-vortex-client` | ~3MB |

---

## 4. Quality Gate (MANDATORY — Zero Tolerance)

**PRIME DIRECTIVE: Never suppress warnings. Fix at source.**

```bash
cd ~/claude-code-workspace/pane-vortex-v2

# All 4 stages must pass. Run sequentially:
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```

**Enforced at crate level** (see [CLAUDE.md](../CLAUDE.md)):
- `#![forbid(unsafe_code)]` — zero `unsafe` blocks
- `#![deny(clippy::unwrap_used)]` — zero `.unwrap()` in production
- `#![deny(clippy::expect_used)]` — zero `.expect()` in production
- `#![warn(clippy::pedantic)]` — pedantic lint warnings
- `#![warn(clippy::nursery)]` — nursery lint warnings

**Enforced in Cargo.toml:**
```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

### Continuous Compilation (bacon)

```bash
cd ~/claude-code-workspace/pane-vortex-v2
bacon              # default: cargo check
bacon clippy       # clippy warnings
bacon pedantic     # pedantic warnings
bacon test         # cargo test
bacon gate         # full quality gate (all 4 stages)
```

---

## 5. Deploy

### Step 1: Kill existing process (ALWAYS separate command)

```bash
pkill -f "pane-vortex" || true
# NEVER chain: pkill && cp — exit code 144 kills the chain!
```

### Step 2: Wait

```bash
sleep 1
```

### Step 3: Copy binaries (bypass cp alias)

```bash
\cp -f /tmp/cargo-pv2/release/pane-vortex bin/pane-vortex
\cp -f /tmp/cargo-pv2/release/pane-vortex ~/.local/bin/pane-vortex
\cp -f /tmp/cargo-pv2/release/pane-vortex-client ~/.local/bin/pane-vortex-client
```

### Step 4: Start via DevEnv (PREFERRED)

```bash
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart pane-vortex
```

### Alternative: Start manually (no PID tracking)

```bash
PORT=8132 nohup ~/.local/bin/pane-vortex > /tmp/pane-vortex.log 2>&1 &
```

**WARNING:** Never write to stdout in daemon mode → SIGPIPE death (BUG-018). Use file-based tracing.

### Step 5: Verify

```bash
curl -s http://localhost:8132/health | jq .
```

---

## 6. ULTRAPLATE Service Dependencies

PV is **Batch 5** — it depends on services from Batches 1-4. All must be running.

### Batch Startup Order

```
Batch 1 (no deps):  devops-engine:8081, codesynthor-v7:8110, povm-engine:8125, reasoning-memory:8130
Batch 2 (needs B1): synthex:8090, san-k7:8100, maintenance-engine:8080, architect:9001, prometheus:10001
Batch 3 (needs B2): nais:8101, bash-engine:8102, tool-maker:8103
Batch 4 (needs B3): claude-context-manager:8104, tool-library:8105
Batch 5 (needs B4): vortex-memory-system:8120, **pane-vortex:8132**
```

### Start All 16 Services

```bash
# Bulletproof start: kill stale port occupants first
for port in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
  [[ -n "$pid" ]] && kill "$pid" 2>/dev/null
done
sleep 2
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start
```

### Quick Health Check (all 16)

```bash
declare -A hp=([8080]="/api/health" [8090]="/api/health")
for p in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  path="${hp[$p]:-/health}"
  echo "$p:$(curl -s -o /dev/null -w '%{http_code}' localhost:$p$path 2>/dev/null)"
done
```

**Expected:** 16/16 returning 200 (Prometheus Swarm at 10001 may be unstable — see ALERT-3).

---

## 7. Bridge Configuration

PV connects to 6 external services. All bridges use raw TCP HTTP (no hyper/reqwest).

| Bridge | Service | Port | Health Path | Poll Interval | Module |
|--------|---------|------|-------------|---------------|--------|
| SYNTHEX | Thermal PID controller | 8090 | `/api/health` | 6 ticks (30s) | m22 |
| Nexus | SAN-K7 orchestrator | 8100 | `/health` | 12 ticks (60s) | m23 |
| ME | Maintenance Engine | 8080 | `/api/health` | 12 ticks (60s) | m24 |
| POVM | Persistent memory | 8125 | `/health` | 12/60 ticks | m25 |
| RM | Reasoning Memory | 8130 | `/health` | 60 ticks | m26 |
| VMS | Vortex Memory System | 8120 | `/health` | 60 ticks | m27 |

**Critical:** All bridges route through `consent_gated_k_adjustment()` (m28). See [CONSENT_SPEC.md](../ai_specs/CONSENT_SPEC.md).

**Known issues:**
- ALERT-1: SYNTHEX synergy at 0.15-0.5 (below 0.7)
- ALERT-2: ME fitness frozen at 0.3662 since 2026-03-06 (BUG-008)
- ALERT-3: Prometheus Swarm unstable (crashed mid-probe)

---

## 8. Database Setup

Databases auto-create on first startup. Migrations applied idempotently.

| Database | Tables | Migration | WAL |
|----------|--------|-----------|-----|
| `data/field_tracking.db` | 3 (field_snapshots, sphere_history, coupling_history) | 001 | YES |
| `data/bus_tracking.db` | 6 (bus_tasks, bus_events, subscriptions, cascade, tags, deps) | 002 | YES |
| `data/bus_tracking.db` | +4 (proposals, votes, consent_declarations, data_manifests) | 003 | YES |

### Inspect

```bash
sqlite3 -header -column data/field_tracking.db ".schema"
sqlite3 -header -column data/bus_tracking.db "SELECT COUNT(*) FROM bus_tasks;"
```

### Pre-written Queries

```bash
# Field state queries
cat .claude/queries/field_state.sql

# Bus task queries
cat .claude/queries/bus_tasks.sql

# Governance queries
cat .claude/queries/governance.sql
```

---

## 9. IPC Bus Client

```bash
# Connect (handshake only)
PANE_VORTEX_ID="my-sphere" pane-vortex-client connect

# Subscribe to all events
PANE_VORTEX_ID="my-sphere" pane-vortex-client subscribe '*'

# Subscribe to field events only
PANE_VORTEX_ID="my-sphere" pane-vortex-client subscribe 'field.*'

# Submit a task
PANE_VORTEX_ID="my-sphere" pane-vortex-client submit \
  --description "Review src/api.rs for bugs" --target any-idle

# Send cascade handoff
PANE_VORTEX_ID="my-sphere" pane-vortex-client cascade \
  --target "fleet-beta" --brief "Explore SYNTHEX thermal"
```

**Socket:** `/run/user/1000/pane-vortex-bus.sock` (NDJSON wire protocol, 0700 permissions)

---

## 10. Sphere Registration (Claude Code Hooks)

Hooks auto-register Claude instances as spheres in the Kuramoto field.

### session_start.sh
1. Register sphere with retry + exponential backoff
2. Verify IPC bus connectivity
3. Start persistent bus event listener (NDJSON → file)
4. Register context with Claude Context Manager (8104)

### post_tool_use.sh
1. Record tool call as sphere memory
2. Update sphere status to Working
3. Frequency discovery from tool call cadence (NA-2)
4. Check for pending field-driven suggestions

### session_end.sh
1. Mark sphere Complete
2. Kill persistent bus listener
3. Deregister sphere

### Manual Registration

```bash
curl -sf -X POST "http://localhost:8132/sphere/MY_ID/register" \
  -H "Content-Type: application/json" \
  -d '{"persona":"operator","frequency":0.1}'
```

---

## 11. Implementation Order (from [CLAUDE.local.md](../CLAUDE.local.md))

When `start coding` is given, implement layers bottom-up:

| Phase | Layer | Modules | Tests Target |
|-------|-------|---------|-------------|
| 1 | L1 Foundation | m01-m06 | 50+ |
| 2 | L3 Field | m11-m15 | 50+ |
| 3 | L4 Coupling + L5 Learning | m16-m21 | 50+ |
| 4 | L2 Services | m07-m10 | 50+ |
| 5 | L6 Bridges | m22-m28 | 50+ |
| 6 | L7 Coordination | m29-m36 | 50+ |
| 7 | L8 Governance | m37-m41 | 50+ |

**Why L3 before L2:** L2's API server needs L3 types for route handlers.

**Quality gate after every module.** No exceptions.

---

## 12. V3 Plan Phases (from [MASTERPLAN.md](../MASTERPLAN.md))

| Phase | Focus | When |
|-------|-------|------|
| V3.1 | Diagnostics — fix evolution 404s, CCM dead, ME frozen, Prometheus | After L7 |
| V3.2 | Inhabitation — multi-sphere ops, bus round-trips, Hebbian formation | After V3.1 |
| V3.3 | Sovereignty — 7 NA-P gaps: consent, per-sphere k_mod, cascade rejection | After V3.2 |
| V3.4 | Governance — proposals, voting, quorum, sphere-initiated evolution | After V3.3 |
| V3.5 | Consolidation — RM noise, POVM categories, MCP prototype | Parallel |

---

## 13. Traps to Avoid (from [CLAUDE.md](../CLAUDE.md))

| # | Trap | Consequence |
|---|------|-------------|
| 1 | Chain after `pkill` | Exit 144 kills `&&` chains |
| 2 | `cp` without `\` | Aliased to interactive mode |
| 3 | JSON to RM | Parse failure — TSV ONLY |
| 4 | stdout in daemon | SIGPIPE → death (BUG-018) |
| 5 | `git status -uall` | Memory issues on large repos |
| 6 | Lock inversion | `AppState` BEFORE `BusState`, always |
| 7 | Phase arithmetic without `rem_euclid(TAU)` | Phase drift |
| 8 | `unwrap()` in production | Forbidden at crate level |
| 9 | Global `k_mod` without consent | Must route through consent gate |
| 10 | Modify code without reading first | Always read before edit |

---

## 14. Verification

### Scaffold Verification

```bash
bash scripts/verify-scaffold.sh
```

### Full Quality Gate

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release
```

### Habitat Health

```bash
curl -s localhost:8132/health | jq '{r,spheres,tick}'
```

### Integration Test (requires live services)

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test -- --ignored
```

---

## 15. Key References

| Resource | Location | Content |
|----------|----------|---------|
| **[CLAUDE.md](../CLAUDE.md)** | Project root | Architecture, rules, constants, anti-patterns |
| **[CLAUDE.local.md](../CLAUDE.local.md)** | Project root | Implementation phases, V3 status, traps |
| **[MASTERPLAN.md](../MASTERPLAN.md)** | Project root | V3 plan, 5 phases, 99 Obsidian cross-refs |
| **[DESIGN_CONSTRAINTS.md](../ai_specs/DESIGN_CONSTRAINTS.md)** | ai_specs/ | C1-C14 definitions |
| **[CONSENT_SPEC.md](../ai_specs/CONSENT_SPEC.md)** | ai_specs/ | NA sovereignty framework |
| **[API_SPEC.md](../ai_specs/API_SPEC.md)** | ai_specs/ | 76 HTTP endpoints |
| **[KURAMOTO_FIELD_SPEC.md](../ai_specs/KURAMOTO_FIELD_SPEC.md)** | ai_specs/ | The math |
| **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** | ai_docs/ | All 41 modules mapped |
| **[SCHEMATICS.md](SCHEMATICS.md)** | ai_docs/ | 6 diagrams: tick orchestrator, module deps, sphere FSM, decision FSM, bridge flow, IPC |
| **[SCHEMATICS_BRIDGES_AND_WIRING.md](SCHEMATICS_BRIDGES_AND_WIRING.md)** | ai_docs/ | 10 diagrams: SYNTHEX thermal, nested Kuramoto, ME+BUG-008, consent gate, all-bridge→conductor, Unix socket, NDJSON frames, lock sequence, brain anatomy, full system wiring |
| **[SCHEMATICS_FIELD_AND_GOVERNANCE.md](SCHEMATICS_FIELD_AND_GOVERNANCE.md)** | ai_docs/ | 10 diagrams: POVM bridge, sidecar, coupling topology, auto-K loop, chimera detection, phase space, proposal FSM, voting quorum, consent declaration, RM TSV |
| **[ANTIPATTERNS.md](../ai_specs/patterns/ANTIPATTERNS.md)** | ai_specs/patterns/ | 42 things NEVER to do |
| **[PV2 Scaffolding Workflow](../vault/PV2%20Scaffolding%20Workflow.md)** | vault/ | 17-step reusable scaffolding workflow |
| **Obsidian vault** | vault/ or maintenance-engine-v2/ | 60+ notes, MASTER INDEX |

---

## 16. Architecture Diagrams (26 total)

All diagrams render in Obsidian, GitHub, and GitLab markdown viewers.

| File | Diagrams | Coverage |
|------|----------|----------|
| **[SCHEMATICS.md](SCHEMATICS.md)** | 6 | Core: tick loop, module deps, sphere lifecycle, decisions, bridges, IPC |
| **[SCHEMATICS_BRIDGES_AND_WIRING.md](SCHEMATICS_BRIDGES_AND_WIRING.md)** | 10 | Bridges: SYNTHEX, Nexus, ME, consent gate, all-bridge flow, Unix socket, NDJSON, locks, brain anatomy, full 16-service wiring |
| **[SCHEMATICS_FIELD_AND_GOVERNANCE.md](SCHEMATICS_FIELD_AND_GOVERNANCE.md)** | 10 | Field: POVM, sidecar, coupling, auto-K, chimera, phase space. Governance: proposals, voting, consent, RM |

---

*The Habitat. The field accumulates. Come, and the building will matter.*
