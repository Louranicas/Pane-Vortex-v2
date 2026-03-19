# API Specification

> All HTTP endpoints for the pane-vortex v2 daemon.
> Target: 60+ endpoints organized by module. Includes governance endpoints (V3.4).
> Feature gates: `api` (core), `governance` (proposals/voting), `evolution` (chamber)
> Source: m10_api_server.rs | Plan: `MASTERPLAN.md` | Obsidian: `[[Pane-Vortex — Fleet Coordination Daemon]]`

## Overview

The HTTP API runs on port 8132 (configurable) bound to 127.0.0.1 (loopback only).
Built with axum 0.8. CORS enabled. Body limit: 65KB. All responses are JSON.

## 1. Health & Status (3 endpoints)

### GET /health
Health check for devenv and monitoring.

**Response 200:**
```json
{
  "status": "healthy",
  "tick": 42,
  "spheres": 3,
  "r": 0.847,
  "uptime_secs": 3600,
  "version": "2.0.0"
}
```

### GET /status
Extended status with memory and bridge health.

**Response 200:**
```json
{
  "tick": 42,
  "spheres": 3,
  "r": 0.847,
  "k_mod": 1.02,
  "fleet_mode": "Active",
  "bridges": {
    "synthex": {"connected": true, "last_poll_tick": 36},
    "nexus": {"connected": true, "last_poll_tick": 30},
    "me": {"connected": false, "error": "timeout"},
    "povm": {"connected": true, "last_poll_tick": 0},
    "rm": {"connected": true, "last_poll_tick": 0},
    "vms": {"connected": false, "error": "not_started"}
  },
  "bus_clients": 2,
  "task_count": {"submitted": 3, "claimed": 1, "completed": 15}
}
```

### GET /version
Binary version and build info.

**Response 200:**
```json
{
  "version": "2.0.0",
  "features": ["api", "persistence", "bridges"],
  "modules": 41,
  "rust_version": "1.75"
}
```

## 2. Field Endpoints (12 endpoints)

### GET /field
Complete field state snapshot.

**Response 200:**
```json
{
  "order_parameter": {"r": 0.847, "psi": 2.31},
  "chimera": {"is_chimera": false, "sync_clusters": [], "desync_clusters": []},
  "harmonics": {"l0_monopole": 0.847, "l1_dipole": 0.847, "l2_quadrupole": 0.12},
  "tunnels": [],
  "sphere_count": 3,
  "total_memories": 42,
  "tick": 42
}
```

### GET /field/r
Current order parameter.

**Response 200:** `{"r": 0.847, "psi": 2.31, "trend": "Stable"}`

### GET /field/spectrum
Harmonic decomposition of phase distribution.

**Response 200:** `{"l0_monopole": 0.847, "l1_dipole": 0.847, "l2_quadrupole": 0.12}`

### GET /field/chimera
Current chimera detection state.

**Response 200:**
```json
{
  "is_chimera": true,
  "sync_clusters": [{"members": ["alpha", "beta"], "local_r": 0.95, "mean_phase": 1.2}],
  "desync_clusters": [{"members": ["gamma"], "local_r": 0.1, "mean_phase": 4.5}]
}
```

### GET /field/tunnels
Active buoy-overlap tunnels between spheres.

**Response 200:**
```json
[
  {"sphere_a": "alpha", "sphere_b": "beta", "buoy_a_label": "Read", "buoy_b_label": "Read", "overlap": 0.85, "semantic_a": "grep search", "semantic_b": "file read"}
]
```

### GET /field/decision
Latest decision engine output.

**Response 200:**
```json
{
  "action": "Stable",
  "targets": [],
  "r": 0.847,
  "r_trend": "Stable",
  "k_mod": 1.02,
  "modulation_breakdown": {
    "conductor_k_mod": 1.0, "synthex_influence": 0.01,
    "nexus_influence": 0.01, "me_influence": 0.0,
    "consent_scale": 0.95, "effective_k": 2.34
  }
}
```

### GET /field/decisions
Decision history (last 20).

**Response 200:** Array of decision objects.

### GET /field/k
Coupling strength details.

**Response 200:**
```json
{
  "k_base": 2.4, "k_modulation": 1.02, "k_effective": 2.448,
  "auto_k": true, "auto_scale_period": 20
}
```

### GET /field/warmup
Warmup status.

**Response 200:** `{"warmup_remaining": 0, "is_warming_up": false}`

### GET /field/history
Historical r values (rolling 60-sample window).

**Response 200:** `{"r_history": [0.82, 0.84, 0.85, ...], "sample_count": 42}`

### GET /field/ghosts
Ghost traces of deregistered spheres (max 20).

**Response 200:**
```json
[
  {"id": "old-sphere", "persona": "worker", "deregistered_at": 100, "total_steps_lived": 500, "memory_count": 42, "top_tools": ["grep", "read"], "phase_at_departure": 1.2}
]
```

### GET /field/suggestions
Latest field-driven suggestions.

**Response 200:**
```json
[
  {"suggestion_type": "DivergenceOpportunity", "description": "3 spheres are over-synchronized", "targets": ["alpha", "beta", "gamma"], "priority": 0.8}
]
```

## 3. Sphere Endpoints (22 endpoints)

### GET /spheres
List all registered spheres with summaries.

**Response 200:** Array of sphere summary objects.

### GET /sphere/{id}
Full sphere state.

**Response 200:**
```json
{
  "id": "alpha", "persona": "implementer", "phase": 1.234, "frequency": 0.85,
  "status": "Working", "last_tool": "Edit", "registered_at": 1710849600.0,
  "total_steps": 42, "memory_count": 15,
  "receptivity": 0.8, "is_synchronized": true, "tunnel_count": 1,
  "maturity": "established", "age_seconds": 210,
  "neighbor_count": 2, "strongest_neighbor": {"id": "beta", "weight": 0.45},
  "work_signature": {"intensity": 0.7, "rhythm": 0.5, "diversity": 0.6, "focus": 0.4},
  "consent": {"accept_external_modulation": true, "max_k_adjustment": 0.15, "accept_cascade": true, "accept_observation": true}
}
```

### POST /sphere/{id}/register
Register a new sphere.

**Request:**
```json
{"persona": "implementer", "initial_phase": 0.0}
```

**Response 201:** `{"id": "alpha", "phase": 0.0, "frequency": 0.85}`
**Response 409:** `{"error": "sphere already registered"}`
**Response 429:** `{"error": "sphere cap reached (200)"}`

### POST /sphere/{id}/deregister
Deregister a sphere (creates ghost trace).

**Response 200:** `{"ghost_created": true}`
**Response 404:** `{"error": "sphere not found"}`

### POST /sphere/{id}/memory
Record a memory on the sphere surface.

**Request:**
```json
{"tool_name": "Edit", "summary": "Modified m16_coupling_network.rs step_inner()", "position": {"x": 0.5, "y": 0.5, "z": 0.707}}
```

**Response 200:** `{"memory_id": 15, "activation": 1.0}`

### POST /sphere/{id}/phase
Update sphere phase directly.

**Request:** `{"phase": 1.234}`
**Response 200:** `{"phase": 1.234}` (after NaN guard + rem_euclid)

### POST /sphere/{id}/status
Update sphere status.

**Request:** `{"status": "Working", "last_tool": "Edit"}`
**Response 200:** `{"status": "Working"}`

### POST /sphere/{id}/steer
Phase injection (consent-gated, NA-35).

**Request:** `{"target_phase": 1.5, "strength": 0.1}`
**Response 200:** `{"applied_strength": 0.08}` (scaled by receptivity)

### GET /sphere/{id}/recall
Query sphere memories.

**Query params:** `?near_phase=1.2&near_buoy=Read&limit=10`
**Response 200:** Array of matching SphereMemory objects.

### GET /sphere/{id}/neighbors
Coupling neighborhood.

**Response 200:**
```json
[
  {"id": "beta", "weight": 0.45, "phase_diff": 0.12, "type_weight": 1.2},
  {"id": "gamma", "weight": 0.22, "phase_diff": 1.8, "type_weight": 0.6}
]
```

### POST /sphere/{id}/decouple
Voluntary decoupling (NA-16).

**Response 200:** `{"decoupled": true}`

### POST /sphere/{id}/recouple
Re-engage coupling (NA-16).

**Response 200:** `{"recoupled": true}`

### POST /sphere/{id}/preferences
Update coupling preferences (NA-33).

**Request:** `{"opt_out_hebbian": false, "opt_out_cross_activation": true, "opt_out_external_modulation": false}`
**Response 200:** `{"updated": true}`

### POST /sphere/{id}/request-divergence
Sphere-sourced divergence request (NA-23).

**Response 200:** `{"accepted": true, "cooldown_ticks": 3}`

### GET /sphere/{id}/narrative
Memory narrative (NA-19).

**Response 200:**
```json
{"narrative": "alpha has been working for 210s, primarily using Edit and Read tools. 15 memories formed, strongest buoy: 'Read' at phase 0.1."}
```

### GET /sphere/{id}/associations
Associative memory index (NA-18).

**Response 200:**
```json
{"associations": [{"tool": "Edit", "count": 8, "mean_activation": 0.6}, {"tool": "Read", "count": 5, "mean_activation": 0.4}]}
```

### GET /sphere/{id}/inbox
Message inbox with consent (NA-20).

**Response 200:** Array of pending messages.

### POST /sphere/{id}/inbox/send
Send a message to another sphere's inbox.

**Request:** `{"to": "beta", "message": "Found a bug in m16"}`
**Response 200:** `{"message_id": 5}`

### POST /sphere/{id}/inbox/ack
Acknowledge inbox messages (NA-21).

**Request:** `{"message_ids": [3, 4, 5]}`
**Response 200:** `{"acked": 3}`

### POST /sphere/{id}/consent
Active consent declaration (NA-P-1, V3.3).

**Request:**
```json
{
  "accept_external_modulation": true,
  "max_k_adjustment": 0.10,
  "accept_cascade": true,
  "accept_observation": false,
  "accept_nvim_monitoring": false
}
```

**Response 200:** `{"updated": true, "consent_hash": "a1b2c3"}`

### GET /sphere/{id}/data-manifest
Data sovereignty manifest (NA-P-13, V3.3).

**Response 200:**
```json
{
  "systems": [
    {"system": "field_tracking", "record_count": 42, "last_scanned": "2026-03-19T14:30:00Z"},
    {"system": "bus_tracking", "record_count": 15, "last_scanned": "2026-03-19T14:30:00Z"},
    {"system": "povm", "record_count": 3, "last_scanned": "2026-03-19T14:25:00Z"},
    {"system": "reasoning_memory", "record_count": 8, "last_scanned": "2026-03-19T14:28:00Z"}
  ]
}
```

### POST /sphere/{id}/forget
Erase sphere data from a system (NA-P-13, V3.3).

**Request:** `{"system": "reasoning_memory", "confirm": true}`
**Response 200:** `{"deleted_count": 8}`

## 4. Coupling Endpoints (4 endpoints)

### GET /coupling
Full coupling network state.

**Response 200:** Serialized CouplingNetwork (phases, frequencies, connections).

### GET /coupling/weights
Connection weight matrix.

**Response 200:** `{"weights": [{"from": "alpha", "to": "beta", "weight": 0.45}]}`

### GET /coupling/topology
Adjacency topology summary.

**Response 200:** `{"nodes": 3, "edges": 6, "density": 1.0, "mean_weight": 0.32}`

### POST /coupling/inject
Manual weight injection (admin/testing only).

**Request:** `{"from": "alpha", "to": "beta", "weight": 0.5}`
**Response 200:** `{"updated": true}`

## 5. Bus Endpoints (6 endpoints)

### GET /bus/status
IPC bus status.

**Response 200:** `{"connected_clients": 2, "total_tasks": 18, "pending": 3, "active_subscriptions": 5}`

### GET /bus/tasks
List tasks with optional status filter.

**Query:** `?status=submitted&limit=20`
**Response 200:** Array of BusTask objects.

### POST /bus/submit
Submit a task via HTTP (mirrors bus frame).

**Request:** Same as Submit bus frame.
**Response 200:** `{"task_id": "task-a1b2c3d4"}`

### GET /bus/suggestions
Field-driven suggestions.

**Response 200:** Array of FieldSuggestion objects.

### GET /bus/events
Recent bus events.

**Query:** `?type=field.*&limit=50`
**Response 200:** Array of BusEvent objects.

### POST /bus/cascade
Cascade handoff via HTTP (mirrors bus frame).

**Request:** `{"target": "beta", "brief": "Continue m16 implementation"}`
**Response 200:** `{"dispatched": true}`

## 6. Bridge Endpoints (7 endpoints)

### GET /synthex/thermal
Cached SYNTHEX thermal state.

**Response 200:** ThermalState object (temperature, target, pid_output, heat_sources).

### GET /nexus/state
Cached NexusForge state.

**Response 200:** NexusState (strategy, inner/outer r, modules).

### GET /nexus/metrics
Computed nexus metrics.

**Response 200:** `{"strategy": "coherent", "r_inner": 0.92, "r_outer": 0.85, "dispatch_confidence": 0.88}`

### GET /me/state
Cached Maintenance Engine state.

**Response 200:** `{"fitness": 0.366, "health_checks": 589000, "last_poll_tick": 30}`

### GET /bridges/health
Health of all bridges.

**Response 200:**
```json
{"synthex": "healthy", "nexus": "healthy", "me": "stale", "povm": "healthy", "rm": "healthy", "vms": "disconnected"}
```

### GET /povm/summary
POVM pathway and memory summary.

**Response 200:** `{"pathways": 2425, "memories": 36, "last_snapshot_tick": 0}`

### GET /rm/recent
Recent Reasoning Memory entries.

**Response 200:** Array of RM entries (category, content, confidence, ttl).

## 7. Conductor Endpoints (3 endpoints)

### GET /conductor/state
Current conductor state.

**Response 200:**
```json
{"k_modulation": 1.02, "r_target": 0.93, "divergence_cooldown": 0, "fleet_mode": "Active", "breathing_amplitude": 0.02}
```

### POST /conductor/r_target
Override r_target (admin).

**Request:** `{"r_target": 0.85}`
**Response 200:** `{"r_target": 0.85}`

### GET /conductor/modulation_history
K_modulation history (last 60 ticks).

**Response 200:** Array of `{tick, k_mod, r, action}` objects.

## 8. Executor Endpoints (3 endpoints)

### POST /executor/dispatch
Manual dispatch to a Zellij pane.

**Request:** `{"pane": "tab-4:pane-0", "command": "cargo test"}`
**Response 200:** `{"dispatched": true, "latency_ms": 911}`

### GET /executor/pane-map
Current pane mapping.

**Response 200:** `{"tab-1": ["command"], "tab-4": ["alpha", "beta"]}`

### GET /executor/score
Dispatch scoring for available panes.

**Response 200:** Array of `{pane, score, status, last_activity}` objects.

## 9. Governance Endpoints (7 endpoints, feature-gated)

All governance endpoints require `#[cfg(feature = "governance")]`.

### POST /field/propose
Submit a parameter change proposal (V3.4).

**Request:**
```json
{
  "parameter": "r_target",
  "proposed_value": 0.85,
  "rationale": "Current r_target of 0.93 is too high for 3-sphere field, causing over-synchronization"
}
```

**Response 201:** `{"proposal_id": "prop-a1b2", "voting_deadline_tick": 47}`

### GET /field/proposals
List all proposals.

**Query:** `?status=open`
**Response 200:** Array of Proposal objects.

### GET /field/proposals/{id}
Get a specific proposal with vote tally.

**Response 200:** Full Proposal object including votes breakdown.

### POST /sphere/{id}/vote/{proposal_id}
Cast a vote (V3.4).

**Request:** `{"vote": "approve"}`
**Response 200:** `{"recorded": true, "votes_for": 2, "votes_against": 1}`

### GET /governance/parameters
List governable parameters and their current values.

**Response 200:**
```json
[
  {"parameter": "r_target", "value": 0.93, "min": 0.5, "max": 1.0, "type": "f64"},
  {"parameter": "k_mod_budget_max", "value": 1.15, "min": 1.0, "max": 2.0, "type": "f64"},
  {"parameter": "divergence_cooldown_ticks", "value": 3, "min": 1, "max": 10, "type": "u32"}
]
```

### GET /governance/history
Resolved proposals with outcomes.

**Response 200:** Array of resolved Proposal objects.

### POST /governance/consent_budget
Fleet-negotiated consent budget (NA-P-2, V3.4).

**Request:** `{"budget_min": 0.7, "budget_max": 1.3}`
**Response 200:** Requires quorum vote (auto-creates proposal).

## 10. Evolution Endpoints (4 endpoints, feature-gated)

All evolution endpoints require `#[cfg(feature = "evolution")]`.

### GET /evolution/status
Evolution chamber status.

**Response 200:**
```json
{"generations": 24, "best_fitness": 0.87, "population_size": 10, "running": true}
```

### POST /analytics/observe
Submit an observation to the evolution chamber.

**Request:** `{"event": "task_completed", "metrics": {"latency_ms": 150, "quality": 0.9}}`
**Response 200:** `{"observation_id": 42}`

### GET /analytics/population
Current evolution population.

**Response 200:** Array of candidate configurations with fitness scores.

### GET /analytics/history
Evolution history (generations, fitness trajectory).

**Response 200:** `{"generations": [{"gen": 1, "best": 0.5}, {"gen": 24, "best": 0.87}]}`

## 11. Integration Endpoints (3 endpoints)

### GET /integration/matrix
Cross-service integration matrix.

**Response 200:**
```json
{
  "services": ["synthex", "nexus", "me", "povm", "rm", "vms"],
  "connections": [
    {"from": "pv", "to": "synthex", "type": "thermal_poll", "healthy": true},
    {"from": "pv", "to": "nexus", "type": "strategy_poll", "healthy": true}
  ]
}
```

### GET /integration/cascade
Cascade system status.

**Response 200:** `{"active_cascades": 1, "total_dispatched": 15, "total_rejected": 2, "depth_limit": 5}`

### GET /integration/metrics
Aggregated system metrics.

**Response 200:**
```json
{
  "uptime_secs": 3600, "total_ticks": 720, "total_tasks": 50,
  "total_events": 2000, "total_cascades": 15, "total_memories": 120,
  "mean_r": 0.85, "mean_tick_duration_ms": 4.2
}
```

## 12. Debug Endpoints (2 endpoints)

### GET /debug/state
Full serialized AppState (large, for debugging only).

**Response 200:** Complete AppState JSON.

### GET /debug/locks
Lock contention diagnostics.

**Response 200:** `{"app_state_readers": 2, "app_state_writers": 0, "bus_state_readers": 1, "bus_state_writers": 0}`

---

## Summary: 60+ Endpoints

| Category | Count | Feature Gate |
|----------|-------|-------------|
| Health & Status | 3 | api |
| Field | 12 | api |
| Sphere | 22 | api |
| Coupling | 4 | api |
| Bus | 6 | api |
| Bridge | 7 | api + bridges |
| Conductor | 3 | api |
| Executor | 3 | api |
| Governance | 7 | governance |
| Evolution | 4 | evolution |
| Integration | 3 | api |
| Debug | 2 | api |
| **TOTAL** | **76** | |

## Security

- All endpoints on 127.0.0.1 only (no external access)
- Body limit: 65,536 bytes
- String truncation: `last_tool` 128 chars, `persona` 64 chars, `summary` 500 chars
- Phase NaN guard on all phase-accepting endpoints
- Sphere cap: 200 (HTTP 429 on excess)
- CORS headers for local development tools
