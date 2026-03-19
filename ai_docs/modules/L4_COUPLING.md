---
title: "Layer 4: Coupling — Module Documentation"
date: 2026-03-19
tags: [documentation, l4_coupling, pane-vortex-v2, modules, kuramoto]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Vortex Sphere Brain-Body Architecture]]"
layer: L4
modules: [m16, m17, m18]
---

# Layer 4: Coupling (m16-m18)

> Kuramoto coupling network, Jacobi integration, auto-K scaling, and topology queries.
> **Depends on:** L1 (Foundation), L3 (Field)
> **Target LOC:** ~650 | **Target tests:** 31+

## Modules: m16_coupling_network m17_auto_k m18_topology

## Purpose

L4 implements the coupling mechanics of the Kuramoto model. This layer is O(N^2) in sphere count because every sphere pair has a coupling weight. At SPHERE_CAP=200, this means up to 40,000 weight pairs per coupling step.

## Design Constraints

- Weight exponent is constant w^2, never variable (V1 bug M12 fix)
- Auto-scale K multiplier 0.5 (was 1.5 in V1, caused over-synchronization)
- Frequency clamped to [0.001, 10.0] (pattern P10)
- Phase wrapping after every coupling step (pattern P01)
- Canonical key ordering for symmetric weight lookup

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list.

## Implementation Status: STUB (awaiting implementation)

---

## m16 -- Coupling Network

**Source:** `src/m4_coupling/m16_coupling_network.rs` | **LOC Target:** ~300

Holds all pairwise weights and performs the Jacobi integration step. The Kuramoto equation:

```
d(theta_i)/dt = omega_i + (K/N) * sum_j[ w_ij^2 * sin(theta_j - theta_i) ]
```

Integration uses Jacobi method (read-only pass for influences, then write pass for updates) to prevent order-dependent artifacts. 15 steps per tick at dt=0.01.

### Key Types

- `CouplingNetwork` -- weights: HashMap<(PaneId, PaneId), f64>, k: f64, k_mod: f64

### Key Functions

- `step(spheres, network, dt=0.01, steps=15)` -- Jacobi integration
- `get_weight(a, b) -> f64` -- Canonical key lookup, default 0.18
- `set_weight(a, b, w)` -- Clamp to [0.0, 2.0]
- `effective_k() -> f64` -- k * k_mod
- `per_sphere_influence(id, spheres) -> f64` -- Total coupling influence

### V1 Bugs Fixed

- **M12:** Weight exponent is constant w^2, not variable 1+k_mod
- **M11:** Auto-scale K multiplier 0.5 (was 1.5)

---

## m17 -- Auto-K

**Source:** `src/m4_coupling/m17_auto_k.rs` | **LOC Target:** ~200

Automatically scales global coupling strength K every 20 ticks. Algorithm: K_new = 0.5 * K_old / mean_effective_weight. Divides by mean effective weight to prevent over-synchronization when Hebbian learning increases average weights.

### Key Functions

- `should_recalculate(tick, period) -> bool`
- `auto_scale_k(network, multiplier) -> f64`

---

## m18 -- Topology

**Source:** `src/m4_coupling/m18_topology.rs` | **LOC Target:** ~150

Topology queries: neighborhood (NA-13), weight amplification (NA-25), statistics.

### Key Types

- `Neighborhood` -- sphere_id, neighbors, strongest_neighbor, mean_weight
- `TopologyStats` -- total_connections, mean/max/min weight, variance

### Key Functions

- `neighbors(id, network) -> Neighborhood`
- `weight_squared_amplification(w) -> f64` -- w^2 fixed exponent
- `topology_stats(network) -> TopologyStats`

---

## Complexity Analysis

| Operation | Complexity | At N=200 |
|-----------|-----------|----------|
| Single coupling step | O(N^2) | ~40K pairs |
| 15 coupling steps | O(15*N^2) | ~600K evaluations |
| Auto-K | O(W) | ~40K |
| Neighborhood query | O(W) | ~40K scan |

---

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan
- [ARCHITECTURE_DEEP_DIVE.md](../ARCHITECTURE_DEEP_DIVE.md) -- Kuramoto model math
- [modules/L3_FIELD.md](L3_FIELD.md) -- Spheres (entities being coupled)
- [modules/L5_LEARNING.md](L5_LEARNING.md) -- Hebbian STDP (modifies weights)
- [PERFORMANCE.md](../PERFORMANCE.md) -- Complexity targets
- Obsidian: `[[Vortex Sphere Brain-Body Architecture]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/coupling.rs`
