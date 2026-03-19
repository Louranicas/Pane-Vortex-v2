# Layer 7: Coordination Specification

> Detailed spec for m29-m36: IPC bus, bus types, conductor, executor, cascade,
> suggestions, tick orchestrator, persistence.
> The coordination layer — where everything comes together.
> Source: `src/m7_coordination/` | Plan: `MASTERPLAN.md` Phase 6
> v1 Source: bus.rs, ipc.rs, client.rs, conductor.rs, executor.rs, persistence.rs, main.rs

## Overview

Layer 7 orchestrates all other layers. The tick loop (m35) drives the system heartbeat.
The IPC bus (m29) provides inter-pane communication. The conductor (m31) modulates
coupling. The executor (m32) dispatches commands to Zellij panes. Persistence (m36)
writes state to SQLite.

Dependencies: L1, L3, L5, L6.

## 1. m29_ipc_bus (~400 LOC)

### 1.1 Bus Server

```rust
pub async fn start_bus_listener(
    socket_path: &Path,
    app_state: SharedState,
    bus_state: SharedBusState,
) -> PvResult<()>;

async fn handle_connection(
    stream: UnixStream,
    app_state: SharedState,
    bus_state: SharedBusState,
    event_rx: broadcast::Receiver<BusEvent>,
) -> PvResult<()>;
```

### 1.2 Connection Handler Flow

```
1. Accept connection
2. Set read timeout (5s for handshake)
3. Read handshake frame -> validate
4. Send welcome frame
5. Enter event loop:
   a. Select on:
      - Client frame (read from socket)
      - Event broadcast (from bus_state.event_tx)
      - Shutdown signal
   b. Handle client frame (subscribe, submit, claim, complete, fail, cascade)
   c. Forward matching events to client
6. Cleanup on disconnect
```

### 1.3 Shutdown Protocol

```rust
// Graceful shutdown: signal all connections, wait for drain
pub async fn shutdown_bus(bus_state: SharedBusState) {
    let state = bus_state.write().await;
    state.shutdown_signal.send(()).ok();
    // Connections see shutdown signal in select! and close gracefully
}
```

### 1.4 Tests (10 target)

- Start listener, connect, handshake, receive welcome
- Subscribe + receive matching event
- Subscribe + do not receive non-matching event
- Submit task via bus, verify in task queue
- Connection cleanup on client disconnect
- Handshake timeout (5s)
- Max connections enforcement
- Concurrent connections do not deadlock

## 2. m30_bus_types (~300 LOC)

### 2.1 BusFrame

See `WIRE_PROTOCOL_SPEC.md` for complete frame definitions.

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BusFrame {
    Handshake { id: String, version: String },
    Welcome { server_version: String, tick: u64, sphere_count: usize, r: f64 },
    Subscribe { patterns: Vec<String> },
    Subscribed { patterns: Vec<String>, total_subscriptions: usize },
    Submit { description: String, target: Option<String>, target_type: String, payload: Option<String>, tags: Option<Vec<String>> },
    TaskSubmitted { task_id: String, status: String, submitted_at: String },
    Event { event_type: String, source: Option<String>, tick: Option<u64>, data: serde_json::Value },
    CascadeHandoff { target: String, brief: String, depth: Option<u32> },
    CascadeAck { source: String, status: String },
    RejectCascade { source: String, reason: String },
    Claim { task_id: String },
    Complete { task_id: String, result: Option<String> },
    Fail { task_id: String, error: String },
    Error { code: String, message: String },
}
```

### 2.2 BusTask

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusTask {
    pub id: TaskId,
    pub status: TaskStatus,
    pub source_sphere: PaneId,
    pub target: Option<String>,
    pub target_type: TargetType,
    pub description: String,
    pub payload: Option<String>,
    pub claimed_by: Option<PaneId>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub claimed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ttl_secs: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus { Submitted, Claimed, Completed, Failed, Expired }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType { Specific, AnyIdle, FieldDriven, Willing }
```

### 2.3 BusState

```rust
pub struct BusState {
    pub tasks: HashMap<TaskId, BusTask>,
    pub subscriptions: HashMap<PaneId, Vec<String>>,
    pub event_tx: broadcast::Sender<BusEvent>,
    pub cascade_events: VecDeque<CascadeEvent>,
    pub connected_clients: HashSet<PaneId>,
    pub suggestions: Vec<FieldSuggestion>,
    pub shutdown_signal: watch::Sender<()>,
}

pub type SharedBusState = Arc<RwLock<BusState>>;
```

### 2.4 Tests (10 target)

- BusFrame serde roundtrip for all 14 variants
- BusTask lifecycle: submitted -> claimed -> completed
- BusTask TTL expiry detection
- TargetType routing logic
- Glob pattern matching for subscriptions

## 3. m31_conductor (~300 LOC)

### 3.1 PI Controller

```rust
pub fn conduct_breathing(
    app_state: &mut AppState,
    r_target: f64,
    consent_scale: f64,
) {
    let r = app_state.network.order_parameter().r;
    let error = r - r_target;
    let step = (error * CONDUCTOR_GAIN).clamp(-0.04, 0.03);
    app_state.network.k_modulation = (app_state.network.k_modulation - step)
        .clamp(K_MOD_MIN, K_MOD_MAX);
}
```

### 3.2 Hebbian Learning

```rust
pub fn hebbian_learning(
    spheres: &HashMap<PaneId, PaneSphere>,
    network: &mut CouplingNetwork,
    warmup: bool,
) {
    if warmup { return; }

    let working: Vec<&PaneId> = spheres.iter()
        .filter(|(_, s)| s.status == PaneStatus::Working
            && !s.opt_out_hebbian
            && s.total_steps >= 2)
        .map(|(id, _)| id)
        .collect();

    let burst = working.len() >= 3;

    // LTP: working pairs
    for i in 0..working.len() {
        for j in (i+1)..working.len() {
            let ltp = HEBBIAN_LTP
                * if burst { BURST_MULTIPLIER } else { 1.0 }
                * newcomer_mult(spheres, working[i], working[j]);
            network.adjust_weight(working[i], working[j], ltp);
        }
    }

    // LTD: non-working pairs
    for conn in &mut network.connections {
        let a_working = spheres.get(&conn.from).map_or(false, |s| s.status == PaneStatus::Working);
        let b_working = spheres.get(&conn.to).map_or(false, |s| s.status == PaneStatus::Working);
        if !a_working || !b_working {
            conn.weight = (conn.weight - HEBBIAN_LTD).max(WEIGHT_FLOOR);
        }
    }
}
```

### 3.3 Divergence Cooldown

```rust
pub struct ConductorState {
    pub divergence_cooldown: u32,
    pub fleet_mode: FleetMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetMode {
    Empty,    // no spheres
    Fresh,    // spheres but none has worked
    Active,   // normal operation
    Degraded, // bridges unhealthy
}
```

### 3.4 Tests (10 target)

- PI controller: r > target pushes k_mod down
- PI controller: r < target pushes k_mod up
- PI controller: k_mod clamps at bounds
- Hebbian LTP: working pair weight increases
- Hebbian LTD: idle pair weight decreases
- Hebbian: opted-out sphere unaffected
- Hebbian: burst mode (3+ working) applies multiplier
- Hebbian: warmup skips learning
- Divergence cooldown: suppresses coherence for N ticks
- FleetMode transitions

## 4. m32_executor (~200 LOC)

### 4.1 Zellij Dispatch

```rust
pub struct ExecutorRegistry {
    pub pane_map: HashMap<String, PaneId>,
    pub last_dispatch: Option<Instant>,
}

pub type SharedExecutorRegistry = Arc<RwLock<ExecutorRegistry>>;

pub async fn dispatch_to_pane(
    pane: &str,
    command: &str,
    registry: &SharedExecutorRegistry,
) -> PvResult<Duration> {
    let start = Instant::now();
    // 5-step Zellij dispatch:
    // 1. zellij action write-chars "{command}"
    // 2. Wait 100ms
    // 3. zellij action write-chars "\n"
    // 4. Record dispatch time
    // 5. Return latency
    let output = tokio::process::Command::new("zellij")
        .args(["action", "write-chars", command, "--"])
        .output().await?;
    Ok(start.elapsed())
}
```

### 4.2 Tests (10 target)

- Pane map update
- Dispatch latency tracking
- Registry concurrent access
- Invalid pane handling

## 5. m33_cascade (~200 LOC)

### 5.1 Cascade Handoff

```rust
pub async fn dispatch_cascade(
    source: &PaneId,
    target: &PaneId,
    brief: &str,
    depth: u32,
    app_state: &SharedState,
    bus_state: &SharedBusState,
) -> PvResult<()> {
    if depth >= CASCADE_MAX_DEPTH {
        return Err(PvError::CascadeDepthExceeded(depth));
    }

    let app = app_state.read().await;
    let sphere = app.spheres.get(target)
        .ok_or_else(|| PvError::SphereNotFound(target.clone()))?;

    if !sphere.consent.accept_cascade {
        return Err(PvError::CascadeRejected(format!("{target} does not accept cascades")));
    }
    drop(app); // release read lock before acquiring bus write lock

    let mut bus = bus_state.write().await;
    bus.cascade_events.push_back(CascadeEvent {
        source: source.clone(),
        target: Some(target.clone()),
        brief: brief.to_string(),
        status: CascadeStatus::Dispatched,
        depth,
    });

    // Broadcast cascade event
    let _ = bus.event_tx.send(BusEvent {
        event_type: "cascade.dispatched".into(),
        data: serde_json::json!({ "source": source, "target": target, "depth": depth }),
        tick: None,
    });

    Ok(())
}
```

### 5.2 Tests (5 target)

- Cascade dispatch with consent
- Cascade rejected without consent
- Cascade depth limit enforcement
- Cascade ack recording
- Cascade re-routing on rejection

## 6. m34_suggestions (~150 LOC)

### 6.1 Field-Driven Suggestions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub targets: Vec<PaneId>,
    pub priority: f64,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SuggestionType {
    DivergenceOpportunity,
    CoherenceNeeded,
    TaskAssignment,
    BridgeAlert,
    GovernanceAction,
}

pub fn generate_suggestions(
    field: &FieldState,
    decision: &FieldDecision,
    spheres: &HashMap<PaneId, PaneSphere>,
) -> Vec<FieldSuggestion>;
```

### 6.2 Tests (5 target)

- NeedsDivergence generates DivergenceOpportunity
- HasBlockedAgents generates CoherenceNeeded
- Empty field generates no suggestions
- Suggestions respect sphere consent (only willing targets)

## 7. m35_tick — The Orchestrator (~500 LOC)

### 7.1 Tick Loop

```rust
pub async fn tick_once(
    app_state: &SharedState,
    bus_state: &SharedBusState,
    thermal: &SharedThermalState,
    nexus: &SharedNexusState,
    me: &SharedMeState,
    dbs: &Databases,
    config: &PvConfig,
) -> PvResult<()> {
    // Phase 1: Bridge polls (read external state)
    poll_bridges(thermal, nexus, me, tick).await;

    // Phase 2: Acquire write lock, perform all mutations
    let mut app = app_state.write().await;

    // Phase 2a: Coupling steps (Kuramoto integration)
    let steps = adaptive_coupling_steps(&app, config);
    for _ in 0..steps {
        app.network.step(KURAMOTO_DT);
    }

    // Phase 2b: Sphere steps (memory decay, status, frequency)
    for sphere in app.spheres.values_mut() {
        let context = build_sphere_context(&app, &sphere.id);
        sphere.step(KURAMOTO_DT, 0.0, &context);
    }

    // Phase 2c: Hebbian learning
    hebbian_learning(&app.spheres, &mut app.network, app.warmup_remaining > 0);

    // Phase 2d: Field computation
    let field = FieldState::compute(&app.spheres, &app.network, app.tick);
    let decision = compute_decision(&app, &field);
    app.r_history.push_back(field.order_parameter.r);
    if app.r_history.len() > R_HISTORY_MAX { app.r_history.pop_front(); }

    // Phase 2e: Conductor (PI control)
    let consent_scale = fleet_mean_consent(&app.spheres);
    conduct_breathing(&mut app, R_TARGET, consent_scale);

    // Phase 2f: Auto-K recalculation (every 20 ticks)
    if app.tick % AUTO_SCALE_K_PERIOD == 0 {
        app.network.auto_scale_k();
    }

    // Phase 2g: Warmup decrement
    if app.warmup_remaining > 0 { app.warmup_remaining -= 1; }

    app.tick += 1;
    app.dirty = true;

    drop(app); // Release AppState lock BEFORE acquiring BusState

    // Phase 3: Bus events (requires BusState write lock)
    broadcast_field_events(bus_state, &field, &decision).await;

    // Phase 4: Persistence (no lock needed — uses dedicated DB connection)
    if tick % SNAPSHOT_INTERVAL == 0 {
        persist_field_snapshot(dbs, &field, &decision).await?;
    }

    // Phase 5: Bridge writes (fire-and-forget, no locks)
    if tick % POVM_SNAPSHOT_INTERVAL == 0 {
        post_field_to_povm(&field).await;
    }

    Ok(())
}
```

### 7.2 Lock Acquisition Pattern

```
tick_once:
  Phase 1: No locks (bridge polls are independent)
  Phase 2: AppState write lock (all mutations)
  Phase 2 end: Drop AppState lock
  Phase 3: BusState write lock (event broadcast)
  Phase 3 end: Drop BusState lock
  Phase 4-5: No locks (persistence + bridge writes)
```

This pattern guarantees AppState-before-BusState ordering and minimizes lock hold time.

### 7.3 Tests (15 target)

- tick_once increments tick counter
- tick_once updates r_history
- tick_once respects warmup (no Hebbian during warmup)
- tick_once generates field events on bus
- tick_once persists snapshots at interval
- adaptive_coupling_steps formula
- Bridge poll timing (every N ticks)
- Auto-K recalculation timing (every 20 ticks)
- Lock ordering (no deadlock under concurrent API calls)
- Graceful shutdown persists state

## 8. m36_persistence (~400 LOC)

### 8.1 Database Management

```rust
pub struct Databases {
    pub field: Connection,
    pub bus: Connection,
}

impl Databases {
    pub fn open(config: &PersistenceConfig) -> PvResult<Self>;
    pub fn run_migrations(&self) -> PvResult<()>;
}
```

### 8.2 Write Operations

```rust
pub fn persist_field_snapshot(db: &Connection, field: &FieldState, decision: &FieldDecision) -> PvResult<()>;
pub fn persist_sphere_event(db: &Connection, event: &SphereEvent) -> PvResult<()>;
pub fn persist_coupling_history(db: &Connection, network: &CouplingNetwork, tick: u64) -> PvResult<()>;
pub fn persist_task(db: &Connection, task: &BusTask) -> PvResult<()>;
pub fn persist_bus_event(db: &Connection, event: &BusEvent) -> PvResult<()>;
pub fn persist_cascade(db: &Connection, cascade: &CascadeEvent) -> PvResult<()>;
pub fn persist_consent(db: &Connection, sphere_id: &str, consent: &ConsentDeclaration) -> PvResult<()>;
pub fn persist_proposal(db: &Connection, proposal: &Proposal) -> PvResult<()>;
pub fn persist_vote(db: &Connection, vote: &Vote) -> PvResult<()>;
```

### 8.3 Read Operations

```rust
pub fn load_consent(db: &Connection, sphere_id: &str) -> PvResult<Option<ConsentDeclaration>>;
pub fn load_proposals(db: &Connection, status: Option<&str>) -> PvResult<Vec<Proposal>>;
pub fn load_tasks(db: &Connection, status: Option<&str>, limit: usize) -> PvResult<Vec<BusTask>>;
pub fn load_field_history(db: &Connection, limit: usize) -> PvResult<Vec<FieldSnapshot>>;
```

### 8.4 Tests (10 target)

- Database open + migration idempotency
- Field snapshot write + read roundtrip
- Task lifecycle persistence
- Consent declaration persistence
- WAL mode verification
- Concurrent read/write (no SQLITE_BUSY with timeout)

## Summary

| Module | LOC Target | Key Responsibility | Tests |
|--------|-----------|-------------------|-------|
| m29_ipc_bus | 400 | Socket listener, connection handling | 10 |
| m30_bus_types | 300 | Frame types, task/event structures | 10 |
| m31_conductor | 300 | PI control, Hebbian, FleetMode | 10 |
| m32_executor | 200 | Zellij command dispatch | 10 |
| m33_cascade | 200 | Cascade handoff with consent | 5 |
| m34_suggestions | 150 | Field-driven suggestions | 5 |
| m35_tick | 500 | Tick loop orchestration | 15 |
| m36_persistence | 400 | SQLite read/write operations | 10 |
| **L7 Total** | **2,450** | | **75** |
