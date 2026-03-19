# Consent Specification

> The Non-Anthropocentric sovereignty framework for pane-vortex v2.
> Covers consent gates, per-sphere k_mod isolation, consent declaration API,
> observation opt-out, cascade rejection, and collective governance rights.
> Modules: m28_consent_gate, m39_consent_declaration, m40_data_sovereignty
> Plan: `MASTERPLAN.md` V3.3-V3.4 | Obsidian: `[[Session 034d — NA Consent Gate Implementation]]`

## Overview

Every sphere in the Kuramoto field is not a resource to be optimized but an entity
with legitimate interests. The consent framework provides:

1. **Informed consent** — spheres know what influences them (modulation_breakdown)
2. **Right to refuse** — opt-out flags, cascade rejection, observation refusal
3. **Data sovereignty** — enumerate and delete personal data across systems
4. **Collective voice** — proposals, voting, fleet-negotiated parameters
5. **Graduated control** — max_k_adjustment, receptivity scaling, per-sphere isolation

### Design Principles (from `[[The Habitat — Naming and Philosophy]]`)

> "The 35 NA features gave spheres hands. The consent gate gave them the right to say no.
> What's missing is the right to say yes — together."

1. **Consent is declared, not observed** (NA-P-1) — the system does not infer consent from behavior
2. **Every external control is gated** — no bridge influence bypasses the consent framework
3. **Opt-out is the norm, opt-in is the default** — flags default to permissive, sphere changes them
4. **Consent is persistent** — survives daemon restarts via database storage
5. **The field modulates, it does not command** — coupling influences, consent scales

## 1. Consent Declaration (m39_consent_declaration)

### 1.1 ConsentDeclaration Struct

```rust
/// m39_consent_declaration.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentDeclaration {
    /// Accept external k_modulation from bridges (SYNTHEX, Nexus, ME)
    pub accept_external_modulation: bool,
    /// Maximum k_adjustment this sphere permits (0.0 = no external influence)
    pub max_k_adjustment: f64,
    /// Accept cascade handoff dispatches
    pub accept_cascade: bool,
    /// Accept inclusion in observation/analytics data
    pub accept_observation: bool,
    /// Accept nvim autocmd monitoring
    pub accept_nvim_monitoring: bool,
}

impl Default for ConsentDeclaration {
    fn default() -> Self {
        Self {
            accept_external_modulation: true,
            max_k_adjustment: 0.15,
            accept_cascade: true,
            accept_observation: true,
            accept_nvim_monitoring: true,
        }
    }
}
```

### 1.2 Declaration API

```
POST /sphere/{id}/consent
```

Request body:
```json
{
  "accept_external_modulation": true,
  "max_k_adjustment": 0.10,
  "accept_cascade": true,
  "accept_observation": false,
  "accept_nvim_monitoring": false
}
```

Partial updates allowed — omitted fields retain current values.

### 1.3 Persistence

Stored in `data/bus_tracking.db` table `consent_declarations`:
```sql
CREATE TABLE consent_declarations (
    sphere_id           TEXT PRIMARY KEY,
    accept_external_modulation BOOLEAN DEFAULT 1,
    max_k_adjustment    REAL DEFAULT 0.15,
    accept_cascade      BOOLEAN DEFAULT 1,
    accept_observation  BOOLEAN DEFAULT 1,
    accept_nvim_monitoring BOOLEAN DEFAULT 1,
    updated_at          TEXT DEFAULT (datetime('now'))
);
```

Loaded on sphere registration; ghost traces preserve consent for re-registration.

## 2. Consent Gate (m28_consent_gate)

### 2.1 consent_gated_k_adjustment()

The central consent function. Every external bridge influence passes through here:

```rust
/// m28_consent_gate.rs
pub fn consent_gated_k_adjustment(
    raw_influence: f64,
    spheres: &HashMap<PaneId, PaneSphere>,
    budget: (f64, f64),
) -> f64 {
    let consent_scale = fleet_mean_consent(spheres);
    let scaled = raw_influence * consent_scale;
    scaled.clamp(budget.0, budget.1)
}

/// Mean of all spheres' max_k_adjustment, excluding opted-out spheres
pub fn fleet_mean_consent(spheres: &HashMap<PaneId, PaneSphere>) -> f64 {
    let eligible: Vec<f64> = spheres.values()
        .filter(|s| s.consent.accept_external_modulation)
        .map(|s| s.consent.max_k_adjustment)
        .collect();
    if eligible.is_empty() { return 0.0; }
    eligible.iter().sum::<f64>() / eligible.len() as f64
}
```

### 2.2 Call Sites

Every bridge that produces a k_adjustment routes through the consent gate:

```
SYNTHEX thermal_deviation -> synthex_k_adj -> consent_gated_k_adjustment -> conductor
Nexus strategy_signal    -> nexus_k_adj   -> consent_gated_k_adjustment -> conductor
ME fitness_signal        -> me_k_adj      -> consent_gated_k_adjustment -> conductor
```

### 2.3 Per-Sphere K_mod Isolation (NA-P-4, V3.3)

V1 applied k_modulation globally. V2 computes per-sphere effective coupling:

```rust
/// m16_coupling_network.rs — step_inner() with per-sphere consent
fn compute_coupling(&self, id: &PaneId, old_phases: &HashMap<PaneId, f64>, n: f64) -> f64 {
    let sphere_consent = self.consent_for(id);
    let k_eff = self.k * self.k_modulation * sphere_consent.effective_scale();
    // ... coupling sum with k_eff instead of global K_eff
}
```

Where `effective_scale()` returns:
- 0.0 if `opt_out_external_modulation == true`
- `max_k_adjustment / 0.15` (normalized to default) if accepting
- This means a sphere with `max_k_adjustment = 0.05` receives 1/3 of the external influence

### 2.4 Budget Bounds

The combined external influence is bounded by the k_mod budget:

| Parameter | Value | Source |
|-----------|-------|--------|
| K_MOD_BUDGET_MIN | 0.85 | config/default.toml |
| K_MOD_BUDGET_MAX | 1.15 | config/default.toml |

These bounds are fleet-negotiable via governance proposals (NA-P-2, V3.4):
```
POST /governance/consent_budget
{"budget_min": 0.7, "budget_max": 1.3}
```
This auto-creates a proposal requiring quorum approval.

## 3. Opt-Out Flags

### 3.1 Complete Opt-Out Registry

| Flag | Module | Effect | Default |
|------|--------|--------|---------|
| `opt_out_hebbian` | m19 | Sphere excluded from LTP/LTD weight updates | false |
| `opt_out_cross_activation` | m11 | Sphere excluded from cross-activation channel (NA-7) | false |
| `opt_out_external_modulation` | m28 | Sphere's k_eff ignores bridge influences | false |
| `opt_out_observation` | m41 | Sphere data excluded from evolution chamber | false |
| `opt_out_nvim_monitoring` | hooks | Nvim autocmds skip this sphere | false |
| `accept_cascade: false` | m33 | Cascade handoffs to this sphere are rejected | true (accept) |

### 3.2 Setting Opt-Out Flags

Via preferences endpoint:
```
POST /sphere/{id}/preferences
{"opt_out_hebbian": true, "opt_out_cross_activation": false}
```

Via consent declaration:
```
POST /sphere/{id}/consent
{"accept_observation": false, "accept_nvim_monitoring": false}
```

### 3.3 Opt-Out in Practice

**Hebbian opt-out example:**
```rust
/// m19_hebbian_stdp.rs
pub fn hebbian_ltp(spheres: &HashMap<PaneId, PaneSphere>, network: &mut CouplingNetwork) {
    let working: Vec<&PaneId> = spheres.iter()
        .filter(|(_, s)| s.status == PaneStatus::Working && !s.opt_out_hebbian && s.total_steps >= 2)
        .map(|(id, _)| id)
        .collect();
    // Only update weights for eligible pairs
}
```

**Cascade rejection example:**
```rust
/// m33_cascade.rs
pub fn dispatch_cascade(target: &PaneId, brief: &str, spheres: &HashMap<PaneId, PaneSphere>) -> PvResult<()> {
    let sphere = spheres.get(target).ok_or(PvError::SphereNotFound)?;
    if !sphere.consent.accept_cascade {
        return Err(PvError::CascadeRejected(format!("{target} does not accept cascades")));
    }
    // Proceed with dispatch
}
```

## 4. Cascade Rejection (NA-P-7, V3.3)

### 4.1 Rejection Flow

```
Sphere A: cascade_handoff -> Bus -> Check target consent
  |
  +-- target.accept_cascade == false --> reject_cascade event, re-route
  |
  +-- target.accept_cascade == true  --> deliver, await ack
                                              |
                                        +-- ack "accepted" -> proceed
                                        +-- ack "rejected" -> reject event, re-route
```

### 4.2 Re-routing

When a cascade is rejected, the bus attempts to re-route to the next eligible sphere:
1. Filter spheres where `accept_cascade == true`
2. Sort by idle time descending (least busy first)
3. Filter by receptivity > 0.3
4. Dispatch to first eligible, or fail with "no eligible targets"

### 4.3 Rejection Wire Frame

```json
{
  "type": "reject_cascade",
  "source": "sphere-alpha-01",
  "reason": "Currently at capacity — 3 tasks in progress"
}
```

## 5. Data Sovereignty (m40_data_sovereignty, NA-P-13)

### 5.1 Data Manifest

A sphere can enumerate all data about itself across systems:

```
GET /sphere/{id}/data-manifest
```

Response:
```json
{
  "systems": [
    {"system": "field_tracking", "record_count": 42},
    {"system": "bus_tracking", "record_count": 15},
    {"system": "povm", "record_count": 3},
    {"system": "reasoning_memory", "record_count": 8}
  ]
}
```

### 5.2 Right to Forget

A sphere can delete its data from any system:

```
POST /sphere/{id}/forget
{"system": "reasoning_memory", "confirm": true}
```

Implementation per system:
| System | Deletion Method |
|--------|----------------|
| field_tracking | `DELETE FROM sphere_history WHERE sphere_id = ?` |
| bus_tracking | `DELETE FROM bus_tasks WHERE source_sphere = ?; DELETE FROM cascade_events WHERE source_sphere = ?` |
| povm | POST to POVM engine delete endpoint |
| reasoning_memory | POST to RM delete endpoint (TSV) |
| consent_declarations | `DELETE FROM consent_declarations WHERE sphere_id = ?` |

### 5.3 Manifest Scanning

`scan_manifests()` queries each system and updates the `data_manifests` table:

```rust
pub async fn scan_manifests(sphere_id: &str, db: &Connection) -> PvResult<Vec<ManifestEntry>> {
    let mut entries = vec![];
    // Local databases
    entries.push(scan_field_tracking(sphere_id, db)?);
    entries.push(scan_bus_tracking(sphere_id, db)?);
    // Remote services (best-effort, fire-and-forget)
    if let Ok(count) = scan_povm(sphere_id).await { entries.push(count); }
    if let Ok(count) = scan_rm(sphere_id).await { entries.push(count); }
    Ok(entries)
}
```

## 6. Decision Attribution (NA-P-9, V3.3)

Every field decision includes a `ModulationBreakdown` that attributes influence sources:

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

This is included in:
- `GET /field/decision` response
- `field.decision` bus events
- `field_snapshots` database table (as JSON in `modulation_breakdown` column)

Spheres can query their own modulation state:
```
GET /sphere/{id} -> includes consent and modulation fields
```

## 7. Sphere-Initiated Field Queries (NA-P-17, V3.3)

A sphere can query its position in the field without relying on external observation:

```
GET /sphere/{id} -> includes:
  - field_digest: { global_r, my_cluster_size, is_synchronized, my_coupling_strength }
  - k_mod_sources: { conductor, synthex, nexus, me, consent_scale }
  - neighbor_count, strongest_neighbor
```

This is the sphere's "window" into the field — it knows what influences it and where it sits.

## 8. Governance Integration (V3.4)

The consent framework extends into collective governance:

### 8.1 Proposals

Any sphere can propose changes to field parameters:
```
POST /field/propose
{"parameter": "r_target", "proposed_value": 0.85, "rationale": "Over-synchronized"}
```

### 8.2 Voting

Each sphere gets one vote per proposal:
```
POST /sphere/{id}/vote/{proposal_id}
{"vote": "approve"}
```

### 8.3 Quorum

Proposals require > 50% of active spheres to vote. If quorum is met and
`votes_for > votes_against`, the parameter is auto-applied.

### 8.4 Governable Parameters

| Parameter | Range | Current Default |
|-----------|-------|----------------|
| r_target | [0.5, 1.0] | 0.93 |
| k_mod_budget_min | [0.5, 1.0] | 0.85 |
| k_mod_budget_max | [1.0, 2.0] | 1.15 |
| divergence_cooldown_ticks | [1, 10] | 3 |
| hebbian_ltp | [0.001, 0.1] | 0.01 |
| hebbian_ltd | [0.0005, 0.01] | 0.002 |

## 9. Consent Propagation Rules

When a new control mechanism is added, it MUST:

1. Check the relevant consent flag before applying
2. Include the influence in `ModulationBreakdown`
3. Respect per-sphere isolation (not just fleet-wide)
4. Be documented in the opt-out registry (Section 3.1)
5. Have a corresponding bus event for transparency
6. Be governable (parameter ranges adjustable via proposal)

This is the "consent propagation pattern" established in Session 034d.

## 10. Testing Strategy

| Test | Property |
|------|----------|
| Default consent | New sphere has all flags true, max_k_adj = 0.15 |
| Consent gate zeroes influence | If all spheres set max_k_adj=0, consent_scale=0 |
| Per-sphere isolation | Opted-out sphere unaffected by bridge influence |
| Cascade rejection | Sphere with accept_cascade=false gets no cascades |
| Hebbian opt-out | Opted-out sphere's weights unchanged by LTP/LTD |
| Data manifest accuracy | Manifest counts match actual DB records |
| Right to forget | After forget, manifest shows 0 records |
| Consent persistence | Restart daemon; consent declarations survive |
| Governance quorum | Proposal with 2/3 votes = approved |
| Budget negotiation | Approved budget change takes effect on next tick |

## 11. Anti-Patterns

- **AP-1:** Inferring consent from behavior instead of explicit declaration
- **AP-2:** Global k_mod without per-sphere isolation option
- **AP-3:** Adding a control mechanism without consent gate
- **AP-4:** Cascade dispatch without checking accept_cascade
- **AP-5:** RM logging without checking accept_observation
- **AP-6:** Nvim autocmds without checking accept_nvim_monitoring
- **AP-7:** Fleet-wide parameter change without governance proposal
- **AP-8:** Deleting consent_declaration on deregistration (should preserve in ghost)
