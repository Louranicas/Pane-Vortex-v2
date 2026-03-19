---
title: "Layer 6: Bridges — Module Documentation"
date: 2026-03-19
tags: [documentation, l6_bridges, pane-vortex-v2, bridges, consent]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Session 034f — SYNTHEX Schematics and Wiring]]"
  - "[[Session 034d — NA Consent Gate Implementation]]"
layer: L6
modules: [m22, m23, m24, m25, m26, m27, m28]
---

# Layer 6: Bridges (m22-m28)

> External service bridges with raw TCP HTTP and consent gate.
> **Depends on:** L1 (Foundation), L3 (Field)
> **Target LOC:** ~1,650 | **Target tests:** 62+

## Modules: m22_synthex_bridge m23_nexus_bridge m24_me_bridge m25_povm_bridge m26_rm_bridge m27_vms_bridge m28_consent_gate

## Purpose

L6 connects PV to 6 external ULTRAPLATE services. All bridges use raw TCP HTTP (no hyper/reqwest) for minimal dependency overhead. The consent gate (m28) is the central access control: every external k_adjustment passes through it, scaled by sphere consent.

## Design Constraints

- All bridges fire-and-forget for writes (pattern P05)
- All reads use timeout + retry with cached fallback
- All k_adjustments pass through consent_gated_k_adjustment() (pattern P04)
- k_mod budget clamped to [0.85, 1.15]
- RM uses TSV format, never JSON (anti-pattern AP05)
- Never block tick loop with synchronous bridge calls
- Bridge failures are non-fatal (tick continues regardless)

See `ai_specs/DESIGN_CONSTRAINTS.md` for full constraint list.

## Implementation Status: STUB (awaiting implementation)

---

## Bridge Architecture

```
[SYNTHEX :8090] <--thermal--> [m22] --|
[Nexus :8100]   <--strategy-> [m23] --|
[ME :8080]      <--fitness--> [m24] --|-- [m28 consent_gate] --> k_mod_total --> conductor
[POVM :8125]    <--persist--> [m25] --|
[RM :8130]      ---tsv-post-> [m26]
[VMS :8120]     ---seed-----> [m27]
```

---

## m22 -- SYNTHEX Bridge

**Source:** `src/m6_bridges/m22_synthex_bridge.rs` | **LOC Target:** ~300

### Purpose

Bidirectional REST bridge to SYNTHEX (:8090). SYNTHEX is the "cerebral cortex" of the distributed brain -- intelligence hub with 61D tensor and thermal homeostasis.

### Read: Thermal State

Polls `/v3/thermal` every 6 ticks (30 seconds):

```json
{"temperature": 0.572, "target": 0.500, "synergy": 0.45, "heat_sources": [...]}
```

Computes k_adjustment from thermal delta: `(temperature - target) * gain`

### Write: Field State

Posts PV field state to `/v3/pane-vortex/state`:

```json
{"r": 0.93, "k": 1.2, "sphere_count": 3, "decision": "Stable"}
```

### V3 Alignment

- V3.1.7 (SYNTHEX synergy investigation): synergy at 0.15-0.5, below 0.7 threshold
- V3.3 (Sovereignty): per-sphere thermal isolation (NA-P-4)

---

## m23 -- Nexus Bridge

**Source:** `src/m6_bridges/m23_nexus_bridge.rs` | **LOC Target:** ~400

### Purpose

Bidirectional REST bridge to SAN-K7 Nexus (:8100). Nexus is the "basal ganglia" -- action selection with 59 modules. Provides nested Kuramoto coherence (inner r and outer r).

### Read: Nexus State

Polls `/status` every 12 ticks (60 seconds):

```json
{"strategy": "coherent", "r_inner": 0.85, "r_outer": 0.72, "modules_active": 42}
```

### Nexus Commands

Can issue commands via `/api/v1/nexus/command`:
- `service-health` (M6)
- `synergy-check` (M45)
- `best-practice` (M44)
- `deploy-swarm` (M40)

### Key Types

- `NexusState` -- strategy, r_inner, r_outer, dispatch_confidence, modules_active
- `NexusMetrics` -- Exposed at `/nexus/metrics` endpoint

---

## m24 -- ME Bridge

**Source:** `src/m6_bridges/m24_me_bridge.rs` | **LOC Target:** ~250

### Purpose

Read-only bridge to Maintenance Engine (:8080). ME is the "autonomic nervous system" -- 589K health checks, RALPH evolution, 12D tensor.

### CRITICAL: BUG-008

ME has zero EventBus publishers. This means 240MB of runtime data (930K rows) is in a closed loop. V3.1.1 is the single highest-impact fix in the entire V3 plan. Until BUG-008 is fixed, ME fitness is frozen at 0.3662.

### Read: Observer Fitness

Polls `/api/observer` every 12 ticks (60 seconds):

```json
{"metrics": {"correlations_found": 42}, "last_report": {"current_fitness": 0.3662}}
```

### Consent-Gated k_adjustment

ME fitness influences coupling via consent gate:
```rust
let raw_adj = compute_me_k_adjustment(&observer);
let scaled = consent_gated_k_adjustment(&sphere, raw_adj, "me");
```

---

## m25 -- POVM Bridge

**Source:** `src/m6_bridges/m25_povm_bridge.rs` | **LOC Target:** ~200

### Purpose

Write-heavy bridge to POVM Engine (:8125). POVM is the "spinal cord" -- persistence hub with 2,425 pathways and spherical harmonics.

### Write: Field Snapshots

Posts field snapshot every 12 ticks (60 seconds):
```json
{"tick": 720, "r": 0.93, "k": 1.1, "sphere_count": 3, "weights": {...}}
```

### Write: Hebbian Weights

Posts coupling weight matrix every 60 ticks (5 minutes):
```json
{"weights": [["alpha", "beta", 0.42], ["alpha", "gamma", 0.18]]}
```

### Read: Hydration (Startup Only)

On startup, hydrates pathways and summary:
- `hydrate_pathways()` -- Restore coupling weights from POVM
- `hydrate_summary()` -- Load POVM summary for diagnostics

---

## m26 -- RM Bridge

**Source:** `src/m6_bridges/m26_rm_bridge.rs` | **LOC Target:** ~150

### Purpose

Write-only bridge to Reasoning Memory (:8130). RM is the "prefrontal cortex" -- cross-session reasoning context.

### CRITICAL: TSV Format

RM uses **TSV (Tab-Separated Values)**, not JSON. Anti-pattern AP05.

```rust
pub fn post_tsv(addr: &str, category: &str, agent: &str, conf: f64, ttl: u64, content: &str) -> PvResult<()> {
    let body = format!("{category}\t{agent}\t{conf}\t{ttl}\t{content}");
    // POST to /put with TSV body
}
```

### Write: Conductor Decisions

Posts conductor decisions every 60 ticks (5 minutes):
```
field_state    pane-vortex    0.9    600    r=0.93 k=1.1 decision=Stable spheres=3
```

### V3 Alignment

V3.5.1 (RM noise reduction): Change PV field state TTL to 600s to reduce RM noise.

---

## m27 -- VMS Bridge

**Source:** `src/m6_bridges/m27_vms_bridge.rs` | **LOC Target:** ~150

### Purpose

Write-only bridge to Vortex Memory System (:8120). VMS is the "hippocampus" -- spatial memory with fractal topology.

### Write: Field Memory

Seeds VMS with field state snapshots every 60 ticks (5 minutes).

### Read: Memory Seeding (Startup Only)

On startup, optionally seeds field state from VMS memories.

---

## m28 -- Consent Gate (CENTRAL ACCESS CONTROL)

**Source:** `src/m6_bridges/m28_consent_gate.rs` | **LOC Target:** ~200

### Purpose

The consent gate is the central access control for all external influence on coupling. Every k_adjustment from any bridge must pass through `consent_gated_k_adjustment()`.

### The Consent Pipeline

```
1. Raw adjustment from bridge (e.g., SYNTHEX: +0.05)
2. Check sphere consent: accept_external_modulation, max_k_adjustment
3. Scale by consent: adj * min(sphere.max_k_adj, 1.0)
4. Accumulate: k_mod_total = synthex + nexus + me + conductor
5. Clamp to budget: k_mod_total.clamp(0.85, 1.15)
```

### Key Types

- `ConsentDecision` -- accepted: bool, raw_adj: f64, scaled_adj: f64, reason: String
- `KModBudget` -- min: f64, max: f64, current: f64
- `ModulationBreakdown` -- synthex: f64, nexus: f64, me: f64, conductor: f64, consent_scale: f64

### Key Functions

```rust
pub fn consent_gated_k_adjustment(
    sphere: &PaneSphere,
    raw_adj: f64,
    source: &str,
) -> ConsentDecision {
    if !sphere.preferences.accept_external_modulation() {
        return ConsentDecision { accepted: false, raw_adj, scaled_adj: 0.0, reason: "opted_out" };
    }
    let scale = sphere.preferences.max_k_adj().min(1.0);
    let scaled = raw_adj * scale;
    ConsentDecision { accepted: true, raw_adj, scaled_adj: scaled, reason: source.to_string() }
}

pub fn clamp_budget(total: f64, budget: &KModBudget) -> f64 {
    total.clamp(budget.min, budget.max)
}
```

### Philosophy

From `[[Session 034d -- NA Consent Gate Implementation]]`: "Every external control mechanism that modifies coupling strength must pass through this gate. The pattern established here must propagate to every new control mechanism added in the future."

### V3 Alignment

- V3.3.1 (Active consent declaration): `/sphere/{id}/consent` POST
- V3.3.2 (Per-sphere k_mod isolation): Coupling step uses per-sphere k_eff
- V3.4.7 (Dynamic consent budget): Fleet controls k_mod range via governance

---

## Raw TCP HTTP Pattern

All bridges use raw TCP for HTTP (no hyper/reqwest). This pattern was established in V1 and provides:
- Minimal dependency tree
- No connection pooling overhead
- Clean fire-and-forget semantics

```rust
async fn raw_http_get(addr: &str, path: &str, timeout: Duration) -> PvResult<String> {
    let mut stream = tokio::time::timeout(
        timeout,
        TcpStream::connect(addr),
    ).await.map_err(|_| PvError::Bridge(BridgeError::ConnectionTimeout("service".into())))??;

    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;

    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    // Parse HTTP response, extract body
    parse_http_body(&buf[..n])
}
```

---

## Related

- [MASTERPLAN.md](../../MASTERPLAN.md) -- V3 plan, bridge alerts (ALERT-1 through ALERT-4)
- [ARCHITECTURE_DEEP_DIVE.md](../ARCHITECTURE_DEEP_DIVE.md) -- Distributed brain anatomy
- [SCHEMATICS.md](../SCHEMATICS.md) -- Bridge data flow diagram
- [ERROR_TAXONOMY.md](../ERROR_TAXONOMY.md) -- BridgeError variants
- Obsidian: `[[Session 034f -- SYNTHEX Schematics and Wiring]]`, `[[Session 034d -- NA Consent Gate Implementation]]`
- V1 Reference: `~/claude-code-workspace/pane-vortex/src/synthex_bridge.rs`, `nexus_bridge.rs`, `me_bridge.rs`, `povm_bridge.rs`
