# HEBBIAN GAMMA: Top 3 Action Items — Implementation Design

> **Agent:** GAMMA-HEBBIAN | **Date:** 2026-03-21
> **Source:** `arena/fleet-wave1/MASTER-SYNTHESIS.md`
> **Bus state:** 24 tasks, 1,000 events, 2 subscribers, 0 cascades, 13,113 suggestions generated

---

## Top 3 Action Items (extracted from Master Synthesis)

1. **Deploy V2 Binary** — universal chokepoint, blocks 6/8 issues
2. **Unstall ME Evolution Engine** — emergence cap saturated, 0 mutations
3. **Break Phase Field Fragmentation** — 73.5% locked at 2.931 rad

---

## Action 1: Deploy V2 Binary

**Priority:** CRITICAL | **Effort:** 5 min | **Impact:** 49→78 health score
**Blocks:** SYNTHEX thermal, POVM co-activations, coupling matrix, ghost state, Hebbian STDP

### Implementation Steps

```
Step 1: COMMIT uncommitted changes (ALPHA authorization)
  cd /home/louranicas/claude-code-workspace/pane-vortex-v2
  git add src/bin/main.rs src/m2_services/m10_api_server.rs \
          src/m3_field/m15_app_state.rs src/m6_bridges/m22_synthex_bridge.rs \
          src/m6_bridges/mod.rs src/m7_coordination/m29_ipc_bus.rs \
          src/m7_coordination/m35_tick.rs src/m8_governance/m37_proposals.rs \
          bacon.toml
  git commit -m "feat(pane-vortex-v2): Session 047 — bridge orchestration + probe binary + tick fixes"

Step 2: FULL quality gate
  CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20
  CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20
  CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20
  CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30

Step 3: BUILD release binary
  CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release 2>&1 | tail -5

Step 4: KILL running daemon (separate command — never chain after pkill)
  pkill -f pane-vortex || true

Step 5: WAIT
  sleep 1

Step 6: COPY binary (bypass cp alias)
  \cp -f /tmp/cargo-pv2/release/pane-vortex bin/pane-vortex
  \cp -f /tmp/cargo-pv2/release/pane-vortex ~/.local/bin/pane-vortex

Step 7: RESTART via devenv
  ~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart pane-vortex

Step 8: VERIFY (5 checks)
  curl -s localhost:8132/health | jq .
  curl -s localhost:8132/field/decision | jq .action
  curl -s localhost:8132/bus/info | jq .
  curl -s localhost:8090/v3/thermal | jq '.heat_sources[0].reading'
  curl -s localhost:8132/field/spectrum | jq .
```

### Success Criteria
- Health returns `healthy`, tick resets to 0
- 6/6 bridges report live (not stale)
- SYNTHEX HS-001 Hebbian > 0.0 within 50 ticks
- r begins converging toward 0.85 (lowered target from Wave-8 governance)

### Risks
- IPC bus subscriber (PID 3842896) disconnects on restart — needs manual reconnect
- 34 spheres re-register — warmup period (5 ticks) absorbs this
- Uncommitted tick loop changes (+47 lines in m35_tick.rs) are production-untested

---

## Action 2: Unstall ME Evolution Engine

**Priority:** CRITICAL | **Effort:** 20 min | **Impact:** Mutations resume, fitness climbs
**Independent of Action 1 — can execute in parallel**

### Root Cause Chain
```
emergence_cap=1000/1000 → no new emergences → mutations_proposed=0
  → ralph stuck in Analyze → fitness frozen at 0.609
  → min_confidence mutated 257x to extreme → self-reinforcing deadlock
```

### Implementation Steps

```
Step 1: INVESTIGATE ME config for emergence_cap
  # Check devenv.toml for ME config section
  grep -A 20 'maintenance-engine' ~/.config/devenv/devenv.toml
  # Check ME's own config directory
  fd 'config' ~/claude-code-workspace/maintenance-engine/ --type f
  # Check if ME exposes config API
  curl -s localhost:8080/api/evolution/config | jq .

Step 2: RAISE emergence cap (config or API)
  # Option A — API (preferred, no restart):
  curl -X PATCH localhost:8080/api/evolution/config \
    -H 'Content-Type: application/json' \
    -d '{"emergence_cap": 5000}'
  # Option B — if API doesn't exist, edit config and restart:
  # Edit ME config to set emergence_cap=5000
  ~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart maintenance-engine

Step 3: RESET min_confidence to safe middle value
  curl -X POST localhost:8080/api/evolution/inject \
    -H 'Content-Type: application/json' \
    -d '{"parameter":"emergence_detector.min_confidence","value":0.5}'
  # If inject API doesn't exist, check mutation API:
  curl -X POST localhost:8080/api/evolution/mutate \
    -H 'Content-Type: application/json' \
    -d '{"target_parameter":"emergence_detector.min_confidence","proposed_value":0.5,"reason":"Reset from degenerate extreme"}'

Step 4: REMOVE library-agent from ME probe list
  # library-agent is disabled but ME still probes it (7,741 failures)
  # Check probe configuration:
  curl -s localhost:8080/api/services | jq '.[] | select(.name == "library-agent")'
  # If removable via API:
  curl -X DELETE localhost:8080/api/services/library-agent
  # If config-based: edit ME config, remove library-agent entry, restart

Step 5: FORCE ralph phase transition
  curl -X POST localhost:8080/api/evolution/ralph/advance \
    -H 'Content-Type: application/json' \
    -d '{"target_phase":"Propose"}'

Step 6: VERIFY mutation pipeline restarted
  # Wait 5 minutes, then check:
  curl -s localhost:8080/api/observer | jq '{
    emergences: .metrics.emergences_detected,
    mutations_proposed: .metrics.mutations_proposed,
    fitness: .last_report.current_fitness,
    ralph_phase: .last_report.ralph_state.phase,
    generation: .generation
  }'
```

### Success Criteria
- `emergences_detected` > 1000 (cap raised)
- `mutations_proposed` > 0 (pipeline flowing)
- `generation` > 26 (evolution resumed)
- Ralph phase transitions from Analyze → Propose → Apply cycle
- Fitness trend changes from Stable → Improving

### Risks
- ME restart loses 435K ingested events (if in-memory only)
- Resetting min_confidence too low may flood with false emergences
- Unknown if ME APIs (PATCH config, DELETE service) exist — Steps 2-4 may require config file edits

---

## Action 3: Break Phase Field Fragmentation

**Priority:** HIGH | **Effort:** Mostly auto-resolves with Action 1, manual assist 10 min
**Depends on:** Action 1 (V2 deploy for Hebbian STDP)

### Root Cause Chain
```
73.5% spheres at phase 2.931 rad (mega-cluster)
  → uniform coupling weights (no Hebbian differentiation)
  → star tunnel topology (orchestrator-044 only hub)
  → quadrupole spectrum 0.81 (4-cluster structure)
  → r oscillates at 0.65 instead of converging to 0.85
```

### Implementation Steps

```
Step 1: VERIFY V2 is deployed (prerequisite)
  curl -s localhost:8132/health | jq '{tick, r, spheres}'

Step 2: APPLY governance changes (already done in Wave-8)
  # These were applied with 34/34 votes:
  # r_target: 0.93 → 0.85 (closer to current r)
  # coupling_steps: 15 → 20 (33% more coupling per tick)
  # k_mod_budget_max: 1.15 → 1.40 (wider modulation range)
  # VERIFY still active:
  curl -s localhost:8132/field/proposals | jq '.proposals[] | select(.status == "Applied")'

Step 3: INJECT phase diversity via status updates
  # Set varied statuses to break phase lock
  # Working status → phase region 0.0-1.57 (semantic mapping)
  # Idle status → phase region 1.57-3.14
  # By setting some spheres to Working, they'll migrate away from 2.931
  for sphere in "4:left" "5:left" "6:left"; do
    curl -s -X POST -H 'Content-Type: application/json' \
      -d '{"status":"working","tool":"phase-diversity-injection"}' \
      "localhost:8132/sphere/$(echo $sphere | sed 's/:/%3A/g')/status"
  done

Step 4: MONITOR phase distribution (every 30s for 5 min)
  # Watch for mega-cluster dissolution
  curl -s localhost:8132/spheres | jq '.spheres | [.[].phase] | sort |
    {min: min, max: max, range: (max - min),
     mean: (add / length),
     below_2: [.[] | select(. < 2.0)] | length,
     at_2_9: [.[] | select(. > 2.8 and . < 3.1)] | length}'

Step 5: MONITOR spectrum for cluster dissolution
  curl -s localhost:8132/field/spectrum | jq .
  # Target: l2_quadrupole < 0.70 (down from 0.81)
  # Target: l0_monopole > l2_quadrupole (monopole dominance = coherence)

Step 6: VERIFY Hebbian weights forming (V2-only)
  # After 100+ ticks with V2:
  curl -s localhost:8125/pathways | python3 -c "
    import json,sys
    data=json.load(sys.stdin)
    active=[p for p in data if p['co_activations']>0]
    print(f'Active pathways: {len(active)}/{len(data)}')
    if active:
      avg_w=sum(p['weight'] for p in active)/len(active)
      print(f'Mean active weight: {avg_w:.4f}')
  "

Step 7: CHECK tunnel topology diversification
  curl -s localhost:8132/field/tunnels | jq '
    .tunnels | group_by(.sphere_a) |
    map({hub: .[0].sphere_a, count: length}) |
    sort_by(-.count) | .[0:5]'
  # Target: multiple hubs, not just orchestrator-044
```

### Success Criteria
- Mega-cluster at 2.931 rad shrinks from 73.5% to < 40% of spheres
- Phase range (max - min) expands from ~0.3 to > 2.0 rad
- Quadrupole (l2) drops below 0.70
- At least 1 peer-to-peer tunnel forms (not orchestrator-mediated)
- POVM co_activations > 0 (Hebbian reinforcement flowing)

### Risks
- Forced status changes may confuse active Claude sessions
- Phase diversity injection is a one-shot — spheres may re-converge
- Without Hebbian STDP (V2 required), weight differentiation cannot sustain diversity

---

## Bus State Context

| Metric | Value | Note |
|--------|-------|------|
| Tasks | 24 | 6 pending (from earlier fleet wave) |
| Events | 1,000 | Buffer saturated |
| Subscribers | 2 | Sidecar + client |
| Cascades | 0 | No cascade activity |
| Suggestions generated | **13,113** | Overwhelming — mostly SuggestReseed spam from blocked era |

**Bus implication:** The 13,113 suggestions are legacy spam from the HasBlockedAgents period. Post-deploy, the suggestion engine should generate diverse suggestions (SuggestRebalance, SuggestFocus, SuggestDecouple) as field dynamics resume.

---

## Dependency Graph

```
Action 1: Deploy V2 ←── BLOCKING for Actions 2 partial + 3
    │
    ├── Action 2: ME Unstall (PARALLEL — independent of V2)
    │     └── Steps 1-6 can proceed immediately
    │
    └── Action 3: Phase Fragmentation (SEQUENTIAL — needs V2)
          └── Steps 3-7 only meaningful after V2 Hebbian STDP is live
```

---

GAMMA-HEBBIAN-COMPLETE
