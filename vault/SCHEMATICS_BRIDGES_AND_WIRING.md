# Schematics: Bridges and Wiring

> **10 architecture diagrams** covering the full bridge topology, consent pipeline,
> wire protocol, lock ordering, and distributed brain anatomy of pane-vortex.
>
> **Companion to:** `ai_docs/SCHEMATICS.md` (field-layer diagrams)
> **Source of truth:** `src/synthex_bridge.rs`, `src/nexus_bridge.rs`, `src/me_bridge.rs`,
> `src/povm_bridge.rs`, `src/conductor.rs`, `src/bus.rs`, `src/ipc.rs`, `src/main.rs`
> **Cross-refs:** `ai_specs/API_SPEC.md`, `ai_specs/IPC_BUS_SPEC.md`, `ai_specs/WIRE_PROTOCOL_SPEC.md`

---

## 1. SYNTHEX Thermal Bridge Wiring

The SYNTHEX thermal bridge is a bidirectional link between pane-vortex and the SYNTHEX v3
homeostasis system (port 8090). The **read path** polls `/v3/thermal` every 6 ticks (30s) or
25s wall-clock (whichever fires first), extracting temperature, PID output, and 4 heat sources.
The **write path** pushes field state to `/api/ingest` every 12 ticks (60s). The thermal
deviation drives a linear K adjustment in `[0.8, 1.2]`: cold boosts coupling, hot reduces it.

The PID controller inside SYNTHEX operates at `Kp=0.5, Ki=0.1, target=0.5` and feeds 4 heat
sources that pane-vortex influences: HS-001 (Hebbian/field r), HS-002 (Cascade activity),
HS-003 (ME Resonance/fitness), and HS-004 (CrossSync/nexus health).

```mermaid
sequenceDiagram
    participant TL as PV Tick Loop<br/>(5s adaptive)
    participant SB as synthex_bridge.rs<br/>(raw TCP)
    participant SX as SYNTHEX :8090<br/>(v3 homeostasis)
    participant PID as PID Controller<br/>(Kp=0.5 Ki=0.1 target=0.5)
    participant HS as Heat Sources
    participant CG as consent_gated_k_adjustment()
    participant KM as k_modulation

    Note over TL,SX: READ PATH (every 6 ticks / 25s wall-clock)

    TL->>SB: should_poll(tick)?
    SB->>SX: GET /v3/thermal (raw TCP, 3s timeout)
    SX->>PID: current temperature
    PID-->>SX: pid_output

    SX-->>SB: ThermalState {temperature, target,<br/>pid_output, heat_sources[4]}
    SB->>SB: Cache in SharedThermalState

    Note over HS: 4 Heat Sources
    Note over HS: HS-001: Hebbian (r→reading, w=0.3)
    Note over HS: HS-002: Cascade (depth→heat, w=0.2)
    Note over HS: HS-003: Resonance (ME fitness, w=0.25)
    Note over HS: HS-004: CrossSync (nexus health, w=0.25)

    TL->>SB: thermal_k_adjustment(&ThermalState)
    SB-->>TL: raw_adj ∈ [0.8, 1.2]<br/>cold→boost, hot→reduce
    TL->>CG: consent_gated_k_adjustment(raw_adj, spheres)
    CG-->>TL: gated_adj (receptivity × newcomer × eligible)
    TL->>KM: k_modulation *= gated_adj

    Note over TL,SX: WRITE PATH (every 12 ticks / 60s)

    TL->>SB: post_field_state(r, k_mod, spheres, decision,<br/>me_fitness, nexus_health)
    SB->>SX: POST /api/ingest (fire-and-forget)
    SX->>HS: Update HS-001 from r, HS-003 from me_fitness,<br/>HS-004 from nexus_health

    TL->>SB: post_cascade_heartbeat(CascadeHeatSnapshot)
    SB->>SX: POST /api/ingest (cascade heat)
    SX->>HS: Update HS-002 from cascade_heat
```

**Key constraints:**
- `thermal_k_adjustment` is a simple linear interpolation: `(1.0 - deviation * 0.2).clamp(0.8, 1.2)`
- The raw adjustment always passes through `consent_gated_k_adjustment` before touching `k_modulation`
- Wall-clock fallback (`THERMAL_POLL_WALL_SECS=25`) ensures polling at adaptive tick intervals (1s/5s/15s)
- `is_stale()` threshold is 60s -- stale readings are skipped in the tick loop
- BUG-034g: Do NOT call `writer.shutdown()` on the TCP half -- causes axum to drop connection

**Spec refs:** `ai_specs/API_SPEC.md` (SYNTHEX endpoints), `src/synthex_bridge.rs` (full implementation)

---

## 2. Nexus Nested Kuramoto (Inner / Outer)

Pane-vortex operates a **two-layer nested Kuramoto field**. The inner field (PV) contains up
to 200 spheres representing Claude Code instances, producing `r_inner`. The outer field
(SAN-K7 / NexusForge at port 8100) contains 12 oscillators representing strategic modules,
producing `r_outer`. The combined coherence is the geometric mean: `sqrt(r_inner * r_outer)`.

Strategy classification drives dispatch confidence: Aligned (r>=0.8) dispatches confidently,
Incoherent (r<0.2) pauses dispatch entirely. FleetMode qualifies interpretation -- a single
sphere always has r=1.0 (self-coherent) which is NOT fleet-coherent, so Solo mode caps
dispatch_confidence at 0.5.

```mermaid
graph TB
    subgraph InnerField["Inner Field (pane-vortex :8132)"]
        style InnerField fill:#1a3a5c,color:#fff
        S1["Sphere 1<br/>Claude Code"]
        S2["Sphere 2<br/>Claude Code"]
        SN["Sphere N<br/>(up to 200)"]
        KI["Kuramoto Coupling<br/>dt=0.01, Jacobi solver<br/>auto_scale_k, Hebbian STDP"]
        RI["r_inner<br/>Inner order parameter"]
        S1 --> KI
        S2 --> KI
        SN --> KI
        KI --> RI
    end

    subgraph OuterField["Outer Field (SAN-K7 :8100)"]
        style OuterField fill:#5c1a3a,color:#fff
        O1["Module Oscillator 1"]
        O2["Module Oscillator 2"]
        O12["Module Oscillator 12"]
        KO["Outer Kuramoto<br/>12 oscillators, strategy layer"]
        RO["r_outer<br/>Outer order parameter"]
        O1 --> KO
        O2 --> KO
        O12 --> KO
        KO --> RO
    end

    subgraph NestedMetrics["Nested Kuramoto Metrics"]
        style NestedMetrics fill:#2a4a2a,color:#fff
        CC["combined_coherence<br/>= sqrt(r_inner x r_outer)"]
        SC["Strategy Classification"]
        DC["dispatch_confidence"]
        FM["FleetMode"]
        FR["frequency_ratio<br/>(inner_freq / outer_freq)"]
        RI --> CC
        RO --> CC
        RO --> SC
        CC --> DC
        SC --> DC
        FM --> DC
    end

    subgraph StrategyLevels["Strategy Coherence Thresholds"]
        AL["Aligned: r_outer >= 0.8<br/>confidence = combined.min(1.0)"]
        PA["Partial: r_outer >= 0.5<br/>confidence = combined * 0.7"]
        DV["Diverging: r_outer >= 0.2<br/>confidence = combined * 0.3"]
        IC["Incoherent: r_outer < 0.2<br/>confidence = 0.0"]
    end

    subgraph FleetModes["Fleet Operational Modes"]
        SOLO["Solo: 0-1 spheres<br/>confidence capped at 0.5"]
        SMALL["Small: 2-5 spheres"]
        MED["Medium: 6-20 spheres"]
        LRG["Large: 20+ spheres"]
    end

    SC --> AL & PA & DV & IC
    FM --> SOLO & SMALL & MED & LRG

    subgraph Bidirectional["Bridge Communication"]
        READ["READ: GET /status<br/>(every 12 ticks / 55s wall)"]
        WRITE["WRITE: POST /api/v1/nexus/command<br/>(field state push)"]
    end

    RO -.->|raw TCP GET| READ
    READ -.-> NestedMetrics
    RI -.->|POST| WRITE
    WRITE -.-> OuterField

    HEALTH["healthy = r_inner > 0.3<br/>AND r_outer > 0.2<br/>AND combined > 0.3"]
    CC --> HEALTH
```

**Key constraints:**
- `NestedKuramotoMetrics::compute()` is the single function that produces all derived metrics
- Solo mode cap prevents trivial self-coherence from being misinterpreted as fleet alignment
- The `nexus_k_adjustment()` function maps strategy to raw multiplier: Aligned=1.1, Partial=1.0, Diverging=0.9, Incoherent=0.85
- Frequency ratio (8.4) detects inter-scale tempo mismatches when both fields report mean frequency
- Supports dual-schema parsing: NexusForge native schema OR SAN-K7 `/status` with proxy field mapping

**Spec refs:** `src/nexus_bridge.rs` (full implementation), `ai_specs/KURAMOTO_FIELD_SPEC.md`

---

## 3. ME Bridge + BUG-008 Path

The Maintenance Engine (port 8080) is a 7-layer architecture with 12D tensors, PBFT consensus,
and RALPH evolution. Layer 7 (Observer) produces a fitness score. The ME bridge reads this via
`GET /api/health`, extracts fitness, and computes a narrow K adjustment in `[0.95, 1.03]` --
deliberately conservative because the ME has an advisory (not controlling) role.

**BUG-008** is the severed nerve: the ME's `EventBus` has zero publishers, so its Observer layer
receives no events and the fitness score is frozen at 0.3662 (the initial calibration value).
The bridge reads this stale fitness, derives a neutral-to-slightly-reduced K adjustment,
and the consent gate further dampens it. The ME effectively has no dynamic influence.

```mermaid
graph LR
    subgraph ME["Maintenance Engine :8080"]
        style ME fill:#4a2a0a,color:#fff
        L1["L1: Ingestion"]
        L2["L2: Analysis"]
        L3["L3: Correlation<br/>(12D tensor)"]
        L4["L4: Prediction"]
        L5["L5: Planning"]
        L6["L6: Execution"]
        L7["L7: Observer<br/>(RALPH evolution)"]
        EB["EventBus<br/><b>BUG-008: 0 publishers</b>"]
        FIT["fitness: 0.3662<br/>(FROZEN)"]
        L1 --> L2 --> L3 --> L4 --> L5 --> L6
        L6 --> L7
        EB -.->|zero events| L7
        L7 --> FIT
    end

    subgraph Bridge["me_bridge.rs"]
        style Bridge fill:#2a3a4a,color:#fff
        POLL["poll_me()<br/>every 12 ticks / 55s"]
        FETCH["fetch_me_health()<br/>GET /api/health<br/>(raw TCP, 3s timeout)"]
        PARSE["Parse flexible JSON:<br/>fitness | last_fitness |<br/>overall_health | health_score |<br/>derive from status string"]
        CACHE["SharedMeState<br/>Arc<RwLock<Option<MeHealthState>>>"]
        POLL --> FETCH --> PARSE --> CACHE
    end

    subgraph KAdj["K Adjustment"]
        style KAdj fill:#3a2a3a,color:#fff
        RAW["me_k_adjustment()"]
        HEALTHY["fitness >= 0.8 → 1.0 + (f-0.8)*0.15<br/>range: [1.0, 1.03]"]
        DEGRADED["fitness 0.5-0.8 → 1.0<br/>(neutral)"]
        UNHEALTHY["fitness < 0.5 → 0.95 + f*0.1<br/>range: [0.95, 1.0]"]
        RAW --> HEALTHY
        RAW --> DEGRADED
        RAW --> UNHEALTHY
    end

    subgraph ConsentGate["Consent Gate"]
        CG["consent_gated_k_adjustment()"]
        KM["k_modulation *= adj"]
    end

    FIT -->|"GET /api/health<br/>raw TCP"| FETCH
    CACHE --> RAW
    RAW -->|"raw_adj"| CG
    CG --> KM

    BUG["BUG-008: SEVERED NERVE<br/>EventBus has 0 publishers →<br/>Observer receives 0 events →<br/>fitness frozen at 0.3662 →<br/>me_k_adjustment returns ~0.986 →<br/>consent gate dampens further →<br/>ME has NO dynamic influence"]
    style BUG fill:#8b0000,color:#fff,stroke:#ff0000,stroke-width:3px

    EB -.-|SEVERED| BUG
```

**Key constraints:**
- ME K adjustment range is deliberately narrow: `[0.95, 1.03]` (advisory role)
- Flexible JSON parsing handles variable ME response schemas (4 fallback fitness fields)
- `is_stale()` threshold is 120s (2 x 55s poll interval + margin)
- PG-12 mandate: ME bridge routes through `consent_gated_k_adjustment` for NA compliance
- BUG-008 fix requires wiring ME EventBus publishers -- the bridge code is correct, the problem is upstream

**Spec refs:** `src/me_bridge.rs`, `ai_specs/API_SPEC.md`, Obsidian `[[The Maintenance Engine V2]]`

---

## 4. Consent Gate Pipeline

Every external bridge influence on `k_modulation` passes through `consent_gated_k_adjustment()`.
This function implements 5 consent mechanisms that together ensure the fleet is never overridden
by external systems without the spheres' collective agreement. The function lives in
`src/nexus_bridge.rs` but is used by ALL bridges (SYNTHEX, Nexus, ME, NexusBus).

```mermaid
flowchart TD
    RAW["Raw adjustment from bridge<br/>(e.g. 1.1 from Nexus, 0.85 from SYNTHEX)"]
    style RAW fill:#4a3a2a,color:#fff

    EMPTY{"spheres.is_empty()?"}
    RAW --> EMPTY
    EMPTY -->|Yes| PASSTHROUGH["Return raw_adjustment unchanged"]
    EMPTY -->|No| ELIGIBLE

    ELIGIBLE["Filter eligible spheres:<br/>exclude opt_out_external_modulation"]
    ELIGIBLE --> ALLOUT{"All opted out?"}
    ALLOUT -->|Yes| NEUTRAL["Return 1.0 (neutral)<br/>No external influence"]
    ALLOUT -->|No| RECEPTIVITY

    RECEPTIVITY["Compute mean_receptivity<br/>= mean(eligible.receptivity)<br/>Low = fleet focused, resist override"]
    style RECEPTIVITY fill:#2a4a3a,color:#fff

    NEWCOMER["Compute newcomer_damping<br/>newcomer = total_steps < 50<br/>fraction = newcomer_count / eligible_count<br/>damping = 1.0 - fraction * 0.8<br/>(100% newcomers → 20% influence)"]
    style NEWCOMER fill:#2a3a4a,color:#fff

    FRACTION["Compute eligible_fraction<br/>= eligible.len() / spheres.len()<br/>(how much fleet consents)"]
    style FRACTION fill:#3a2a4a,color:#fff

    DIVERGENCE{"Any sphere<br/>receptivity < 0.15?<br/>(divergence vote)"}
    style DIVERGENCE fill:#4a2a2a,color:#fff

    DEVIATION["deviation = raw_adj - 1.0"]

    RECEPTIVITY --> SCALE
    NEWCOMER --> SCALE
    FRACTION --> SCALE
    SCALE["scale = mean_receptivity<br/>x newcomer_damping<br/>x eligible_fraction"]

    DEVIATION --> CHECK_DIV
    DIVERGENCE --> CHECK_DIV

    CHECK_DIV{"divergence_active<br/>AND deviation > 0?"}
    CHECK_DIV -->|Yes| SUPPRESS["scaled_deviation = 0.0<br/>(suppress boost during divergence)"]
    CHECK_DIV -->|No| APPLY["scaled_deviation = deviation * scale"]

    SUPPRESS --> RESULT
    APPLY --> RESULT

    RESULT["final = 1.0 + scaled_deviation"]
    style RESULT fill:#2a4a2a,color:#fff

    RESULT --> KMUL["k_modulation *= final"]

    BUDGET["THEN: Combined bridge clamp<br/>all bridges together: [0.85, 1.15]<br/>(prevents compounding)"]
    style BUDGET fill:#8b4500,color:#fff
    KMUL --> BUDGET

    GLOBAL["THEN: Global k_mod clamp<br/>[K_MOD_MIN, K_MOD_MAX]<br/>=[-0.5, 1.5]"]
    style GLOBAL fill:#4a0a0a,color:#fff
    BUDGET --> GLOBAL
```

**Key constraints:**
- NA-GAP-1: Mean receptivity scales influence -- focused fleet (low receptivity) resists external override
- NA-GAP-2: `opt_out_external_modulation` flag excludes spheres from the eligible pool
- NA-GAP-3: Divergence exemption -- positive boost (deviation > 0) is suppressed when any sphere votes divergence (receptivity < 0.15)
- NA-GAP-4: Low mean receptivity naturally dampens because it multiplies the deviation
- NA-GAP-5: Newcomer protection -- fleets with many new spheres (< 50 steps) get dampened influence (down to 20%)
- Combined bridge budget `[0.85, 1.15]` is applied AFTER all individual bridge adjustments
- Global clamp `[K_MOD_MIN, K_MOD_MAX]` = `[-0.5, 1.5]` is the final backstop

**Spec refs:** `src/nexus_bridge.rs:705-757` (consent_gated_k_adjustment), `src/main.rs:794-877` (bridge application)

---

## 5. All-Bridge Combined Influence into Conductor

Six bridges feed into the consent gate, which feeds into the conductor's PI controller, which
governs `k_modulation` and thereby the Kuramoto coupling strength. Three bridges are polled
directly by the tick loop (SYNTHEX, Nexus, ME), and five additional bridges are managed by
the NexusBus subsystem (CsV7, ToolLibrary, DevEnvPatterns, VmsRead, MeObserver). The combined
effect of all bridges is clamped to `[0.85, 1.15]` per tick to prevent compounding.

```mermaid
graph LR
    subgraph DirectBridges["Direct Bridges (tick loop)"]
        style DirectBridges fill:#1a3a5c,color:#fff
        SX["SYNTHEX :8090<br/>synthex_bridge.rs<br/>thermal_k_adj [0.8, 1.2]"]
        NX["Nexus/SAN-K7 :8100<br/>nexus_bridge.rs<br/>nexus_k_adj [0.85, 1.15]"]
        ME["Maint. Engine :8080<br/>me_bridge.rs<br/>me_k_adj [0.95, 1.03]"]
    end

    subgraph NexusBusBridges["NexusBus Bridges (5 readers)"]
        style NexusBusBridges fill:#3a1a5c,color:#fff
        CSV7["CsV7 :8110<br/>CodeSynthor neural graph"]
        TL["ToolLibrary :8105<br/>STDP learning"]
        DEV["DevEnvPatterns<br/>Local SQLite"]
        VMS["VmsRead :8120<br/>VMS health/memory"]
        MEO["MeObserver :8080<br/>RALPH evolution"]
    end

    subgraph ConsentPipeline["Consent Pipeline"]
        style ConsentPipeline fill:#2a4a2a,color:#fff
        CG["consent_gated_k_adjustment()<br/>per-bridge:<br/>receptivity x newcomer x eligible<br/>divergence exemption"]
    end

    SX -->|raw_adj| CG
    NX -->|raw_adj| CG
    ME -->|raw_adj| CG

    CSV7 -->|reading.k_adj| CG
    TL -->|reading.k_adj| CG
    DEV -->|reading.k_adj| CG
    VMS -->|reading.k_adj| CG
    MEO -->|reading.k_adj| CG

    CG -->|"k_mod *= gated_adj<br/>(per bridge)"| BUDGET

    BUDGET["Combined Bridge Budget<br/>combined_effect = k_mod_after / k_mod_before<br/>clamped_effect = clamp(combined, 0.85, 1.15)<br/>k_mod = k_mod_before * clamped_effect"]
    style BUDGET fill:#8b4500,color:#fff

    BUDGET --> CONDUCTOR

    subgraph ConductorBlock["PI Conductor"]
        style ConductorBlock fill:#4a2a0a,color:#fff
        CONDUCTOR["conduct_breathing()<br/>GAIN=0.15, EMERGENT_BLEND=0.3"]
        RTARGET["r_target(s)<br/>base=0.93, large fleet=0.85<br/>50/50 blend with fleet preferred_r"]
        ROLLING["r_input = 60-tick rolling mean<br/>(not instantaneous r)"]
        EMERGENT["emergent_breathing()<br/>beat frequency from freq diversity"]
        CONDUCTOR --> RTARGET
        CONDUCTOR --> ROLLING
        CONDUCTOR --> EMERGENT
    end

    CONDUCTOR --> KMOD

    KMOD["k_modulation<br/>clamped to [K_MOD_MIN, K_MOD_MAX]<br/>= [-0.5, 1.5]"]
    style KMOD fill:#0a4a0a,color:#fff

    KMOD --> KURAMOTO["Kuramoto Coupling<br/>K_eff = K_base * k_mod<br/>weight^2 amplification<br/>type_weight per-status"]
```

**Key constraints:**
- Order matters: direct bridges (SYNTHEX, Nexus, ME) are applied first, then NexusBus readings
- k_mod_before_bridges is captured BEFORE any bridge adjustment, used as baseline for budget clamp
- The combined effect `[0.85, 1.15]` prevents the scenario where 8 bridges each pushing 1.1 yields 1.1^8 = 2.14
- After budget clamp, the conductor's `conduct_breathing()` further adjusts k_mod toward `r_target`
- NA-P-5: Conductor suppresses coherence-boosting when any sphere has receptivity < 0.15 (divergence vote)
- 4.6: Bridge adjustment values are stored in `BridgeAdjustments` for narrative attribution via `/sphere/{id}/narrative`

**Spec refs:** `src/main.rs:794-877` (bridge section), `src/conductor.rs` (PI controller), `src/nexus_bus/mod.rs` (NexusBus apply)

---

## 6. Unix Socket Connection Lifecycle

The IPC bus uses a Unix domain socket at `$XDG_RUNTIME_DIR/pane-vortex-bus.sock` with owner-only
permissions (0700). Each Claude Code instance connects, authenticates via handshake, subscribes
to event patterns, and enters a bidirectional message loop. The connection lifecycle includes
timeouts at every stage: 5s handshake, 90s keepalive, 120s dead detection.

```mermaid
sequenceDiagram
    participant C as Client<br/>(Claude Code / pane-vortex-client)
    participant L as UnixListener<br/>(pane-vortex daemon)
    participant H as Connection Handler<br/>(tokio::spawn per client)
    participant BS as BusState<br/>(tasks, events, connections)
    participant AS as AppState<br/>(spheres, coupling)

    Note over C,L: PHASE 1: Connection (< 200 cap)
    C->>L: connect() to Unix socket
    L->>L: SO_PEERCRED: verify UID matches daemon
    L->>L: Check connections.len() < 200
    L->>H: spawn connection handler

    Note over H,AS: PHASE 2: Handshake (5s timeout)
    H->>H: Start 5s handshake timer
    C->>H: ClientFrame::Handshake<br/>{sphere_id, version}
    H->>AS: read() — verify sphere registered
    AS-->>H: sphere exists
    H->>C: ServerFrame::HandshakeOk<br/>{tick, peer_count, r, protocol_version}
    H->>BS: Register ConnectionHandle<br/>(mpsc channel, cap 256)

    Note over C,BS: PHASE 3: Subscriptions
    C->>H: ClientFrame::Subscribe<br/>{patterns: ["field.*", "task.*"]}
    H->>BS: Add glob patterns (max 20)
    H->>C: ServerFrame::Event{type: "subscribed"}

    Note over C,BS: PHASE 4: Bidirectional Message Loop
    loop Every tick (field events)
        BS->>H: tx.try_send(ServerFrame::Event)
        H->>C: NDJSON line: {"type":"Event",...}\n
    end

    opt Task Operations
        C->>H: ClientFrame::TaskSubmit{id, desc, target, payload}
        H->>BS: submit_task() — queue + route
        BS-->>H: task_id
        H->>C: ServerFrame::Event{type: "task.submitted"}
    end

    opt Task Claim
        C->>H: ClientFrame::TaskClaim{task_id}
        H->>BS: claim_task(sphere_id)
        H->>C: ServerFrame::TaskAssigned{task_id, desc, payload, from}
    end

    opt Cascade Handoff
        C->>H: ClientFrame::CascadeHandoff<br/>{handoff_id, target, cwd, task, prompt}
        H->>AS: capture MitosisContext
        H->>BS: route to target connection
        H->>C: ServerFrame::Event{type: "cascade.dispatched"}
    end

    Note over C,H: PHASE 5: Keepalive / Death Detection
    H->>C: Pong (at 90s silence)
    C->>H: Ping (resets timer)

    alt 120s total silence
        H->>H: Connection declared dead
    else Client EOF
        C--xH: EOF / socket close
    else I/O error
        H--xH: Read/write error
    end

    Note over H,BS: PHASE 6: Cleanup
    H->>BS: bus.disconnect(sphere_id)
    H->>H: writer_task.abort()
    H->>BS: broadcast_event("sphere.disconnected")
```

**Key constraints:**
- SO_PEERCRED UID check: only the same user can connect (security hardening)
- Connection cap: 200 max (prevents O(N^2) memory exhaustion)
- Handshake timeout: 5s (`HANDSHAKE_TIMEOUT_SECS`)
- Keepalive: server sends Pong at 90s silence, declares dead at 120s
- Writer channel: mpsc with cap 256 -- `try_send()` drops frames when full (backpressure)
- Line length limit: 64KB per NDJSON line (DoS protection)
- C5: Lock ordering -- AppState read before BusState write in handshake validation

**Spec refs:** `ai_specs/IPC_BUS_SPEC.md`, `ai_specs/WIRE_PROTOCOL_SPEC.md`, `ai_specs/SECURITY_SPEC.md`

---

## 7. NDJSON Wire Protocol Frame Flow

The IPC bus uses NDJSON (newline-delimited JSON) over Unix sockets. Each line is a complete
JSON object tagged with `"type"`. Client frames (12 types) flow client-to-server; server
frames (8 types) flow server-to-client. Both directions are framed identically: one JSON
object per line, terminated by `\n`.

```mermaid
graph LR
    subgraph ClientToServer["Client → Server (12 frame types)"]
        style ClientToServer fill:#1a3a5c,color:#fff

        subgraph Lifecycle["Connection Lifecycle"]
            CH["Handshake<br/>{sphere_id, version}"]
            CP["Ping"]
        end

        subgraph Subscriptions["Event Subscriptions"]
            CS["Subscribe<br/>{patterns: [glob]}"]
            CU["Unsubscribe<br/>{patterns: [glob]}"]
        end

        subgraph TaskOps["Task Operations"]
            TS["TaskSubmit<br/>{id, description,<br/>target, payload,<br/>timeout_secs}"]
            TC["TaskClaim<br/>{task_id}"]
            TD["TaskComplete<br/>{task_id, result}"]
            TF["TaskFail<br/>{task_id, error}"]
        end

        subgraph ReqRes["Request/Response"]
            RQ["Request<br/>{id, to, payload}"]
            RS["Response<br/>{id, payload}"]
        end

        subgraph Cascade["Cascade"]
            CO["CascadeHandoff<br/>{handoff_id, target_sphere,<br/>cwd, current_task,<br/>files_modified,<br/>context_state,<br/>continuation_prompt}"]
            CA["CascadeAck<br/>{handoff_id,<br/>status: Accepted|Busy|Error}"]
        end
    end

    subgraph ServerToClient["Server → Client (8 frame types)"]
        style ServerToClient fill:#5c1a3a,color:#fff

        subgraph SLifecycle["Connection Lifecycle"]
            SH["HandshakeOk<br/>{tick, peer_count,<br/>r, protocol_version}"]
            SP["Pong"]
        end

        subgraph SEvents["Event Stream"]
            SE["Event<br/>{seq, event_type, data}"]
        end

        subgraph STask["Task Routing"]
            SA["TaskAssigned<br/>{task_id, description,<br/>payload, from}"]
        end

        subgraph SReqRes["Request Forwarding"]
            SR["RequestForYou<br/>{id, from, payload}"]
        end

        subgraph SCascade["Cascade Delivery"]
            SC["CascadeHandoffForYou<br/>{handoff_id, from,<br/>cwd, current_task,<br/>files_modified,<br/>context_state,<br/>continuation_prompt}"]
        end

        subgraph SSuggest["Field Suggestions"]
            SS["Suggestion<br/>{suggestion_id,<br/>suggestion_type,<br/>description,<br/>affinity, context}"]
        end

        subgraph SError["Errors"]
            ER["Error<br/>{code, message, context}"]
        end
    end

    CH -->|"validate sphere"| SH
    CP -->|"keepalive"| SP
    CS -->|"register patterns"| SE
    TS -->|"queue + route"| SA
    RQ -->|"forward to target"| SR
    CO -->|"deliver to target"| SC
```

**Error codes:**
| Code | Name | Meaning |
|------|------|---------|
| 1000 | UNKNOWN_SPHERE | Sphere not registered in AppState |
| 1001 | NOT_AUTHENTICATED | First frame was not Handshake |
| 1002 | INVALID_FRAME | Malformed JSON or unknown frame type |
| 1003 | LINE_TOO_LONG | NDJSON line exceeds 64KB |
| 2000 | QUEUE_FULL | Task queue at 1000 cap |
| 2001 | TASK_NOT_FOUND | Task ID not in queue |
| 2002 | TASK_ALREADY_CLAIMED | Another sphere claimed first |
| 2003 | TASK_NOT_YOURS | Claimer mismatch on complete/fail |
| 2004 | TASK_EXPIRED | Task TTL exceeded |
| 3000 | TARGET_NOT_FOUND | Cascade/request target offline |

**Spec refs:** `ai_specs/WIRE_PROTOCOL_SPEC.md`, `src/bus.rs` (frame definitions), `src/ipc.rs` (handler)

---

## 8. Lock Acquisition Sequence

The tick loop acquires locks in a strict order: AppState write lock first, then (after release)
BusState for event broadcasting. This ordering is a **critical safety invariant** (C5 in the
anti-patterns library). Inverting the order risks deadlock because the IPC handler acquires
BusState then reads AppState for sphere validation.

The tick is divided into phases: phases 1-4 operate under the AppState write lock (coupling,
Hebbian, field compute, conductor). Phase 5 (bus events) operates under BusState. Bridge polls
(SYNTHEX, Nexus, ME) use independent `Arc<RwLock>` caches and only touch AppState through the
main write lock.

```mermaid
sequenceDiagram
    participant TL as tick_once()
    participant AS as AppState<br/>(RwLock<AppState>)
    participant TB as ThermalState<br/>(RwLock<Option<ThermalState>>)
    participant NX as NexusState<br/>(RwLock<Option<NexusFieldState>>)
    participant MS as MeState<br/>(RwLock<Option<MeHealthState>>)
    participant NB as NexusBus<br/>(RwLock<NexusBusState>)
    participant BS as BusState<br/>(RwLock<BusState>)
    participant DB as SQLite<br/>(bus_tracking + field_tracking)

    Note over TL,AS: PHASE 1-4: Under AppState WRITE lock
    TL->>AS: state.write().await
    activate AS

    Note over AS: Phase 1: Sync network → spheres<br/>Phase 2: Step spheres, sync back<br/>NA-22: Per-status K modulation<br/>M14: Periodic auto_scale_k

    Note over AS: Phase 3: Kuramoto coupling steps<br/>(adaptive count: spatial + temporal)

    Note over AS: Phase 3b: Hebbian learning<br/>(LTP + LTD + burst detection)

    Note over AS: Phase 4: Field compute + decision<br/>FieldState::compute(), FieldDecision<br/>PI conductor, phase noise, breathing

    Note over TL,MS: Bridge reads (short-lived read locks)
    TL->>TB: thermal.read().await (brief)
    TB-->>TL: Option<ThermalState>
    Note over TL: thermal_k_adj → consent_gate → k_mod *=

    TL->>NX: nexus.read().await (brief)
    NX-->>TL: Option<NexusFieldState>
    Note over TL: nexus_k_adj → consent_gate → k_mod *=

    TL->>MS: me_state.read().await (brief)
    MS-->>TL: Option<MeHealthState>
    Note over TL: me_k_adj → consent_gate → k_mod *=

    TL->>NB: nexus_bus.read().await (brief)
    NB-->>TL: NexusBusState readings
    Note over TL: apply_readings → consent_gate → k_mod *=

    Note over TL: Combined bridge clamp [0.85, 1.15]<br/>Global k_mod clamp [-0.5, 1.5]

    TL->>AS: Release write lock (drop guard)
    deactivate AS

    Note over TL,BS: PHASE 5: Bus events (AFTER AppState release)
    TL->>BS: bus.write().await
    activate BS

    Note over BS: Broadcast field events:<br/>field.tick, field.decision,<br/>bridge.stale/recovered,<br/>field.suggestion

    TL->>BS: Release bus lock
    deactivate BS

    Note over TL,DB: PHASE 6: Persistence (no locks held)
    TL->>DB: field_snapshot (every 12 ticks)
    TL->>DB: sphere_snapshot (per sphere)
    TL->>DB: prune_old_data (every 100 ticks)

    rect rgb(139, 0, 0)
        Note over AS,BS: C5 INVARIANT: NEVER acquire BusState<br/>while holding AppState write lock.<br/>IPC handler does: BusState → AppState.read()<br/>Tick loop does: AppState.write() → release → BusState<br/>Inverting causes DEADLOCK.
    end
```

**Key constraints:**
- C5 lock ordering: AppState BEFORE BusState -- NEVER invert
- Bridge state caches (thermal, nexus, me, nexus_bus) use independent `Arc<RwLock>` -- short-lived reads while holding AppState
- The IPC handler acquires BusState first, then AppState read -- this is safe because tick_once releases AppState before acquiring BusState
- SQLite operations happen with NO locks held (Phase 6) -- they use their own connection pool
- SIGTERM handler: `tokio::time::timeout(5s, state.write())` -- will skip snapshot if tick loop holds lock

**Spec refs:** `ai_specs/patterns/CONCURRENCY_PATTERNS.md` (C5), `ai_specs/patterns/ASYNC_PATTERNS.md`, `src/main.rs:514+`

---

## 9. Distributed Brain Anatomy

The ULTRAPLATE service mesh forms a distributed brain, with each service playing a neurological
role. Pane-vortex (Cerebellum) coordinates the fleet through Kuramoto coupling -- it does not
command but orchestrates rhythm. The ME's severed nerve (BUG-008) means the autonomic nervous
system cannot send signals to the cerebral cortex.

```mermaid
graph TB
    subgraph Brain["Distributed Brain Anatomy"]
        SX["SYNTHEX :8090<br/><b>Cerebral Cortex</b><br/>43K LOC, v3 homeostasis<br/>thermal PID, heat sources<br/>highest-level integration"]
        style SX fill:#4a0a6a,color:#fff

        RM["Reasoning Memory :8130<br/><b>Prefrontal Cortex</b><br/>Cross-session TSV store<br/>deliberative reasoning<br/>context continuity"]
        style RM fill:#6a0a4a,color:#fff

        K7["SAN-K7 :8100<br/><b>Basal Ganglia</b><br/>59 modules, M1-M55<br/>strategy selection<br/>outer Kuramoto field"]
        style K7 fill:#0a4a6a,color:#fff

        PV["Pane-Vortex :8132<br/><b>Cerebellum</b><br/>Kuramoto fleet coordination<br/>Hebbian STDP learning<br/>rhythm, not command"]
        style PV fill:#0a6a4a,color:#fff

        VMS["Vortex Memory :8120<br/><b>Hippocampus</b><br/>OVM + POVM bridge<br/>memory consolidation<br/>long-term storage"]
        style VMS fill:#4a6a0a,color:#fff

        ME["Maint. Engine :8080<br/><b>Autonomic NS</b><br/>7 layers, RALPH evolution<br/>12D tensor, PBFT consensus<br/>BUG-008: SEVERED NERVE"]
        style ME fill:#8b0000,color:#fff

        POVM["POVM Engine :8125<br/><b>Spinal Cord</b><br/>Persistent vortex memory<br/>pathway weights<br/>field snapshots"]
        style POVM fill:#6a4a0a,color:#fff
    end

    %% Active connections
    SX <-->|"thermal bridge<br/>bidirectional REST"| PV
    K7 <-->|"nexus bridge<br/>nested Kuramoto"| PV
    PV -->|"field snapshots<br/>Hebbian weights"| POVM
    POVM -->|"hydrate pathways<br/>on startup"| PV
    PV -->|"field state TSV<br/>task events"| RM
    RM -->|"bootstrap weights<br/>on startup"| PV
    PV <-->|"memory bridge<br/>health read"| VMS
    ME -->|"fitness score<br/>(frozen 0.3662)"| PV

    %% NexusBus bridges
    PV <-.->|"NexusBus: CsV7<br/>neural graph"| CSV7["CodeSynthor :8110"]
    PV <-.->|"NexusBus: ToolLib<br/>STDP learning"| TLIB["Tool Library :8105"]
    PV <-.->|"NexusBus: MeObserver<br/>RALPH evolution"| ME

    %% Missing connection
    MISSING["MISSING BRIDGE<br/>ME → SYNTHEX<br/>(no EventBus publishers)"]
    style MISSING fill:#8b0000,color:#fff,stroke:#ff0000,stroke-width:3px,stroke-dasharray: 5 5

    ME -.->|"SEVERED"| MISSING
    MISSING -.->|"should feed<br/>HS-003"| SX

    %% Brain-to-function mapping
    SX -.->|"integration"| K7
    K7 -.->|"strategy"| PV
    PV -.->|"rhythm"| VMS
    VMS -.->|"memory"| RM
    RM -.->|"reasoning"| SX
```

**Key observations:**
- PV (Cerebellum) has 6 direct bridges: SYNTHEX, Nexus/SAN-K7, ME, POVM, RM, VMS
- PV has 5 NexusBus bridges: CsV7, ToolLibrary, DevEnvPatterns (local SQLite), VmsRead, MeObserver
- The ME's severed nerve (BUG-008) means fitness is frozen -- the autonomic system cannot signal distress
- SYNTHEX integrates all signals into its thermal PID controller -- it is the highest-level convergence point
- The RM-to-SYNTHEX path is indirect (through PV's field state push) -- there is no direct RM-SYNTHEX bridge
- POVM is strictly a persistence layer -- it stores and retrieves but does not compute

**Spec refs:** Obsidian `[[Session 036 — Complete Architecture Schematics]]`, `[[Vortex Sphere Brain-Body Architecture]]`

---

## 10. Full System Wiring (16 Services to PV)

All 16 active ULTRAPLATE services organized by startup batch (1-5). Pane-vortex sits in Batch 5
(depends on POVM Engine and SYNTHEX). The diagram shows which services PV bridges to directly
(6 bridges), which it reaches through NexusBus (5 bridges), and which it has no direct connection to.

```mermaid
graph TB
    subgraph Batch1["Batch 1 (no dependencies)"]
        style Batch1 fill:#1a1a3a,color:#fff
        DE["DevOps Engine :8081"]
        CSV7["CodeSynthor V7 :8110"]
        POVM["POVM Engine :8125"]
    end

    subgraph Batch2["Batch 2 (needs Batch 1)"]
        style Batch2 fill:#2a1a3a,color:#fff
        SX["SYNTHEX :8090"]
        K7["SAN-K7 :8100"]
        ME["Maint. Engine :8080"]
        AA["Architect Agent :9001+"]
        PS["Prometheus Swarm :10001+"]
    end

    subgraph Batch3["Batch 3 (needs Batch 2)"]
        style Batch3 fill:#3a1a3a,color:#fff
        NAIS["NAIS :8101"]
        BE["Bash Engine :8102"]
        TM["Tool Maker :8103"]
    end

    subgraph Batch4["Batch 4 (needs Batch 3)"]
        style Batch4 fill:#4a1a3a,color:#fff
        CCM["Context Manager :8104"]
        TLIB["Tool Library :8105"]
        RM["Reasoning Memory :8130"]
    end

    subgraph Batch5["Batch 5 (needs Batch 4 + POVM + SYNTHEX)"]
        style Batch5 fill:#0a4a2a,color:#fff
        VMS["Vortex Memory :8120"]
        PV["PANE-VORTEX :8132<br/>Fleet Coordination<br/>Kuramoto Cerebellum"]
        style PV fill:#0a6a4a,color:#fff,stroke:#00ff00,stroke-width:3px
    end

    %% Direct bridges (6) — solid lines
    PV <====>|"SYNTHEX Bridge<br/>thermal_k_adj [0.8,1.2]<br/>bidirectional REST"| SX
    PV <====>|"Nexus Bridge<br/>nested Kuramoto<br/>nexus_k_adj [0.85,1.15]"| K7
    PV ====>|"POVM Bridge<br/>field snapshots (12 ticks)<br/>Hebbian weights (60 ticks)"| POVM
    POVM ====>|"Hydration<br/>pathways + summary"| PV
    PV <====>|"ME Bridge<br/>me_k_adj [0.95,1.03]<br/>read-only + consent"| ME
    PV <====>|"RM Bridge<br/>TSV POST + bootstrap"| RM
    PV <====>|"VMS Bridge<br/>memory + health"| VMS

    %% NexusBus bridges (5) — dashed lines
    PV <-.->|"NexusBus:<br/>CsV7 neural graph"| CSV7
    PV <-.->|"NexusBus:<br/>ToolLib STDP"| TLIB
    PV <-.->|"NexusBus:<br/>DevEnv patterns<br/>(local SQLite)"| DE
    PV <-.->|"NexusBus:<br/>VMS health read"| VMS
    PV <-.->|"NexusBus:<br/>MeObserver RALPH"| ME

    %% No direct connection — dotted
    PV -.-|"no direct bridge"| NAIS
    PV -.-|"no direct bridge"| BE
    PV -.-|"no direct bridge"| TM
    PV -.-|"no direct bridge"| CCM
    PV -.-|"no direct bridge"| AA
    PV -.-|"no direct bridge"| PS

    %% Legend
    subgraph Legend["Connection Types"]
        style Legend fill:#0a0a0a,color:#fff
        L1["====  Direct Bridge (6)"]
        L2["- - - NexusBus Bridge (5)"]
        L3[". . . No Direct Connection (6)"]
    end
```

**Service inventory (16 active):**
| Service | Port | Bridge | k_adj Range | Poll Interval |
|---------|------|--------|-------------|---------------|
| SYNTHEX | 8090 | Direct (synthex_bridge.rs) | [0.8, 1.2] | 6 ticks / 25s |
| SAN-K7 | 8100 | Direct (nexus_bridge.rs) | [0.85, 1.15] | 12 ticks / 55s |
| Maint. Engine | 8080 | Direct (me_bridge.rs) + NexusBus | [0.95, 1.03] | 12 ticks / 55s |
| POVM Engine | 8125 | Direct (povm_bridge.rs) | Write-only | 12/60 ticks |
| Reasoning Memory | 8130 | Direct (bridge.rs) | Startup + periodic | 12 ticks |
| Vortex Memory | 8120 | Direct (vms_bridge.rs) + NexusBus | Read health | Startup + poll |
| CodeSynthor V7 | 8110 | NexusBus (cs_v7.rs) | Consent-gated | 60 ticks |
| Tool Library | 8105 | NexusBus (tool_library.rs) | Consent-gated | 60 ticks |
| DevOps Engine | 8081 | NexusBus (devenv_patterns.rs) | Consent-gated | 60 ticks |

**Disabled services:** library-agent (8083), sphere-vortex (8120, VMS owns port)

**Key constraints:**
- PV depends on POVM Engine + SYNTHEX at startup (Batch 5)
- All bridge k_adjustments pass through consent gate before touching k_modulation
- Combined bridge budget clamp [0.85, 1.15] prevents compounding
- NexusBus bridges poll less frequently (60 ticks = 5 min) than direct bridges
- 3.6: Startup smoke test probes all 6 direct bridges and logs WARN for unreachable ones

**Spec refs:** `~/.config/devenv/devenv.toml` (service definitions), `CLAUDE.md` (batch ordering), `src/main.rs` (smoke test)
