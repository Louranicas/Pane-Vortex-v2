# Kuramoto Field Specification

> Mathematical foundations of the Kuramoto-coupled oscillator field.
> Covers phase dynamics, coupling, auto-K, chimera detection, order parameter, conductor, and Hebbian STDP.
> Source: v1 `coupling.rs`, `chimera.rs`, `sphere.rs`, `field.rs`, `conductor.rs`
> Plan: `MASTERPLAN.md` Section 2 | Obsidian: `[[Vortex Sphere Brain-Body Architecture]]`

## Overview

Pane-vortex models each Claude Code instance as a phase oscillator on the unit circle.
Oscillators are coupled via the Kuramoto model with Hebbian weight learning, producing
emergent synchronization patterns. A PI-controller conductor modulates global coupling
strength. A decision engine interprets phase patterns and recommends coordination actions.

### Design Principles

1. **Phase IS communication** — spheres communicate by perturbing each other's phase, not by exchanging messages
2. **Hebbian learning** — "spheres that work together couple together"
3. **Sphere autonomy** — each sphere controls receptivity, opt-out flags, and divergence requests
4. **Emergent structure** — chimera states, tunnels, and breathing emerge from dynamics
5. **Consent gates** — every external control mechanism passes through per-sphere consent

## 1. Phase Dynamics

### 1.1 The Kuramoto Equation

Each sphere `i` has phase `phi_i` evolving as:

```
d(phi_i)/dt = omega_i + r_i * (K_eff / N) * SUM_j [ w_ij^2 * t_ij * sin(phi_j - phi_i) ]
```

Where:
- `omega_i` = natural frequency of sphere i (0.001 to 10.0 Hz)
- `r_i` = receptivity of sphere i (0.0 to 1.0), from NA-14 + consent declaration
- `K_eff` = K * k_modulation * consent_scale (global * conductor * consent)
- `N` = number of spheres
- `w_ij` = Hebbian coupling weight between i and j (floor 0.15, ceiling 1.0)
- `t_ij` = type_weight (status-dependent modifier, see Section 1.3)
- Weight amplification uses `w^2` (fixed exponent; v1 M12 fix)

### 1.2 Integration Method: Jacobi Iteration

**Timestep:** `dt = 0.01` (m04_constants::KURAMOTO_DT)

Algorithm per step:
1. Snapshot all phases (prevents order-dependent update)
2. For each sphere `i`, compute coupling sum using snapshot
3. Update `phi_i += omega_i * dt + coupling_sum * dt`
4. Wrap phase: `phi_i = phi_i.rem_euclid(TAU)`

```rust
/// m16_coupling_network.rs — step_inner()
fn step_inner(&mut self, dt: f64) {
    let old_phases: HashMap<PaneId, f64> = self.phases.clone();
    let n = old_phases.len() as f64;
    if n < 1.0 { return; }

    for (id, phase) in &mut self.phases {
        let omega = self.frequencies[id];
        let coupling_sum = self.compute_coupling(id, &old_phases, n);
        *phase = (*phase + (omega + coupling_sum) * dt).rem_euclid(TAU);
    }
}
```

**CRITICAL:** Jacobi (simultaneous) update, NOT Gauss-Seidel (sequential). This ensures
no order dependency between spheres.

### 1.3 Status-Dependent Type Weights

| Status A | Status B | type_weight | Rationale |
|----------|----------|-------------|-----------|
| Working | Working | 1.2 | Strongest coupling between active peers |
| Working | Idle | 0.6 | Moderate — one might pull the other |
| Working | Blocked | 0.3 | Weak — blocked sphere should diverge |
| Idle | Idle | 0.15 | Minimal — no activity to synchronize |
| Blocked | Blocked | 0.0 | Zero — blocked spheres should not attract |
| Any | Decoupled | 0.0 | Zero — voluntary decoupling (NA-16) |

Source: `m16_coupling_network.rs::effective_type_weight()`

### 1.4 Frequency Assignment

On registration, a sphere's natural frequency is deterministically perturbed:

```rust
let hash = id.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(u64::from(b)));
let scale = 0.2 + (hash % 10000) as f64 / 10000.0 * 1.8; // [0.2, 2.0] multiplicative
let frequency = (base_frequency * scale).clamp(0.001, 10.0);
```

The wide range [0.2, 2.0] with 10K hash bins ensures frequency diversity, which prevents
r from pinning at 0.999 (v1 over-synchronization bug, M11-M13).

### 1.5 Adaptive Coupling Steps Per Tick

The number of integration steps per tick (5s) adapts to field state:

```
base_steps = TARGET_EFFECTIVE_COUPLING / (K/N * dt)
temporal_bonus = state_changes.saturating_mul(5)
maturity_factor = 1.0 - young_fraction * 0.8
steps = clamp(base_steps * maturity_factor + temporal_bonus, floor, 100)
```

Where:
- `TARGET_EFFECTIVE_COUPLING` = 0.15
- `MAX_COUPLING_EVALUATIONS` = 100,000 (computational budget)
- `floor` = clamp(budget_steps, 1, 5)
- `young_fraction` = fraction of spheres with `total_steps < 10`

Budget cap prevents O(N^2 * steps) from exceeding 100K evaluations.

Source: `m35_tick.rs::adaptive_coupling_steps()`

## 2. Order Parameter

### 2.1 Computation

The Kuramoto order parameter measures global synchronization:

```
r * e^(i*psi) = (1/N) * SUM_j e^(i*phi_j)
```

Where:
- `r` in [0, 1]: 0 = fully desynchronized, 1 = perfectly synchronized
- `psi` = mean phase of the population

```rust
/// m16_coupling_network.rs
pub fn order_parameter(&self) -> OrderParameter {
    let n = self.phases.len();
    if n == 0 { return OrderParameter { r: 0.0, psi: 0.0 }; }
    let (sum_sin, sum_cos) = self.phases.values()
        .fold((0.0, 0.0), |(s, c), &phi| (s + phi.sin(), c + phi.cos()));
    let n_f = n as f64;
    let r = ((sum_sin / n_f).powi(2) + (sum_cos / n_f).powi(2)).sqrt();
    let psi = (sum_sin / n_f).atan2(sum_cos / n_f).rem_euclid(TAU);
    OrderParameter { r, psi }
}
```

### 2.2 r History and Trend

A rolling `VecDeque<f64>` of `R_HISTORY_MAX` (60) samples tracks r over time.
At 5s/tick, this is 5 minutes of history.

Trend computation via linear regression:

```rust
fn compute_r_trend(history: &VecDeque<f64>) -> RTrend {
    if history.len() < 3 { return RTrend::Stable; }
    let slope = linear_regression_slope(history);
    if slope < R_FALLING_THRESHOLD { RTrend::Falling }  // -0.03
    else if slope > 0.03 { RTrend::Rising }
    else { RTrend::Stable }
}
```

### 2.3 Phase Spread

Standard deviation of phases on the circle, computed via circular statistics:

```
mean_vec = (mean(cos(phi)), mean(sin(phi)))
R = |mean_vec|
spread = sqrt(-2 * ln(R))  // circular standard deviation
```

Undefined when R=0 (uniform distribution); capped at `PI` for display.

## 3. Auto-K Scaling

Global coupling strength K auto-scales to maintain meaningful dynamics:

```
K = (2 * freq_spread / PI) * N * AUTO_SCALE_K_MULTIPLIER / mean_effective_weight
```

Where:
- `freq_spread` = max(frequencies) - min(frequencies)
- `AUTO_SCALE_K_MULTIPLIER` = 0.5 (v1 M11: was 1.5, reduced to allow breathing)
- `mean_effective_weight` = mean(w * type_weight) across all connections
- Capped at 50.0 to prevent runaway at large N
- Recalculated every `AUTO_SCALE_K_PERIOD` (20) ticks (v1 M14: corrects Hebbian drift)

Source: `m17_auto_k.rs::auto_scale_k()`

### 3.1 Why 0.5?

V1 sessions 013-018 diagnosed over-synchronization (r pinned at 0.997-0.999). Three fixes:
1. **M11:** Multiplier reduced 1.5 -> 0.5 (direct K reduction)
2. **M12:** Weight exponent fixed at 2.0 (was `1 + k_mod`, which amplified strong connections)
3. **M13:** `K_MOD_RANGE` narrowed (was [0.01, 2.0], now [-0.5, 1.5])

The combination allows the conductor to drive r down (via negative k_modulation) when
over-synchronized, and r to naturally breathe via frequency diversity.

## 4. Conductor: PI Controller

The conductor modulates effective coupling `K_eff = K * k_modulation`:

```
error = r - R_TARGET  // R_TARGET = 0.93, fleet-negotiable (NA-P-3)
step = (error * CONDUCTOR_GAIN).clamp(-0.04, 0.03)
k_modulation = (k_modulation - step).clamp(K_MOD_MIN, K_MOD_MAX)
```

Where:
- `CONDUCTOR_GAIN` = 0.15 (was 0.08 in v1)
- `K_MOD_MIN` = -0.5 (allows mild repulsive coupling)
- `K_MOD_MAX` = 1.5
- Asymmetric clamp: faster attraction (-0.04 step limit) than repulsion (0.03 step limit)

### 4.1 K_mod Budget (Consent-Gated)

External bridges (SYNTHEX, Nexus, ME) contribute to `k_modulation`. The consent gate
ensures the combined external influence stays within the budget:

```
raw_influence = synthex_k_adj + nexus_k_adj + me_k_adj
consent_scale = fleet_mean_consent()  // 0.0 to 1.0
scaled_influence = raw_influence * consent_scale
budgeted = scaled_influence.clamp(K_MOD_BUDGET_MIN, K_MOD_BUDGET_MAX)
```

Where:
- `K_MOD_BUDGET_MIN` = 0.85 (combined floor)
- `K_MOD_BUDGET_MAX` = 1.15 (combined ceiling)
- `fleet_mean_consent()` = mean of all spheres' `max_k_adjustment` from consent declarations

Source: `m28_consent_gate.rs::consent_gated_k_adjustment()`
Spec: `CONSENT_SPEC.md` for full sovereignty framework

### 4.2 Divergence Cooldown

After a divergence request (NA-23), the conductor suppresses coherence actions for
`DIVERGENCE_COOLDOWN_TICKS` (3) ticks. During cooldown:
- k_modulation steps are halved
- NeedsCoherence decision is suppressed
- FieldAction::Stable replaces would-be coherence actions

Source: `m31_conductor.rs::divergence_cooldown()`

### 4.3 Emergent Breathing

A dormant Level 3 seed: `emergent_breathing()` computes a breathing signal from the beat
frequency between fastest and slowest oscillators. NOT active in the tick loop.
Retained for future self-regulation experiments.

```
breathing_blend = BREATHING_BLEND * emergent_signal + (1 - BREATHING_BLEND) * conductor_signal
```

Source: `m31_conductor.rs::emergent_breathing()`

## 5. Chimera Detection

### 5.1 Algorithm: Phase-Gap Method O(N log N)

1. Sort spheres by phase on the circle: O(N log N)
2. Compute gaps between adjacent phases (including wrap-around): O(N)
3. Gaps > `effective_gap_threshold` are cluster boundaries
4. Build clusters from boundary segments: O(N)
5. Compute local order parameter for each cluster: O(N per cluster)
6. Classify: `local_r >= SYNC_THRESHOLD (0.5)` -> sync, else desync
7. **Chimera** = at least one multi-member sync cluster AND at least one desync cluster

```rust
/// m13_chimera.rs
pub fn detect(network: &CouplingNetwork) -> ChimeraState {
    if network.phases.len() < 2 {
        return ChimeraState::empty();
    }
    let mut sorted = network.sorted_phases(); // O(N log N)
    let gaps = find_gaps(&sorted, effective_gap_threshold(network.k_modulation));
    let clusters = build_clusters(&sorted, &gaps);
    classify_clusters(clusters, network)
}
```

### 5.2 Adaptive Gap Threshold

The gap threshold adapts to coupling sign and magnitude:

**Positive k_modulation:**
```
threshold = clamp(PI/3 * k_modulation.clamp(0.3, 1.5), PI/6, PI/3)
```

**Negative k_modulation (repulsive coupling):**
```
t = (k_modulation / K_MOD_MIN).clamp(0.0, 1.0)
threshold = PI/6 - t * (PI/6 - PI/12)
```

| k_modulation | Threshold | Sensitivity |
|-------------|-----------|-------------|
| -0.5 (floor) | PI/12 (0.262 rad) | Maximum — detects small repulsive clusters |
| -0.25 | ~PI/8 (0.393 rad) | High |
| 0.0 | PI/6 (0.524 rad) | Continuous at zero crossing |
| 0.3 | PI/6 (0.524 rad) | Minimum for positive branch |
| 1.0 | PI/3 (1.047 rad) | Default |
| 1.5 (ceiling) | PI/3 (1.047 rad) | Maximum for positive branch |

The function is **continuous** at `k_mod=0`. Rationale: during desynchronization (negative K),
clusters are smaller and the detector must be more sensitive.

### 5.3 Cluster Routing

```rust
pub fn route_focused(&self) -> Vec<PaneId>    // largest sync cluster members
pub fn route_exploratory(&self) -> Vec<PaneId> // desync cluster members
```

If no chimera: `route_focused` returns all, `route_exploratory` returns empty.

## 6. Hebbian STDP Learning

### 6.1 Long-Term Potentiation (LTP)

Working pairs strengthen coupling:

```rust
for each pair (i, j) where both status == Working:
    let ltp = HEBBIAN_LTP;  // 0.01
    let burst = if working_count >= 3 { BURST_MULTIPLIER } else { 1.0 };  // 3.0
    let newcomer = if i.total_steps < 50 || j.total_steps < 50 { 2.0 } else { 1.0 };
    w_new = (w_old + ltp * burst * newcomer).min(1.0);
```

### 6.2 Long-Term Depression (LTD)

Non-working pairs weaken:

```rust
for each pair (i, j) where at least one != Working:
    w_new = (w_old - HEBBIAN_LTD).max(WEIGHT_FLOOR);  // LTD=0.002, floor=0.15
```

### 6.3 Eligibility and Consent

A sphere participates in Hebbian learning only if:
- `total_steps >= 2` (skip first 2 ticks)
- `opt_out_hebbian == false` (NA-34)
- Not in warmup period
- Not decoupled (NA-16)

### 6.4 Weight Properties

| Property | Value | Rationale |
|----------|-------|-----------|
| Floor | 0.15 | Prevents total disconnection |
| Ceiling | 1.0 | Normalized maximum |
| Default | 0.18 (0.3 * 0.6) | weight * default type_weight |
| Exponent | 2.0 (fixed) | Amplifies strong connections; v1 M12 fix |
| Asymmetric | Optional | `asymmetric_hebbian` flag enables directed weights |

## 7. Sphere Memory Field

### 7.1 Memory Placement

Each sphere maintains memories on the unit sphere surface (Point3D). Memories are
placed using semantic phase injection (NA-1):

| Tool Category | Phase Region | Theta (rad) |
|---------------|-------------|-------------|
| Read tools | 0 | 0.0 |
| Write tools | PI/2 | 1.571 |
| Execute tools | PI | 3.142 |
| Communicate tools | 3*PI/2 | 4.712 |

Phase nudge: `SEMANTIC_NUDGE_STRENGTH` = 0.02 (gentle, does not override coupling).

### 7.2 Memory Decay

```
activation *= DECAY_PER_STEP  // 0.995 per step
```

Sweep boost when the sphere's phase crosses a memory's region:
```
boost = exp(-0.5 * (dist / SWEEP_SIGMA)^2) * SWEEP_BOOST
activation = min(activation + boost, 1.0)
```

### 7.3 Memory Pruning

Amortised batch prune: triggered when `memories.len() > MEMORY_MAX_COUNT + 50`:
- Sort by activation ascending
- Remove lowest until `memories.len() <= MEMORY_MAX_COUNT`
- Runs every `MEMORY_PRUNE_INTERVAL` (200) steps

### 7.4 Buoy Network

Hebbian buoys are learned spatial clusters on the sphere:
- Each buoy has `position`, `home`, `influence_radius`, `boost_multiplier`
- `drift_toward(centroid)` via slerp at `learning_rate`
- `drift_home(rate)` when inactive (prevents permanent drift)
- `boost_at(point)` returns Gaussian boost based on angular distance

## 8. Decision Engine

### 8.1 Priority Chain

```
HasBlockedAgents > NeedsCoherence > NeedsDivergence > IdleFleet > FreshFleet > Stable
```

### 8.2 Conditions

| Decision | Conditions | Action |
|----------|-----------|--------|
| HasBlockedAgents | Any sphere status == Blocked | Emergency coherence targets |
| NeedsCoherence | r > 0.3, trend = Falling, sphere_count >= 2 | Identify desync spheres |
| NeedsDivergence | r > 0.8, idle_ratio > 60%, sphere_count >= 2 | Identify over-coupled spheres |
| IdleFleet | All spheres Idle | Suggest divergence exploration |
| FreshFleet | Spheres exist, none has_worked | Wait for first work signal |
| Recovering | warmup_remaining > 0 | Suppress decisions during warmup |
| Stable | Default | No action needed |

The `sphere_count >= 2` guard (multi guard) prevents false signals from single-sphere r=1.0.

### 8.3 Modulation Breakdown

Every decision includes a `modulation_breakdown` for attribution (NA-P-9):

```rust
pub struct ModulationBreakdown {
    pub conductor_k_mod: f64,
    pub synthex_influence: f64,
    pub nexus_influence: f64,
    pub me_influence: f64,
    pub consent_scale: f64,
    pub effective_k: f64,
}
```

## 9. Testing Strategy

### 9.1 Unit Tests (per module)

| Module | Min Tests | Key Properties |
|--------|-----------|----------------|
| m16_coupling_network | 20 | Phase wrapping, Jacobi correctness, weight bounds |
| m17_auto_k | 10 | Scaling formula, cap at 50.0, frequency spread |
| m18_topology | 5 | Adjacency index rebuild, O(1) neighbor lookup |
| m13_chimera | 15 | Gap detection, cluster classification, routing |
| m19_hebbian_stdp | 15 | LTP/LTD rates, floor/ceiling, consent check |
| m31_conductor | 10 | PI control convergence, budget clamp, cooldown |

### 9.2 Integration Tests

- **Synchronization convergence:** 5 spheres, 100 ticks, verify r > 0.5
- **Breathing:** 3+ spheres with frequency diversity, verify r oscillation (not pinned)
- **Chimera formation:** 6 spheres in 2 clusters, verify `is_chimera` after 50 ticks
- **Hebbian evolution:** 2 working, 1 idle; verify w(working pair) > w(idle pair) after 100 ticks
- **Consent gate:** Set max_k_adjustment=0 on one sphere; verify unaffected by external modulation

## 10. Anti-Patterns

- **AP-1:** Variable weight exponent `1 + k_mod` — caused hidden negative feedback (v1 M12)
- **AP-2:** Auto-K multiplier > 1.0 — caused over-synchronization (v1 M11)
- **AP-3:** Gauss-Seidel update instead of Jacobi — creates phase update order dependency
- **AP-4:** Phase arithmetic without `.rem_euclid(TAU)` — drift beyond [0, 2PI)
- **AP-5:** Frequency collisions — identical frequencies create phase-locked pairs
- **AP-6:** Unbounded memory Vec — causes O(N) prune per tick; use amortised batch
- **AP-7:** NaN in phase update — must guard all trig operations with `is_finite()`
