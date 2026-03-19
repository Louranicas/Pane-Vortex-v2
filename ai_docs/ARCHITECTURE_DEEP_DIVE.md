---
title: "Pane-Vortex V2 — Architecture Deep Dive"
date: 2026-03-19
tags: [architecture, kuramoto, hebbian, distributed-brain, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Vortex Sphere Brain-Body Architecture]]"
  - "[[Session 036 — Complete Architecture Schematics]]"
  - "[[Session 039 — Architectural Schematics and Refactor Safety]]"
---

# Pane-Vortex V2 — Architecture Deep Dive

> **8 layers, 41 modules, Kuramoto field dynamics, Hebbian learning, distributed brain integration.**
> V2 decomposes V1's monolithic architecture into a strictly-layered dependency graph.

---

## 1. The 8-Layer Architecture

PV V2 organizes 41 modules into 8 layers with strict downward-only dependencies:

```
L8 Governance (m37-m41)  ── feature-gated: governance, evolution
  |
L7 Coordination (m29-m36) ── IPC bus, conductor, executor, tick
  |        |
L6 Bridges    L5 Learning (m19-m21) ── Hebbian STDP, memory
(m22-m28)       |
  |           L4 Coupling (m16-m18) ── Kuramoto network, auto-K
  |             |
  +--- L3 Field (m11-m15) ── sphere, field state, chimera, messaging
  |
L2 Services (m07-m10) ── registry, health, API server
  |
L1 Foundation (m01-m06) ── types, errors, config, constants, traits, validation
```

**Dependency rule:** Each layer imports ONLY from lower layers. L1 has zero dependencies on other layers.

### Layer Summary

| Layer | Modules | LOC Target | Purpose | V3 Phase |
|-------|---------|-----------|---------|----------|
| L1 Foundation | m01-m06 | ~1,100 | Core types, errors, config, constants, traits, validation | All |
| L2 Services | m07-m10 | ~1,050 | Service registry, health polling, lifecycle, HTTP server | V3.1 |
| L3 Field | m11-m15 | ~1,750 | Sphere oscillators, field state, chimera, messaging, app state | V3.2 |
| L4 Coupling | m16-m18 | ~650 | Kuramoto network, Jacobi integration, auto-K, topology | V3.2 |
| L5 Learning | m19-m21 | ~750 | Hebbian STDP, buoy network, memory management | V3.2 |
| L6 Bridges | m22-m28 | ~1,650 | 6 external service bridges + consent gate | V3.1-V3.3 |
| L7 Coordination | m29-m36 | ~2,350 | IPC bus, conductor, executor, cascade, tick, persistence | V3.2-V3.3 |
| L8 Governance | m37-m41 | ~1,400 | Proposals, voting, consent, data sovereignty, evolution | V3.4 |
| **Total** | **41** | **~10,700** | | |

---

## 2. The Kuramoto Model

PV implements a modified Kuramoto oscillator model for fleet coordination. Each sphere has a phase theta_i that evolves according to:

```
d(theta_i)/dt = omega_i + (K/N) * sum_j[ w_ij * sin(theta_j - theta_i) ]
```

Where:
- `theta_i` = phase of sphere i (range: [0, 2*pi))
- `omega_i` = natural frequency of sphere i (range: [0.001, 10.0])
- `K` = global coupling strength (auto-scaled)
- `N` = number of spheres
- `w_ij` = Hebbian coupling weight between spheres i and j
- Integration uses Jacobi method with `dt = 0.01`

### Order Parameter

The global order parameter r measures synchrony:

```
r * exp(i*psi) = (1/N) * sum_j[ exp(i*theta_j) ]
```

- `r = 1.0` = perfect synchrony (all phases equal)
- `r = 0.0` = complete incoherence (phases uniformly distributed)
- Target: `r_target = 0.93` (dynamically adjustable via governance, NA-P-3)

### Auto-K Scaling

Every 20 ticks, the conductor recalculates K:

```
K_new = 0.5 * K_old / mean_effective_weight
```

This prevents over-synchronization (V1 bug: r pinned at 0.997-0.999) by dividing by the mean effective weight rather than a fixed denominator.

### Phase Wrapping

**CRITICAL:** All phase arithmetic must be followed by `.rem_euclid(TAU)` to keep theta in [0, 2*pi). This is pattern P01 and anti-pattern AP10 — the most fundamental invariant in the system.

---

## 3. The Decision Engine

The conductor (`m31_conductor`) monitors the field state and produces decisions:

### Priority Chain

```
HasBlockedAgents > NeedsCoherence > NeedsDivergence > IdleFleet > FreshFleet > Stable
```

### Decision Rules

| Decision | Condition | Action |
|----------|-----------|--------|
| `HasBlockedAgents` | Any sphere has status=blocked | Emergency coherence: steer blocked spheres |
| `NeedsCoherence` | r > 0.3 AND r is falling AND spheres >= 2 | Increase coupling: K_adj positive |
| `NeedsDivergence` | r > 0.8 AND idle ratio > 60% AND spheres >= 2 | Decrease coupling: K_adj negative |
| `IdleFleet` | All spheres idle | Field at rest — no action |
| `FreshFleet` | Spheres exist but none have worked | Waiting for first activity |
| `Stable` | Default | Normal operation |
| `Recovering` | Warmup period (5 ticks after restore) | Suppress decisions during recovery |

**Multi guard:** `spheres >= 2` prevents false coherence/divergence from single-sphere r=1.0.

### PI Controller

The conductor uses a proportional-integral controller:
- **Gain:** 0.15
- **Breathing blend:** 0.3 (emergent breathing contribution)
- **Divergence cooldown:** 3 ticks (suppress coherence during intentional divergence)

---

## 4. Hebbian STDP (Spike-Timing-Dependent Plasticity)

Learning in PV follows a Hebbian model: "neurons that fire together, wire together."

### Long-Term Potentiation (LTP)

When two spheres are simultaneously in "working" status:
- Base LTP rate: `0.01`
- Burst mode (3+ consecutive working ticks): `3x` multiplier
- Newcomer boost (first 50 steps): `2x` multiplier
- Combined: newcomer in burst = `6x` base rate

### Long-Term Depression (LTD)

When spheres have anti-correlated activity (one working, one idle):
- LTD rate: `0.002` (asymmetric — learning is 5x faster than forgetting)

### Weight Amplification

Coupling weights are amplified by `w^2` (fixed exponent, not variable — V1 bug M12):
- Default weight: `0.18` (= 0.3 * 0.6)
- Effective weight: `0.18^2 = 0.0324`
- After strong LTP: weights can reach ~0.8, effective ~0.64

### Amortised Pruning

Weights below `weight_floor = 0.15` are not immediately pruned. Instead, a batch prune runs when count exceeds `threshold + 50`, removing the lowest-weight connections. This amortises the O(N) scan cost.

---

## 5. Chimera Detection

Chimera states are partially synchronized patterns: some spheres lock-step while others remain incoherent. PV detects these using O(N log N) phase-gap clustering.

### Algorithm

1. Sort sphere phases in ascending order
2. Compute gaps between adjacent phases (including wraparound)
3. Gaps exceeding `PHASE_GAP_THRESHOLD = pi/3` mark cluster boundaries
4. Spheres within the same gap-bounded region form a cluster
5. If cluster count >= 2, chimera is detected

### Significance

Chimera detection feeds into:
- Decision engine (NeedsDivergence considers chimera state)
- Field suggestions (chimera detected -> suggest divergence)
- Analytics/evolution chamber observations

---

## 6. The Bridge Network

PV connects to 6 external ULTRAPLATE services via fire-and-forget raw TCP HTTP bridges:

### Bridge Architecture

```
[SYNTHEX :8090] <--thermal--> [m22] --|
[Nexus :8100]   <--strategy-> [m23] --|
[ME :8080]      <--fitness--> [m24] --|-- [m28 consent_gate] --> k_mod_total --> conductor
[POVM :8125]    <--persist--> [m25] --|
[RM :8130]      ---tsv-post-> [m26]
[VMS :8120]     ---seed-----> [m27]
```

### Consent Gate (m28)

Every external k_adjustment passes through `consent_gated_k_adjustment()`:

1. Raw adjustment from bridge (e.g., SYNTHEX thermal: +0.05)
2. Check sphere consent: `max_k_adjustment` and `accept_external_modulation`
3. Scale by consent: `adj * consent_scale` (where consent_scale = min(sphere.max_k_adj, 1.0))
4. Accumulate: `k_mod_total = synthex_adj + nexus_adj + me_adj + conductor_adj`
5. Clamp to budget: `k_mod_total.clamp(0.85, 1.15)`

### Bridge Patterns

| Bridge | Direction | Frequency | Data |
|--------|-----------|-----------|------|
| SYNTHEX | Bidirectional | Every 6 ticks (30s) | Thermal state, k_adjustment |
| Nexus | Bidirectional | Every 12 ticks (60s) | Strategy, inner/outer Kuramoto r |
| ME | Read | Every 12 ticks (60s) | Fitness, observer metrics |
| POVM | Write | Snapshots 12 ticks, weights 60 ticks | Field state, Hebbian weights |
| RM | Write | Every 60 ticks (5min) | TSV-formatted conductor decisions |
| VMS | Write | Every 60 ticks (5min) | Field memory seeding |

### Fire-and-Forget Pattern

All bridges use raw TCP connections (no hyper/reqwest):

```rust
// Pattern from V1 — no external HTTP client dependency
let mut stream = TcpStream::connect(addr).await?;
stream.write_all(request_bytes).await?;
// For writes: spawn and forget
// For reads: read response with timeout
```

This keeps the dependency tree minimal and avoids blocking the tick loop.

---

## 7. The IPC Bus

The IPC bus provides inter-sphere communication over a Unix domain socket:

### Socket: `/run/user/1000/pane-vortex-bus.sock`

### Wire Protocol: NDJSON

Each message is a single line of JSON (newline-delimited JSON):

```json
{"type":"handshake","sphere_id":"alpha","version":"2.0.0"}
{"type":"submit","task_id":"uuid","source":"alpha","target":"any_idle","desc":"Run tests"}
{"type":"event","event_type":"field.tick","data":{...}}
```

### Task Lifecycle

```
submitted --> claimed --> completed
                    \--> failed
        \--> expired (TTL)
```

Tasks support routing by: specific sphere ID, any idle sphere, field-driven selection, or willing spheres (consent-aware).

### Event Subscriptions

Spheres subscribe to event patterns using glob matching:
- `field.*` matches `field.tick`, `field.decision`, `field.chimera`
- `sphere.alpha.*` matches events for sphere alpha
- `*` matches all events

---

## 8. The Distributed Brain Anatomy

PV does not operate in isolation. It is one organ in a distributed brain:

```
SYNTHEX (:8090)     = Cerebral Cortex
                      Intelligence hub, 61D tensor, 327 NAM components
                      Receives from 3 services, thermal homeostasis

Pane-Vortex (:8132) = Cerebellum
                      Coordination hub, Kuramoto coupling, 6 bridges
                      Drives fleet coherence/divergence

VMS (:8120)         = Hippocampus
                      Spatial memory, fractal topology, OVM field memory

SAN-K7 (:8100)      = Basal Ganglia
                      Action selection, 59 modules, nexus commands

ME (:8080)          = Autonomic Nervous System
                      589K health checks, RALPH evolution, 12D tensor
                      CRITICAL: 240MB data in closed loop (BUG-008)

POVM (:8125)        = Spinal Cord
                      Persistence hub, 2,425 pathways, spherical harmonics

RM (:8130)          = Prefrontal Cortex
                      Cross-session reasoning, TSV format, ~3,250 entries
```

### Critical Integration Gap

ME (autonomic nervous system) has zero EventBus publishers (BUG-008). This means 240MB of runtime data (930K rows, 12D tensor) never feeds back into the distributed brain. V3.1.1 (fixing BUG-008) is the single highest-impact repair in the V3 plan.

---

## 9. The Tick Loop

The tick orchestrator (`m35_tick`) runs every 5 seconds:

### 5-Phase Decomposition (from V1's 829-line tick_once)

```
tick_orchestrator()
  |
  +--> Phase 1: Bridge Polling
  |    - Poll SYNTHEX thermal (every 6 ticks)
  |    - Poll Nexus strategy (every 12 ticks)
  |    - Poll ME fitness (every 12 ticks)
  |    - Each poll is a tokio::spawn fire-and-forget
  |
  +--> Phase 2: Field Update
  |    - Kuramoto coupling steps (15 iterations, adaptive)
  |    - Chimera detection
  |    - Tunnel detection
  |    - Phase wrapping on all spheres
  |
  +--> Phase 3: Learning
  |    - Hebbian LTP for co-active sphere pairs
  |    - Hebbian LTD for anti-correlated pairs
  |    - Burst detection + newcomer boost
  |    - Amortised weight pruning
  |
  +--> Phase 4: Decision
  |    - Compute field state (r, phase spread, idle ratio)
  |    - Run decision engine priority chain
  |    - Apply conductor PI controller
  |    - Consent-gate all bridge k_adjustments
  |    - Generate field suggestions
  |
  +--> Phase 5: Persistence
       - Write field snapshot (every 60 ticks)
       - Write Hebbian weights to POVM (every 60 ticks)
       - Post field state to RM (every 60 ticks)
       - Persist bus events
```

---

## 10. Governance (V3.4 — Feature-Gated)

The governance layer (m37-m41) closes NA-P-15: "the right to say yes — together."

### Proposal System

1. Any sphere can POST to `/field/propose` with: parameter name, proposed value, rationale
2. All active spheres can vote: approve, reject, abstain
3. Quorum: >50% of active spheres, within 5-tick window
4. Approved proposals auto-apply (e.g., change r_target from 0.93 to 0.85)

### Data Sovereignty

Each sphere can:
- GET `/sphere/{id}/data-manifest` — enumerate all data stored about this sphere
- POST `/sphere/{id}/forget` — request deletion of sphere-specific data
- POST `/sphere/{id}/consent` — set explicit consent posture (max_k_adj, opt-outs)

### Evolution Chamber

The evolution module (feature-gated with `evolution`) observes field patterns:
- Anomaly scoring on r_history
- Pattern detection (recurring chimera, oscillating decisions)
- Emergence tracking (novel coupling patterns)

---

## 11. Feature Gates

| Feature | Cargo Flag | Default | Modules |
|---------|-----------|---------|---------|
| `api` | `--features api` | ON | m10_api_server |
| `persistence` | `--features persistence` | ON | m36_persistence |
| `bridges` | `--features bridges` | ON | — (always compiled, flag reserved) |
| `evolution` | `--features evolution` | OFF | m41_evolution_chamber |
| `governance` | `--features governance` | OFF | m37-m41 (entire L8) |
| `full` | `--features full` | OFF | All of the above |

---

## Cross-References

- **[SCHEMATICS.md](SCHEMATICS.md)** — Mermaid diagrams for all architecture views
- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Function-level module index
- **[STATE_MACHINES.md](STATE_MACHINES.md)** — FSM definitions
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan with phase details
- **[config/default.toml](../config/default.toml)** — All constants with defaults
- **Obsidian:** `[[Vortex Sphere Brain-Body Architecture]]`, `[[Session 039 — Architectural Schematics and Refactor Safety]]`
- **V1 Architecture:** `~/claude-code-workspace/pane-vortex/ai_docs/ARCHITECTURE_DEEP_DIVE.md`
