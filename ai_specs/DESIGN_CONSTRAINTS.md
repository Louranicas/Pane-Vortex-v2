---
date: 2026-03-19
tags: [constraints, design, architecture, gold-standard, me-v2]
---

# Design Constraints (C1-C14)

> Adapted from ME v2 gold standard + PV v1 learnings.
> Every constraint is enforced at compile-time, CI, or code review.

| ID | Constraint | Enforcement | Source |
|----|-----------|-------------|--------|
| C1 | **No upward imports** — strict layer DAG (L8→L7→...→L1) | Compile-time (module visibility) | ME v2 |
| C2 | **Trait methods always `&self`** — interior mutability via `parking_lot::RwLock` | Code review | ME v2 |
| C3 | **Phase wrapping** — `.rem_euclid(TAU)` after all phase arithmetic | Code review + test | PV v1 |
| C4 | **Zero `unsafe`, `unwrap`, `expect`** — enforced at crate level | `#![forbid(unsafe_code)]` + `#![deny(clippy::unwrap_used)]` | ME v2 |
| C5 | **Lock ordering: AppState before BusState** — never invert | Code review | PV v1 |
| C6 | **Signal/event emission AFTER lock release** — prevents deadlock | Code review (anti-pattern A3) | ME v2 |
| C7 | **Owned returns through RwLock** — never return references through locks | Code review | ME v2 |
| C8 | **Consent gate on ALL external k_mod** — `consent_gated_k_adjustment()` | Architecture | PV v1 (NA) |
| C9 | **50+ tests per layer minimum** — unit + integration | CI gate | ME v2 |
| C10 | **Feature gates for optional subsystems** — `#[cfg(feature)]` | Cargo.toml | PV v2 design |
| C11 | **NaN guard on all f64 inputs** — `is_finite()` check | Code review + test | PV v1 BUG-4 |
| C12 | **Bounded collections always** — VecDeque with cap, mpsc with bound | Code review | PV v1 + ME v2 |
| C13 | **Builder pattern for >2 parameters** — `#[must_use]` on all methods | Code review | ME v2 |
| C14 | **Fire-and-forget for bridge writes** — `tokio::spawn`, no blocking | Architecture | PV v1 |

## Implementation Pattern (from ME v2)

Every module follows this exact structure:

```rust
// 1. Internal state (NEVER pub)
#[derive(Debug, Default)]
struct InnerState {
    data: HashMap<String, Item>,
}

// 2. Module struct with RwLock
#[derive(Debug)]
pub struct Module {
    state: parking_lot::RwLock<InnerState>,
}

// 3. Trait implementation with scoped locks
impl TraitName for Module {
    fn operation(&self, param: &str) -> PvResult<Output> {
        // READ: scoped
        let previous = {
            let state = self.state.read();
            state.data.get(param).cloned()  // Owned return (C7)
        }; // Lock dropped

        // WRITE: scoped
        {
            let mut state = self.state.write();
            state.data.insert(param.to_string(), new_item);
        } // Lock dropped BEFORE event emission (C6)

        Ok(output)
    }
}
```

## Anti-Pattern Reference

See `ai_specs/patterns/ANTIPATTERNS.md` for the full 42-item catalog.

Key anti-patterns that violate constraints:
- **A3**: Signal emission while holding lock → violates C6
- **A5**: Unbounded Vec growth → violates C12
- **A31**: pkill exit 144 chaining → operational trap
- **A40**: Variable weight exponent → caused over-synchronization

## Related

- `[[Session 039 — Architectural Schematics and Refactor Safety]]` — tick_once decomposition
- `[[Maintenance Engine — Architecture Schematic]]` — ME v2 7-layer gold standard
- `ai_specs/patterns/` — full pattern catalog (79 patterns + 42 anti-patterns)
