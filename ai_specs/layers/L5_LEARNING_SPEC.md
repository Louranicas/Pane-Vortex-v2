# Layer 5: Learning Specification

> Detailed spec for m19-m21: Hebbian STDP, buoy network, memory manager.
> The learning layer — how the field adapts, remembers, and prunes.
> Source: `src/m5_learning/` | Plan: `MASTERPLAN.md` Phase 4
> v1 Source: `coupling.rs` (Hebbian section, ~200 LOC), `sphere.rs` (memory, ~400 LOC)
> Math: `KURAMOTO_FIELD_SPEC.md` Section 6-7 | Obsidian: `[[Session 039 — What I Learned]]`

## Overview

Layer 5 implements three interconnected learning mechanisms:

1. **Hebbian STDP** (m19) — Coupling weights between spheres strengthen when they work
   together (LTP) and weaken when they do not (LTD). This is the Kuramoto field's
   equivalent of synaptic plasticity.
2. **Buoy Network** (m20) — Spatial markers on the unit sphere that learn where tool
   activity concentrates. Buoys drift toward activation centroids and boost nearby memories.
3. **Memory Manager** (m21) — Per-sphere memory storage with decay, pruning, and ID
   reconciliation. Memories are placed on the sphere surface and decay exponentially.

L5 depends on L1 (types, constants) and L3 (sphere data). L4 (Coupling) provides the
weight matrix that m19 modifies. L7 (Coordination) calls L5 from the tick loop.

### Design Constraints

| ID | Constraint | Application in L5 |
|----|-----------|-------------------|
| C1 | No upward imports | L5 imports only from L1 and L3 |
| C11 | NaN guard | Weight adjustments check `is_finite()` before applying |
| C12 | Bounded collections | Memory capped at MEMORY_MAX_COUNT (500), VecDeque for recent_active |

### The POVM Bimodal Weight Distribution

Session 039 analysis of POVM pathway data revealed that Hebbian weights follow a bimodal
distribution, not a Gaussian. This is a fundamental insight about phase-transitive learning:

| Weight Band | Proportion | Interpretation |
|-------------|-----------|----------------|
| 0.15-0.30 (near floor) | ~90% | Default connections — never co-worked |
| 0.30-0.65 (middle) | ~7.2% | Intermittent co-workers — some shared sessions |
| 0.65-1.00 (crystallized) | ~2.8% | Persistent partners — deep collaborative bonds |

Source: `[[Session 039 — What I Learned]]`, `[[Session 039 — Architectural Schematics and Refactor Safety]]`

The bimodal shape emerges because:
- LTD continuously pulls non-working pairs toward the floor (0.15)
- LTP only activates for working pairs, creating a "jump" when both work simultaneously
- Burst detection (3x) and newcomer boost (2x) accelerate crystallization for active pairs
- The result is a phase transition: connections are either "cold" (floor) or "hot" (crystallized)

This bimodality is **desirable** — it means the Hebbian learning is functioning correctly,
creating a sparse topology where only real collaborative relationships are strongly coupled.

## 1. m19_hebbian_stdp (~250 LOC)

### 1.1 Long-Term Potentiation (LTP)

Working pairs strengthen their coupling:

```rust
/// Apply Hebbian LTP to all eligible working pairs.
/// Called once per tick from m35_tick::tick_once().
pub fn hebbian_ltp(
    spheres: &HashMap<PaneId, PaneSphere>,
    network: &mut CouplingNetwork,
    warmup: bool,
) {
    if warmup { return; } // No learning during warmup (snapshot restore)

    // Collect eligible working spheres
    let working: Vec<&PaneId> = spheres.iter()
        .filter(|(_, s)| {
            s.status == PaneStatus::Working
                && !s.opt_out_hebbian      // NA-34: respect opt-out
                && s.total_steps >= 2       // Skip first 2 ticks (noise)
                && s.status != PaneStatus::Decoupled  // NA-16
        })
        .map(|(id, _)| id)
        .collect();

    // Burst detection: 3+ working spheres triggers 3x LTP multiplier
    let burst = working.len() >= 3;
    let burst_mult = if burst { BURST_MULTIPLIER } else { 1.0 }; // 3.0 or 1.0

    // LTP: strengthen weight for all working pairs
    for i in 0..working.len() {
        for j in (i + 1)..working.len() {
            let newcomer_mult = newcomer_multiplier(
                spheres.get(working[i]),
                spheres.get(working[j]),
            );

            let ltp = HEBBIAN_LTP * burst_mult * newcomer_mult;
            network.adjust_weight(working[i], working[j], ltp);
        }
    }
}
```

### 1.2 LTP Constants

| Constant | Value | Source | Purpose |
|----------|-------|--------|---------|
| `HEBBIAN_LTP` | 0.01 | m04_constants | Base learning rate per tick |
| `BURST_MULTIPLIER` | 3.0 | m04_constants | Multiplier when 3+ spheres work simultaneously |
| `NEWCOMER_MULTIPLIER` | 2.0 | m04_constants | Multiplier for spheres with `total_steps < NEWCOMER_STEPS` |
| `NEWCOMER_STEPS` | 50 | m04_constants | Steps below which a sphere is considered a newcomer |

At maximum amplification (burst + both newcomers): `0.01 * 3.0 * 2.0 = 0.06` per tick.
A connection at floor (0.15) reaches crystallization (0.80) in approximately:
`(0.80 - 0.15) / 0.06 = 10.8 ticks = ~54 seconds`.

At minimum (no burst, no newcomer): `0.01` per tick.
Floor to crystallization: `(0.80 - 0.15) / 0.01 = 65 ticks = ~325 seconds = ~5.4 minutes`.

### 1.3 Long-Term Depression (LTD)

Non-working pairs weaken:

```rust
/// Apply Hebbian LTD to all non-working pairs.
/// Called once per tick from m35_tick::tick_once() (after LTP).
pub fn hebbian_ltd(
    spheres: &HashMap<PaneId, PaneSphere>,
    network: &mut CouplingNetwork,
    warmup: bool,
) {
    if warmup { return; }

    for conn in &mut network.connections {
        let a_working = spheres.get(&conn.from)
            .map_or(false, |s| s.status == PaneStatus::Working && !s.opt_out_hebbian);
        let b_working = spheres.get(&conn.to)
            .map_or(false, |s| s.status == PaneStatus::Working && !s.opt_out_hebbian);

        // LTD applies when at least one sphere in the pair is NOT working
        if !a_working || !b_working {
            conn.weight = (conn.weight - HEBBIAN_LTD).max(WEIGHT_FLOOR);
        }
    }
}
```

### 1.4 LTD Constants

| Constant | Value | Source | Purpose |
|----------|-------|--------|---------|
| `HEBBIAN_LTD` | 0.002 | m04_constants | Depression rate per tick |
| `WEIGHT_FLOOR` | 0.15 | m04_constants | Minimum weight — prevents total disconnection |

LTP/LTD ratio: `0.01 / 0.002 = 5:1` asymmetry. This means a connection needs 5 ticks
of inactivity to undo 1 tick of co-activation. The asymmetry ensures that learning is
progressive — brief pauses do not erase deep collaborative bonds.

### 1.5 Newcomer Multiplier (NA-32)

```rust
/// Compute newcomer boost multiplier for a pair of spheres.
/// Either sphere being a newcomer triggers the boost.
fn newcomer_multiplier(
    a: Option<&PaneSphere>,
    b: Option<&PaneSphere>,
) -> f64 {
    let a_new = a.map_or(false, |s| s.total_steps < NEWCOMER_STEPS);
    let b_new = b.map_or(false, |s| s.total_steps < NEWCOMER_STEPS);
    if a_new || b_new { NEWCOMER_MULTIPLIER } else { 1.0 }
}
```

Rationale: new spheres start with all connections at default weight. The 2x boost
accelerates their integration into the established weight topology, reducing the
"cold start" period from ~5 minutes to ~2.5 minutes.

### 1.6 Consent Interactions

| Flag | Effect on Learning |
|------|-------------------|
| `opt_out_hebbian` | Sphere excluded from both LTP and LTD — weights frozen |
| `PaneStatus::Decoupled` | Sphere excluded from LTP (type_weight = 0.0 prevents coupling) |
| Warmup period | All learning disabled for WARMUP_TICKS (5) after snapshot restore |

### 1.7 Tests (15 target)

- LTP: two working spheres' weight increases by exactly HEBBIAN_LTP
- LTP: burst mode (3 working) applies 3x multiplier
- LTP: newcomer boost applies 2x when one sphere has <50 steps
- LTP: opted-out sphere unaffected (weight unchanged)
- LTP: decoupled sphere excluded
- LTP: warmup skips all learning
- LTD: non-working pair weight decreases by HEBBIAN_LTD
- LTD: weight floors at WEIGHT_FLOOR (0.15)
- LTD: opted-out sphere unaffected
- LTD: warmup skips all learning
- LTP + LTD cycle: 10 ticks of co-work then 50 ticks idle -> net increase
- Weight never exceeds 1.0
- Weight never goes below WEIGHT_FLOOR
- NaN weight adjustment is no-op (C11)
- Combined: burst + newcomer = 0.01 * 3.0 * 2.0 = 0.06

## 2. m20_buoy_network (~200 LOC)

### 2.1 Buoy Structure

Buoys are learned spatial markers on the unit sphere that represent clusters of tool activity:

```rust
/// A Hebbian buoy — learned spatial cluster on the sphere surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buoy {
    /// Current position on the unit sphere
    pub position: Point3D,
    /// Home position (original placement — drift anchor)
    pub home: Point3D,
    /// Number of times this buoy has been activated
    pub activation_count: u64,
    /// Radius of influence (radians on the sphere)
    pub influence_radius: f64,
    /// Boost multiplier for memories within influence radius
    pub boost_multiplier: f64,
    /// Learning rate for drift-toward behavior
    pub learning_rate: f64,
    /// Human-readable label (e.g., "Read", "Edit", "Bash")
    pub label: String,
}
```

### 2.2 Activation Zones

A buoy's influence on nearby memories follows a Gaussian profile based on angular distance:

```rust
impl Buoy {
    /// Compute boost at a given point on the sphere.
    /// Returns 0.0 if outside the influence radius.
    pub fn boost_at(&self, point: &Point3D) -> f64 {
        let dist = self.position.angular_distance(point);
        if dist > self.influence_radius { return 0.0; }
        let sigma = self.influence_radius / 3.0;
        self.boost_multiplier * (-0.5 * (dist / sigma).powi(2)).exp()
    }
}
```

Activation zone classification based on angular distance from buoy center:

| Zone | Distance | Boost Level | Interpretation |
|------|----------|-------------|----------------|
| Vivid | 0 - R/3 | 80-100% of boost_multiplier | Core activity region |
| Clear | R/3 - 2R/3 | 30-80% | Active region |
| Dim | 2R/3 - R | 5-30% | Peripheral region |
| Trace | > R | 0% | Outside influence |

Where R = `influence_radius` (default: PI/4 = 0.785 rad).

### 2.3 Drift Dynamics

Buoys drift toward the centroid of their activating memories, creating a self-organizing
spatial map of tool usage:

```rust
impl Buoy {
    /// Drift position toward a target point using spherical interpolation.
    /// Rate is modulated by activation count (more activations = slower drift).
    pub fn drift_toward(&mut self, target: &Point3D) {
        let rate = self.learning_rate / (1.0 + (self.activation_count as f64).ln().max(0.0));
        self.position = self.position.slerp(target, rate);
    }

    /// Drift back toward home position when inactive. Prevents permanent drift
    /// away from initial placement.
    pub fn drift_home(&mut self, rate: f64) {
        self.position = self.position.slerp(&self.home, rate);
    }
}
```

### 2.4 Sweep Boost

When a sphere's phase crosses a buoy's angular region, nearby memories receive a boost:

```rust
/// Apply sweep boost to memories near buoys whose phase region is currently active.
pub fn apply_sweep_boost(sphere: &mut PaneSphere) {
    for memory in &mut sphere.memories {
        for buoy in &sphere.buoys {
            let boost = buoy.boost_at(&memory.position);
            if boost > 0.0 {
                memory.activation = (memory.activation + boost * SWEEP_BOOST).min(1.0);
            }
        }
    }
}
```

`SWEEP_BOOST` = 0.05. This counteracts exponential decay for memories near active buoys,
creating persistent "hot zones" on the sphere surface.

### 2.5 Co-Activation Tracking

The buoy network detects co-occurring tool usage and creates tunnel connections:

```rust
/// Compute co-activation between buoys on different spheres.
/// Two buoys co-activate when their owning spheres are simultaneously Working
/// and the buoys are within TUNNEL_THRESHOLD (0.8 rad) angular distance.
pub fn compute_co_activation(
    buoys_a: &[Buoy],
    buoys_b: &[Buoy],
) -> Vec<CoActivation> {
    let mut results = vec![];
    for a in buoys_a {
        for b in buoys_b {
            let dist = a.position.angular_distance(&b.position);
            if dist < TUNNEL_THRESHOLD {
                results.push(CoActivation {
                    buoy_a: a.label.clone(),
                    buoy_b: b.label.clone(),
                    overlap: 1.0 - (dist / TUNNEL_THRESHOLD),
                });
            }
        }
    }
    results
}
```

### 2.6 Tests (10 target)

- `boost_at()`: maximum at buoy center
- `boost_at()`: zero outside influence radius
- `boost_at()`: Gaussian decay within radius
- `drift_toward()`: position moves toward target
- `drift_toward()`: high activation count slows drift
- `drift_home()`: position moves toward original home
- Sweep boost: memory near buoy gains activation
- Sweep boost: memory far from buoy unchanged
- Co-activation: buoys within threshold produce overlap > 0
- Co-activation: buoys beyond threshold produce no overlap

## 3. m21_memory_manager (~200 LOC)

### 3.1 Memory Recording

```rust
/// Record a new memory on the sphere surface.
/// Returns the assigned memory ID.
pub fn record_memory(sphere: &mut PaneSphere, tool_name: &str, summary: &str) -> u64 {
    let id = sphere.next_memory_id;
    sphere.next_memory_id += 1;

    // Semantic phase injection (NA-1): tool category determines angular region
    let theta = semantic_phase_for_tool(tool_name);
    let phi = pseudo_random_phi(id); // Distribute within the angular band
    let position = Point3D::from_spherical(theta, phi);

    let memory = SphereMemory {
        id,
        position,
        activation: 1.0,              // Start fully active
        tool_name: truncate_string(tool_name, LAST_TOOL_MAX_CHARS), // 128 char cap
        summary: truncate_string(summary, 256),
        timestamp: now_secs(),
        confidence: 1.0,
    };

    sphere.memories.push(memory);

    // Update has_worked flag (monotonic — never reverts to false)
    sphere.has_worked = true;

    // Track first memory timestamp (NA-27)
    if sphere.first_memory_at.is_none() {
        sphere.first_memory_at = Some(now_secs());
    }

    // Update buoy activations
    for buoy in &mut sphere.buoys {
        let dist = buoy.position.angular_distance(&position);
        if dist < buoy.influence_radius {
            buoy.activation_count += 1;
            buoy.drift_toward(&position);
        }
    }

    id
}
```

### 3.2 Semantic Phase Injection (NA-1)

Tool categories map to angular regions on the sphere, creating a spatial organization
of memory by function:

```rust
/// Map tool name to a phase region (theta angle in radians).
fn semantic_phase_for_tool(tool_name: &str) -> f64 {
    match tool_name.to_lowercase().as_str() {
        // Read tools: theta = 0
        s if s.contains("read") || s.contains("glob") || s.contains("grep")
            || s.contains("cat") || s.contains("search") => 0.0,
        // Write tools: theta = PI/2
        s if s.contains("write") || s.contains("edit") || s.contains("create")
            || s.contains("mv") || s.contains("cp") => FRAC_PI_2,
        // Execute tools: theta = PI
        s if s.contains("bash") || s.contains("exec") || s.contains("run")
            || s.contains("cargo") || s.contains("test") => PI,
        // Communication tools: theta = 3PI/2
        s if s.contains("message") || s.contains("send") || s.contains("post")
            || s.contains("curl") || s.contains("fetch") => 3.0 * FRAC_PI_2,
        // Default: random region
        _ => {
            let hash = tool_name.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(u64::from(b)));
            (hash % 628) as f64 / 100.0 // [0, 2PI)
        }
    }
}
```

The nudge strength is controlled by `SEMANTIC_NUDGE_STRENGTH` (0.02), which is gentle
enough to not override Kuramoto coupling dynamics but sufficient to create spatial locality.

### 3.3 Memory Decay

```rust
/// Decay all memories by DECAY_PER_STEP.
/// Called once per sphere step (every tick).
pub fn decay_memories(sphere: &mut PaneSphere) {
    for memory in &mut sphere.memories {
        memory.activation *= DECAY_PER_STEP; // 0.995
    }
}
```

Half-life at `DECAY_PER_STEP = 0.995`:
```
0.995^n = 0.5  =>  n = ln(0.5) / ln(0.995) = 138.3 steps = ~11.5 minutes
```

A memory drops below `ACTIVATION_THRESHOLD` (0.3) after:
```
0.995^n = 0.3  =>  n = ln(0.3) / ln(0.995) = 240.5 steps = ~20 minutes
```

### 3.4 Amortised Batch Pruning

Pruning is amortised to avoid per-tick O(N log N) sorting:

```rust
/// Prune memories if count exceeds MEMORY_MAX_COUNT + 50.
/// Keeps MEMORY_MAX_COUNT memories, removing those with lowest activation.
/// Triggered every MEMORY_PRUNE_INTERVAL (200) steps or when count threshold hit.
pub fn prune_if_needed(sphere: &mut PaneSphere) {
    if sphere.memories.len() <= MEMORY_MAX_COUNT + 50 {
        return; // Not yet at threshold
    }
    if sphere.total_steps % MEMORY_PRUNE_INTERVAL != 0
        && sphere.memories.len() <= MEMORY_MAX_COUNT + 50
    {
        return; // Not at scheduled interval
    }

    // Sort by activation ascending (lowest first)
    sphere.memories.sort_by(|a, b|
        a.activation.partial_cmp(&b.activation).unwrap_or(std::cmp::Ordering::Equal)
    );

    // Remove lowest until at MEMORY_MAX_COUNT
    let remove_count = sphere.memories.len().saturating_sub(MEMORY_MAX_COUNT);
    sphere.memories.drain(0..remove_count);

    // Update temporal marker (NA-27)
    sphere.last_prune_at = Some(now_secs());
}
```

### 3.5 ID Reconciliation on Restore (I-3 Fix)

When restoring from a snapshot, `next_memory_id` must be recomputed from existing memories
to prevent ID collisions:

```rust
/// Reconcile next_memory_id after snapshot restore.
/// Sets it to max(existing IDs) + 1 to prevent collisions.
pub fn reconcile_memory_ids(sphere: &mut PaneSphere) {
    sphere.next_memory_id = sphere.memories.iter()
        .map(|m| m.id)
        .max()
        .map_or(0, |max_id| max_id + 1);
}
```

This fix addresses v1 bug I-3 where `next_memory_id` was not recomputed after snapshot
restore, causing duplicate IDs and data corruption.

### 3.6 Memory Recall

```rust
/// Recall memories from a sphere, optionally filtered by phase region or buoy label.
/// Returns references sorted by activation (highest first).
pub fn recall(
    sphere: &PaneSphere,
    near_phase: Option<f64>,
    near_buoy: Option<&str>,
    limit: usize,
) -> Vec<&SphereMemory> {
    let mut candidates: Vec<&SphereMemory> = sphere.memories.iter()
        .filter(|m| m.activation >= ACTIVATION_THRESHOLD)
        .filter(|m| {
            if let Some(phase) = near_phase {
                let mem_phase = m.position.x.atan2(m.position.y).rem_euclid(TAU);
                let diff = (mem_phase - phase).abs().rem_euclid(TAU);
                diff.min(TAU - diff) < FRAC_PI_2  // Within PI/2 of target phase
            } else {
                true
            }
        })
        .filter(|m| {
            if let Some(label) = near_buoy {
                sphere.buoys.iter().any(|b|
                    b.label == label
                    && b.position.angular_distance(&m.position) < b.influence_radius
                )
            } else {
                true
            }
        })
        .collect();

    candidates.sort_by(|a, b| b.activation.partial_cmp(&a.activation).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(limit);
    candidates
}
```

### 3.7 Narrative Generation (NA-19)

```rust
/// Generate a narrative summary of the sphere's memory history.
/// Groups memories by tool category and temporal window.
pub fn narrative(sphere: &PaneSphere) -> String {
    if sphere.memories.is_empty() {
        return "No memories recorded yet.".into();
    }

    let active: Vec<&SphereMemory> = sphere.memories.iter()
        .filter(|m| m.activation >= ACTIVATION_THRESHOLD)
        .collect();

    let total = sphere.memories.len();
    let active_count = active.len();

    // Group by tool category
    let mut categories: HashMap<String, usize> = HashMap::new();
    for m in &active {
        let cat = tool_category(&m.tool_name);
        *categories.entry(cat).or_default() += 1;
    }

    format!(
        "{} of {} memories active. Primary focus: {}. Most recent: {}",
        active_count, total,
        categories.iter().max_by_key(|(_, &v)| v).map_or("none", |(k, _)| k),
        active.first().map_or("none", |m| &m.tool_name),
    )
}
```

### 3.8 Tests (10 target)

- `record_memory()`: ID increments sequentially
- `record_memory()`: activation starts at 1.0
- `record_memory()`: sets `has_worked = true` (monotonic)
- `record_memory()`: updates `first_memory_at` once
- `decay_memories()`: activation decreases by `DECAY_PER_STEP`
- `prune_if_needed()`: triggers at `MEMORY_MAX_COUNT + 50`, leaves `MEMORY_MAX_COUNT`
- `prune_if_needed()`: removes lowest activation first
- `reconcile_memory_ids()`: sets `next_memory_id` to max + 1
- `recall()`: `near_phase` filter returns only memories in angular range
- `recall()`: respects `ACTIVATION_THRESHOLD` (0.3)

## Summary

| Module | LOC Target | Key Responsibility | Tests |
|--------|-----------|-------------------|-------|
| m19_hebbian_stdp | 250 | LTP/LTD weight updates, burst detection, newcomer boost | 15 |
| m20_buoy_network | 200 | Spatial markers, drift dynamics, sweep boost, co-activation | 10 |
| m21_memory_manager | 200 | Record, decay, prune, recall, narrative, ID reconciliation | 10 |
| **L5 Total** | **650** | | **35** |

## Weight Evolution Timeline

Tracking a single connection over a typical session:

```
t=0     w=0.30  Registration (default weight)
t=1     w=0.30  Both idle, LTD skipped (steps < 2)
t=5     w=0.27  Sphere A idle, B idle: 5 × LTD = -0.010
t=10    w=0.24  Still idle: 5 × LTD = -0.010
t=15    w=0.21  Still idle: 5 × LTD = -0.010
t=20    w=0.18  Still idle: 5 × LTD = -0.010
t=25    w=0.15  Floor reached. LTD no longer decreases.
t=30    w=0.16  Both start working: LTP = +0.01
t=31    w=0.17  Still working: LTP = +0.01
t=32    w=0.18  Still working: LTP = +0.01
t=33    w=0.24  Third sphere joins → burst: LTP = +0.01 × 3.0 = +0.03 × 2 ticks
t=40    w=0.45  Burst continues: 7 ticks × 0.03 = +0.21
t=50    w=0.65  Crystallization zone reached
t=100   w=0.80  Deep collaboration bond
```

## POVM Bimodal Discovery (Session 039)

Analysis of 2,425 POVM pathways revealed:
- **90%** clustered at default weight (0.15-0.30) — pairs that never co-worked
- **2.8%** crystallized at high weight (0.65-1.00) — persistent collaborative partners
- **7.2%** in transition (0.30-0.65) — intermittent co-workers

This distribution is a direct consequence of the LTP/LTD asymmetry (5:1 ratio)
combined with the weight floor. The floor prevents total disconnection, while the
asymmetry ensures that only sustained co-activation produces high weights.

The bimodal shape implies a phase transition in the learning dynamics: once a connection
crosses approximately w=0.4, the w^2 amplification in coupling (see L4 spec) makes it
strong enough that synchronized phases reinforce co-working, creating a positive feedback
loop that drives the weight to crystallization.

Source: `[[Session 039 — Architectural Schematics and Refactor Safety]]` (POVM bimodal distribution diagram)

## Anti-Patterns

- **AP-1:** LTP without warmup guard — learning from stale snapshot state corrupts weights
- **AP-2:** Unbounded memory Vec — O(N) scan per tick. Use amortised batch prune at +50 threshold.
- **AP-3:** `next_memory_id` from 0 after restore — creates duplicate IDs. Always reconcile (I-3 fix).
- **AP-4:** UTF-8 byte-slice panic — `&s[..40]` on multi-byte strings panics. Use `chars().take(40)` (C1 fix).
- **AP-5:** Symmetric LTP/LTD rates — equal rates cause all weights to converge to a single value.
  The 5:1 asymmetry is essential for bimodal distribution.
- **AP-6:** LTP without opt-out check — violates sphere autonomy (NA-34). Always check `opt_out_hebbian`.
- **AP-7:** Memory pruning every tick — O(N log N) sort is expensive. Only trigger at +50 threshold.

## Related

- `KURAMOTO_FIELD_SPEC.md` Sections 6-7 — Full Hebbian STDP and memory field mathematics
- `layers/L4_COUPLING_SPEC.md` — Weight matrix that m19 modifies
- `layers/L3_FIELD_SPEC.md` Section 1 — PaneSphere struct that owns memories and buoys
- `CONSENT_SPEC.md` Section 3 — Opt-out flag registry
- `[[Session 039 — What I Learned]]` — Bimodal weight distribution discovery
- `[[Session 039 — Architectural Schematics and Refactor Safety]]` — Memory data flow, risk hotspots
- `[[Vortex Sphere Brain-Body Architecture]]` — Theoretical foundations
- `MASTERPLAN.md` Phase V3.2 — Inhabitation plan (requires Hebbian pathway formation)
