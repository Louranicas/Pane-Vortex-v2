# Schematics: Field & Governance (Medium + Low Priority)

> 10 architecture diagrams covering bridge protocols, coupling topology,
> detection algorithms, phase space, governance lifecycle, and data flows.
>
> **Companion file:** `SCHEMATICS.md` (high-priority system context, module deps, tick loop, API routes, lifecycle FSMs, IPC bus, cascade flow, bridge topology, field computation)
>
> **Cross-references:** `ai_specs/KURAMOTO_FIELD_SPEC.md` | `ai_specs/IPC_BUS_SPEC.md` | `ai_specs/API_SPEC.md` | `ai_specs/DATABASE_SPEC.md` | `ai_specs/SECURITY_SPEC.md`

---

## 1. POVM Bidirectional Bridge

The POVM Engine (port 8125) provides cross-session persistence for the Kuramoto field state and Hebbian coupling weights. The bridge uses raw TCP HTTP (no client library) with fire-and-forget semantics on the write path and blocking hydration on the read path during startup only.

All constants are defined in `src/povm_bridge.rs`:
- `POVM_SYNC_INTERVAL` = 12 ticks (every ~60s at 5s/tick)
- `HEBBIAN_SYNC_MULTIPLIER` = 5 (every 60 ticks, ~5min)
- `TIMEOUT_SECS` = 3 (connect + read timeout per request)

```mermaid
sequenceDiagram
    participant PV as Pane-Vortex<br/>:8132
    participant POVM as POVM Engine<br/>:8125

    Note over PV: === STARTUP (Read Path) ===

    PV->>POVM: GET /pathways
    POVM-->>PV: Vec<PovmPathway><br/>{pre_id, post_id, weight}
    Note over PV: hydrate_pathways()<br/>Seeds CouplingNetwork<br/>with persisted Hebbian weights

    PV->>POVM: GET /hydrate
    POVM-->>PV: PovmHydrationSummary<br/>{memory_count, pathway_count, latest_r}
    Note over PV: hydrate_summary()<br/>Log startup state,<br/>optional r seeding

    Note over PV: === TICK LOOP (Write Path) ===

    loop Every 12 ticks (~60s)
        PV->>POVM: POST /snapshots<br/>{session_id, coefficients: [[0,0,r]],<br/>kuramoto_r, kuramoto_k,<br/>memory_count, crystallised_count}
        Note right of POVM: Fire-and-forget<br/>No response checked
    end

    loop Every 60 ticks (~5min)
        PV->>POVM: POST /pathways (per edge)<br/>{pre_id, post_id, weight}
        Note right of POVM: Iterates all connections<br/>One POST per (from, to, weight)<br/>Fire-and-forget
    end

    Note over PV: === SHUTDOWN ===

    PV->>POVM: POST /snapshots (final flush)
    PV->>POVM: POST /pathways (final flush)
    Note over PV: SIGTERM handler:<br/>write lock + 3s timeout<br/>before process exit
```

**Key constraints:**
- Write path is fire-and-forget: POVM downtime does not affect PV operation.
- Read path runs once at startup, after RM bootstrap (Hebbian seeding order matters).
- Raw TCP HTTP avoids adding `reqwest`/`hyper` client dependencies.
- Shutdown flush uses tokio `timeout(3s)` to prevent hanging on SIGTERM.
- Address configurable via `POVM_ADDR` env var (default `127.0.0.1:8125`).

**Cross-references:** `src/povm_bridge.rs` | `ai_specs/DATABASE_SPEC.md` (field_snapshots table) | `CLAUDE.md` (POVM Bridge Details)

---

## 2. Sidecar WASM-to-Bus Bridge

The Swarm WASM plugin runs inside the Zellij WASI sandbox, which cannot hold Unix sockets. The `swarm-sidecar` Rust binary bridges the gap using filesystem intermediaries: a named FIFO pipe for commands (WASM-to-bus) and a ring-buffered JSONL file for events (bus-to-WASM).

Constants from `swarm-sidecar/src/main.rs`:
- `RING_CAP` = 1000 lines (event ring buffer)
- `MAX_RETRIES` = 3 (reconnect attempts with exponential backoff)
- `EVENTS_PATH` = `/tmp/swarm-events.jsonl`
- `COMMANDS_PATH` = `/tmp/swarm-commands.pipe`

```mermaid
graph LR
    subgraph Zellij["Zellij WASI Sandbox"]
        WASM["Swarm WASM Plugin<br/>(swarm-orchestrator)"]
    end

    subgraph FS["Filesystem Intermediaries"]
        PIPE["/tmp/swarm-commands.pipe<br/>(Named FIFO)"]
        RING["/tmp/swarm-events.jsonl<br/>(Ring file, 1000 lines)"]
    end

    subgraph Sidecar["swarm-sidecar<br/>(Rust binary, persistent)"]
        CMD_RD["Command Reader<br/>(async FIFO read)"]
        EVT_WR["Event Writer<br/>(ring buffer flush)"]
        CONN["Unix Socket Connection<br/>(BufReader/BufWriter)"]
        RECONN["Reconnect Logic<br/>(3 retries, exp backoff)"]
        HSHK["Handshake<br/>(sphere_id: swarm-sidecar)"]
    end

    subgraph PVBus["Pane-Vortex IPC Bus<br/>(Unix socket)"]
        ACCEPT["Connection Handler"]
        EVENTS["Event Broadcast<br/>(field.*, cascade.*, task.*)"]
    end

    WASM -->|"write NDJSON<br/>commands"| PIPE
    PIPE --> CMD_RD
    CMD_RD -->|"ClientFrame<br/>(TaskSubmit, CascadeHandoff,<br/>Request, Subscribe)"| CONN
    CONN -->|"NDJSON over<br/>Unix socket"| ACCEPT
    ACCEPT -->|"ServerFrame<br/>(Event, TaskAssigned,<br/>Pong, Error)"| CONN
    CONN --> EVT_WR
    EVT_WR -->|"append NDJSON<br/>(truncate at 1000 lines)"| RING
    RING -->|"read tail<br/>poll interval"| WASM
    CONN -.->|"disconnect"| RECONN
    RECONN -.->|"retry 1s/2s/4s"| CONN
    HSHK -->|"first frame"| CONN

    style PIPE fill:#ff9,stroke:#333
    style RING fill:#9ff,stroke:#333
```

**Key constraints:**
- WASM plugins cannot open Unix sockets (WASI limitation) -- filesystem is the only IPC channel.
- The event ring file is truncated to `RING_CAP` lines to prevent unbounded growth.
- Commands pipe is a named FIFO (`mkfifo`); sidecar reopens on EOF to handle plugin restarts.
- Sidecar subscribes to `field.*`, `task.*`, `cascade.*`, `decision.*` event patterns.
- Rate limiting on the command path: sidecar validates frame structure before forwarding.
- Graceful shutdown on SIGTERM/SIGINT writes final event and removes PID file.

**Cross-references:** `swarm-sidecar/src/main.rs` (380 LOC) | `swarm-orchestrator/src/lib.rs` (67 tests) | `src/client.rs` (Sidecar subcommand) | `ai_specs/IPC_BUS_SPEC.md`

---

## 3. Coupling Network Topology

The Kuramoto coupling network is a fully-connected directed graph where each pair of spheres has two directed edges (ordered-pair keys). Edges carry a base `weight` (Hebbian-learned, 0.0..1.0) and a `type_weight` (status-based modifier). The effective coupling is `weight * type_weight`.

An adjacency index (`adj_index: HashMap<PaneId, Vec<usize>>`) maps each sphere to its outgoing connection indices for O(degree) lookup instead of O(E) linear scan.

```mermaid
graph TD
    subgraph Network["Coupling Network (5 spheres, 20 directed edges)"]
        A["Sphere A<br/>phase=0.3 rad<br/>freq=0.42 Hz<br/>status=Working"]
        B["Sphere B<br/>phase=1.1 rad<br/>freq=0.85 Hz<br/>status=Working"]
        C["Sphere C<br/>phase=2.8 rad<br/>freq=0.21 Hz<br/>status=Idle"]
        D["Sphere D<br/>phase=4.2 rad<br/>freq=1.37 Hz<br/>status=Working"]
        E["Sphere E<br/>phase=5.5 rad<br/>freq=0.63 Hz<br/>status=Blocked"]
    end

    A -- "w=0.82 tw=1.2<br/>eff=0.984" --> B
    B -- "w=0.75 tw=1.2<br/>eff=0.900" --> A
    A -- "w=0.30 tw=0.6<br/>eff=0.180" --> C
    C -- "w=0.30 tw=0.6<br/>eff=0.180" --> A
    A -- "w=0.55 tw=1.2<br/>eff=0.660" --> D
    D -- "w=0.48 tw=1.2<br/>eff=0.576" --> A
    B -- "w=0.65 tw=1.2<br/>eff=0.780" --> D
    D -- "w=0.60 tw=1.2<br/>eff=0.720" --> B
    A -. "w=0.15 tw=0.0<br/>eff=0.000<br/>(Blocked)" .-> E
    E -. "w=0.15 tw=0.0<br/>eff=0.000<br/>(Blocked)" .-> A

    subgraph Legend["Weight Legend"]
        L1["Default: w=0.30, tw=0.60 → eff=0.18"]
        L2["NA-22 type_weight:<br/>Working×Working = 1.2<br/>Working×Idle = 0.6<br/>Mixed = 0.5<br/>Blocked×Any = 0.0"]
        L3["Hebbian range: w ∈ [0.15, 1.0]<br/>Floor = HEBBIAN_WEIGHT_FLOOR"]
        L4["Coupling: K_eff × w × tw × sin(phi_j - phi_i)"]
    end

    style A fill:#4a4,stroke:#333,color:#fff
    style B fill:#4a4,stroke:#333,color:#fff
    style C fill:#aa4,stroke:#333,color:#fff
    style D fill:#4a4,stroke:#333,color:#fff
    style E fill:#a44,stroke:#333,color:#fff
```

**Key constraints:**
- Edges use ordered-pair keys: `(from, to)` is distinct from `(to, from)` when `asymmetric_hebbian=true`.
- Default `asymmetric_hebbian=false` keeps both directions symmetric on `set_weight()`.
- New registrations create connections to ALL existing spheres (fully connected).
- `adj_index` is rebuilt on every `register()`/`deregister()` call.
- Frequency is hash-scaled: `base_freq * hash_scale` where `hash_scale` in [0.2, 2.0] with 10K bins.
- Integration uses Jacobi method: snapshot all phases, then update (no in-place mutation during step).

**Cross-references:** `src/coupling.rs` | `ai_specs/KURAMOTO_FIELD_SPEC.md` | `ai_specs/patterns/CONCURRENCY_PATTERNS.md` (CP-7: Jacobi integration)

---

## 4. Auto-K Feedback Loop

The coupling strength K is automatically scaled to maintain the system near the critical coupling threshold (Kuramoto transition). The conductor PI controller then modulates `k_modulation` to push the system sub- or supercritical depending on the field decision. External bridges (SYNTHEX, NexusForge, ME) further influence `k_modulation` through the consent gate.

```mermaid
flowchart TD
    subgraph AutoK["auto_scale_k() — every 20 ticks"]
        FREQ["Collect all frequencies<br/>freqs: Vec&lt;f64&gt;"]
        SPREAD["Compute spread<br/>spread = max(freq) - min(freq)"]
        KC["K_c = 2 × spread / π × N<br/>(critical coupling for<br/>uniform g(ω))"]
        CAP["K = min(K_c, N)<br/>(cap at sphere count)"]
    end

    subgraph Conductor["PI Conductor — every tick"]
        R_TARGET["r_target(state)<br/>base=0.93, scales for<br/>large fleets, blends<br/>fleet preferred_r"]
        ERROR["error = r - r_target"]
        PI["k_mod += P×error + I×∫error<br/>clamp to [K_MOD_MIN, K_MOD_MAX]<br/>= [-0.5, 1.5]"]
    end

    subgraph Bridges["External Bridge Influence"]
        SX["SYNTHEX thermal_k_adjustment<br/>cold → 1.2× (boost)<br/>hot → 0.8× (reduce)"]
        NX["NexusForge nexus_k_adjustment<br/>Aligned → 1.1×<br/>Diverging → 0.9×<br/>Incoherent → 0.85×"]
        ME["ME me_k_adjustment<br/>fitness-based [0.9, 1.1]"]
        CONSENT["consent_gated_k_adjustment()<br/>Scales by: mean receptivity,<br/>newcomer damping, eligible fraction,<br/>divergence exemption"]
    end

    subgraph Coupling["Kuramoto Step"]
        KEFF["K_eff = K × k_modulation"]
        STEP["d_phi/dt = ω_i + receptivity_i ×<br/>(K_eff/N) × Σ w_ij × sin(φ_j - φ_i)"]
        R_OUT["r = |mean(e^(iφ))| ∈ [0, 1]"]
    end

    FREQ --> SPREAD --> KC --> CAP
    CAP -->|"Sets base K"| KEFF

    R_OUT -->|"Measured r"| ERROR
    R_TARGET --> ERROR
    ERROR --> PI
    PI -->|"k_modulation"| KEFF

    SX --> CONSENT
    NX --> CONSENT
    ME --> CONSENT
    CONSENT -->|"multiplicative"| PI

    KEFF --> STEP --> R_OUT

    style AutoK fill:#e8f4e8,stroke:#333
    style Conductor fill:#e8e8f4,stroke:#333
    style Bridges fill:#f4e8e8,stroke:#333
    style Coupling fill:#f4f4e8,stroke:#333
```

**Key constraints:**
- `auto_scale_k` runs every 20 ticks (configurable), NOT every tick -- avoids K oscillation.
- K_c formula assumes uniform frequency distribution: `K_c = 2 * spread / pi * N`.
- K is capped at N to prevent runaway coupling in small fleets.
- `k_modulation` is clamped to `[K_MOD_MIN, K_MOD_MAX]` = `[-0.5, 1.5]` (authoritative values in `conductor.rs`).
- All external bridge adjustments route through `consent_gated_k_adjustment()` for NA compliance.
- The consent gate checks: receptivity, opt-out flags, newcomer protection, divergence exemption.
- Negative k_modulation produces repulsive coupling (deliberate desynchronization).

**Cross-references:** `src/coupling.rs` (auto_scale_k) | `src/conductor.rs` (K_MOD_MIN/MAX, r_target) | `src/nexus_bridge.rs` (consent_gated_k_adjustment) | `src/main.rs` (tick loop wiring)

---

## 5. Chimera Detection Algorithm

Chimera states are the coexistence of synchronized and desynchronized clusters in the same oscillator network. Detection uses the phase-gap method with adaptive gap threshold that scales with `k_modulation`.

Algorithm complexity: O(N log N) from the sort step.

```mermaid
flowchart TD
    START["ChimeraState::detect(network)"]
    GUARD{"N < 2?"}
    SORT["Sort spheres by phase<br/>O(N log N)<br/>phases mod 2π"]
    GAPS["Find gaps > threshold<br/>Scan sorted list + wraparound<br/>O(N)"]
    THRESH["effective_gap_threshold(k_mod):<br/>k_mod > 0: π/3 × clamp(k_mod, 0.3, 1.5)<br/>→ range [π/6, π/3]<br/>k_mod < 0: π/6 → π/12<br/>(finer detection during desync)"]
    NOGAP{"Any gaps<br/>found?"}
    ONECLUSTER["Single cluster:<br/>all spheres together"]
    SPLIT["Split at gap boundaries<br/>→ Vec&lt;Vec&lt;PaneId&gt;&gt;"]
    CLASSIFY["For each cluster:<br/>local_r = |mean(e^(iφ))|<br/>over cluster members"]
    SYNC{"local_r ≥ 0.5?"}
    SYNC_C["→ sync_clusters"]
    DESYNC_C["→ desync_clusters"]
    CHIMERA{"Has multi-member<br/>sync cluster AND<br/>any desync cluster?"}
    YES["is_chimera = true"]
    NO["is_chimera = false"]
    ROUTE["route_focused() → largest sync cluster<br/>route_exploratory() → all desync members"]

    START --> GUARD
    GUARD -->|"Yes"| NO
    GUARD -->|"No"| SORT
    SORT --> GAPS
    THRESH -.->|"threshold"| GAPS
    GAPS --> NOGAP
    NOGAP -->|"No gaps"| ONECLUSTER --> CLASSIFY
    NOGAP -->|"Yes"| SPLIT --> CLASSIFY
    CLASSIFY --> SYNC
    SYNC -->|"Yes"| SYNC_C
    SYNC -->|"No"| DESYNC_C
    SYNC_C --> CHIMERA
    DESYNC_C --> CHIMERA
    CHIMERA -->|"Yes"| YES --> ROUTE
    CHIMERA -->|"No"| NO --> ROUTE

    style THRESH fill:#ffd,stroke:#333
    style CHIMERA fill:#fdf,stroke:#333
```

**Key constraints:**
- `SYNC_THRESHOLD` = 0.5 (local order parameter threshold for sync classification).
- Gap threshold is adaptive: scales with `k_modulation` for sensitivity in different regimes.
- Negative k_mod (repulsive coupling) uses finer detection: `[pi/12, pi/6]` range.
- Single-member clusters are NOT counted as evidence for chimera (prevents false positives).
- `local_order_parameter()` uses `found.len()` as denominator (not `members.len()`), handling missing phases.
- Wraparound gap (last element to first) is explicitly computed.
- Route functions: `route_focused()` returns the largest sync cluster; `route_exploratory()` returns all desync members.

**Cross-references:** `src/chimera.rs` | `ai_specs/KURAMOTO_FIELD_SPEC.md` (chimera section) | `src/conductor.rs` (K_MOD_MIN for negative threshold branch)

---

## 6. Phase Space Visualization

The Kuramoto order parameter r measures phase coherence on the unit circle. This cannot be rendered in Mermaid (no polar coordinates), so an ASCII representation illustrates the three key regimes.

```
                    COHERENT (r → 1.0)              CHIMERA (r ~ 0.5-0.7)           INCOHERENT (r → 0)
                    Tight cluster                    Two clusters                     Uniform spread

                        ·  B                              C                               E
                      · A ·                           ·     ·                          ·       ·
                     ·  C  ·                         ·       ·                        ·    B    ·
                    ·   D   ·                    A ·           · D                   ·           ·
                    ·       ·                    B ·           · E                F ·           · A
                     ·     ·                         ·       ·                        ·    D    ·
                      ·   ·                           ·     ·                          ·       ·
                        ·                               F                               C

                    r = 0.98                         r = 0.62                         r = 0.08
                    K_eff > K_c                      K_eff ≈ K_c                     K_eff < K_c
                    All Working                      Sync: {A,B}                     Free-running
                    Decision: Stable                 Desync: {D,E,F}                 Decision: NeedsCoherence
                                                     Decision: chimera routing

    ┌──────────────────────────────────────────────────────────────────────────────────────────────┐
    │                                                                                              │
    │  ORDER PARAMETER:  r = |1/N × Σ e^(iφ_k)|     where φ_k ∈ [0, 2π)                          │
    │                                                                                              │
    │  MEAN PHASE:       ψ = arg(1/N × Σ e^(iφ_k))  (direction of centroid on unit circle)       │
    │                                                                                              │
    │  INTERPRETATION:                                                                             │
    │    r = 1.0  →  All oscillators phase-locked (identical phases)                               │
    │    r > 0.8  →  Strong synchronization (conductor may trigger NeedsDivergence if idle > 60%)  │
    │    r < 0.3  →  Weak coherence (conductor triggers NeedsCoherence if r is falling)            │
    │    r ≈ 0.0  →  Uniform distribution (maximum entropy, no coupling effect)                    │
    │                                                                                              │
    │  FIELD DECISIONS (priority chain):                                                           │
    │    HasBlockedAgents > NeedsCoherence (r>0.3, falling, ≥2 spheres)                           │
    │    > NeedsDivergence (r>0.8, idle>60%, ≥2 spheres)                                          │
    │    > IdleFleet > FreshFleet > Stable                                                         │
    │                                                                                              │
    │  R_THRESHOLDS:                                                                               │
    │    R_HIGH = 0.8    R_LOW = 0.3    R_FALLING = -0.03/tick                                    │
    │                                                                                              │
    └──────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Key constraints:**
- The "multi guard" (`spheres.len() >= 2`) prevents false coherence/divergence signals from single-sphere r=1.0.
- `Recovering` is returned during warmup (5 ticks after snapshot restore).
- `FreshFleet` is returned when spheres exist but none have the `has_worked` flag set.
- r history is maintained as a VecDeque of 60 samples (`R_HISTORY_MAX`), used to compute dr/dt for falling detection.

**Cross-references:** `src/field.rs` (FieldDecision, thresholds) | `src/coupling.rs` (order_parameter) | `src/state.rs` (R_HISTORY_MAX)

---

## 7. Proposal Lifecycle FSM

The governance proposal system (NA-P-15) enables spheres to collectively decide on field parameter changes. This is a planned feature (V3.4) building on the consent gate pattern established by `consent_gated_k_adjustment()`.

Proposals target parameter changes (r_target, k_mod bounds, coupling preferences) and require quorum to resolve.

```mermaid
stateDiagram-v2
    [*] --> Open : Sphere POST /governance/propose<br/>{parameter, value, justification}

    Open --> Voting : Voting window opens<br/>(default: 60s)

    state Voting {
        [*] --> Collecting
        Collecting --> Collecting : Sphere POST /governance/vote<br/>{proposal_id, vote: For|Against}
        Collecting --> EarlyResolve : All active spheres voted
        Collecting --> WindowExpired : voting_window expires
    }

    Voting --> QuorumCheck : Tally votes

    state QuorumCheck {
        [*] --> CheckQuorum
        CheckQuorum --> HasQuorum : voted_count >= ceil(active_spheres × 0.5)
        CheckQuorum --> NoQuorum : voted_count < quorum threshold
    }

    QuorumCheck --> Approved : Has quorum AND<br/>for_votes > 50% of votes cast
    QuorumCheck --> Rejected : Has quorum AND<br/>for_votes <= 50% of votes cast
    QuorumCheck --> Expired : No quorum reached

    Approved --> AutoApply : Apply parameter change<br/>(e.g., set r_target, adjust k_mod bounds)
    AutoApply --> Archived : Record in RM + POVM<br/>broadcast governance.resolved event

    Rejected --> Archived : Record rejection reason<br/>broadcast governance.resolved event

    Expired --> Archived : Record expiry<br/>broadcast governance.expired event

    Archived --> [*]

    note right of Open
        Proposal fields:
        - proposal_id (UUID)
        - proposer_sphere_id
        - parameter (enum)
        - proposed_value (f64)
        - justification (string, 256 char cap)
        - created_at (epoch)
        - voting_window_secs (default 60)
    end note

    note right of Approved
        Auto-apply targets:
        - r_target override
        - k_mod_min / k_mod_max
        - coupling preference
        - Hebbian opt-out flags
    end note
```

**Key constraints:**
- **STATUS: PLANNED (V3.4)** -- not yet implemented. Design based on NA-P-15 gap analysis.
- Quorum threshold: `ceil(active_spheres * 0.5)` (majority of active spheres must participate).
- Approval requires > 50% of cast votes to be `For` (simple majority).
- Voting window default: 60 seconds (configurable per proposal).
- Early resolution: if all active spheres have voted, skip waiting for window expiry.
- Each sphere may cast exactly one vote per proposal (duplicates rejected with error).
- Proposals are immutable after creation; only the voting state changes.
- Archived proposals are persisted to RM (TSV) and optionally POVM for cross-session continuity.

**Cross-references:** `CLAUDE.local.md` (V3.4 Governance phase) | `src/nexus_bridge.rs` (consent_gated_k_adjustment -- pattern to extend) | NA-P-15 gap analysis

---

## 8. Voting Quorum Flow

Detailed sequence for vote submission through resolution, showing validation checks and early-exit conditions.

```mermaid
sequenceDiagram
    participant S as Sphere<br/>(voter)
    participant API as PV HTTP API<br/>:8132
    participant GOV as Governance<br/>Module
    participant STATE as AppState<br/>(proposals map)
    participant BUS as IPC Bus<br/>(event broadcast)
    participant RM as Reasoning Memory<br/>:8130

    S->>API: POST /governance/vote<br/>{proposal_id, vote: For|Against}

    API->>GOV: validate_vote(proposal_id, sphere_id, vote)

    GOV->>STATE: lookup proposal
    alt Proposal not found
        GOV-->>API: 404 PROPOSAL_NOT_FOUND
        API-->>S: Error response
    end

    alt Proposal not Open/Voting
        GOV-->>API: 409 PROPOSAL_CLOSED
        API-->>S: Error response
    end

    GOV->>STATE: check already_voted(sphere_id)
    alt Already voted
        GOV-->>API: 409 ALREADY_VOTED
        API-->>S: Error response
    end

    GOV->>STATE: record_vote(sphere_id, vote)
    GOV-->>API: 200 Vote recorded

    GOV->>STATE: count active spheres
    GOV->>STATE: count votes cast

    alt All active spheres voted (early resolution)
        GOV->>GOV: tally_votes()
        Note over GOV: for_count, against_count,<br/>total = votes cast

        alt for_count > total / 2
            GOV->>STATE: set status = Approved
            GOV->>STATE: auto_apply(parameter, value)
            GOV->>BUS: broadcast governance.resolved<br/>{proposal_id, result: approved}
            GOV->>RM: POST /put (TSV)<br/>governance\tpane-vortex\t0.95\t604800\t<br/>Proposal approved: {parameter}={value}
        else for_count <= total / 2
            GOV->>STATE: set status = Rejected
            GOV->>BUS: broadcast governance.resolved<br/>{proposal_id, result: rejected}
            GOV->>RM: POST /put (TSV)<br/>governance\tpane-vortex\t0.90\t604800\t<br/>Proposal rejected: {parameter}
        end
    else Votes still outstanding
        Note over GOV: Wait for voting_window<br/>expiry (checked each tick)
    end

    API-->>S: 200 OK {vote_recorded: true,<br/>votes_cast, votes_remaining}

    Note over GOV: On tick: check expired proposals
    loop Every tick during voting window
        GOV->>STATE: check voting_window expiry
        alt Window expired
            GOV->>GOV: check quorum<br/>voted >= ceil(active × 0.5)
            alt Quorum met
                GOV->>GOV: tally and resolve
            else No quorum
                GOV->>STATE: set status = Expired
                GOV->>BUS: broadcast governance.expired
            end
        end
    end
```

**Key constraints:**
- **STATUS: PLANNED (V3.4)** -- design spec, not yet implemented.
- Vote validation order: proposal exists -> proposal is open -> sphere has not voted -> record vote.
- Early resolution triggers immediately when `votes_cast == active_sphere_count`.
- Quorum check only runs on window expiry (not on each vote, to avoid premature resolution edge case with sphere deregistration mid-vote).
- Active sphere count for quorum is computed at resolution time, not at proposal creation (handles dynamic fleet).
- RM entries use category `governance` with 7-day TTL (604800 seconds).

**Cross-references:** `src/bridge.rs` (post_reasoning_memory pattern) | `ai_specs/API_SPEC.md` (planned governance routes) | NA-P-15 gap analysis

---

## 9. Consent Declaration Data Flow

The consent gate is the central NA (Non-Anthropocentric) mechanism ensuring external bridge influences on the coupling field are proportional to fleet agreement. Every external k_modulation adjustment (from SYNTHEX thermal, NexusForge strategy, ME fitness) routes through `consent_gated_k_adjustment()`.

```mermaid
flowchart LR
    subgraph Spheres["Sphere Consent Signals"]
        SP_A["Sphere A<br/>receptivity=0.9<br/>opt_out=false<br/>steps=200"]
        SP_B["Sphere B<br/>receptivity=0.3<br/>opt_out=false<br/>steps=150"]
        SP_C["Sphere C<br/>receptivity=1.0<br/>opt_out=true<br/>steps=30"]
    end

    subgraph Bridges["External Bridges"]
        SX["SYNTHEX<br/>thermal_k_adjustment()<br/>raw_adj=1.15"]
        NX["NexusForge<br/>nexus_k_adjustment()<br/>raw_adj=1.10"]
        ME_B["ME Bridge<br/>me_k_adjustment()<br/>raw_adj=0.95"]
    end

    subgraph Gate["consent_gated_k_adjustment()"]
        FILTER["Filter eligible:<br/>!opt_out_external_modulation<br/>→ {A, B} (C excluded)"]
        RECEP["Mean receptivity:<br/>(0.9 + 0.3) / 2 = 0.6"]
        NEWCOMER["Newcomer damping:<br/>C has 30 steps < 50<br/>but C is opted out<br/>→ damping = 1.0"]
        ELIGIBLE["Eligible fraction:<br/>2/3 = 0.667"]
        DIVERGE{"Any sphere<br/>receptivity < 0.15?"}
        SCALE["scale = 0.6 × 1.0 × 0.667<br/>= 0.400"]
        DEVIATE["deviation = raw_adj - 1.0"]
        RESULT["gated = 1.0 + deviation × scale"]
    end

    subgraph Apply["Applied to k_modulation"]
        KMOD["k_modulation *= gated<br/>(multiplicative, per bridge)"]
        CLAMP["Clamp to [K_MOD_MIN, K_MOD_MAX]<br/>= [-0.5, 1.5]"]
    end

    SX --> Gate
    NX --> Gate
    ME_B --> Gate

    SP_A --> FILTER
    SP_B --> FILTER
    SP_C --> FILTER

    FILTER --> RECEP
    FILTER --> NEWCOMER
    FILTER --> ELIGIBLE
    RECEP --> SCALE
    NEWCOMER --> SCALE
    ELIGIBLE --> SCALE
    DIVERGE -->|"No"| DEVIATE
    DIVERGE -->|"Yes: suppress<br/>positive boost"| DEVIATE
    SCALE --> RESULT
    DEVIATE --> RESULT

    RESULT --> KMOD --> CLAMP

    subgraph OptOut["Opt-Out Path"]
        ALL_OUT["All spheres opted out<br/>→ return 1.0 (neutral)<br/>No external influence"]
    end

    FILTER -.->|"eligible.is_empty()"| ALL_OUT

    style Gate fill:#e8f0ff,stroke:#336
    style OptOut fill:#ffe8e8,stroke:#633
```

**Key constraints:**
- `opt_out_external_modulation` is a per-sphere boolean set via `POST /sphere/{id}/preferences`.
- Receptivity ranges from 0.0 (fully resistant) to 1.0 (fully open); set automatically by activation density (NA-14).
- Newcomer protection: spheres with `total_steps < 50` dampen external influence. At 100% newcomers, only 20% of adjustment passes through.
- Divergence exemption (NA-GAP-3): if ANY sphere has `receptivity < 0.15`, positive boosts (adj > 1.0) are suppressed to 0.0 deviation.
- Negative adjustments (reducing coupling) are never suppressed -- spheres can always request less coupling.
- Empty fleet (no spheres): raw adjustment passes through unchanged.
- Each bridge applies independently and multiplicatively to `k_modulation`.

**Cross-references:** `src/nexus_bridge.rs` (consent_gated_k_adjustment, lines 705-755) | `src/main.rs` (tick loop wiring, lines 810-848) | `src/nexus_bus/mod.rs` (NexusBus consent routing) | `src/sphere.rs` (opt_out_external_modulation field)

---

## 10. RM TSV Bridge Flow

The Reasoning Memory bridge persists field state, conductor decisions, and task lifecycle events to the Reasoning Memory service (port 8130). The wire format is **strictly TSV** (tab-separated values), never JSON. This is the most common integration mistake in the codebase.

Format: `category\tagent\tconfidence\tttl\tcontent`

```mermaid
sequenceDiagram
    participant TICK as Tick Loop<br/>(conductor)
    participant BUS as IPC Bus<br/>(task events)
    participant BRIDGE as bridge.rs<br/>(RM bridge)
    participant SANITIZE as Sanitize<br/>(E9)
    participant RM as Reasoning Memory<br/>:8130

    Note over TICK,RM: === PERIODIC FIELD SNAPSHOTS (every 12 ticks) ===

    TICK->>BRIDGE: record_field_snapshot(<br/>tick, r, k_mod, sphere_count)
    BRIDGE->>SANITIZE: Replace \t and \n with space<br/>in category + content
    Note over SANITIZE: "shared_state" → safe<br/>"Field state: tick=100..." → safe
    SANITIZE->>BRIDGE: safe_category, safe_content

    BRIDGE->>BRIDGE: Format TSV body:<br/>shared_state\tpane-vortex\t0.90\t604800\tField state: tick=100 r=0.9500...

    BRIDGE->>RM: POST /put HTTP/1.1<br/>Content-Type: text/plain<br/><TSV body>
    Note over RM: Store with 7-day TTL<br/>(604800 seconds)

    Note over TICK,RM: === CONDUCTOR DECISIONS (significant only) ===

    TICK->>BRIDGE: record_conductor_decision(<br/>tick, action, r, k_mod, spheres)
    BRIDGE->>SANITIZE: Sanitize
    BRIDGE->>RM: POST /put<br/>conductor\tpane-vortex\t0.90\t604800\t<br/>Conductor: tick=160 action=NeedsCoherence r=0.2800...

    Note over TICK,RM: === TASK LIFECYCLE ===

    BUS->>BRIDGE: record_task_complete(<br/>task_id, desc, submitted_by,<br/>completed_by, tick)
    BRIDGE->>RM: POST /put<br/>discovery\tpane-vortex\t0.90\t604800\t<br/>Task completed: build | id=abc...

    BUS->>BRIDGE: record_task_failed(<br/>task_id, desc, submitted_by,<br/>failed_by, error, tick)
    BRIDGE->>RM: POST /put<br/>discovery\tpane-vortex\t0.90\t604800\t<br/>Task failed: test | id=def error=timeout...

    Note over TICK,RM: === STARTUP BOOTSTRAP (Read Path) ===

    TICK->>RM: GET /entries HTTP/1.1
    RM-->>TICK: Text response (conductor entries)
    TICK->>BRIDGE: parse_conductor_data(body)
    Note over BRIDGE: Single-pass parser:<br/>1. Extract sphere pairs<br/>(sphere_a, sphere_b, weight)<br/>2. Extract last k_mod value
    BRIDGE-->>TICK: BootstrapResult {<br/>pairs: Vec<(String,String,f64)>,<br/>last_k_mod: Option<f64><br/>}
    Note over TICK: Seed Hebbian weights<br/>+ restore last k_mod

    Note over RM: RM garbage collection:<br/>TTL=604800s (7 days)<br/>Entries auto-expire
```

**Key constraints:**
- **NEVER send JSON to RM.** The wire format is `text/plain` with TSV fields. This is the most common integration mistake (recurred 3x).
- TSV fields: `category\tagent\tconfidence\tttl\tcontent` (exactly 5 fields, tab-separated).
- E9 sanitization: all tabs and newlines in `category` and `content` are replaced with spaces before formatting.
- Agent is always `pane-vortex`. Confidence is `0.90`. TTL is `604800` (7 days).
- Categories used: `shared_state` (field snapshots), `conductor` (decisions), `discovery` (task events), `governance` (planned).
- Fire-and-forget on write path: 2-second connect timeout, errors logged at debug level.
- Bootstrap read path: bounded to 64KB response to prevent memory pressure.
- `parse_conductor_data()` is a single-pass parser extracting both sphere pairs AND last k_mod from `Conductor:` lines.
- Weight values from bootstrap are clamped to `[0.05, 1.0]`.
- Address configurable via `REASONING_MEMORY_ADDR` env var (default `127.0.0.1:8130`).

**Cross-references:** `src/bridge.rs` | `ai_specs/patterns/SERDE_PATTERNS.md` (SP-9: TSV format) | `CLAUDE.md` (Trap #4: RM is TSV, not JSON) | `ai_specs/DATABASE_SPEC.md`
