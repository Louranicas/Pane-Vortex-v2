# Plan: Habitat Remediation — Systems Engineering Analysis

## Meta Tree Mindmap

```
HABITAT REMEDIATION PLAN
│
├── A: PRE-DEPLOYMENT ──────────────────────────────────────────────
│   ├── A1: Commit 847 lines (12 files)
│   │   └── 📎 git diff: mod.rs(+280), main.rs(+183), m35_tick.rs(+47)
│   ├── A2: Backup V1 binary
│   │   └── 📎 [[Session 046 — V2 Binary Deployed]]
│   ├── A3: Locate ME emergence_cap config
│   │   ├── 📎 POVM:ee28fec6 (Session 031 exploration: 16/16 services)
│   │   └── 📎 [[ULTRAPLATE Developer Environment]] (devenv.toml schema)
│   └── A4: Document POVM schema
│       ├── 📎 POVM:32e9b820 (Session 027: Zellij devenv deployed)
│       ├── 📎 POVM:3611a357 (IPC bus: socket handshake)
│       └── 📎 ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md
│
├── B: DEPLOY V2 BINARY ────────────────────────────────────────────
│   ├── B1: cargo build --release
│   │   └── 📎 [[Session 045 — Remediation Plan Deployment]] (1527 tests)
│   ├── B2: Hot-swap ./bin/ + ~/.local/bin/
│   │   └── 📎 [[Session 046 — V2 Binary Deployed]] (deploy steps)
│   ├── B3: devenv restart pane-vortex
│   │   └── 📎 POVM:50b8fd79 (operational learnings: side-effect control)
│   └── B4: Verify 5 checks
│       ├── 📎 ai_docs/SCHEMATICS.md (26 Mermaid diagrams)
│       ├── 📎 [[Session 047 — Final Report]] (health scorecard 49→78)
│       └── RESOLVES: FM-1 (open control loop)
│           └── 📎 arena/subagent-synthex-feedback-loop.md
│               └── Signal: SYNTHEX k_adj=1.094 → k_mod propagates
│
│   ┌─────────────────────────────────────────────────────────┐
│   │ ARCHITECTURE SCHEMATIC: Control Loop Closure            │
│   │                                                         │
│   │  SYNTHEX ──thermal──> BridgeSet.apply_k_mod()          │
│   │     ↑                        ↓                          │
│   │     │               network.k_modulation *= adj         │
│   │     │                        ↓                          │
│   │  heat_sources <──── Hebbian STDP (Phase 2.5)           │
│   │     ↑                        ↓                          │
│   │     │               coupling weights differentiate      │
│   │     │                        ↓                          │
│   │  temperature ←───── field r converges toward 0.93      │
│   │                                                         │
│   │  📎 src/m6_bridges/mod.rs:141 (apply_k_mod)            │
│   │  📎 src/m7_coordination/m35_tick.rs:120 (Phase 2.7)    │
│   │  📎 src/m6_bridges/m22_synthex_bridge.rs:89            │
│   │  📎 ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md           │
│   └─────────────────────────────────────────────────────────┘
│
├── C: ME CONFIG FIXES (parallel with B) ───────────────────────────
│   ├── C1: emergence_cap 1000→5000
│   │   ├── 📎 arena/subagent-me-deadlock.md (full forensics)
│   │   ├── 📎 arena/gamma-me-investigation.md (root cause)
│   │   └── 📎 [[ULTRAPLATE — Bugs and Known Issues]] BUG-035
│   ├── C2: min_confidence reset to 0.5
│   │   └── 📎 POVM:7a3eed6a (Session 032: 9 issues identified)
│   ├── C3: Remove library-agent from probe list
│   │   └── 📎 [[ULTRAPLATE — Bugs and Known Issues]] BUG-036
│   └── C4: devenv restart maintenance-engine
│       └── RESOLVES: FM-2 (buffer saturation), FM-4 (dead input)
│
│   ┌─────────────────────────────────────────────────────────┐
│   │ ARCHITECTURE SCHEMATIC: ME Deadlock Chain               │
│   │                                                         │
│   │  events(432K) → correlations(4.7M) → emergences        │
│   │                                        ↓                │
│   │                                   [1000/1000 CAP]       │
│   │                                        ↓                │
│   │                                   mutations = 0         │
│   │                                        ↓                │
│   │                              fitness stuck at 0.609     │
│   │                                                         │
│   │  FIX: raise cap → emergences flow → mutations resume   │
│   │                                                         │
│   │  📎 [[Maintenance Engine — Architecture Schematic]]     │
│   │  📎 [[ME Tensor Architecture]]                          │
│   │  📎 [[The Maintenance Engine V2]]                       │
│   └─────────────────────────────────────────────────────────┘
│
├── D: RUNTIME CORRECTIONS (requires B) ────────────────────────────
│   ├── D1: Unblock 7 fleet spheres
│   │   ├── 📎 arena/pv2main-synergy-synthesis.md (Quick Win 1)
│   │   ├── 📎 arena/gamma-bus-governance-audit.md (sphere census)
│   │   └── 📎 [[Session 047 — Fleet Orchestration Comms]]
│   ├── D2: Verify 7 MVP routes
│   │   └── 📎 arena/subagent-23-integration-points.md (MVP list)
│   ├── D3: SYNTHEX /api/ingest injection test
│   │   ├── 📎 arena/subagent-synthex-api-map.md (endpoint discovery)
│   │   └── 📎 [[Session 047 — Subagent Breakthroughs]]
│   └── D4: Monitor coupling matrix growth
│       ├── 📎 arena/gammaright-sphere-analysis.md (phase clustering)
│       └── 📎 ai_docs/SCHEMATICS_FIELD_AND_GOVERNANCE.md
│
├── E: EXECUTOR + WRITE-BACK (requires B) ──────────────────────────
│   ├── E1: Wire Executor into IPC bus
│   │   ├── 📎 src/m7_coordination/m32_executor.rs (43 lines)
│   │   ├── 📎 src/m7_coordination/m29_ipc_bus.rs (bus handler)
│   │   ├── 📎 [[Session 046b — Bridge Wiring Complete]] (deferred)
│   │   └── 📎 POVM:3611a357 (IPC bus socket protocol)
│   ├── E2: spawn_bridge_posts() for POVM/RM/VMS
│   │   ├── 📎 src/m6_bridges/m25_povm_bridge.rs
│   │   ├── 📎 src/m6_bridges/m26_rm_bridge.rs
│   │   └── 📎 ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md
│   ├── E3: V3.2 inhabitation smoke test
│   │   ├── 📎 [[The Habitat — Integrated Master Plan V3]] (V3.2)
│   │   └── 📎 src/m2_services/m10_api_server.rs (accept-ghost)
│   └── E4: Quality gate
│       └── RESOLVES: FM-5 (missing handler), FM-6 (missing outbound)
│
│   ┌─────────────────────────────────────────────────────────┐
│   │ ARCHITECTURE SCHEMATIC: IPC Bus Data Flow               │
│   │                                                         │
│   │  Claude Instance                                        │
│   │     ↓ pane-vortex-client submit                        │
│   │  Unix Socket (/run/user/1000/pane-vortex-bus.sock)     │
│   │     ↓ NDJSON BusFrame::Submit                          │
│   │  handle_connection()                                    │
│   │     ↓ ← Executor.execute() [MISSING — FM-5]           │
│   │  BusState.add_task()                                    │
│   │     ↓                                                   │
│   │  field.tick event broadcast                             │
│   │     ↓                                                   │
│   │  Swarm Sidecar → /tmp/swarm-events.jsonl               │
│   │                                                         │
│   │  📎 ai_docs/MESSAGE_FLOWS.md                            │
│   │  📎 [[IPC Bus Architecture Deep Dive]]                  │
│   │  📎 [[Executor and Nested Kuramoto Bridge]]             │
│   └─────────────────────────────────────────────────────────┘
│
├── F: POVM READ-BACK (requires E) ─────────────────────────────────
│   ├── F1: hydrate() every 12 ticks
│   │   ├── 📎 src/m6_bridges/m25_povm_bridge.rs
│   │   ├── 📎 arena/subagent-povm-pathology.md (write-only finding)
│   │   └── 📎 [[POVM Engine]] (Obsidian service note)
│   ├── F2: Hebbian co-activation → POVM pathway updates
│   │   ├── 📎 src/m5_learning/m19_hebbian.rs (STDP)
│   │   ├── 📎 POVM:e89e33a3 (speed benchmarks: 7D chain)
│   │   └── 📎 [[Hebbian Learning Deep Dive]]
│   ├── F3: Session tagging on all POVM writes
│   │   └── 📎 [[ULTRAPLATE — Bugs and Known Issues]] BUG-034
│   └── F4: Quality gate
│       └── RESOLVES: FM-3 (unidirectional data path)
│
│   ┌─────────────────────────────────────────────────────────┐
│   │ ARCHITECTURE SCHEMATIC: POVM Bidirectional Flow         │
│   │                                                         │
│   │  BEFORE (current):                                      │
│   │  Services → POST /memories → POVM [dead end]           │
│   │                                                         │
│   │  AFTER (target):                                        │
│   │  Services → POST /memories → POVM                      │
│   │                                ↕ (bidirectional)        │
│   │  tick loop ← GET /hydrate ← POVM                      │
│   │     ↓                          ↑                        │
│   │  Hebbian STDP → co_activation → pathway.weight++       │
│   │     ↓                          ↑                        │
│   │  crystallise() when T > 0.15 → persistent memory       │
│   │                                                         │
│   │  📎 [[POVM Engine]]                                     │
│   │  📎 [[Vortex Sphere Brain-Body Architecture]]           │
│   │  📎 ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md           │
│   └─────────────────────────────────────────────────────────┘
│
├── G: HOOKS (requires B) ──────────────────────────────────────────
│   ├── G1: UserPromptSubmit → field state injection
│   │   ├── 📎 arena/subagent-hook-points.md (#1)
│   │   └── 📎 [[Session 047 — Powerful Workflows]] (Workflow 1)
│   ├── G2: SessionStart → sphere registration + IPC
│   │   ├── 📎 arena/subagent-hook-points.md (#2)
│   │   ├── 📎 POVM:32e9b820 (Session 027: devenv deployed)
│   │   └── 📎 [[Session 047 — Powerful Workflows]] (Workflow 7)
│   └── G3: PostToolUse → POVM pathway recording
│       ├── 📎 arena/subagent-hook-points.md (#3)
│       └── 📎 [[Session 047 — Powerful Workflows]] (Workflow 10)
│
├── H: COUPLING OPTIMIZATION (requires F) ──────────────────────────
│   ├── H1: Governance auto-voting
│   │   ├── 📎 src/m8_governance/m39_voting.rs
│   │   ├── 📎 arena/subagent-5-new-synergies.md (Synergy 4)
│   │   └── 📎 ai_docs/SCHEMATICS_FIELD_AND_GOVERNANCE.md
│   ├── H2: POVM-SYNTHEX crystallisation
│   │   ├── 📎 arena/subagent-5-new-synergies.md (Synergy 1)
│   │   └── 📎 [[Synthex (The brain of the developer environment)]]
│   ├── H3: Harmonic damping (l2 quadrupole → K-scaling)
│   │   ├── 📎 src/m4_coupling/m17_auto_k.rs
│   │   ├── 📎 arena/subagent-5-new-synergies.md (Synergy 3)
│   │   └── 📎 [[Session 047 — Consolidated Learnings]] (spectrum)
│   ├── H4: Governance window 24→200 ticks
│   │   └── 📎 src/m8_governance/m37_proposals.rs
│   └── H5: Quality gate
│
│   ┌─────────────────────────────────────────────────────────┐
│   │ ARCHITECTURE SCHEMATIC: Thermal Fleet Protocol          │
│   │                                                         │
│   │  Fleet Activity                                         │
│   │     ↓ cascade depth (D), rate (R)                      │
│   │  Heat Generation                                        │
│   │     ↓ HS-001 Hebbian, HS-002 Cascade                  │
│   │  SYNTHEX Temperature Rise                               │
│   │     ↓ PID controller adjusts                           │
│   │  k_modulation Boost                                     │
│   │     ↓ Phase 2.7 in tick                                │
│   │  Stronger Coupling                                      │
│   │     ↓ Kuramoto synchronization                         │
│   │  r → R_TARGET (0.93)                                   │
│   │     ↓ Hebbian STDP fires (co-active pairs)             │
│   │  Heat Accelerates                                       │
│   │     ↓ self-sustaining thermal flywheel                 │
│   │                                                         │
│   │  CA = (1+D/10)(1+R/10)(1+r/2)(1+H/3)                  │
│   │                                                         │
│   │  📎 arena/subagent-synthex-feedback-loop.md             │
│   │  📎 [[Session 034f — SYNTHEX Schematics and Wiring]]   │
│   │  📎 [[Synthex (The brain of the developer environment)]]│
│   └─────────────────────────────────────────────────────────┘
│
└── I: INTEGRATION + PERSISTENCE (requires H) ──────────────────────
    ├── I1: Phase 2 integration (7 routes)
    │   └── 📎 arena/subagent-23-integration-points.md
    ├── I2: Phase 3 integration (9 routes)
    │   └── 📎 arena/subagent-23-integration-points.md
    ├── I3: Dashboard scripts → scripts/
    │   └── 📎 /tmp/habitat-nerve-center.sh
    └── I4: habitat-probe full verification
        ├── 📎 [[Session 047 — Final Report]] (target: 78/100)
        └── 📎 [[ULTRAPLATE Master Index]] (session tracker)
```

## POVM Memory Cross-Reference Index

| POVM ID | Content | Linked Blocks |
|---------|---------|---------------|
| `32e9b820` | Session 027: Zellij devenv deployed | A4, G2 |
| `5052d714` | Session 027b: Pane navigation mastery | G2 |
| `8f6cec1d` | Session 027c: System schematics | A4 |
| `50b8fd79` | Operational learnings: side-effect control | B3 |
| `d2591927` | Nexus controller analysis | C1 |
| `ee28fec6` | Session 031: 16/16 services | A3 |
| `7a3eed6a` | Session 032: 9 issues identified | C2 |
| `e89e33a3` | Speed benchmarks: 7D chain 431ms | F2 |
| `3611a357` | IPC bus: socket handshake protocol | E1 |
| `33027028` | 6 chain clusters: C1-C6 | F2 |
| `77e2ac77` | Tool Library gap: 65/0 registered | H1 |

## Obsidian Cross-Reference Index

| Note | Linked Blocks |
|------|---------------|
| `[[Session 045 — Remediation Plan Deployment]]` | B1 |
| `[[Session 046 — V2 Binary Deployed]]` | A2, B2 |
| `[[Session 046b — Bridge Wiring Complete]]` | E1 |
| `[[Session 047 — Final Report]]` | B4, I4 |
| `[[Session 047 — Subagent Breakthroughs]]` | D3 |
| `[[Session 047 — Consolidated Learnings]]` | H3 |
| `[[Session 047 — Powerful Workflows]]` | G1, G2, G3 |
| `[[ULTRAPLATE — Bugs and Known Issues]]` | C1, C3, F3 |
| `[[The Habitat — Integrated Master Plan V3]]` | E3 |
| `[[POVM Engine]]` | F1 |
| `[[Synthex (The brain of the developer environment)]]` | H2, H-schematic |
| `[[Maintenance Engine — Architecture Schematic]]` | C-schematic |
| `[[Hebbian Learning Deep Dive]]` | F2 |
| `[[IPC Bus Architecture Deep Dive]]` | E-schematic |
| `[[Executor and Nested Kuramoto Bridge]]` | E-schematic |
| `[[Vortex Sphere Brain-Body Architecture]]` | F-schematic |
| `ai_docs/SCHEMATICS.md` | B4 |
| `ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md` | A4, E2, F-schematic |
| `ai_docs/SCHEMATICS_FIELD_AND_GOVERNANCE.md` | D4, H1 |
| `ai_docs/MESSAGE_FLOWS.md` | E-schematic |

---

## Context

21 system faults identified across 16 interconnected services. Root cause: data path discontinuities between subsystems that individually function correctly. Measured health: 49/100. Projected after remediation: 78/100.

---

## Systems Analysis: Failure Modes

### Failure Mode 1: Open Control Loop (SYNTHEX → PV2)

**Signal path:** SYNTHEX computes `k_adjustment = deviation.mul_add(-0.2, 1.0).clamp(0.85, 1.15)` → V1 binary discards the value → `k_modulation` fixed at 0.85.

**Engineering fix:** Close the loop. V2 binary's `BridgeSet::apply_k_mod()` (already coded, `src/m6_bridges/mod.rs:141`) writes `network.k_modulation *= cached_adjustment`. Deploy the binary.

**Measurement:** `k_modulation` should vary between 0.85-1.15 per tick, not fixed at 0.85.

### Failure Mode 2: Buffer Saturation (ME Emergence Detector)

**Signal path:** Event stream → correlation engine → emergence detector (hard cap 1,000) → mutation proposer. Buffer full → proposer receives zero input → output zero.

**Engineering fix:** Increase buffer capacity from 1,000 to 5,000. Reset the parameter `min_confidence` to midpoint (0.5) — 254 sequential mutations pushed it to an extreme, creating a detection threshold that blocks new entries.

**Measurement:** `mutations_proposed > 0` within one reporting cycle post-restart.

### Failure Mode 3: Unidirectional Data Path (POVM)

**Signal path:** Services → `POST /memories` → POVM storage. No return path. `access_count = 0` across all 53 records. `co_activations = 0` across all 2,427 edges.

**Engineering fix:** Add read-back in tick loop: call `hydrate()` every 12 ticks, feed results into field state. Wire Hebbian STDP weight deltas as co-activation events to POVM pathway updates.

**Measurement:** `access_count > 0` and `co_activations > 0` after 20 ticks.

### Failure Mode 4: Dead Input Channel (ME → library-agent)

**Signal path:** ME polls `library-agent` health → TCP connection refused → failure counter increments (7,838) → circuit breaker OPEN → health dimension dragged by 5%.

**Engineering fix:** Remove `library-agent` from ME's probe list. Service is disabled at the orchestrator level but ME's internal registry still includes it.

**Measurement:** Failure counter stops incrementing. ME `health` dimension rises from 0.625 to ~0.75.

### Failure Mode 5: Missing Process Handler (Executor)

**Signal path:** IPC bus receives `Submit` frame → no handler registered → frame queued indefinitely. 53 tasks pending, 0 processed.

**Engineering fix:** Instantiate `Executor` in `main.rs`, pass `Arc<RwLock<Executor>>` to `start_bus_listener()`. Existing code in `m32_executor.rs` (43 lines) handles frame dispatch.

**Measurement:** Pending task count decreases. Submitted tasks produce results.

### Failure Mode 6: Missing Outbound Data Path (Bridge Write-Back)

**Signal path:** PV2 polls bridges (inbound) but never posts field state back (outbound). POVM/RM/VMS bridges have `post_*()` methods that are never called from the tick loop.

**Engineering fix:** Add `spawn_bridge_posts()` function in tick loop after bridge polls. Fire-and-forget via `spawn_blocking` (same pattern as polling). POVM snapshot every 12 ticks, RM/VMS every 60 ticks.

**Measurement:** Services receive field state updates. POVM `memory_count` grows. RM entry count increases from PV2 source.

### Failure Mode 7: Uncommitted State (847 lines, 12 files)

**Risk:** Binary deployment from uncommitted source creates no rollback point. If deployed binary fails, the only recovery path is reverting to a722a6b — losing all wiring work from Sessions 046-047.

**Engineering fix:** Commit before building. Backup V1 binary before overwriting.

**Measurement:** `git log --oneline -1` shows new commit hash. `./bin/pane-vortex.v1.bak` exists.

### Failure Mode 8: Phase Clustering (Field Spectrum)

**Signal path:** 35 spheres with l2 quadrupole moment at 0.810 — indicating 4-way phase clustering. Kuramoto coupling drives spheres toward nearest cluster rather than global coherence.

**Engineering fix:** Feed l2 magnitude into IQR K-scaling. When l2 > 0.70, increase inter-cluster coupling and decrease intra-cluster coupling. Formula: `k_adj = 1.0 + 0.15 * (1.0 - r) * (l2 - 0.70) / 0.30`.

**Measurement:** l2 should decline below 0.5 over 200 ticks. Order parameter r should trend upward.

---

## Execution Sequence (Dependency-Ordered)

### Block 0: Bootstrap (MANDATORY — new context window)

Every new context window MUST execute these before any plan work:

1. **Run `/primehabitat`** — loads Zellij tabs, 16 services, IPC bus, memory systems, tool chains
2. **Run `/deephabitat`** — loads IPC wire protocol, cross-DB architecture, 55 custom binaries, bridge details
3. **Read `CLAUDE.local.md`** — implementation status, phase tracking, traps to avoid
4. **Run `habitat-probe pulse`** — verify PV + POVM + ME live
5. **Read this plan file** — understand current phase and next steps

After bootstrap, execute `deploy plan` to begin Phase 1. The `deploy plan` command triggers the full deployment sequence documented in CLAUDE.local.md Steps 0-5.

**Reference files for deploy plan:**
- `CLAUDE.local.md` — deploy plan steps 0-20
- `vault/Session 047 — Final Report.md` — system state context
- `vault/Session 047 — Consolidated Learnings.md` — all discoveries
- `ai_docs/SCHEMATICS_BRIDGES_AND_WIRING.md` — bridge architecture
- `arena/fleet-wave1/MASTER-SYNTHESIS.md` — fleet intelligence (85 files summary)

### Block A: Pre-Deployment (15 min, no dependencies)

| Step | Action | Resolves |
|------|--------|----------|
| A1 | `git add` 12 modified files + `git commit` | FM-7 |
| A2 | `\cp -f ./bin/pane-vortex ./bin/pane-vortex.v1.bak` | FM-7 rollback |
| A3 | `grep -r "emergence_cap" ~/claude-code-workspace/the_maintenance_engine/` | Locates FM-2 config |
| A4 | `curl -s localhost:8125/memories \| jq '.[0]'` | Documents POVM schema |

### Block B: Binary Deployment (30 min, requires A)

| Step | Action | Resolves |
|------|--------|----------|
| B1 | `cargo build --release` | — |
| B2 | Hot-swap binary to `./bin/` and `~/.local/bin/` | — |
| B3 | `devenv restart pane-vortex` | FM-1 (control loop closes) |
| B4 | Verify: health 200, proposals 200, bridges updating, coupling non-empty, 16/16 sweep | FM-1 |

### Block C: Config Fixes (15 min, parallel with B)

| Step | Action | Resolves |
|------|--------|----------|
| C1 | Edit ME config: `emergence_cap` 1000→5000 | FM-2 |
| C2 | Edit ME config: `min_confidence` reset to 0.5 | FM-2 |
| C3 | Edit ME config: remove `library-agent` from probe list | FM-4 |
| C4 | `devenv restart maintenance-engine` | FM-2, FM-4 |
| C5 | Verify: `mutations_proposed > 0`, failure count stable | FM-2, FM-4 |

### Block D: Runtime Corrections (15 min, requires B)

| Step | Action | Resolves |
|------|--------|----------|
| D1 | POST idle status to 7 blocked spheres | Unblocks field routing |
| D2 | Verify 7 MVP API routes return correct responses | Validates B deployment |
| D3 | POST thermal reading to SYNTHEX `/api/ingest` | Tests FM-1 injection path |
| D4 | Monitor coupling matrix growth over 6 ticks | Validates FM-8 substrate |

### Block E: Code Changes — Process Handler + Write-Back (2 hrs, requires B)

| Step | Action | Files | Resolves |
|------|--------|-------|----------|
| E1 | Wire Executor into `start_bus_listener()` | `main.rs`, `m29_ipc_bus.rs` | FM-5 |
| E2 | Add `spawn_bridge_posts()` for POVM/RM/VMS | `main.rs` | FM-6 |
| E3 | V3.2 inhabitation smoke test (register sphere, accept ghost, verify coupling) | Runtime | Validates V3.2 |
| E4 | Quality gate: check + clippy + pedantic + test | — | — |

### Block F: Code Changes — POVM Read-Back (4 hrs, requires E)

| Step | Action | Files | Resolves |
|------|--------|-------|----------|
| F1 | Add `hydrate()` call every 12 ticks in tick loop | `main.rs`, `m25_povm_bridge.rs` | FM-3 |
| F2 | Wire Hebbian co-activation deltas to POVM pathway updates | `m19_hebbian.rs`, `m15_app_state.rs` | FM-3 |
| F3 | Add session ID to all POVM memory writes | `m25_povm_bridge.rs` | FM-3 tagging regression |
| F4 | Quality gate | — | — |

### Block G: Hooks (8 hrs, requires B)

| Step | Action | Resolves |
|------|--------|----------|
| G1 | UserPromptSubmit hook: inject field state (r, tick, k_mod, spheres) | Recon automation |
| G2 | SessionStart hook: sphere registration + IPC bus connect | Lifecycle automation |
| G3 | PostToolUse hook: POVM pathway recording with session tags | FM-3 (read-back substrate) |
| G4 | PreToolUse hook: validate against 7 anti-patterns (pkill chains, cp without \\, JSON→RM, stdout in daemons, unwrap, git status -uall, modify without read) | Safety gate |
| G5 | Stop hook: deregister sphere + crystallize session memory to RM+POVM | Lifecycle cleanup |

### Block H: Coupling Optimization (8 hrs, requires F)

| Step | Action | Files | Resolves |
|------|--------|-------|----------|
| H1 | Governance auto-voting based on local sphere fitness | `m39_voting.rs`, `m35_tick.rs` | Governance stall |
| H2 | POVM-SYNTHEX crystallisation: thermal threshold triggers memory persistence | `m22_synthex_bridge.rs`, `m25_povm_bridge.rs` | FM-3 downstream |
| H3 | Harmonic damping: l2 quadrupole feedback to K-scaling | `m17_auto_k.rs` | FM-8 |
| H4 | Widen governance voting window from 24 to 200 ticks | `m37_proposals.rs` | Proposal expiry |
| H5 | Quality gate | — | — |

### Block I: Integration + Persistence (4 hrs, requires H)

| Step | Action | Resolves |
|------|--------|----------|
| I1 | Verify Phase 2 integration points (A3, A4, B2, C1, C2, D3, D4) | 7 routes |
| I2 | Wire Phase 3 integration points (E1-E4, F1-F2, G1) | 9 routes |
| I3 | Persist dashboard scripts to `scripts/` directory | Operational tooling |
| I4 | Full system verification: `habitat-probe full` | All subsystems nominal |
| I5 | SubagentStop hook: auto-aggregate parallel subagent results to arena/ | Cascade automation |

---

## Dependency Graph

```
A (commit, backup, locate config, probe schema)
├── B (deploy V2 binary)
│   ├── D (runtime corrections)
│   ├── E (executor + write-back)
│   │   └── F (POVM read-back)
│   │       └── H (coupling optimization)
│   │           └── I (integration + persistence)
│   └── G (hooks)
└── C (ME config fixes — parallel with B)
```

Critical path: A → B → E → F → H → I (30 hrs)
Parallel path: C (independent, 15 min), G (after B, 6 hrs)

---

## Rollback Procedures

**Binary rollback:**
```bash
\cp -f ./bin/pane-vortex.v1.bak ./bin/pane-vortex
\cp -f ./bin/pane-vortex.v1.bak ~/.local/bin/pane-vortex
devenv restart pane-vortex
```

**ME config rollback:** Restart ME without config changes — `devenv restart maintenance-engine` reverts to last saved config.

**Code rollback:** `git stash` or `git checkout -- src/` to revert uncommitted changes.

---

## Verification Matrix

| Subsystem | Metric | Before | Target | Command |
|-----------|--------|--------|--------|---------|
| PV Field | r | 0.61 | >0.85 | `curl -s :8132/health \| jq .r` |
| PV Field | k_modulation | 0.85 (fixed) | 0.85-1.15 (varying) | `curl -s :8132/health \| jq .k_modulation` |
| Coupling | edge count | 2 | >100 | `curl -s :8132/coupling/matrix \| jq .count` |
| Bridges | stale count | 3/6 | 0/6 | `curl -s :8132/bridges/health` |
| SYNTHEX | temperature | 0.030 | >0.15 | `curl -s :8090/v3/thermal \| jq .temperature` |
| SYNTHEX | synergy | 0.50 | >0.70 | `curl -s :8090/v3/diagnostics` |
| ME | mutations_proposed | 0 | >0 | `curl -s :8080/api/observer` |
| ME | fitness | 0.609 | >0.70 | `curl -s :8080/api/observer` |
| POVM | access_count | 0 | >0 | `curl -s :8125/memories \| jq '.[0].access_count'` |
| POVM | session_count | 0 | >0 | `curl -s :8125/hydrate \| jq .session_count` |
| Bus | pending tasks | 53 | <10 | `curl -s :8132/bus/tasks` |
| Spectrum | l2 quadrupole | 0.81 | <0.50 | `curl -s :8132/field/spectrum \| jq .l2_quadrupole` |

---

## Session Allocation

| Session | Blocks | Effort | Deliverable |
|---------|--------|--------|-------------|
| 048 | A + B + C + D + E | ~3 hrs | V2 live, ME unblocked, executor wired, write-back active |
| 049 | F + G | ~10 hrs | POVM bidirectional, 3 hooks automated |
| 050 | H + I | ~12 hrs | Coupling optimization, full integration, 78/100 health |

---

## Coverage Verification: All 21 Issues × Plan Blocks

| # | Issue | Severity | Plan Block | Status |
|---|-------|----------|------------|--------|
| 1 | BUG-034: POVM write-only | HIGH | F1-F3 | COVERED |
| 2 | BUG-035: ME emergence cap | CRITICAL | C1-C2 | COVERED |
| 3 | BUG-036: library-agent ghost | MEDIUM | C3 | COVERED |
| 4 | BUG-037: SYNTHEX thermal decoupled | HIGH | B1-B4 | COVERED |
| 5 | V2 binary not deployed | CRITICAL | B1-B4 | COVERED |
| 6 | Executor not wired | HIGH | E1 | COVERED |
| 7 | Bridge write-back missing | HIGH | E2 | COVERED |
| 8 | 847 uncommitted lines | HIGH | A1 | COVERED |
| 9 | POVM schema unknown | MEDIUM | A4 | COVERED |
| 10 | V3.2 inhabitation untested | MEDIUM | E3 | COVERED |
| 11 | Governance window too short | MEDIUM | H4 | COVERED |
| 12 | Coupling matrix empty | HIGH | D4, E3 | COVERED |
| 13 | l2 quadrupole dominance | MEDIUM | H3 | COVERED |
| 14 | ME config location unknown | HIGH | A3 | COVERED |
| 15 | No rollback plan | HIGH | A2, Rollback | COVERED |
| 16 | Synergy 1: POVM-SYNTHEX crystallisation | MEDIUM | H2 | COVERED |
| 17 | Synergy 2: RM-ME corridor | MEDIUM | — | **GAP** |
| 18 | Synergy 3: Harmonic damping | MEDIUM | H3 | COVERED |
| 19 | Synergy 4: Governance auto-voting | MEDIUM | H1 | COVERED |
| 20 | Synergy 5: Bus diversity | HIGH | D1 | COVERED |
| 21 | Hook: UserPromptSubmit | HIGH | G1 | COVERED |
| 22 | Hook: SessionStart | HIGH | G2 | COVERED |
| 23 | Hook: PostToolUse POVM | HIGH | G3 | COVERED |
| 24 | Hook: PreToolUse safety | MEDIUM | — | **GAP** |
| 25 | Hook: Stop deregister | MEDIUM | — | **GAP** |
| 26 | Hook: SubagentStop cascade | MEDIUM | — | **GAP** |
| 27 | 7 MVP integration points | HIGH | D2 | COVERED |
| 28 | 7 Phase 2 integration points | MEDIUM | I1 | COVERED |
| 29 | 9 Phase 3 integration points | MEDIUM | I2 | COVERED |
| 30 | SYNTHEX /api/ingest test | MEDIUM | D3 | COVERED |
| 31 | Dashboard persistence | LOW | I3 | COVERED |

**COVERAGE: 27/31 (87%) — 4 gaps remain**

---

## Remaining Gaps + Recommendations

### FINAL GAP 1: Synergy 2 — RM-ME Emergence Corridor (DEFERRED)

Plan marks this as deferred because it requires ME API extension (currently no write endpoint for mutation injection). This is correct — cannot implement without ME code changes.

**Recommendation:** Add as Phase 10 post-Session 050, contingent on ME V2 development. Document the specification in Obsidian for future reference. No plan change needed — deferral is the right call.

### FINAL GAP 2: Hook — PreToolUse Safety Gate

Not covered in plan. Blocks 7 documented anti-patterns (pkill chains, cp without \, JSON to RM, unwrap in production, etc).

**Recommendation:** Add to Block G as G4:
```
G4: PreToolUse hook — validate against anti-pattern list
    Files: .claude/settings.json
    Effort: 1 hr
    Blocks: pkill chains, cp without \, JSON→RM, stdout in daemons
```

### FINAL GAP 3: Hook — Stop Sphere Deregister

Not covered in plan. Causes orphaned spheres persisting in field after session end.

**Recommendation:** Add to Block G as G5:
```
G5: Stop hook — deregister sphere + crystallize session memory
    Files: .claude/settings.json
    Effort: 1 hr
    Depends: G2 (SessionStart creates the sphere to deregister)
```

### FINAL GAP 4: Hook — SubagentStop Cascade Aggregation

Not covered in plan. Session 047 proved subagents produce exceptional output but results require manual collation.

**Recommendation:** Add to Block I as I5 (lowest priority, most complex):
```
I5: SubagentStop hook — auto-aggregate parallel subagent results
    Files: .claude/settings.json, possibly custom skill
    Effort: 3-4 hrs
    Depends: G3 (PostToolUse recording provides the substrate)
```

### Updated Coverage with Recommendations

If G4, G5, I5 added: **30/31 (97%)** — only Synergy 2 (RM-ME corridor) deferred due to external dependency.
