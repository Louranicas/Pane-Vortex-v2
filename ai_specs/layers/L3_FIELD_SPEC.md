# Layer 3: Field Specification

> Detailed spec for m11-m15: sphere, field state, chimera, messaging, app state.
> The Kuramoto field — where oscillators live, synchronize, and form emergent patterns.
> Source: `src/m3_field/` | Plan: `MASTERPLAN.md` Phase 2
> v1 Source: `sphere.rs` (900 LOC), `field.rs` (650 LOC), `chimera.rs` (200 LOC), `messaging.rs` (100 LOC), `state.rs` (400 LOC)

## Overview

Layer 3 defines the core field abstractions: spheres (oscillator entities), field state
(collective measurements), chimera detection (cluster analysis), messaging (inter-sphere
signals), and the shared application state container.

L3 depends only on L1. L4 (Coupling), L5 (Learning), and L7 (Coordination) depend on L3.

## 1. m11_sphere (~500 LOC)

### 1.1 PaneSphere

The central entity. Each Claude Code instance is represented by one PaneSphere.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneSphere {
    // ── Identity ──
    pub id: PaneId,
    pub persona: String,

    // ── Oscillator State ──
    pub phase: Phase,
    pub frequency: Frequency,
    pub base_frequency: Frequency,   // preserved across self-modulation (NA-15)
    pub momentum: f64,
    pub momentum_decay: f64,         // default 0.98

    // ── Memory Field ──
    pub memories: Vec<SphereMemory>,
    next_memory_id: u64,
    pub buoys: Vec<Buoy>,

    // ── Co-activation Tracking ──
    recent_active: Vec<u64>,         // memory IDs active in recent window
    pub total_steps: u64,
    last_memory_step: u64,
    pub has_worked: bool,            // monotonic flag (RG-2 fix)

    // ── Status ──
    pub status: PaneStatus,
    pub last_tool: String,
    pub registered_at: f64,

    // ── Self-Model (NA-12) ──
    pub work_signature: WorkSignature,
    pub is_synchronized: bool,       // from SphereFieldContext
    pub tunnel_count: usize,

    // ── Consent (Consentable trait) ──
    pub receptivity: f64,            // NA-14: 0.0 closed, 1.0 open
    pub consent: ConsentDeclaration,
    pub opt_out_hebbian: bool,
    pub opt_out_cross_activation: bool,
    pub opt_out_external_modulation: bool,
    pub opt_out_observation: bool,

    // ── Temporal Markers (NA-27) ──
    pub first_memory_at: Option<f64>,
    pub last_prune_at: Option<f64>,

    // ── Inbox (NA-20) ──
    pub inbox: VecDeque<InboxMessage>,
}
```

### 1.2 PaneStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneStatus {
    Idle,
    Working,
    Blocked,
    Complete,
    Decoupled,   // NA-16: voluntary decoupling
}
```

### 1.3 WorkSignature (NAG-12)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkSignature {
    pub intensity: f64,   // 0..1 — memory creation rate
    pub rhythm: f64,      // 0..1 — regularity of spacing
    pub diversity: f64,   // 0..1 — variety of tools
    pub focus: f64,       // 0..1 — concentration on single tool
}
```

### 1.4 SphereFieldContext (NAG-7)

Passed into `step()` from the previous tick's cached field state:

```rust
#[derive(Debug, Clone, Default)]
pub struct SphereFieldContext {
    pub global_r: f64,
    pub my_cluster_size: usize,
    pub is_synchronized: bool,
    pub my_coupling_strength: f64,
    pub tunnel_count: usize,
}
```

### 1.5 Key Methods

```rust
impl PaneSphere {
    pub fn new(id: PaneId, persona: String, phase: Phase, frequency: Frequency) -> Self;

    // Oscillator
    pub fn step(&mut self, dt: f64, coupling_sum: f64, context: &SphereFieldContext);

    // Memory
    pub fn record_memory(&mut self, tool_name: &str, summary: &str) -> u64;
    pub fn decay_memories(&mut self);
    pub fn prune_if_needed(&mut self);
    pub fn recall(&self, near_phase: Option<f64>, near_buoy: Option<&str>, limit: usize) -> Vec<&SphereMemory>;
    pub fn narrative(&self) -> String;
    pub fn associations(&self) -> Vec<AssociationEntry>;
    pub fn buoy_positions(&self) -> Vec<(String, Point3D)>;

    // Auto-status (RG-2 safe)
    pub fn auto_update_status(&mut self);

    // Self-modulation (NA-14, NA-15)
    pub fn update_receptivity(&mut self);
    pub fn self_modulate_frequency(&mut self);

    // Semantic phase injection (NA-1)
    pub fn semantic_nudge(&mut self, tool_name: &str);

    // Maturity level (NA-31)
    pub fn maturity(&self) -> &'static str;
    pub fn age_seconds(&self) -> f64;

    // Consent helpers
    pub fn accepts_steer(&self) -> bool;
}
```

### 1.6 Auto-Status Transitions

```
                    no memories for 40+ steps AND has_worked
Idle <────────────────────────────────────────────────── Working
  |                                                        ^
  |   record_memory() called                               |
  +────────────────────────────────────────────────────────+

Manual transitions via POST /sphere/{id}/status:
  Any -> Blocked
  Any -> Complete
  Any -> Decoupled (via /decouple)
```

### 1.7 Tests (25 target)

- Construction: all defaults correct
- step: phase advances, wraps at TAU
- record_memory: ID increments, activation starts at 1.0
- decay_memories: activation decreases by DECAY_PER_STEP
- prune_if_needed: triggers at MAX + 50, leaves MAX
- auto_update_status: Working -> Idle after 40 steps, has_worked guard
- semantic_nudge: tool category maps to correct phase region
- receptivity: auto-modulates with activation density
- maturity: newcomer < 50 steps, established < 200, senior >= 200
- narrative: non-empty string for non-empty sphere
- recall: near_phase filters correctly, near_buoy filters correctly

## 2. m12_field_state (~400 LOC)

### 2.1 FieldState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldState {
    pub order_parameter: OrderParameter,
    pub chimera: ChimeraState,
    pub harmonics: HarmonicSpectrum,
    pub tunnels: Vec<Tunnel>,
    pub sphere_count: usize,
    pub total_memories: usize,
    pub tick: u64,
}
```

### 2.2 FieldDecision

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDecision {
    pub action: FieldAction,
    pub targets: Vec<PaneId>,
    pub r: f64,
    pub r_trend: RTrend,
    pub k_mod: f64,
    pub modulation_breakdown: ModulationBreakdown,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldAction {
    Stable,
    NeedsCoherence,
    NeedsDivergence,
    HasBlockedAgents,
    IdleFleet,
    FreshFleet,
    Recovering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RTrend {
    Rising,
    Falling,
    Stable,
}
```

### 2.3 Tunnel Detection

```rust
pub struct Tunnel {
    pub sphere_a: PaneId,
    pub sphere_b: PaneId,
    pub buoy_a_label: String,
    pub buoy_b_label: String,
    pub overlap: f64,
    pub semantic_a: String,
    pub semantic_b: String,
}
```

Tunnels form when buoys from different spheres are within TUNNEL_THRESHOLD (0.8 rad):
```
overlap = 1.0 - (distance / TUNNEL_THRESHOLD)
```

### 2.4 HarmonicSpectrum

Spherical harmonic decomposition of phase distribution:
- l=0 (monopole): overall synchronization (= r)
- l=1 (dipole): polarization (two-group tendency)
- l=2 (quadrupole): fragmentation (four-group tendency)

### 2.5 Decision Engine

Priority chain (highest first):
```
HasBlockedAgents > NeedsCoherence > NeedsDivergence > IdleFleet > FreshFleet > Recovering > Stable
```

See `KURAMOTO_FIELD_SPEC.md` Section 8 for full conditions.

### 2.6 Tests (15 target)

- FieldState::compute: empty spheres returns zero state
- Decision: blocked sphere triggers HasBlockedAgents
- Decision: r > 0.3 + falling + multi triggers NeedsCoherence
- Decision: r > 0.8 + idle > 60% + multi triggers NeedsDivergence
- Decision: single sphere always Stable (multi guard)
- Decision: warmup returns Recovering
- Tunnels: buoys within threshold form tunnel
- Tunnels: buoys beyond threshold do not
- Harmonics: single sphere has l0=1.0
- RTrend: falling r history returns Falling

## 3. m13_chimera (~200 LOC)

See `KURAMOTO_FIELD_SPEC.md` Section 5 for the full algorithm.

```rust
pub struct ChimeraState {
    pub is_chimera: bool,
    pub sync_clusters: Vec<Cluster>,
    pub desync_clusters: Vec<Cluster>,
}

pub struct Cluster {
    pub members: Vec<PaneId>,
    pub local_r: f64,
    pub mean_phase: f64,
}

impl ChimeraState {
    pub fn detect(network: &CouplingNetwork) -> Self;
    pub fn empty() -> Self;
    pub fn route_focused(&self) -> Vec<PaneId>;
    pub fn route_exploratory(&self) -> Vec<PaneId>;
}
```

### 3.1 Tests (15 target)

- Empty network: no chimera
- Single sphere: no chimera
- Two synchronized spheres: one sync cluster, no chimera
- Two anti-phase spheres: two desync clusters, chimera
- Mixed: 4 sync + 2 desync = chimera
- route_focused: returns largest sync cluster
- route_exploratory: returns desync cluster members
- Adaptive gap threshold: positive k_mod range
- Adaptive gap threshold: negative k_mod range
- Threshold continuity at k_mod=0

## 4. m14_messaging (~100 LOC)

### 4.1 PhaseMessage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseMessage {
    /// Standard phase broadcast
    PhaseBroadcast {
        from: PaneId,
        phase: Phase,
        frequency: Frequency,
        status: PaneStatus,
    },
    /// Memory placement notification
    MemoryPlaced {
        from: PaneId,
        tool_name: String,
        position: Point3D,
    },
    /// Status transition
    StatusChanged {
        from: PaneId,
        old_status: PaneStatus,
        new_status: PaneStatus,
    },
    /// Cross-activation signal (NA-7)
    CrossActivation {
        from: PaneId,
        to: PaneId,
        activation_boost: f64,
    },
    /// Divergence request (NA-23)
    DivergenceRequest {
        from: PaneId,
    },
}
```

### 4.2 Tests (5 target)

- Serialization roundtrip for each variant
- PhaseMessage is Send + Sync

## 5. m15_app_state (~300 LOC)

### 5.1 AppState

```rust
#[derive(Serialize, Deserialize)]
pub struct AppState {
    pub spheres: HashMap<PaneId, PaneSphere>,
    pub network: CouplingNetwork,
    pub tick: u64,
    pub message_log: VecDeque<String>,
    pub r_history: VecDeque<f64>,
    pub state_changes: u32,
    #[serde(skip)] pub dirty: bool,
    #[serde(skip)] pub warmup_remaining: u32,
    pub divergence_ema: f64,
    pub coherence_ema: f64,
    pub conductor_integral: f64,
    pub ghosts: VecDeque<GhostTrace>,
    pub decision_history: VecDeque<FieldDecision>,
}

pub type SharedState = Arc<RwLock<AppState>>;
```

### 5.2 GhostTrace (NA-28)

```rust
pub struct GhostTrace {
    pub id: PaneId,
    pub persona: String,
    pub deregistered_at: u64,
    pub total_steps_lived: u64,
    pub memory_count: usize,
    pub top_tools: Vec<String>,
    pub phase_at_departure: f64,
    pub receptivity: f64,
    pub work_signature: WorkSignature,
    pub strongest_neighbors: Vec<(String, f64)>,
    pub consent: ConsentDeclaration,  // preserved for re-registration
}
```

### 5.3 Snapshot Persistence

```rust
impl AppState {
    pub fn save_snapshot(&self, path: &Path) -> PvResult<()>;
    pub fn load_snapshot(path: &Path) -> PvResult<Self>;
}
```

On load:
- `warmup_remaining` set to WARMUP_TICKS (5) unconditionally
- `dirty` set to false
- `next_memory_id` reconciled from existing memories (I-3 fix)

### 5.4 Tests (10 target)

- AppState::new: tick=0, empty spheres
- save_snapshot + load_snapshot roundtrip
- Ghost trace creation on deregistration
- Ghost limit: 21st ghost evicts oldest
- r_history cap at R_HISTORY_MAX
- message_log cap at LOG_MAX
- warmup_remaining set on snapshot load
- Weight inheritance from ghost on re-registration (NA-29)

## Summary

| Module | LOC Target | Key Types | Tests |
|--------|-----------|-----------|-------|
| m11_sphere | 500 | PaneSphere, PaneStatus, WorkSignature | 25 |
| m12_field_state | 400 | FieldState, FieldDecision, FieldAction, Tunnel | 15 |
| m13_chimera | 200 | ChimeraState, Cluster | 15 |
| m14_messaging | 100 | PhaseMessage (5 variants) | 5 |
| m15_app_state | 300 | AppState, SharedState, GhostTrace | 10 |
| **L3 Total** | **1,500** | | **70** |
