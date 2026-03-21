# Hebbian Learning Deep Dive

> Generated Session 044 by Claude-PV1-Sidecar analyzing V1 source code (19KB analysis)

## Overview
Rate-based Hebbian learning on Kuramoto coupling network. "Neurons that fire together wire together" — co-active Claude Code instances get stronger coupling.

**NOT classical STDP** — closer to rate-based Hebbian with LTP/LTD, plus a causal STDP flag (S6.6) for asymmetric directed weight updates.

## Constants
| Constant | Value | Purpose |
|----------|-------|---------|
| HEBBIAN_LTP | 0.01 | Base potentiation per tick |
| HEBBIAN_LTD | 0.002 | Base depression per tick |
| HEBBIAN_BURST_MULTIPLIER | 3.0 | LTP multiplier when 3+ spheres co-active |
| HEBBIAN_WEIGHT_FLOOR | 0.15 | Minimum weight (prevents total decoupling) |

## Learning Rule: `hebbian_learning()` (src/conductor.rs)

Called per tick AFTER Kuramoto coupling, BEFORE field state.

### Phase 1: LTP (Strengthen co-active pairs)
- Trigger: 2+ eligible spheres with status=Working
- Burst: 3+ Working spheres → LTP × 3.0 = 0.03
- Newcomer boost: sphere with <50 steps → × 2.0
- STDP direction: sphere with earlier `last_active_at` gets asymmetric boost

### Phase 2: LTD (Weaken inactive pairs)
- For each pair where at least one is NOT Working
- Weight decays by LTD=0.002 per tick
- Floor at 0.15 — never fully decouples

### Consent Gate
- `opt_out_hebbian = true` → sphere excluded from learning entirely (NA-34)

## Buoy Semantic Anchoring (src/types.rs, src/sphere.rs)

Buoys provide semantic labels that enable **resonant tunnels** between spheres.

```
Buoy { label: String, strength: f64, category: BuoyCategory }
```

- Categories: Primary, Tool, Topic, Bridge
- Spheres accumulate buoys from tool use and task context
- `detect_tunnels()` finds sphere pairs with overlapping buoy labels
- Tunnel threshold: 0.8 rad phase proximity + shared buoys
- Tunnels create **preferential routing** for cascade handoffs

## Implementation Files (V1)
| File | Component |
|------|-----------|
| src/conductor.rs | `hebbian_learning()` — the learning rule (~140 LOC) |
| src/coupling.rs | `CouplingNetwork` — weight storage, Kuramoto step |
| src/types.rs | `Buoy` struct and methods |
| src/sphere.rs | `PaneSphere` buoy update/boost/prune logic |
| src/field.rs | `detect_tunnels()` — buoy overlap → resonant channel |

## Links
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[IPC Bus Architecture Deep Dive]]
- [[KURAMOTO_FIELD_SPEC]]
- [[Consent Flow Analysis]]
