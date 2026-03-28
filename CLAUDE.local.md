# Pane-Vortex V2 — The Habitat (Private Project Instructions)

## BOOTSTRAP PROTOCOL (New Context Window)

**MANDATORY — execute these 3 steps at the start of EVERY new context window:**

1. **Run `/primehabitat`** — loads The Habitat: Zellij tabs, 16 services, IPC bus, memory systems, tool chains, NEVER list
2. **Run `/deephabitat`** — loads deep substrate: yazi, btm, bacon, MCP, pipe protocol, autocmds, cross-DB, vault nav, 55 custom binaries, all bridge details
3. **Read this file** — `CLAUDE.local.md` — implementation status, phase tracking, traps to avoid

**After bootstrap, WAIT for user to type `deploy plan` before taking ANY action.**

Bootstrap gives you god-tier understanding. But deployment and code changes require explicit authorization via `deploy plan`.

---

## Session 047 — Fleet Orchestration Complete (2026-03-21)

**Status:** 85 arena files (788KB, 13,877 lines) from 9 Claude instances across 10+ waves.
**Tests:** 1,527 (unchanged from Session 046b)
**Health:** 49/100 current → 78/100 projected with V2 deploy

### Bugs Discovered (Session 047)
- **BUG-034** POVM write-only pathology (access_count=0 all memories)
- **BUG-035** ME emergence cap 1000/1000 deadlocked (CRITICAL)
- **BUG-036** library-agent ghost probing (7,838 failures)
- **BUG-037** SYNTHEX thermal feedback decoupled on V1

### Breakthroughs
- SYNTHEX `POST /api/ingest` is WRITABLE (thermal injection vector)
- Cross-service synergy matrix mapped (PV-SYNTHEX 0.82 strongest)
- 10 hook points identified (40-50% automation)
- 23 code integration points across 7 files
- 5 synergies: POVM crystallisation, RM-ME corridor, harmonic damping, governance auto-voting, bus diversity

### Session 048 Remediation Plan
**Plan file:** `.claude/plans/zesty-shimmying-fountain.md`
**Also at:** `ai_docs/SESSION_048_REMEDIATION_PLAN.md` and `vault/Session 048 — Remediation Plan.md`

**Execution: `deploy plan` triggers Blocks A→I**
1. Block 0: Bootstrap (/primehabitat + /deephabitat + habitat-probe pulse)
2. Block A: Commit 847 lines + backup V1 + locate ME config + probe POVM schema
3. Block B: Build V2 + hot-swap + restart (closes thermal loop)
4. Block C: ME emergence_cap 1000→5000 + remove library-agent (parallel with B)
5. Block D: Unblock 7 spheres + verify 7 MVP routes + SYNTHEX injection test
6. Block E: Wire Executor + spawn_bridge_posts + V3.2 inhabitation smoke test
7. Block F: POVM hydration read-back + Hebbian co-activation + session tagging
8. Block G: 5 hooks (UserPromptSubmit, SessionStart, PostToolUse, PreToolUse, Stop)
9. Block H: 4 synergies (auto-voting, crystallisation, harmonic damping, voting window)
10. Block I: Phase 2-3 integration (16 routes) + dashboards + SubagentStop hook

**Rollback:** `\cp -f ./bin/pane-vortex.v1.bak ./bin/pane-vortex && devenv restart pane-vortex`

---

## Session 040 — Scaffold Phase (2026-03-19)

**Status:** SCAFFOLDED — 198 files, 8 layers, 41 modules, 17,120 lines docs, 26 diagrams, quality gate 4/4 CLEAN
**Plan:** `MASTERPLAN.md` in this directory (499 lines, 99 Obsidian cross-refs)
**Obsidian:** `[[The Habitat — Integrated Master Plan V3]]`
**Workflow:** `vault/PV2 Scaffolding Workflow.md` — 17-step reusable process
**Schematics:** 3 files in ai_docs/ (26 Mermaid diagrams covering all bridges, IPC, field, governance)

## V3 Plan Status

| Phase | Focus | Status |
|-------|-------|--------|
| V3.1 | Diagnostics & Repair | DONE (Session 045) |
| V3.2 | Inhabitation | READY (ghost reincarnation coded) |
| V3.3 | Sovereignty | CODED (GAP-3/4 consent fixes, needs V2 deploy) |
| V3.4 | Governance | CODED (GAP-1/2/5/6/7, 5 API routes, needs V2 deploy) |
| V3.5 | Consolidation | PARALLEL with V3.2+ |

## Session 045 — Remediation COMPLETE, Deploy V2 Binary (2026-03-21)

**Status:** ALL CODE COMPLETE — V2 binary NOT yet deployed to live daemon
**Commits:** `73314ad` (remediation) → `ea06b35` (BUG-029) → `6fa51d9` (BUG-031) → `a722a6b` (BUG-028)
**Tests:** 1,516 (was 1,379, +137)
**Obsidian:** `[[Session 045 — Remediation Plan Deployment]]`, `[[Session 045 — Sidecar and Fleet Failure Analysis]]`
**POVM:** `a33eca4a` (deployment), `376dd9e7` (sidecar), `91ea60f4` (failure analysis)
**RM:** `r69bdb76b01f8` (deployment), `r69bdc5b40201` (synthesis), `r69bdda5f023a` (failure analysis)

### What Was Coded (Session 045)
- **7 GAPs closed:** GAP-1 governance actuator, GAP-2 runtime budget, GAP-3 6-bridge consent, GAP-4 divergence exemption, GAP-5 sphere override, GAP-6 opt-out policy, GAP-7 voting window 5→24
- **5 bugs fixed:** BUG-027 stuck cp, BUG-028 V1 sidecar wire compat, BUG-029 client arg parse, BUG-030 unblocked, BUG-031 Hebbian STDP wired to tick Phase 2.5
- **Architecture:** IQR K-scaling, ghost reincarnation, multiplicative conductor/bridge composition, EMA decay α=0.95, ProposalManager persistence
- **5 governance API routes:** /field/propose, /field/proposals, /sphere/{id}/vote/{proposal_id}, /sphere/{id}/consent, /sphere/{id}/data-manifest

### What Needs Deploying (NEXT SESSION — via `deploy plan`)
The live daemon runs V1 binary (`./bin/pane-vortex`, PID from Mar 20). All V2 code exists only in source.

## The `deploy plan` Command

When user types **`deploy plan`**, execute in this order:

**Step 0 — Bootstrap:**
1. Run `/primehabitat` — loads full Habitat knowledge
2. Run `/deephabitat` — loads substrate knowledge
3. Run `habitat-probe pulse` — verify PV + POVM + ME live
4. Run `fleet-ctl status` — verify fleet pane state

**Step 1 — Verify codebase:**
5. Read this file (`CLAUDE.local.md`) — confirm Session 045 state
6. Read `vault/Session 044 — Deep Synthesis.md` for architectural context
7. Run quality gate — confirm 1,516 tests pass, zero warnings
8. Verify 4 commits exist: `git log --oneline -4`

**Step 2 — Build V2 release:**
9. `CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release 2>&1 | tail -5`

**Step 3 — Deploy V2 binary (ULTRAPLATE convention):**
10. Hot-swap BOTH paths (devenv uses `./bin/`, hooks use `~/.local/bin/`):
    ```bash
    \cp -f /tmp/cargo-pv2/release/pane-vortex ./bin/pane-vortex
    \cp -f /tmp/cargo-pv2/release/pane-vortex ~/.local/bin/pane-vortex
    ```
11. Restart via devenv: `~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart pane-vortex`

**Step 4 — Verify V2 is live (5 checks):**
12. `curl -s localhost:8132/health | jq '{status, tick}'` — healthy
13. `curl -s -o /dev/null -w '%{http_code}' localhost:8132/field/proposals` — **200 (not 404)**
14. `tail -5 /tmp/swarm-sidecar.log` — sustained session (no "bus disconnected")
15. `curl -s localhost:8132/coupling/matrix | jq '[.matrix[].weight] | unique | length'` — should show weight differentiation after Hebbian ticks
16. `habitat-probe sweep` — 16/16 healthy

**Step 5 — Post-deploy:**
17. Record to RM + POVM
18. Save to Obsidian vault
19. Update devenv.toml description: `1516 tests | 38 API routes`
20. Continue with V3.2 Inhabitation or other tasks

## Gold Standard Reference

**ME v2:** `~/claude-code-workspace/the_maintenance_engine_v2/`
- Module naming: `m01_`, `m02_`, etc.
- Layer structure: `src/m1_foundation/`, `src/m2_services/`
- Error handling: Custom error enum with `thiserror`
- Config: Figment with TOML + env overlay
- Tests: In-file `#[cfg(test)]` modules

**PV v1:** `~/claude-code-workspace/pane-vortex/`
- 79 patterns in `ai_specs/patterns/`
- 42 anti-patterns in `ai_specs/patterns/ANTIPATTERNS.md`
- .claude folder: context.json, patterns.json, anti_patterns.json, queries/, schemas/
- Operational hooks: session_start.sh, post_tool_use.sh, session_end.sh

## Traps to Avoid

1. Never chain after pkill (exit 144)
2. Always `\cp -f` (cp aliased — BUG-027 killed 2 stuck processes)
3. TSV only for Reasoning Memory
4. Lock ordering: AppState before BusState
5. Phase wrapping: `.rem_euclid(TAU)` always
6. No stdout in daemons (SIGPIPE death)
7. BUG-008: ME EventBus — subscriber_count=0 is cosmetic, not the real issue
8. fleet-ctl cache is STALE (300s TTL) — screen dump is ONLY reliable pane state source
9. PV sphere status LAGS — hooks don't always fire on session end
10. Running daemon is V1 binary at `./bin/pane-vortex` — V2 at `~/.local/bin/` needs explicit deploy
11. Sidecar (V1 binary) disconnects after handshake until V2 deployed (BUG-028 fix in code)

## Working Directory
`/home/louranicas/claude-code-workspace/pane-vortex-v2`

## Quality Gate
```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```


---

## Habitat Bootstrap Protocol (Fresh Context Window)

**Execute these in order at the start of EVERY new context window:**

| # | Command | What It Loads |
|---|---------|---------------|
| 1 | `/primehabitat` | Zellij tabs, 17 services, IPC bus, memory systems |
| 2 | `/deephabitat` | Wire protocol, 173 DBs, devenv batches, 100+ binaries |
| 3 | Read `CLAUDE.local.md` | Current session state, phase tracking, session history |

**After bootstrap, WAIT for user instruction before taking action.**

### Operational Commands (use as needed after bootstrap)

| Command | When To Use |
|---------|-------------|
| `/gate` | Before every commit — 4-stage quality gate: check → clippy → pedantic → test |
| `/sweep` | Health check — probe all 17 services + ORAC + thermal + field |
| `/deploy-orac` | After ORAC code changes — build → deploy → verify (encodes all traps) |
| `/forge` | After ANY service code changes — generic build → deploy → verify |
| `/genesis` | Create new service from zero — scaffold + register + deploy |
| `/integrate` | Wire service into Habitat — hooks + bridges + PV2 + RM + POVM |
| `/acp` | Complex decisions — Adversarial Convergence Protocol (3 rounds) |
| `/battern` | Multi-pane work — fleet batch dispatch with roles + gates |
| `/nerve` | Live monitoring — continuous Nerve Center dashboard (10s refresh) |
| `/propagate` | After adding commands — push command table to all service CLAUDE.md files |
| `/nvim-mastery` | Neovim RPC: LSP, treesitter, 37 keymaps, 22 snacks features, structural analysis |
| `/atuin-mastery` | Shell history intelligence: search, stats, service density, time-of-day, KV store |

> Commands defined at `orac-sidecar/.claude/commands/` and workspace `.claude/skills/`. Work from ANY service directory.
