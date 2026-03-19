# Layer 1: Foundation Specification

> Detailed spec for m01-m06: types, errors, config, constants, traits, validation.
> The bedrock — every other module depends on L1. L1 has zero internal dependencies.
> Source: `src/m1_foundation/` | Plan: `MASTERPLAN.md` Phase 1
> v1 Source: `pane-vortex/src/types.rs` (177 LOC, 4 tests)

## Overview

Layer 1 provides the type system, error handling, configuration, constants, trait
definitions, and input validation for the entire codebase. No module in L1 imports
from any other layer. This makes L1 the stable foundation that all other layers build on.

## 1. m01_core_types (~300 LOC)

### 1.1 Spatial Types

```rust
/// 3D point on the unit sphere — embedding surface for memory placement
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self;
    pub fn north() -> Self;                        // (0, 0, 1)
    pub fn from_spherical(theta: f64, phi: f64) -> Self;
    pub fn dot(&self, other: &Self) -> f64;
    pub fn norm(&self) -> f64;
    pub fn normalized(&self) -> Self;
    pub fn angular_distance(&self, other: &Self) -> f64;
    pub fn arc_distance(&self, other: &Self) -> f64;  // alias for angular_distance
    pub fn slerp(&self, other: &Self, t: f64) -> Self; // spherical interpolation
}
```

### 1.2 Identity Types

```rust
/// Unique pane identifier — Zellij tab:pane or custom name
pub type PaneId = String;

/// Task identifier — UUID v4
pub type TaskId = String;

/// Phase in radians [0, TAU)
pub type Phase = f64;

/// Natural frequency in Hz [0.001, 10.0]
pub type Frequency = f64;
```

### 1.3 Memory Types

```rust
/// A memory placed on the sphere surface by a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphereMemory {
    pub id: u64,
    pub position: Point3D,
    pub activation: f64,      // [0.0, 1.0] — decays over time
    pub tool_name: String,
    pub summary: String,
    pub timestamp: f64,       // epoch seconds
    pub confidence: f64,      // [0.0, 1.0]
}

/// Hebbian buoy — learned spatial cluster on the sphere
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buoy {
    pub position: Point3D,
    pub home: Point3D,        // original position (drift anchor)
    pub activation_count: u64,
    pub influence_radius: f64,
    pub boost_multiplier: f64,
    pub learning_rate: f64,
    pub label: String,
}
```

### 1.4 Coupling Types

```rust
/// Connection between two pane-spheres
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: PaneId,
    pub to: PaneId,
    pub weight: f64,          // [0.0, 1.0]
    pub type_weight: f64,     // status-dependent modifier
}

/// Kuramoto order parameter
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderParameter {
    pub r: f64,               // [0.0, 1.0] synchronization
    pub psi: f64,             // [0, TAU) mean phase
}
```

### 1.5 Utility Functions

```rust
pub fn now_secs() -> f64;    // SystemTime since UNIX_EPOCH
```

### 1.6 Tests (15 target)

- Point3D: north is unit, from_spherical roundtrip, dot product, norm, normalized
- angular_distance: same point = 0, antipodal = PI, orthogonal = PI/2
- arc_distance: matches angular_distance
- slerp: t=0 returns start, t=1 returns end, midpoint on geodesic
- SphereMemory::new: activation starts at 1.0, timestamp is now
- Buoy: boost_at Gaussian shape, drift_toward moves position

## 2. m02_error_handling (~200 LOC)

### 2.1 Error Enum

```rust
/// Unified error type for all pane-vortex operations
#[derive(Debug)]
pub enum PvError {
    /// Invalid user input (phase, frequency, string length)
    InvalidInput(String),
    /// Sphere not found in registry
    SphereNotFound(String),
    /// Sphere already registered
    SphereAlreadyRegistered(String),
    /// Sphere cap reached
    SphereCapReached,
    /// Task not found
    TaskNotFound(String),
    /// Task already claimed
    TaskAlreadyClaimed(String),
    /// Cascade rejected by target
    CascadeRejected(String),
    /// Cascade depth limit exceeded
    CascadeDepthExceeded(u32),
    /// Consent violation
    ConsentViolation(String),
    /// Database error
    Database(String),
    /// Configuration error
    Config(String),
    /// Bridge communication error
    Bridge(String),
    /// IPC bus error
    Bus(String),
    /// Connection closed
    ConnectionClosed,
    /// Frame too large
    FrameTooLarge,
    /// IO error
    Io(std::io::Error),
    /// Serialization error
    Serde(serde_json::Error),
    /// Internal error (should never happen)
    Internal(String),
}

pub type PvResult<T> = Result<T, PvError>;
```

### 2.2 From Implementations

```rust
impl From<std::io::Error> for PvError;
impl From<serde_json::Error> for PvError;
impl From<rusqlite::Error> for PvError;     // cfg(feature = "persistence")
impl From<figment::Error> for PvError;
impl From<toml::de::Error> for PvError;

impl std::fmt::Display for PvError;
impl std::error::Error for PvError;
```

### 2.3 Axum Integration

```rust
impl axum::response::IntoResponse for PvError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            PvError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PvError::SphereNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            PvError::SphereAlreadyRegistered(_) => (StatusCode::CONFLICT, self.to_string()),
            PvError::SphereCapReached => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            PvError::ConsentViolation(_) => (StatusCode::FORBIDDEN, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
```

### 2.4 Tests (10 target)

- From<io::Error> conversion
- From<serde_json::Error> conversion
- Display formatting for each variant
- IntoResponse status codes (400, 404, 409, 429, 403, 500)

## 3. m03_config (~250 LOC)

### 3.1 Configuration Loading

Uses Figment for layered configuration:

```rust
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

pub fn load_config() -> PvResult<PvConfig> {
    let config: PvConfig = Figment::new()
        .merge(Toml::file("config/default.toml"))
        .merge(Toml::file("config/production.toml"))
        .merge(Env::prefixed("PV2_"))
        .extract()?;
    Ok(config)
}
```

### 3.2 Override Hierarchy

1. `config/default.toml` (base)
2. `config/production.toml` (overrides)
3. Environment variables with `PV2_` prefix (highest priority)

Example: `PV2_SERVER__PORT=9000` overrides `[server].port`.

### 3.3 Tests (10 target)

- Default config loads without errors
- Production overrides apply
- Environment variables override TOML
- Missing config file gracefully defaults
- Invalid values produce PvError::Config

## 4. m04_constants (~100 LOC)

All magic numbers as named constants. Values sourced from `config/default.toml`.

```rust
// Tick loop
pub const TICK_INTERVAL_SECS: u64 = 5;
pub const COUPLING_STEPS_PER_TICK: usize = 15;

// Kuramoto
pub const KURAMOTO_DT: f64 = 0.01;
pub const R_TARGET: f64 = 0.93;
pub const R_HIGH_THRESHOLD: f64 = 0.8;
pub const R_LOW_THRESHOLD: f64 = 0.3;
pub const R_FALLING_THRESHOLD: f64 = -0.03;
pub const IDLE_RATIO_THRESHOLD: f64 = 0.6;
pub const PHASE_GAP_THRESHOLD: f64 = std::f64::consts::FRAC_PI_3;
pub const SYNC_THRESHOLD: f64 = 0.5;
pub const TUNNEL_THRESHOLD: f64 = 0.8;

// Sphere
pub const SPHERE_CAP: usize = 200;
pub const MEMORY_MAX_COUNT: usize = 500;
pub const MEMORY_PRUNE_INTERVAL: u64 = 200;
pub const DECAY_PER_STEP: f64 = 0.995;
pub const SWEEP_BOOST: f64 = 0.05;
pub const ACTIVATION_THRESHOLD: f64 = 0.3;
pub const SEMANTIC_NUDGE_STRENGTH: f64 = 0.02;
pub const LAST_TOOL_MAX_CHARS: usize = 128;
pub const NEWCOMER_STEPS: u64 = 50;

// Coupling
pub const DEFAULT_WEIGHT: f64 = 0.18;
pub const WEIGHT_EXPONENT: f64 = 2.0;
pub const AUTO_SCALE_K_PERIOD: u64 = 20;
pub const AUTO_SCALE_K_MULTIPLIER: f64 = 0.5;
pub const FREQUENCY_MIN: f64 = 0.001;
pub const FREQUENCY_MAX: f64 = 10.0;
pub const STRENGTH_MIN: f64 = 0.0;
pub const STRENGTH_MAX: f64 = 2.0;

// Learning
pub const HEBBIAN_LTP: f64 = 0.01;
pub const HEBBIAN_LTD: f64 = 0.002;
pub const BURST_MULTIPLIER: f64 = 3.0;
pub const NEWCOMER_MULTIPLIER: f64 = 2.0;
pub const WEIGHT_FLOOR: f64 = 0.15;

// Bridges
pub const K_MOD_MIN: f64 = -0.5;
pub const K_MOD_MAX: f64 = 1.5;
pub const K_MOD_BUDGET_MIN: f64 = 0.85;
pub const K_MOD_BUDGET_MAX: f64 = 1.15;

// Conductor
pub const CONDUCTOR_GAIN: f64 = 0.15;
pub const BREATHING_BLEND: f64 = 0.3;
pub const DIVERGENCE_COOLDOWN_TICKS: u32 = 3;

// IPC
pub const SOCKET_PATH: &str = "/run/user/1000/pane-vortex-bus.sock";
pub const MAX_BUS_CONNECTIONS: usize = 50;
pub const EVENT_BUFFER_SIZE: usize = 256;
pub const CASCADE_MAX_DEPTH: u32 = 5;
pub const CASCADE_RATE_LIMIT: usize = 10;

// Persistence
pub const SNAPSHOT_INTERVAL: u64 = 60;
pub const WAL_BUSY_TIMEOUT_MS: i32 = 5000;
pub const R_HISTORY_MAX: usize = 60;
pub const WARMUP_TICKS: u32 = 5;
pub const GHOST_MAX: usize = 20;

// Governance
pub const PROPOSAL_VOTING_WINDOW_TICKS: u64 = 5;
pub const QUORUM_THRESHOLD: f64 = 0.5;
pub const MAX_ACTIVE_PROPOSALS: usize = 10;
```

### 4.1 Tests (5 target)

- All constants are finite (no NaN, no infinity)
- Phase constants are in valid range [0, TAU)
- Weight constants respect floor < ceiling < 1.0
- Budget min < budget max
- Frequency min < frequency max

## 5. m05_traits (~150 LOC)

### 5.1 Trait Definitions

See MODULE_MATRIX.md Section 1 for full signatures.

### 5.2 Tests (5 target)

- Mock struct implementing Oscillator: step advances phase
- Mock struct implementing Learnable: record + prune works
- Mock struct implementing Consentable: defaults are correct
- Trait objects are object-safe (can be `dyn Oscillator`)

## 6. m06_validation (~100 LOC)

### 6.1 Validation Functions

```rust
pub fn validate_phase(phase: f64) -> PvResult<f64>;
pub fn validate_frequency(freq: f64) -> PvResult<f64>;
pub fn validate_weight(w: f64) -> PvResult<f64>;
pub fn validate_strength(s: f64) -> PvResult<f64>;
pub fn validate_sphere_id(id: &str) -> PvResult<()>;
pub fn truncate_string(s: &str, max_chars: usize) -> String;
pub fn validate_tags(tags: &[String]) -> PvResult<()>;
pub fn validate_patterns(patterns: &[String]) -> PvResult<()>;
```

### 6.2 Tests (15 target)

- Phase: NaN rejected, Inf rejected, wraps to [0, TAU)
- Frequency: clamped to [0.001, 10.0], NaN rejected
- Weight: clamped to [0.0, 1.0], NaN rejected
- Sphere ID: empty rejected, >128 rejected, special chars rejected
- truncate_string: multi-byte UTF-8 safe (chars not bytes)
- Tags: >10 tags rejected, >64 char tag rejected
- Patterns: >20 patterns rejected, >128 char pattern rejected

## Summary

| Module | LOC Target | Exports | Tests |
|--------|-----------|---------|-------|
| m01_core_types | 300 | 8 types, 20+ methods | 15 |
| m02_error_handling | 200 | PvError (18 variants), PvResult | 10 |
| m03_config | 250 | PvConfig, load_config() | 10 |
| m04_constants | 100 | ~50 named constants | 5 |
| m05_traits | 150 | 4 traits | 5 |
| m06_validation | 100 | 8 validation functions | 15 |
| **L1 Total** | **1,100** | | **60** |
