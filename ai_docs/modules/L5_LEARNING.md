---
title: "Layer 5: Learning — Module Documentation"
date: 2026-03-19
tags: [documentation, l5_learning, pane-vortex-v2, hebbian, stdp]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Vortex Sphere Brain-Body Architecture]]"
layer: L5
modules: [m19, m20, m21]
---

# Layer 5: Learning (m19-m21)

> Hebbian STDP, buoy network, and memory management.
> **Depends on:** L1 (Foundation), L3 (Field), L4 (Coupling)
> **Target LOC:** ~750 | **Target tests:** 35+

## Modules: m19_hebbian_stdp m20_buoy_network m21_memory_manager

## Purpose

L5 implements the learning mechanisms that allow the Kuramoto field to adapt over time. Hebbian STDP (Spike-Timing-Dependent Plasticity) modifies coupling weights based on sphere co-activity. Buoys provide 3D navigational landmarks in phase space. The memory manager handles sphere memory lifecycle with amortised pruning.

## Design Constraints

- LTP rate (0.01) is 5x faster than LTD rate (0.002) -- learning is faster than forgetting
- Burst detection: 3+ consecutive working ticks triggers 3x LTP multiplier
- Newcomer boost: first 50 steps get 2x LTP multiplier
- Combined newcomer+burst = 6x base rate
- Weight floor at 0.15 -- weights below this are candidates for pruning
- Amortised batch prune at MEMORY_MAX+50, not every tick
- Never prune during burst (sphere is actively learning)

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list.

## Implementation Status: STUB (awaiting implementation)

---

## m19 -- Hebbian STDP

**Source:** `src/m5_learning/m19_hebbian_stdp.rs` | **LOC Target:** ~300

### Purpose

Implements Hebbian learning on coupling weights. "Neurons that fire together, wire together."

### Long-Term Potentiation (LTP)

When two spheres are simultaneously Working:
- Base rate: 0.01
- Burst mode (3+ consecutive working ticks): 3x multiplier
- Newcomer (step_count < 50): 2x multiplier (NA-32)
- Combined: newcomer in burst = 6x base = 0.06 per tick

```rust
pub fn apply_ltp(
    network: &mut CouplingNetwork,
    a: &PaneId, b: &PaneId,
    config: &HebbianConfig,
    is_burst: bool,
    is_newcomer: bool,
) -> f64 {
    let mut rate = config.ltp_rate;
    if is_burst { rate *= config.burst_multiplier; }
    if is_newcomer { rate *= config.newcomer_multiplier; }
    let current = network.get_weight(a, b);
    let new_weight = (current + rate).min(STRENGTH_MAX);
    network.set_weight(a, b, new_weight);
    new_weight - current
}
```

### Long-Term Depression (LTD)

When spheres have anti-correlated activity (one Working, one Idle):
- Rate: 0.002
- Asymmetric: learning is 5x faster than forgetting

```rust
pub fn apply_ltd(
    network: &mut CouplingNetwork,
    a: &PaneId, b: &PaneId,
    config: &HebbianConfig,
) -> f64 {
    let current = network.get_weight(a, b);
    let new_weight = (current - config.ltd_rate).max(config.weight_floor);
    network.set_weight(a, b, new_weight);
    new_weight - current
}
```

### Hebbian Step

Called once per tick in Phase 3 of the tick orchestrator:

```rust
pub fn hebbian_step(
    spheres: &HashMap<PaneId, PaneSphere>,
    network: &mut CouplingNetwork,
    config: &HebbianConfig,
) -> Vec<LearningEvent> {
    let mut events = Vec::new();
    let working: Vec<&PaneId> = spheres.iter()
        .filter(|(_, s)| s.status == SphereStatus::Working)
        .map(|(id, _)| id)
        .collect();
    let idle: Vec<&PaneId> = spheres.iter()
        .filter(|(_, s)| s.status == SphereStatus::Idle)
        .map(|(id, _)| id)
        .collect();

    // LTP: all working pairs
    for (i, a) in working.iter().enumerate() {
        for b in &working[i+1..] {
            let delta = apply_ltp(network, a, b, config,
                detect_burst(&spheres[*a]),
                is_newcomer(&spheres[*a]) || is_newcomer(&spheres[*b]));
            events.push(LearningEvent { sphere_a: (*a).clone(), sphere_b: (*b).clone(), delta_w: delta, reason: "ltp" });
        }
    }

    // LTD: working-idle pairs
    for w in &working {
        for i in &idle {
            let delta = apply_ltd(network, w, i, config);
            events.push(LearningEvent { sphere_a: (*w).clone(), sphere_b: (*i).clone(), delta_w: delta, reason: "ltd" });
        }
    }

    events
}
```

### Key Types

- `HebbianConfig` -- ltp_rate, ltd_rate, burst_multiplier, newcomer_multiplier, weight_floor
- `LearningEvent` -- sphere_a, sphere_b, delta_w, reason

### Burst Detection

A sphere is in burst mode when it has been Working for 3+ consecutive ticks. Tracked via work_signature.burst_count.

### Respect for Consent (NA-34)

If a sphere has `preferences.opt_out_hebbian == true`, it is excluded from both LTP and LTD. Its weights are never modified.

---

## m20 -- Buoy Network

**Source:** `src/m5_learning/m20_buoy_network.rs` | **LOC Target:** ~200

### Purpose

Buoys are 3D navigational markers in phase space. They provide landmarks that spheres can use for spatial recall (NA-17). Each buoy has a position, activation level, label, and decay rate.

### Key Types

- `BuoyNetwork` -- buoys: Vec<Buoy>

### Key Functions

- `nearest_buoy(position: Point3D) -> Option<&Buoy>` -- Euclidean distance search
- `activate_buoy(idx, amount)` -- Increase activation (clamped to [0, 1])
- `decay_all(rate: f64)` -- Apply multiplicative decay to all buoys
- `prune_below(threshold: f64)` -- Remove buoys with activation below threshold
- `add_buoy(position, label)` -- Add a new navigational marker

### Buoy Lifecycle

1. Created when a sphere records a significant memory (tool use at notable phase)
2. Activation increases on revisit (nearby phase + same tool type)
3. Decays by DECAY_PER_STEP=0.995 each tick
4. Pruned when activation drops below ACTIVATION_THRESHOLD=0.3

---

## m21 -- Memory Manager

**Source:** `src/m5_learning/m21_memory_manager.rs` | **LOC Target:** ~250

### Purpose

Manages sphere memory lifecycle: creation, recall, amortised pruning, and narrative generation.

### Amortised Pruning (V1 Bug RG-3)

V1 pruned every tick, causing O(M) overhead. V2 uses amortised batch pruning:

```rust
pub fn amortised_batch_prune(sphere: &mut PaneSphere, max_plus: usize) -> usize {
    if sphere.memories.len() <= MEMORY_MAX_COUNT + max_plus {
        return 0;
    }
    // Sort by activation (ascending), keep top MEMORY_MAX_COUNT
    sphere.memories.sort_by(|a, b| a.activation.partial_cmp(&b.activation).unwrap_or(Ordering::Equal));
    let removed = sphere.memories.len() - MEMORY_MAX_COUNT;
    sphere.memories.truncate(MEMORY_MAX_COUNT);
    removed
}
```

Pruning triggers when memory count exceeds MAX + 50 (default: 500 + 50 = 550). This amortises the O(M) sort cost over 50 memory additions.

### ID Reconciliation (V1 Bug I3)

After snapshot restore, `next_memory_id` may be stale:

```rust
pub fn reconcile_ids(sphere: &mut PaneSphere) {
    sphere.next_memory_id = sphere.memories.iter()
        .map(|m| m.id)
        .max()
        .unwrap_or(0) + 1;
}
```

### Narrative Generation (NA-19)

Generates a text narrative of a sphere's memory:

```rust
pub fn narrative(sphere: &PaneSphere) -> String {
    // Group memories by tool, order by timestamp
    // Generate: "Used {tool} {count} times, most recently at phase {phase}"
}
```

---

## Cross-References

- [modules/L3_FIELD.md](L3_FIELD.md) -- PaneSphere (entity with memories)
- [modules/L4_COUPLING.md](L4_COUPLING.md) -- CouplingNetwork (weights modified by STDP)
- [ARCHITECTURE_DEEP_DIVE.md](../ARCHITECTURE_DEEP_DIVE.md) -- Hebbian STDP section
- [PERFORMANCE.md](../PERFORMANCE.md) -- Learning phase timing budget
- Obsidian: `[[Vortex Sphere Brain-Body Architecture]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/sphere.rs` (memory + buoy methods)

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan
- V3.2 (Inhabitation) drives multi-sphere testing of Hebbian pathways
