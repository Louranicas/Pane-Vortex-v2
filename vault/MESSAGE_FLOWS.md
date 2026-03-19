---
title: "Pane-Vortex V2 — Message Flows"
date: 2026-03-19
tags: [sequence-diagrams, message-flows, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[Pane-Vortex System Schematics — Session 027c]]"
---

# Pane-Vortex V2 — Message Flows

> Sequence diagrams for key message flows in PV V2.
> Each diagram shows the modules involved, message format, and timing.

---

## 1. Tick Cycle (Every 5 Seconds)

The core tick loop orchestrates all field dynamics:

```mermaid
sequenceDiagram
    participant Timer as Tokio Timer
    participant Tick as m35 tick_orchestrator
    participant Bridges as m22-m27 bridges
    participant Gate as m28 consent_gate
    participant Coupling as m16 coupling_network
    participant STDP as m19 hebbian_stdp
    participant Chimera as m13 chimera
    participant Conductor as m31 conductor
    participant Suggestions as m34 suggestions
    participant Persist as m36 persistence
    participant Bus as m29 ipc_bus

    Timer->>Tick: tick interval elapsed (5s)

    Note over Tick: Phase 1: Bridge Polling
    Tick->>Bridges: spawn poll tasks (fire-and-forget)
    Bridges-->>Gate: raw k_adjustments
    Gate-->>Tick: consent-scaled k_mod_total

    Note over Tick: Phase 2: Field Update
    Tick->>Coupling: step(spheres, network, dt=0.01, steps=15)
    Coupling-->>Tick: phases updated
    Tick->>Chimera: detect_chimera(phases)
    Chimera-->>Tick: ChimeraResult

    Note over Tick: Phase 3: Learning
    Tick->>STDP: hebbian_step(spheres, network)
    STDP-->>Tick: Vec<LearningEvent>

    Note over Tick: Phase 4: Decision
    Tick->>Conductor: decide(field_state, spheres)
    Conductor-->>Tick: FieldDecision
    Tick->>Suggestions: generate_suggestions(field_state, spheres)
    Suggestions-->>Tick: Vec<FieldSuggestion>

    Note over Tick: Phase 5: Persistence
    Tick->>Persist: save_field_snapshot (every 60 ticks)
    Tick->>Bus: broadcast field.tick event
    Tick-->>Timer: TickMetrics
```

### Timing Budget

| Phase | Target | Description |
|-------|--------|-------------|
| Bridge Polling | <100ms | Fire-and-forget spawns, non-blocking |
| Field Update | <20ms | 15 coupling steps at dt=0.01 |
| Learning | <10ms | Hebbian LTP/LTD scan |
| Decision | <5ms | Priority chain + PI controller |
| Persistence | <10ms | SQLite WAL write |
| **Total** | **<50ms** | Well under 5s tick interval |

---

## 2. Sphere Registration

A Claude Code instance registers as a sphere:

```mermaid
sequenceDiagram
    participant Client as Claude Code Instance
    participant API as m10 api_server
    participant State as m15 app_state
    participant Ghost as m12 field_state
    participant Coupling as m16 coupling_network
    participant Bus as m29 ipc_bus
    participant Persist as m36 persistence

    Client->>API: POST /register {id: "alpha", persona: "analyst"}

    API->>State: validate_sphere_id("alpha")
    alt sphere_count >= SPHERE_CAP
        State-->>API: Err(SphereLimitExceeded)
        API-->>Client: 429 Too Many Requests
    else sphere already exists
        State->>State: deregister_sphere("alpha")
        State->>Ghost: create GhostTrace
    end

    State->>State: create PaneSphere("alpha", "analyst")

    alt ghost exists for "alpha"
        Ghost->>Coupling: inherit weights from ghost
        Note over Coupling: consent-gated weight inheritance
    else no ghost
        Coupling->>Coupling: initialize default weights (0.18)
    end

    State-->>API: Ok(SphereSummary)
    API-->>Client: 200 {sphere summary JSON}

    API->>Bus: broadcast sphere.registered event
    API->>Persist: save_sphere_history("alpha", "registered", tick)
```

### Registration Fields

```json
{
    "id": "alpha",
    "persona": "analyst",
    "initial_frequency": 1.0
}
```

All fields except `id` are optional. Defaults: persona="default", frequency=1.0.

---

## 3. Bus Task Submission and Claim

A sphere submits a task that gets claimed by another sphere:

```mermaid
sequenceDiagram
    participant A as Sphere Alpha (source)
    participant Socket as Unix Socket
    participant Bus as m29 ipc_bus
    participant Tasks as m30 bus_types
    participant B as Sphere Beta (claimer)
    participant Persist as m36 persistence

    Note over A: 1. Handshake
    A->>Socket: {"type":"handshake","sphere_id":"alpha","version":"2.0.0"}
    Socket->>Bus: register connection
    Bus-->>A: {"type":"handshake_ack","status":"ok"}

    Note over A: 2. Subscribe to events
    A->>Bus: {"type":"subscribe","pattern":"task.*"}
    Bus-->>A: {"type":"subscribe_ack","pattern":"task.*"}

    Note over A: 3. Submit task
    A->>Bus: {"type":"submit","task_id":"uuid-1","target":"any_idle","desc":"Run cargo test"}
    Bus->>Tasks: create BusTask(status=Submitted)
    Tasks->>Persist: save bus_task
    Bus-->>A: {"type":"submit_ack","task_id":"uuid-1"}
    Bus->>B: {"type":"event","event_type":"task.submitted","data":{...}}

    Note over B: 4. Claim task
    B->>Bus: {"type":"claim","task_id":"uuid-1","sphere_id":"beta"}
    Bus->>Tasks: transition Submitted -> Claimed
    Tasks->>Persist: update bus_task
    Bus-->>B: {"type":"claim_ack","task_id":"uuid-1"}
    Bus->>A: {"type":"event","event_type":"task.claimed","data":{"claimed_by":"beta"}}

    Note over B: 5. Complete task
    B->>Bus: {"type":"complete","task_id":"uuid-1","result":"42 tests passed"}
    Bus->>Tasks: transition Claimed -> Completed
    Tasks->>Persist: update bus_task
    Bus-->>B: {"type":"complete_ack","task_id":"uuid-1"}
    Bus->>A: {"type":"event","event_type":"task.completed","data":{"result":"42 tests passed"}}
```

### NDJSON Wire Protocol

Each message is a single JSON line terminated by `\n`:
```
{"type":"handshake","sphere_id":"alpha","version":"2.0.0"}\n
```

Field summary:
- `type` (required): Frame type identifier
- `sphere_id`: Sender identity (required in handshake, optional in others)
- `task_id`: UUID for task-related frames
- `pattern`: Glob pattern for subscribe/unsubscribe
- `data`: Arbitrary JSON payload for events

---

## 4. Cascade Handoff

A sphere delegates work to another sphere via the cascade system:

```mermaid
sequenceDiagram
    participant A as Sphere Alpha (Tab 4)
    participant Bus as m29 ipc_bus
    participant Cascade as m33 cascade
    participant Consent as m39 consent_declaration
    participant B as Sphere Beta (Tab 5)
    participant Persist as m36 persistence

    A->>Bus: {"type":"cascade_handoff","source":"alpha","target":"beta","brief":"Refactor m01","depth":0}

    Bus->>Cascade: validate rate limit
    alt rate limited (>10/min)
        Cascade-->>A: {"type":"error","code":"rate_limited"}
    end

    Bus->>Consent: check beta.accept_cascade
    alt cascade not consented
        Cascade-->>A: {"type":"cascade_ack","status":"rejected","reason":"consent_denied"}
        Cascade->>Persist: save cascade_event(rejected)
    else cascade consented
        Bus->>B: {"type":"cascade_handoff","source":"alpha","brief":"Refactor m01","depth":0}

        alt beta accepts
            B-->>Bus: {"type":"cascade_ack","source":"beta","target":"alpha","status":"acked"}
            Bus->>A: forward cascade_ack
            Cascade->>Persist: save cascade_event(acked)
        else beta rejects
            B-->>Bus: {"type":"cascade_ack","source":"beta","target":"alpha","status":"rejected"}
            Bus->>A: forward cascade_ack
            Cascade->>Persist: save cascade_event(rejected)
            Note over Cascade: Re-route to next eligible sphere
        end
    end
```

### Fallback (No Bus Connection)

If the target sphere has no active bus connection:
1. Cascade module writes a markdown brief to `~/projects/shared-context/tasks/cascade-{uuid}.md`
2. The brief contains: source, target, description, timestamp, depth
3. Target sphere picks it up on next session start (hook reads shared-context)

---

## 5. Governance Proposal (V3.4)

A sphere proposes changing a field parameter:

```mermaid
sequenceDiagram
    participant A as Sphere Alpha (proposer)
    participant API as m10 api_server
    participant Proposals as m37 proposals
    participant B as Sphere Beta (voter)
    participant C as Sphere Gamma (voter)
    participant Voting as m38 voting
    participant Conductor as m31 conductor
    participant Bus as m29 ipc_bus

    Note over A: 1. Submit proposal
    A->>API: POST /field/propose {parameter: "r_target", proposed_value: 0.85, rationale: "More divergence needed"}
    API->>Proposals: create_proposal(alpha, "r_target", 0.85, ...)
    Proposals-->>API: Proposal(id="prop-1", deadline=tick+5)
    API-->>A: 200 {proposal_id: "prop-1"}
    API->>Bus: broadcast governance.proposal.created event

    Note over A,C: 2. Voting window (5 ticks = 25s)
    B->>API: POST /sphere/beta/vote/prop-1 {vote: "approve"}
    API->>Voting: cast_vote("prop-1", "beta", Approve)
    Voting-->>API: Ok

    C->>API: POST /sphere/gamma/vote/prop-1 {vote: "approve"}
    API->>Voting: cast_vote("prop-1", "gamma", Approve)
    Voting->>Voting: check_quorum(3 spheres, 2 votes = 66% > 50%)

    Note over Voting: 3. Quorum reached, majority approves
    Voting->>Proposals: resolve_proposal("prop-1") -> Approved
    Proposals->>Conductor: apply_approved(r_target = 0.85)
    Conductor->>Conductor: update r_target to 0.85
    Proposals->>Bus: broadcast governance.proposal.applied {parameter: "r_target", new_value: 0.85}

    Bus->>A: governance.proposal.applied event
    Bus->>B: governance.proposal.applied event
    Bus->>C: governance.proposal.applied event
```

### Quorum Rules

- Quorum threshold: >50% of active (registered) spheres must vote
- Approval: votes_for > votes_against (simple majority)
- Voting window: 5 ticks (25 seconds)
- Max active proposals: 10
- One vote per sphere per proposal (UNIQUE constraint)

---

## 6. Bridge Polling (SYNTHEX Thermal)

Bidirectional bridge poll with consent gate:

```mermaid
sequenceDiagram
    participant Tick as m35 tick_orchestrator
    participant Bridge as m22 synthex_bridge
    participant SYNTHEX as SYNTHEX :8090
    participant Gate as m28 consent_gate
    participant Spheres as m15 app_state

    Note over Tick: Every 6 ticks (30s)
    Tick->>Bridge: spawn poll_thermal("127.0.0.1:8090")

    Bridge->>SYNTHEX: GET /v3/thermal
    SYNTHEX-->>Bridge: {temperature: 0.57, target: 0.5, synergy: 0.45, heat_sources: [...]}

    Bridge->>Bridge: compute_k_adjustment(thermal)
    Note over Bridge: raw_adj = (0.57 - 0.5) * 0.1 = +0.007

    Bridge->>Gate: consent_gated_k_adjustment(spheres, +0.007, "synthex")

    loop for each sphere
        Gate->>Spheres: check sphere.preferences.accept_external_modulation
        Gate->>Spheres: check sphere.preferences.max_k_adj
        alt consented
            Gate->>Gate: scaled_adj = 0.007 * consent_scale
        else opted out
            Gate->>Gate: scaled_adj = 0.0 for this sphere
        end
    end

    Gate-->>Bridge: ConsentDecision {accepted: true, raw: 0.007, scaled: 0.005}

    Note over Bridge: Also post field state back to SYNTHEX
    Bridge->>SYNTHEX: POST /v3/pane-vortex/state {r, k, sphere_count, decision}
```

---

## Message Format Reference

### HTTP API Messages

All HTTP endpoints accept and return JSON. Content-Type: application/json.

| Direction | Format | Example |
|-----------|--------|---------|
| Request body | JSON | `{"id": "alpha", "persona": "analyst"}` |
| Response body | JSON | `{"status": "ok", "sphere_count": 3}` |
| Error response | JSON | `{"error": "SphereNotFound", "message": "..."}` |

### IPC Bus Messages

NDJSON over Unix domain socket. One JSON object per line.

| Direction | Format | Example |
|-----------|--------|---------|
| Client -> Server | NDJSON | `{"type":"handshake","sphere_id":"alpha","version":"2.0.0"}` |
| Server -> Client | NDJSON | `{"type":"event","event_type":"field.tick","data":{...}}` |
| Error | NDJSON | `{"type":"error","code":"parse_error","message":"..."}` |

### Bridge Messages

Raw TCP HTTP (no hyper). Request/response follow HTTP/1.1 format.

| Direction | Format | Notes |
|-----------|--------|-------|
| PV -> Service | GET/POST HTTP/1.1 | Raw TCP, fire-and-forget for writes |
| Service -> PV | HTTP/1.1 200 + JSON body | Parsed with serde_json |
| PV -> RM | POST with TSV body | `printf 'cat\tagent\tconf\tttl\tcontent'` format |

---

## Cross-References

- **[STATE_MACHINES.md](STATE_MACHINES.md)** — FSM definitions for all state transitions
- **[SCHEMATICS.md](SCHEMATICS.md)** — Static architecture diagrams
- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Module-level function signatures
- **[ERROR_TAXONOMY.md](ERROR_TAXONOMY.md)** — Error types referenced in error flows
- **[config/default.toml](../config/default.toml)** — Timing and threshold values
- **Obsidian:** `[[Pane-Vortex System Schematics — Session 027c]]` (V1 sequence diagrams)
