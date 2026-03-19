---
title: "Layer 1: Foundation — Module Documentation"
date: 2026-03-19
tags: [modules, foundation, L1, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
layer: L1
modules: [m01, m02, m03, m04, m05, m06]
---

# Layer 1: Foundation (m01-m06)

> Core types, error handling, configuration, constants, traits, and validation.
> **Zero dependencies on other layers.** Every other module in the system depends on L1.
> **Target LOC:** ~1,100 | **Target tests:** 60+

---

## Overview

L1 is the bedrock of PV V2. It provides:
- All core domain types (PaneId, Phase, Frequency, SphereMemory, Buoy)
- A unified error type (PvError) with 6 categories
- Configuration loading via Figment (TOML + env vars)
- Named constants for all magic numbers
- Core traits that define the oscillator and consent contracts
- Input validation functions used at every system boundary

**Implementation order:** m04 (constants, no deps) -> m01 (types) -> m02 (errors) -> m06 (validation) -> m05 (traits) -> m03 (config)

---

## m01 — Core Types

**Source:** `src/m1_foundation/m01_core_types.rs`
**LOC Target:** ~300
**Depends on:** nothing

### Design Decisions

All core types are defined here to minimize fan-in. In V1, `types.rs` had fan-in=9 (9 modules depended on it), making it the highest-risk file for modifications. V2 preserves this centralization but with newtypes and proper validation.

### Types to Implement

#### PaneId — Sphere Identity

```rust
/// Validated sphere identifier. Max 128 characters, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PaneId(String);

impl PaneId {
    pub fn new(s: impl Into<String>) -> PvResult<Self> { ... }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

#### Phase — Oscillator Phase

```rust
/// Phase in [0, TAU). Always wrapped via rem_euclid(TAU).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Phase(f64);

impl Phase {
    pub fn new(value: f64) -> Self { Self(value.rem_euclid(TAU)) }
    pub fn value(&self) -> f64 { self.0 }
    pub fn wrap(value: f64) -> Self { Self(value.rem_euclid(TAU)) }
    pub fn diff(&self, other: &Phase) -> f64 { ... }  // Shortest angular distance
}
```

CRITICAL: Phase wrapping via `.rem_euclid(TAU)` is the most fundamental invariant in the system (pattern P01). Every arithmetic operation on phases must produce a wrapped result.

#### Frequency — Oscillator Frequency

```rust
/// Natural frequency, clamped to [0.001, 10.0].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Frequency(f64);

impl Frequency {
    pub fn new(value: f64) -> Self { Self(value.clamp(FREQ_MIN, FREQ_MAX)) }
    pub fn value(&self) -> f64 { self.0 }
}
```

#### SphereStatus — Activity State

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SphereStatus {
    Idle,
    Working,
    Complete,
    Blocked,
}
```

#### Point3D — Buoy Position

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

#### SphereMemory — Tool Memory

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereMemory {
    pub id: u64,
    pub tool: String,     // Truncated to 128 chars (pattern P13)
    pub phase: Phase,
    pub timestamp: u64,   // Step count when recorded
    pub activation: f64,  // Decays over time
}
```

#### Buoy — Navigational Marker

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buoy {
    pub position: Point3D,
    pub activation: f64,
    pub label: String,
    pub decay_rate: f64,
}
```

#### WorkSignature — Activity Pattern

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkSignature {
    pub tool_histogram: HashMap<String, u32>,
    pub burst_count: u32,
    pub total_tool_uses: u64,
}
```

### Tests (15+)

- PaneId: empty string rejected, 128-char limit enforced, equality, hash
- Phase: wrapping at TAU, wrapping negative values, diff calculation
- Frequency: clamping at bounds, NaN handling
- SphereStatus: serialization round-trip
- Point3D: basic construction
- SphereMemory: tool truncation

---

## m02 — Error Handling

**Source:** `src/m1_foundation/m02_error_handling.rs`
**LOC Target:** ~200
**Depends on:** nothing

### Design Decisions

V1 used string errors and `.map_err(|e| format!(...))` everywhere. V2 defines a single `PvError` enum with 6 categories, each containing a sub-enum. All errors propagate via `?` operator.

### Types to Implement

```rust
/// Unified error type for all PV V2 operations.
#[derive(Debug)]
pub enum PvError {
    Field(FieldError),
    Bridge(BridgeError),
    Bus(BusError),
    Persistence(PersistenceError),
    Validation(ValidationError),
    Config(ConfigError),
}

pub type PvResult<T> = Result<T, PvError>;
```

See [ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md) for full variant list.

### Conversion Implementations

```rust
impl From<std::io::Error> for PvError { ... }
impl From<serde_json::Error> for PvError { ... }
impl From<rusqlite::Error> for PvError { ... }  // behind #[cfg(feature = "persistence")]
impl From<figment::Error> for PvError { ... }
```

### Display Implementation

Each variant produces a human-readable message suitable for API error responses:
```rust
impl std::fmt::Display for PvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PvError::Field(e) => write!(f, "field error: {e}"),
            PvError::Bridge(e) => write!(f, "bridge error: {e}"),
            // ...
        }
    }
}
```

### Tests (10+)

- From conversions: io::Error, serde, rusqlite
- Display output for each variant
- Error categorization

---

## m03 — Config

**Source:** `src/m1_foundation/m03_config.rs`
**LOC Target:** ~250
**Depends on:** m02

### Design Decisions

V1 hardcoded constants and used environment variables ad-hoc. V2 uses Figment to load config from: `default.toml` -> `production.toml` -> environment variables (`PV2_` prefix). This gives a clean hierarchy with compile-time defaults as fallback.

### Types to Implement

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PvConfig {
    pub server: ServerConfig,
    pub field: FieldConfig,
    pub sphere: SphereConfig,
    pub coupling: CouplingConfig,
    pub learning: LearningConfig,
    pub bridges: BridgeConfig,
    pub conductor: ConductorConfig,
    pub ipc: IpcConfig,
    pub persistence: PersistenceConfig,
    pub governance: GovernanceConfig,
}
```

See `config/default.toml` for all fields and defaults (11 sections, 87 lines).

### Config Loading

```rust
impl PvConfig {
    pub fn load() -> PvResult<Self> {
        Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Toml::file("config/production.toml"))
            .merge(Env::prefixed("PV2_").split("__"))
            .extract()
            .map_err(PvError::from)
    }

    pub fn validate(&self) -> PvResult<()> { ... }
}
```

### Validation

Config validation checks:
- Port in valid range (1024-65535)
- Tick interval > 0
- r_target in [0.3, 0.99]
- Sphere cap > 0
- Socket path exists (parent directory)
- k_mod budget min < max

### Tests (10+)

- Load from default.toml
- Env var override
- Invalid values produce ConfigError
- Missing file falls back to defaults
- Validation catches out-of-range values

---

## m04 — Constants

**Source:** `src/m1_foundation/m04_constants.rs`
**LOC Target:** ~100
**Depends on:** nothing

### Design Decisions

V1 scattered constants across 8 source files. V2 centralizes all magic numbers in one file. Constants are `pub const` rather than config values when they represent mathematical or protocol invariants that should not be user-configurable.

### Constants to Define

```rust
use std::f64::consts::TAU;

// Field dynamics
pub const TICK_INTERVAL_SECS: u64 = 5;
pub const COUPLING_STEPS_PER_TICK: usize = 15;
pub const KURAMOTO_DT: f64 = 0.01;
pub const R_TARGET_DEFAULT: f64 = 0.93;

// Thresholds
pub const R_HIGH_THRESHOLD: f64 = 0.8;
pub const R_LOW_THRESHOLD: f64 = 0.3;
pub const R_FALLING_THRESHOLD: f64 = -0.03;
pub const IDLE_RATIO_THRESHOLD: f64 = 0.6;
pub const PHASE_GAP_THRESHOLD: f64 = TAU / 6.0;  // pi/3
pub const SYNC_THRESHOLD: f64 = 0.5;
pub const TUNNEL_THRESHOLD: f64 = 0.8;

// Learning
pub const HEBBIAN_LTP: f64 = 0.01;
pub const HEBBIAN_LTD: f64 = 0.002;
pub const BURST_MULTIPLIER: f64 = 3.0;
pub const NEWCOMER_MULTIPLIER: f64 = 2.0;
pub const NEWCOMER_STEPS: u64 = 50;
pub const WEIGHT_FLOOR: f64 = 0.15;

// Coupling
pub const DEFAULT_WEIGHT: f64 = 0.18;
pub const WEIGHT_EXPONENT: f64 = 2.0;
pub const AUTO_SCALE_K_PERIOD: u64 = 20;
pub const AUTO_SCALE_K_MULTIPLIER: f64 = 0.5;
pub const FREQ_MIN: f64 = 0.001;
pub const FREQ_MAX: f64 = 10.0;
pub const STRENGTH_MIN: f64 = 0.0;
pub const STRENGTH_MAX: f64 = 2.0;

// Limits
pub const SPHERE_CAP: usize = 200;
pub const MEMORY_MAX_COUNT: usize = 500;
pub const MEMORY_PRUNE_INTERVAL: u64 = 200;
pub const GHOST_MAX_COUNT: usize = 20;
pub const R_HISTORY_MAX: usize = 60;
pub const SNAPSHOT_INTERVAL: u64 = 60;
pub const LAST_TOOL_MAX_CHARS: usize = 128;

// Bridges
pub const K_MOD_MIN: f64 = -0.5;
pub const K_MOD_MAX: f64 = 1.5;
pub const K_MOD_BUDGET_MIN: f64 = 0.85;
pub const K_MOD_BUDGET_MAX: f64 = 1.15;

// Sphere maturity
pub const DECAY_PER_STEP: f64 = 0.995;
pub const SWEEP_BOOST: f64 = 0.05;
pub const ACTIVATION_THRESHOLD: f64 = 0.3;
pub const SEMANTIC_NUDGE_STRENGTH: f64 = 0.02;
```

### Tests (5+)

- Compile-time assertions (const_assert where applicable)
- TAU correctness
- Threshold ordering (LOW < HIGH, BUDGET_MIN < BUDGET_MAX)

---

## m05 — Traits

**Source:** `src/m1_foundation/m05_traits.rs`
**LOC Target:** ~150
**Depends on:** m01, m02

### Design Decisions

V1 had no trait abstractions — all behavior was directly on concrete types. V2 defines 4 core traits that establish the contracts for the system. These enable testing with mock implementations and future extensibility.

### Traits to Define

```rust
/// An entity that oscillates with phase and frequency.
pub trait Oscillator {
    fn phase(&self) -> Phase;
    fn frequency(&self) -> Frequency;
    fn step(&mut self, dt: f64, influence: f64);
}

/// An entity that participates in Hebbian learning.
pub trait Learnable {
    fn ltp(&mut self, other: &Self, rate: f64);
    fn ltd(&mut self, other: &Self, rate: f64);
    fn is_active(&self) -> bool;
}

/// An external service bridge.
pub trait Bridgeable: Send + Sync {
    fn poll(&self) -> impl std::future::Future<Output = PvResult<()>> + Send;
    fn health(&self) -> bool;
    fn service_name(&self) -> &str;
}

/// An entity that can grant or deny consent.
pub trait Consentable {
    fn accepts_modulation(&self) -> bool;
    fn max_k_adj(&self) -> f64;
    fn consent_scale(&self) -> f64;
    fn accepts_observation(&self) -> bool;
    fn accepts_cascade(&self) -> bool;
}
```

### Tests (8+)

- Mock Oscillator implementation: phase wrapping, frequency clamping
- Mock Consentable: consent scale calculation, opt-out behavior
- Trait object creation and dispatch

---

## m06 — Validation

**Source:** `src/m1_foundation/m06_validation.rs`
**LOC Target:** ~100
**Depends on:** m01, m02, m04

### Design Decisions

Validation happens at system boundaries (API handlers, bus frame parsing, bridge responses). V2 centralizes all validation functions in one module so they are consistent and tested.

### Functions to Implement

```rust
/// Validate and wrap a phase value.
pub fn validate_phase(value: f64) -> PvResult<Phase> {
    is_finite_or_err(value, "phase")?;
    Ok(Phase::wrap(value))
}

/// Validate and clamp a frequency value.
pub fn validate_frequency(value: f64) -> PvResult<Frequency> {
    is_finite_or_err(value, "frequency")?;
    Ok(Frequency::new(value))
}

/// Validate sphere ID string.
pub fn validate_sphere_id(s: &str) -> PvResult<PaneId> {
    if s.is_empty() {
        return Err(PvError::Validation(ValidationError::SphereIdEmpty));
    }
    if s.len() > LAST_TOOL_MAX_CHARS {
        return Err(PvError::Validation(ValidationError::SphereIdTooLong(s.len())));
    }
    PaneId::new(s)
}

/// Validate string length, truncating if necessary.
pub fn validate_string_length(s: &str, max: usize) -> PvResult<String> {
    Ok(s.chars().take(max).collect())  // chars().take(), never byte slice (P13)
}

/// Validate a k_mod value is within budget.
pub fn validate_k_mod(value: f64) -> PvResult<f64> {
    is_finite_or_err(value, "k_mod")?;
    Ok(value.clamp(K_MOD_BUDGET_MIN, K_MOD_BUDGET_MAX))
}

/// Check that a float is finite (not NaN, not Infinity).
pub fn is_finite_or_err(value: f64, name: &str) -> PvResult<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PvError::Validation(ValidationError::NotFinite(name.to_string())))
    }
}
```

### Tests (12+)

- Phase: NaN, infinity, negative, > TAU
- Frequency: below min, above max, NaN
- Sphere ID: empty, too long, valid
- String truncation: ASCII, multi-byte UTF-8, emoji
- k_mod: below budget, above budget, NaN
- is_finite: NaN, +inf, -inf, normal values

---

## Cross-References

- **[CODE_MODULE_MAP.md](../CODE_MODULE_MAP.md)** — All 41 modules at a glance
- **[ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md)** — Full PvError variant list
- **[config/default.toml](../../config/default.toml)** — Config defaults matching m04 constants
- **[.claude/patterns.json](../../.claude/patterns.json)** — P01 (phase wrapping), P03 (error propagation), P08 (NaN guard), P10 (frequency clamp), P13 (chars not bytes)
- **V1 Reference:** `~/claude-code-workspace/pane-vortex/src/types.rs` (V1 types, fan-in=9)
