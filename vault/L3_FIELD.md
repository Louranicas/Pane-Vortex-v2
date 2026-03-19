---
title: "Layer 3: Field — Module Documentation"
date: 2026-03-19
tags: [documentation, l3_field, pane-vortex-v2, modules, kuramoto]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Vortex Sphere Brain-Body Architecture]]"
layer: L3
modules: [m11, m12, m13, m14, m15]
---

# Layer 3: Field (m11-m15)

> Kuramoto oscillator field: spheres, field state, chimera detection, messaging, and shared state.
> **The most critical layer.** This is where the Kuramoto dynamics live.
> **Depends on:** L1 (Foundation)
> **Target LOC:** ~1,750 | **Target tests:** 83+

## Modules: m11_sphere m12_field_state m13_chimera m14_messaging m15_app_state

## Purpose

L3 implements the core field abstraction. Each Claude Code instance registers as a PaneSphere oscillator on a shared Kuramoto field. The field state tracks aggregate metrics (order parameter r, coupling strength, phase spread), detects chimera patterns, manages ghost traces, and houses the shared state container.

## Design Constraints

- Phase wrapping (`.rem_euclid(TAU)`) after ALL phase arithmetic (pattern P01)
- Lock ordering: AppState before BusState (pattern P02)
- NaN guard on all phase/frequency updates (pattern P08)
- Sphere cap at 200 (security, prevents O(N^2) exhaustion)
- VecDeque for all logs and inboxes, never Vec (pattern P12)
- Multi guard (spheres >= 2) on coherence/divergence decisions

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list. Constraints applicable to this layer are documented in `src/m3_field/mod.rs`.

## Implementation Status: STUB (awaiting implementation)

---

## m11 -- Sphere (The Core Entity)

**Source:** `src/m3_field/m11_sphere.rs` | **LOC Target:** ~500

PaneSphere is the representation of a Claude Code instance in the Kuramoto field. It is the most complex L3 type: an oscillator with phase, frequency, memories, buoys, a self-model, consent preferences, and an inbox.

### Key Types

- `PaneSphere` -- Full oscillator state: id, phase, frequency, status, persona, memories, buoys, self_model, preferences, inbox, step_count, work_signature
- `SelfModel` -- is_synchronized, tunnel_count, total_steps, maturity, age_secs (NA-12)
- `Maturity` -- Newcomer (step < 50, 2x LTP), Established (50-199), Senior (200+)
- `CouplingPreferences` -- opt_out_hebbian, opt_out_cross_activation, max_k_adj, accept_observation (NA-33, NA-34)
- `InboxMessage` -- id, from, content, timestamp (NA-20, NA-21)

### Key Functions

- `PaneSphere::new(id, persona)` -- Create sphere with default phase/frequency
- `step(dt, influence)` -- Advance phase, always wraps with rem_euclid(TAU)
- `add_memory(tool, phase)` -- Record tool use, truncates to 128 chars (P13)
- `recall(near_phase, near_buoy)` -- Spatial + phase proximity query (NA-17)
- `summary()` -- Generate SphereSummary with self-model, neighborhood, maturity
- `set_status(status)` -- Update status; has_worked monotonic (RG-2 fix)
- `receive_message(msg)` -- Push to inbox VecDeque (capped 50, FIFO eviction)
- `acknowledge_message(id)` -- Remove from inbox (NA-21)
- `prune_memories()` -- Amortised batch prune at MAX+50 (RG-3)

### V1 Bugs Fixed in Design

- C1 (UTF-8 panic): chars().take() replaces byte slice
- RG-2 (auto-status false positive): has_worked is monotonic
- RG-3 (memory cap unbounded): amortised batch prune
- I3 (next_memory_id stale): reconcile_memory_ids() on restore

---

## m12 -- Field State

**Source:** `src/m3_field/m12_field_state.rs` | **LOC Target:** ~400

Aggregate state of the Kuramoto field, computed from all spheres.

### Key Types

- `FieldState` -- r, r_history (VecDeque, max 60), k, k_mod, effective_k, phase_spread, mean_phase, sphere/idle/working/blocked counts, tunnels, chimera_detected, modulation_breakdown
- `FieldDecision` -- action, targets, rationale, attribution
- `DecisionAction` -- Stable, NeedsCoherence, NeedsDivergence, HasBlockedAgents, IdleFleet, FreshFleet, Recovering
- `Tunnel` -- sphere_a, sphere_b, phase_diff, label (NA-6)
- `GhostTrace` -- id, persona, weights, top_memories, departed_at, consent_given (NA-28, NA-29)
- `ModulationBreakdown` -- synthex, nexus, me, conductor, consent_scale (NA-P-9)

### Key Functions

- `FieldState::compute(spheres)` -- Compute r, phase_spread, status counts
- `compute_r(phases)` -- Order parameter from phase vector
- `detect_tunnels(spheres, weights)` -- Find phase-locked pairs
- `decide(state, spheres)` -- Priority chain decision engine

### Decision Priority Chain

HasBlockedAgents > NeedsCoherence (r>0.3, falling, multi) > NeedsDivergence (r>0.8, idle>60%, multi) > IdleFleet > FreshFleet > Stable

---

## m13 -- Chimera Detection

**Source:** `src/m3_field/m13_chimera.rs` | **LOC Target:** ~200

O(N log N) phase-gap cluster detection.

### Algorithm

1. Sort sphere phases ascending
2. Compute inter-phase gaps (including wraparound)
3. Gaps exceeding PHASE_GAP_THRESHOLD (pi/3) mark cluster boundaries
4. Group spheres between boundaries into PhaseCluster
5. cluster_count >= 2 means chimera detected

### Key Types

- `ChimeraResult` -- detected, cluster_count, clusters
- `PhaseCluster` -- sphere_ids, mean_phase, spread

### V1 Bug Fix (I1)

V1 used inflated denominator. V2 uses actual found count.

---

## m14 -- Messaging

**Source:** `src/m3_field/m14_messaging.rs` | **LOC Target:** ~150

5 PhaseMessage types for field communication.

### Types

- `Steer` -- target, phase
- `CrossActivation` -- source, targets, strength (clamped [0.0, 2.0])
- `EmergencyCoherence` -- targets (capped at 50)
- `SemanticNudge` -- target, tool, phase_region
- `FieldBroadcast` -- action, data

---

## m15 -- App State

**Source:** `src/m3_field/m15_app_state.rs` | **LOC Target:** ~500

Central shared state container with RwLock-protected collections.

### Structure

- `AppState` -- spheres: RwLock<HashMap>, field: RwLock<FieldState>, ghosts: RwLock<VecDeque<GhostTrace>>, config, tick: AtomicU64, started_at, warmup_remaining: AtomicU32, dirty: AtomicBool
- `SharedState` -- app: Arc<AppState>, bus: Arc<BusState>

### Lock Ordering (CRITICAL)

Always AppState before BusState. This is pattern P02 and the single most important concurrency invariant.

### Key Functions

- `register_sphere(id, persona)` -- Create sphere, check ghost inheritance
- `deregister_sphere(id)` -- Remove, create GhostTrace, push to ghosts deque
- `snapshot()` -- Extract serializable field state for persistence
- `restore_snapshot(snapshot)` -- Restore + enter warmup (5 ticks)
- `reconcile_memory_ids()` -- Fix next_memory_id after restore (V1 bug I3)

---

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan
- [ARCHITECTURE_DEEP_DIVE.md](../ARCHITECTURE_DEEP_DIVE.md) -- Kuramoto model, decision engine
- [STATE_MACHINES.md](../STATE_MACHINES.md) -- Sphere lifecycle, decision FSM
- [SCHEMATICS.md](../SCHEMATICS.md) -- Mermaid diagrams
- [modules/L4_COUPLING.md](L4_COUPLING.md) -- Coupling network
- [modules/L5_LEARNING.md](L5_LEARNING.md) -- Hebbian STDP
- Obsidian: `[[Vortex Sphere Brain-Body Architecture]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/sphere.rs`, `field.rs`, `chimera.rs`, `state.rs`
