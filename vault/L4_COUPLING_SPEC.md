# Layer 4: Coupling Specification

> Detailed spec for m16-m18: coupling network, auto-K, topology.
> The Kuramoto engine — phase dynamics, weight matrices, and coupling strength control.
> Source: `src/m4_coupling/` | Plan: `MASTERPLAN.md` Phase 3
> v1 Source: `coupling.rs` (600 LOC, 35 tests)
> Math: `KURAMOTO_FIELD_SPEC.md` | Obsidian: `[[Vortex Sphere Brain-Body Architecture]]`

## Overview

Layer 4 implements the Kuramoto coupled oscillator model that is the computational heart
of pane-vortex. Each Claude Code instance is modelled as a phase oscillator. Oscillators
are coupled through a weight matrix learned by Hebbian STDP (L5). The coupling network
steps at dt=0.01 using Jacobi iteration, computing the order parameter r that measures
fleet synchronization.

L4 depends on L1 (types, constants) and L3 (sphere data). L5 (Learning) and L7 (Coordination)
depend on L4 for the coupling state.

### Design Constraints

| ID | Constraint | Application in L4 |
|----|-----------|-------------------|
| C1 | No upward imports | L4 imports only from L1 and L3 |
| C3 | Phase wrapping | `.rem_euclid(TAU)` after every phase arithmetic operation |
| C11 | NaN guard | `is_finite()` check on all frequency, phase, and weight inputs |
| C12 | Bounded collections | Connection list bounded by SPHERE_CAP^2; adjacency index rebuilt on change |

### The Over-Synchronization Fix (M11-M14)

V1 sessions 013-018 diagnosed a critical issue: r pinned at 0.997-0.999, preventing
any meaningful field dynamics. The fix involved four coordinated changes:

| Fix | What | Why |
|-----|------|-----|
| M11 | `AUTO_SCALE_K_MULTIPLIER` 1.5 -> 0.5 | Direct K reduction prevents r saturation |
| M12 | Weight exponent fixed at 2.0 (was `1 + k_mod`) | Variable exponent created hidden negative feedback: when k_mod was high, exponent amplified weights, pushing r higher, which kept k_mod high |
| M13 | `K_MOD_RANGE` narrowed [-0.5, 1.5] (was [0.01, 2.0]) | Negative k_mod enables mild repulsive coupling for divergence |
| M14 | Auto-K recalculated every 20 ticks (was only at registration) | Corrects Hebbian drift where LTP slowly inflated mean weight |

These four fixes are preserved in v2 as invariants. Violating any one re-introduces
over-synchronization. See Anti-Patterns section.

## 1. m16_coupling_network (~400 LOC)

### 1.1 The Kuramoto Equation

Each sphere `i` has phase `phi_i` evolving as:

```
d(phi_i)/dt = omega_i + r_i * (K_eff / N) * SUM_j [ w_ij^2 * t_ij * sin(phi_j - phi_i) ]
```

Where:
- `omega_i` = natural frequency of sphere i, in [0.001, 10.0] Hz
- `r_i` = receptivity (consent-gated, 0.0 to 1.0)
- `K_eff` = `K * k_modulation * consent_scale` (global * conductor * consent)
- `N` = number of coupled spheres
- `w_ij` = Hebbian coupling weight between i and j (floor 0.15, ceiling 1.0)
- `t_ij` = status-dependent type weight (see Section 1.4)
- `w^2` = fixed quadratic amplification (M12 fix: exponent is always 2.0)

### 1.2 CouplingNetwork Data Structure

```rust
/// The Kuramoto coupling network — owns all phase and connection state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingNetwork {
    /// Per-sphere phase (radians, [0, TAU))
    phases: HashMap<PaneId, Phase>,
    /// Per-sphere natural frequency (Hz, [0.001, 10.0])
    frequencies: HashMap<PaneId, Frequency>,
    /// Connection list — all sphere pairs have bidirectional connections
    connections: Vec<Connection>,
    /// Global coupling strength K (auto-scaled)
    k: f64,
    /// Whether auto-K is enabled (default: true)
    auto_k: bool,
    /// Multiplicative modulation factor for K (1.0 = no change)
    /// Driven by PI conductor in m31.
    k_modulation: f64,
    /// Causal STDP flag — when true, set_weight only updates the directed edge
    asymmetric_hebbian: bool,
    /// Adjacency index for O(1) neighbor lookup. Rebuilt on register/deregister.
    #[serde(skip)]
    adj_index: HashMap<PaneId, Vec<usize>>,
}
```

### 1.3 Jacobi Integration Step

The step function uses Jacobi (simultaneous) iteration: all phases are read from a
snapshot before any are written. This prevents order-dependent phase updates.

```rust
impl CouplingNetwork {
    /// Advance all phases by one Kuramoto integration step.
    /// dt = KURAMOTO_DT (0.01). Called COUPLING_STEPS_PER_TICK times per tick.
    pub fn step(&mut self, dt: f64) {
        // 1. Snapshot all current phases (Jacobi requirement)
        let old_phases: HashMap<PaneId, f64> = self.phases.clone();
        let n = old_phases.len() as f64;
        if n < 1.0 { return; }

        // 2. Compute effective global K
        let k_eff = self.k * self.k_modulation;

        // 3. Update each phase using snapshot values
        for (id, phase) in &mut self.phases {
            let omega = self.frequencies.get(id).copied().unwrap_or(1.0);
            let coupling_sum = self.compute_coupling_for(id, &old_phases, k_eff, n);
            *phase = (*phase + (omega + coupling_sum) * dt).rem_euclid(TAU); // C3
        }
    }

    /// Compute the coupling sum for sphere `id` from the phase snapshot.
    fn compute_coupling_for(
        &self,
        id: &PaneId,
        old_phases: &HashMap<PaneId, f64>,
        k_eff: f64,
        n: f64,
    ) -> f64 {
        let my_phase = old_phases.get(id).copied().unwrap_or(0.0);
        let receptivity = self.receptivities.get(id).copied().unwrap_or(1.0);

        // Use adjacency index for O(degree) instead of O(N) scan
        let indices = match self.adj_index.get(id) {
            Some(idx) => idx,
            None => return 0.0,
        };

        let mut sum = 0.0;
        for &conn_idx in indices {
            let conn = &self.connections[conn_idx];
            let other_phase = old_phases.get(&conn.to).copied().unwrap_or(0.0);
            let phase_diff = other_phase - my_phase;

            // Fixed w^2 exponent (M12 fix — NEVER use variable exponent)
            let weight_factor = conn.weight * conn.weight; // w^2
            let type_factor = conn.type_weight;

            sum += weight_factor * type_factor * phase_diff.sin();
        }

        // Apply K_eff/N and sphere receptivity
        receptivity * (k_eff / n) * sum
    }
}
```

**CRITICAL:** The `weight * weight` computation uses `w^2` with a fixed exponent of 2.0.
V1 bug M12 used `w.powf(1.0 + k_mod)`, which created a hidden feedback loop:
high k_mod -> high exponent -> stronger effective weights -> higher r -> higher k_mod.

### 1.4 Status-Dependent Type Weights

Type weights modulate coupling based on the pair's work status:

```rust
/// Compute the effective type weight for a connection between two spheres.
pub fn effective_type_weight(status_a: PaneStatus, status_b: PaneStatus) -> f64 {
    match (status_a, status_b) {
        (PaneStatus::Working, PaneStatus::Working)   => 1.2,  // Strongest: active peers
        (PaneStatus::Working, PaneStatus::Idle)
        | (PaneStatus::Idle, PaneStatus::Working)     => 0.6,  // Moderate: one active
        (PaneStatus::Working, PaneStatus::Blocked)
        | (PaneStatus::Blocked, PaneStatus::Working)  => 0.3,  // Weak: blocked should diverge
        (PaneStatus::Idle, PaneStatus::Idle)          => 0.15, // Minimal: no activity
        (PaneStatus::Blocked, PaneStatus::Blocked)    => 0.0,  // Zero: blocked should not attract
        (_, PaneStatus::Decoupled)
        | (PaneStatus::Decoupled, _)                  => 0.0,  // Zero: voluntary decoupling (NA-16)
        _                                             => 0.6,  // Default
    }
}
```

### 1.5 Weight Matrix (HashMap with Ordered-Pair Keys)

The weight matrix is stored as a `Vec<Connection>` with an adjacency index for O(1) lookup.
On registration, new connections are created to all existing spheres with default weight 0.3
and default type_weight 0.6 (giving effective weight 0.3 * 0.6 = 0.18).

```rust
/// Connection between two pane-spheres
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: PaneId,
    pub to: PaneId,
    pub weight: f64,      // [WEIGHT_FLOOR, 1.0] — learned by Hebbian STDP
    pub type_weight: f64,  // Status-dependent modifier (see table above)
}

impl CouplingNetwork {
    /// Get weight for a directed edge (a -> b).
    pub fn get_weight(&self, from: &PaneId, to: &PaneId) -> f64 {
        self.connections.iter()
            .find(|c| c.from == *from && c.to == *to)
            .map(|c| c.weight)
            .unwrap_or(DEFAULT_WEIGHT)
    }

    /// Set weight for edge a -> b. If asymmetric_hebbian is false, also sets b -> a.
    pub fn set_weight(&mut self, from: &PaneId, to: &PaneId, weight: f64) {
        let w = weight.clamp(WEIGHT_FLOOR, 1.0); // C11: NaN becomes WEIGHT_FLOOR
        for conn in &mut self.connections {
            if conn.from == *from && conn.to == *to {
                conn.weight = w;
            }
            if !self.asymmetric_hebbian && conn.from == *to && conn.to == *from {
                conn.weight = w;
            }
        }
    }

    /// Adjust weight by delta (clamped to floor/ceiling).
    pub fn adjust_weight(&mut self, from: &PaneId, to: &PaneId, delta: f64) {
        let current = self.get_weight(from, to);
        self.set_weight(from, to, current + delta);
    }
}
```

### 1.6 Frequency Assignment

On registration, a sphere's natural frequency is deterministically perturbed using a
hash of its ID to ensure frequency diversity:

```rust
pub fn register(&mut self, id: PaneId, phase: f64, base_frequency: f64) {
    // Wide multiplicative range [0.2, 2.0] with 10K hash bins
    // Prevents frequency collisions that create phase-locked pairs
    #[allow(clippy::cast_precision_loss)]
    let hash_scale = {
        let hash: u64 = id.bytes()
            .fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(u64::from(b)));
        0.2 + (hash % 10000) as f64 / 10000.0 * 1.8
    };
    let frequency = (base_frequency * hash_scale).clamp(FREQUENCY_MIN, FREQUENCY_MAX);

    // Create bidirectional connections to all existing spheres
    let existing: Vec<PaneId> = self.phases.keys().cloned().collect();
    for other in &existing {
        self.connections.push(Connection {
            from: id.clone(), to: other.clone(),
            weight: 0.3, type_weight: 0.6,
        });
        self.connections.push(Connection {
            from: other.clone(), to: id.clone(),
            weight: 0.3, type_weight: 0.6,
        });
    }

    self.phases.insert(id.clone(), phase.rem_euclid(TAU));
    self.frequencies.insert(id, frequency);
    self.rebuild_adj_index();

    if self.auto_k { self.auto_scale_k(); }
}
```

### 1.7 Order Parameter

```rust
/// Compute the Kuramoto order parameter: r * e^(i*psi) = (1/N) * SUM_j e^(i*phi_j)
pub fn order_parameter(&self) -> OrderParameter {
    let n = self.phases.len();
    if n == 0 { return OrderParameter { r: 0.0, psi: 0.0 }; }

    let (sum_sin, sum_cos) = self.phases.values()
        .fold((0.0, 0.0), |(s, c), &phi| (s + phi.sin(), c + phi.cos()));

    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;
    let r = ((sum_sin / n_f).powi(2) + (sum_cos / n_f).powi(2)).sqrt();
    let psi = (sum_sin / n_f).atan2(sum_cos / n_f).rem_euclid(TAU);

    OrderParameter { r: r.clamp(0.0, 1.0), psi }
}
```

### 1.8 Tests (20 target)

- `step()`: single sphere phase advances by `omega * dt`
- `step()`: two in-phase spheres remain in phase
- `step()`: two anti-phase spheres converge (r increases)
- `step()`: phase wraps correctly at TAU boundary (C3)
- `step()`: NaN frequency produces no coupling (C11)
- `step()`: zero spheres is no-op
- `order_parameter()`: 1 sphere -> r = 1.0
- `order_parameter()`: 2 in-phase -> r = 1.0
- `order_parameter()`: 2 anti-phase -> r near 0.0
- `order_parameter()`: empty network -> r = 0.0
- `register()`: creates 2N connections for Nth sphere
- `register()`: frequency perturbed from base
- `deregister()`: removes all connections involving sphere
- `get_weight()`: returns DEFAULT_WEIGHT for unknown pair
- `set_weight()`: clamps at WEIGHT_FLOOR
- `set_weight()`: symmetric mode updates both directions
- `set_weight()`: asymmetric mode updates only from->to
- `effective_type_weight()`: Working/Working = 1.2
- `effective_type_weight()`: Decoupled/Any = 0.0
- `adj_index`: rebuilt after register, enables O(degree) lookup

## 2. m17_auto_k (~150 LOC)

### 2.1 Auto-Scale K Formula

Global coupling strength K must auto-adjust to maintain meaningful dynamics regardless
of fleet size and frequency distribution:

```
K = (2 * freq_spread / PI) * N * AUTO_SCALE_K_MULTIPLIER / mean_effective_weight
```

Where:
- `freq_spread` = max(frequencies) - min(frequencies)
- `N` = number of spheres
- `AUTO_SCALE_K_MULTIPLIER` = 0.5 (M11 fix: was 1.5)
- `mean_effective_weight` = mean of (w * type_weight) across all connections
- K is capped at 50.0 to prevent runaway at large N

```rust
impl CouplingNetwork {
    /// Recalculate global coupling strength K from current frequency spread and weights.
    /// Called every AUTO_SCALE_K_PERIOD (20) ticks and on registration.
    pub fn auto_scale_k(&mut self) {
        if !self.auto_k || self.phases.len() < 2 { return; }

        let freqs: Vec<f64> = self.frequencies.values().copied().collect();
        let freq_spread = freqs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - freqs.iter().copied().fold(f64::INFINITY, f64::min);

        if freq_spread < 1e-9 { return; } // Avoid near-zero divide

        // Mean effective weight (M11 fix: divide by mean to normalize)
        let mean_eff = if self.connections.is_empty() {
            DEFAULT_WEIGHT
        } else {
            #[allow(clippy::cast_precision_loss)]
            let sum: f64 = self.connections.iter()
                .map(|c| c.weight * c.type_weight)
                .sum();
            sum / self.connections.len() as f64
        };

        let mean_eff = mean_eff.max(0.01); // Floor to prevent divide by near-zero

        #[allow(clippy::cast_precision_loss)]
        let n = self.phases.len() as f64;
        self.k = ((2.0 * freq_spread / PI) * n * AUTO_SCALE_K_MULTIPLIER / mean_eff)
            .min(50.0);
    }
}
```

### 2.2 Why AUTO_SCALE_K_PERIOD = 20 Ticks

Recalculating K every tick would amplify noise in the weight matrix. Every 20 ticks
(100 seconds) provides enough time for Hebbian learning to shift weights meaningfully
while still correcting for drift. This period was chosen empirically in v1 M14 to balance
responsiveness against stability.

### 2.3 Tests (10 target)

- `auto_scale_k()`: single sphere is no-op
- `auto_scale_k()`: zero frequency spread is no-op
- `auto_scale_k()`: K proportional to frequency spread
- `auto_scale_k()`: K proportional to N
- `auto_scale_k()`: K capped at 50.0
- `auto_scale_k()`: higher mean weight reduces K (inverse relationship)
- `auto_scale_k()`: multiplier 0.5 produces lower K than 1.5
- `auto_scale_k()`: called every 20 ticks in tick loop
- `auto_scale_k()`: called on registration
- `auto_scale_k()`: disabled when `auto_k == false`

## 3. m18_topology (~100 LOC)

### 3.1 Adjacency Index

The naive implementation of `compute_coupling_for()` scans all connections O(N^2).
The adjacency index maps each sphere to its outgoing connection indices for O(degree) lookup:

```rust
impl CouplingNetwork {
    /// Rebuild the adjacency index from the connection list.
    /// Called after register/deregister and after deserialization.
    pub fn rebuild_adj_index(&mut self) {
        self.adj_index.clear();
        for (idx, conn) in self.connections.iter().enumerate() {
            self.adj_index.entry(conn.from.clone()).or_default().push(idx);
        }
    }

    /// Get all connection indices where `from == id` (outgoing edges).
    pub fn outgoing(&self, id: &PaneId) -> &[usize] {
        self.adj_index.get(id).map_or(&[], |v| v.as_slice())
    }

    /// Get neighbor IDs for a sphere (unique, excluding self).
    pub fn neighbors(&self, id: &PaneId) -> Vec<PaneId> {
        self.outgoing(id).iter()
            .filter_map(|&idx| {
                let conn = &self.connections[idx];
                if conn.to != *id { Some(conn.to.clone()) } else { None }
            })
            .collect()
    }

    /// Get the N strongest neighbors by effective weight (w * type_weight).
    pub fn strongest_neighbors(&self, id: &PaneId, n: usize) -> Vec<(PaneId, f64)> {
        let mut neighbors: Vec<(PaneId, f64)> = self.outgoing(id).iter()
            .filter_map(|&idx| {
                let conn = &self.connections[idx];
                if conn.to != *id {
                    Some((conn.to.clone(), conn.weight * conn.type_weight))
                } else {
                    None
                }
            })
            .collect();
        neighbors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        neighbors.truncate(n);
        neighbors
    }
}
```

### 3.2 Weight-Squared Amplification

The fixed `w^2` exponent amplifies strong connections and attenuates weak ones,
creating a "rich get richer" effect in the topology:

| w (raw) | w^2 (effective) | Interpretation |
|---------|----------------|----------------|
| 0.15 (floor) | 0.0225 | Near-zero coupling — effectively disconnected |
| 0.30 (default) | 0.0900 | Weak coupling — background influence |
| 0.50 | 0.2500 | Moderate coupling — noticeable phase attraction |
| 0.80 | 0.6400 | Strong coupling — significant synchronization |
| 1.00 (ceiling) | 1.0000 | Maximum coupling — tight phase lock |

This nonlinearity is essential for chimera formation: a few strong connections create
synchronized clusters while many weak connections allow desynchronized periphery.

### 3.3 Tests (5 target)

- `rebuild_adj_index()`: correct index after 3 registrations
- `neighbors()`: returns correct set (excludes self)
- `strongest_neighbors()`: returns top N sorted by effective weight
- `outgoing()`: returns empty slice for unknown sphere
- Index survives serialization round-trip (rebuilt on deserialize)

## Summary

| Module | LOC Target | Key Responsibility | Tests |
|--------|-----------|-------------------|-------|
| m16_coupling_network | 400 | Kuramoto step, weight matrix, order parameter | 20 |
| m17_auto_k | 150 | Auto-scale K formula, periodic recalculation | 10 |
| m18_topology | 100 | Adjacency index, neighborhood queries, w^2 | 5 |
| **L4 Total** | **650** | | **35** |

## Mathematical Reference

### Order Parameter Derivation

The Kuramoto order parameter is the first Fourier mode of the phase distribution on the circle:

```
r * e^(i*psi) = (1/N) * SUM_j=1..N e^(i*phi_j)
```

This can be decomposed into real and imaginary parts:

```
r * cos(psi) = (1/N) * SUM cos(phi_j)    [real]
r * sin(psi) = (1/N) * SUM sin(phi_j)    [imaginary]
r = sqrt( (SUM cos / N)^2 + (SUM sin / N)^2 )
psi = atan2( SUM sin / N, SUM cos / N )
```

Properties:
- r = 1.0 when all phases are identical (perfect synchronization)
- r = 0.0 when phases are uniformly distributed (complete incoherence)
- r oscillates between these extremes in the interesting regime
- Single-sphere r is always 1.0 (multi guard needed in decision engine)

### Critical Coupling Threshold

From Kuramoto theory, the critical coupling for onset of synchronization:

```
K_c = 2 / (pi * g(0))
```

Where `g(0)` is the value of the frequency distribution at its center. For our
hash-perturbed uniform distribution, this gives approximate K_c around the frequency
spread divided by PI. Our `auto_scale_k` formula targets slightly above K_c to allow
breathing dynamics rather than rigid synchronization.

## Anti-Patterns (CRITICAL)

- **AP-1 (M12):** Variable weight exponent `w.powf(1.0 + k_mod)` — creates hidden feedback loop.
  ALWAYS use fixed `w * w`. This was the root cause of the over-synchronization bug.
- **AP-2 (M11):** `AUTO_SCALE_K_MULTIPLIER > 1.0` — pushes K above critical threshold,
  pins r at 0.999. ALWAYS keep at 0.5 or below.
- **AP-3:** Gauss-Seidel update (sequential) instead of Jacobi (simultaneous) — creates
  phase update order dependency. The sphere iterated first "sees" different phases than
  the sphere iterated last. ALWAYS snapshot phases before updating.
- **AP-4:** Phase arithmetic without `.rem_euclid(TAU)` — phase drifts beyond [0, 2PI).
  Eventually causes NaN from accumulated floating-point error.
- **AP-5:** Frequency collisions — identical frequencies create phase-locked pairs that
  never separate. The hash-based perturbation with 10K bins prevents this.
- **AP-6 (M14):** Auto-K only at registration — Hebbian learning slowly inflates mean weight,
  which should increase K proportionally but does not without periodic recalculation.
- **AP-7:** Unbounded connection list — O(N^2) connections. The SPHERE_CAP (200) bounds this
  at 200^2 = 40,000 connections maximum. The adjacency index makes per-step cost O(degree).

## Related

- `KURAMOTO_FIELD_SPEC.md` — Full mathematical specification (Sections 1-4)
- `CONSENT_SPEC.md` Section 2 — Per-sphere K_mod isolation via consent gate
- `layers/L5_LEARNING_SPEC.md` — Hebbian STDP that modifies coupling weights
- `layers/L7_COORDINATION_SPEC.md` Section 3 — Conductor PI control that modifies k_modulation
- `[[Vortex Sphere Brain-Body Architecture]]` — Field theory foundations
- `[[Session 039 — Architectural Schematics and Refactor Safety]]` — Risk hotspots, POVM bimodal weights
- `[[Session 034e — NA Gap Analysis of Master Plan V2]]` — NA-P-4 per-sphere isolation requirement
