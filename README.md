# Pane-Vortex V2 — The Habitat Coordination Daemon

Kuramoto-coupled oscillator field for multi-pane Claude Code fleet coordination.

## Status: SCAFFOLDED

8 layers, 41 modules, 114+ files. Awaiting implementation.

## Architecture

```
L1 Foundation  → Core types, errors, config, validation
L2 Services    → Registry, health, lifecycle, API (axum 0.8)
L3 Field       → Sphere, field state, chimera, messaging
L4 Coupling    → Kuramoto network, Jacobi integration, auto-K
L5 Learning    → Hebbian STDP, buoy network, memory
L6 Bridges     → SYNTHEX, Nexus, ME, POVM, RM, VMS, consent gate
L7 Coordination → IPC bus, conductor, executor, tick orchestrator
L8 Governance   → Proposals, voting, consent, data sovereignty
```

## Build

```bash
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release
```

## Plan

See [MASTERPLAN.md](MASTERPLAN.md) for the V3 plan (499 lines, 45 Obsidian cross-references).

## Part of ULTRAPLATE

Port 8132 | Batch 5 | Depends on: povm-engine + synthex
