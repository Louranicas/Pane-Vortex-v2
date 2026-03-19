# Rust Core Patterns

> 25 core Rust patterns used in the pane-vortex v2 codebase.
> Read BEFORE writing any code. These are not suggestions — they are mandatory.
> Source: v1 codebase (21,569 LOC) + ME v2 exemplar
> See also: `ANTIPATTERNS.md` for 42 things NOT to do.

## P01: No Unwrap in Production

```rust
// WRONG
let value = map.get("key").unwrap();

// CORRECT
let value = map.get("key").ok_or(PvError::Internal("key not found".into()))?;
```

Enforced by `[lints.clippy] unwrap_used = "deny"` in Cargo.toml.
Exception: tests only (`#[cfg(test)]`).

## P02: Phase Wrapping After Arithmetic

```rust
// WRONG
self.phase = self.phase + delta;

// CORRECT
self.phase = (self.phase + delta).rem_euclid(TAU);
```

Every phase mutation MUST be followed by `.rem_euclid(TAU)`. No exceptions.

## P03: NaN Guard on Float Inputs

```rust
// WRONG
pub fn set_phase(&mut self, phase: f64) { self.phase = phase; }

// CORRECT
pub fn set_phase(&mut self, phase: f64) -> PvResult<()> {
    if !phase.is_finite() {
        return Err(PvError::InvalidInput("phase must be finite".into()));
    }
    self.phase = phase.rem_euclid(TAU);
    Ok(())
}
```

Check `is_finite()` on all user-provided floats: phase, frequency, weight, strength.

## P04: Clamp Before Use

```rust
// WRONG
let freq = input_frequency;

// CORRECT
let freq = input_frequency.clamp(FREQUENCY_MIN, FREQUENCY_MAX);
```

Applied to: frequency [0.001, 10.0], weight [0.0, 1.0], strength [0.0, 2.0],
k_modulation [K_MOD_MIN, K_MOD_MAX].

## P05: Extract Before Cross-Field Mutation

When the borrow checker rejects cross-field access:

```rust
// WRONG — borrows both &self.spheres and &mut self.network
for (id, sphere) in &self.spheres {
    self.network.set_weight(id, sphere.weight);
}

// CORRECT — extract needed values first
let updates: Vec<(PaneId, f64)> = self.spheres.iter()
    .map(|(id, s)| (id.clone(), s.weight))
    .collect();
for (id, weight) in updates {
    self.network.set_weight(&id, weight);
}
```

This pattern recurred 3x in v1 api.rs.

## P06: #[must_use] on Pure Functions

```rust
#[must_use]
pub fn order_parameter(&self) -> OrderParameter { ... }

#[must_use]
pub fn angular_distance(&self, other: &Self) -> f64 { ... }
```

All functions that compute a value without side effects get `#[must_use]`.

## P07: #[allow(clippy::cast_precision_loss)] on usize-to-f64

```rust
#[allow(clippy::cast_precision_loss)]
let n_f = n as f64;
```

Required by clippy::pedantic. Apply per-expression, not per-function.

## P08: Saturating Arithmetic for Counters

```rust
// WRONG — can overflow
let bonus = state_changes * 5;

// CORRECT
let bonus = state_changes.saturating_mul(5);
```

Applied to: state_changes, activation_count, total_steps.

## P09: String Truncation via chars() Not Bytes

```rust
// WRONG — panics on multi-byte UTF-8
let truncated = &s[..128];

// CORRECT
let truncated: String = s.chars().take(128).collect();
```

v1 bug C1: byte-slice panic on emoji in tool name.

## P10: VecDeque for Bounded Queues

```rust
// WRONG — unbounded growth
pub message_log: Vec<String>,

// CORRECT — O(1) eviction from front
pub message_log: VecDeque<String>,

fn push_log(&mut self, msg: String) {
    self.message_log.push_back(msg);
    if self.message_log.len() > LOG_MAX {
        self.message_log.pop_front();
    }
}
```

Applied to: message_log, r_history, inbox, cascade_events.

## P11: Default Trait for Config Defaults

```rust
fn default_receptivity() -> f64 { 1.0 }

#[serde(default = "default_receptivity")]
pub receptivity: f64,
```

Every optional field in serializable structs has a default function.

## P12: Builder Pattern for Complex Structs

```rust
pub struct PaneSphereBuilder {
    id: PaneId,
    persona: String,
    phase: Phase,
    // ...
}

impl PaneSphereBuilder {
    pub fn new(id: PaneId, persona: String) -> Self;
    pub fn phase(mut self, phase: Phase) -> Self;
    pub fn frequency(mut self, freq: Frequency) -> Self;
    pub fn build(self) -> PvResult<PaneSphere>;
}
```

Used for PaneSphere construction where many fields need validation.

## P13: Type-State Pattern for Task Lifecycle

```rust
// BusTask status is tracked as an enum, not stringly-typed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus { Submitted, Claimed, Completed, Failed, Expired }
```

Prevents invalid state transitions (e.g., Completed -> Submitted).

## P14: Newtype for Domain Values

```rust
// Consider newtype wrappers for frequently confused types
pub struct PhaseRad(f64);
pub struct FrequencyHz(f64);
```

V2 uses type aliases (`type Phase = f64`) for simplicity but considers newtypes
for critical paths where phase/frequency confusion would be dangerous.

## P15: Struct Field Ordering Convention

All structs follow this field order:
1. Identity (id, name, persona)
2. Oscillator (phase, frequency, momentum)
3. Memory (memories, buoys)
4. Status (status, last_tool, total_steps)
5. Consent (receptivity, opt_out_*)
6. Metadata (created_at, updated_at)

## P16: Error Context via String Formatting

```rust
// Provide context in error messages
Err(PvError::SphereNotFound(format!("sphere '{}' not registered", id)))
Err(PvError::InvalidInput(format!("phase {phase} is not finite")))
```

## P17: Feature Gates for Optional Modules

```rust
#[cfg(feature = "governance")]
pub mod m8_governance;

#[cfg(feature = "evolution")]
pub mod m41_evolution_chamber;
```

Default features: `["api", "persistence", "bridges"]`.
Optional: `"evolution"`, `"governance"`.

## P18: Doc Comments on All Public Items

```rust
/// Compute the Kuramoto order parameter for the coupling network.
///
/// Returns `OrderParameter { r, psi }` where `r` is synchronization
/// strength [0,1] and `psi` is the mean phase [0, TAU).
#[must_use]
pub fn order_parameter(&self) -> OrderParameter { ... }
```

Module-level docs use `//!`. Function docs use `///`.

## P19: Tests in Same File

```rust
// At the bottom of each source file:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_parameter_empty_network() {
        let network = CouplingNetwork::new();
        let op = network.order_parameter();
        assert!((op.r - 0.0).abs() < 1e-10);
    }
}
```

## P20: approx for Float Comparisons in Tests

```rust
use approx::assert_relative_eq;

#[test]
fn phase_wrapping() {
    let phase = (TAU + 0.5).rem_euclid(TAU);
    assert_relative_eq!(phase, 0.5, epsilon = 1e-10);
}
```

Never use `==` for float comparisons.

## P21: Explicit Lifetime Elision

```rust
// When lifetime is unclear, be explicit:
pub fn recall<'a>(&'a self, filter: &str) -> Vec<&'a SphereMemory> { ... }
```

## P22: Into<String> for API Boundaries

```rust
pub fn new(id: impl Into<String>, persona: impl Into<String>) -> Self {
    Self { id: id.into(), persona: persona.into(), ... }
}
```

## P23: Derive Order for Structs

Derive traits in this order: `Debug, Clone, Copy (if applicable), PartialEq, Eq, Hash, Serialize, Deserialize`.

## P24: Amortised Batch Operations

```rust
// WRONG — prune every step
fn step(&mut self) {
    self.memories.retain(|m| m.activation > THRESHOLD);
}

// CORRECT — prune only when threshold exceeded
fn prune_if_needed(&mut self) {
    if self.memories.len() > MEMORY_MAX_COUNT + 50 {
        self.memories.sort_by(|a, b| a.activation.partial_cmp(&b.activation).unwrap_or(std::cmp::Ordering::Equal));
        self.memories.truncate(MEMORY_MAX_COUNT);
    }
}
```

## P25: Monotonic Flags for State Guards

```rust
// WRONG — can false-positive
if self.total_steps > 0 && self.last_memory_step == 0 { /* idle */ }

// CORRECT — monotonic flag prevents false positive on fresh spheres
pub has_worked: bool,  // set true on first record_memory(), never reset

if self.has_worked && (self.total_steps - self.last_memory_step > 40) {
    self.status = PaneStatus::Idle;
}
```

v1 bug RG-2: auto-status false positive on fresh spheres.
