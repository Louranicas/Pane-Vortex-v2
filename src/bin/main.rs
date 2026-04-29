//! # Pane-Vortex V2 Daemon
//!
//! The Habitat Coordination Daemon — Kuramoto-coupled oscillator field for
//! multi-pane Claude Code fleet coordination.
//!
//! ## Startup sequence
//! 1. Init tracing (file-based — no stdout to avoid SIGPIPE from devenv)
//! 2. Load configuration (Figment: TOML + env overlay)
//! 3. Restore snapshot (if exists)
//! 4. Spawn bridge smoke test
//! 5. Spawn tick loop
//! 6. Spawn IPC bus listener
//! 7. Spawn signal handler (graceful shutdown)
//! 8. Bind HTTP server (`SO_REUSEADDR` + exponential retry)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use pane_vortex::m1_foundation::{
    m01_core_types::PaneId,
    m03_config::PvConfig,
    m04_constants,
};
use pane_vortex::m3_field::m11_sphere::PaneSphere;
use pane_vortex::m3_field::m15_app_state::{new_shared_state, SharedState};
use pane_vortex::m4_coupling::m16_coupling_network::CouplingNetwork;
use pane_vortex::m7_coordination::m29_ipc_bus::{start_bus_listener, BusState};
use pane_vortex::m7_coordination::m31_conductor::Conductor;
use pane_vortex::m7_coordination::m35_tick::tick_orchestrator;

// ──────────────────────────────────────────────────────────────
// Tracing
// ──────────────────────────────────────────────────────────────

/// Initialise tracing to a log file. Never write to stdout (SIGPIPE from devenv).
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "pane_vortex=info".into());

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/pane-vortex.log");

    match log_file {
        Ok(file) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::sync::Mutex::new(file))
                .with_ansi(false)
                .init();
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(|| {
                    struct SafeStderr;
                    impl std::io::Write for SafeStderr {
                        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                            match std::io::stderr().write(buf) {
                                Ok(n) => Ok(n),
                                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(buf.len()),
                                Err(e) => Err(e),
                            }
                        }
                        fn flush(&mut self) -> std::io::Result<()> {
                            match std::io::stderr().flush() {
                                Ok(()) => Ok(()),
                                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
                                Err(e) => Err(e),
                            }
                        }
                    }
                    SafeStderr
                })
                .init();
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Snapshot
// ──────────────────────────────────────────────────────────────

/// Resolve the snapshot path (owner-only directory).
fn snapshot_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("pane-vortex");
    std::fs::create_dir_all(&dir).ok();

    // Restrict snapshot directory to owner-only (0700)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).ok();
    }

    dir.join("state-v2.json")
}

/// Attempt to restore state from snapshot.
fn restore_snapshot(path: &std::path::Path) -> SharedState {
    let state = new_shared_state();

    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                match serde_json::from_str(&json) {
                    Ok(restored) => {
                        let mut guard = state.write();
                        *guard = restored;
                        guard.reconcile_after_restore();
                        info!(
                            tick = guard.tick,
                            spheres = guard.spheres.len(),
                            "snapshot restored from {}",
                            path.display()
                        );
                    }
                    Err(e) => {
                        warn!("snapshot parse failed (starting fresh): {e}");
                    }
                }
            }
            Err(e) => {
                warn!("snapshot read failed (starting fresh): {e}");
            }
        }
    } else {
        info!("no snapshot found — starting with fresh state");
    }

    state
}

/// Save snapshot to disk.
fn save_snapshot(state: &SharedState, path: &std::path::Path) {
    let guard = state.read();
    match serde_json::to_string_pretty(&*guard) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                error!("snapshot write failed: {e}");
            } else {
                info!(tick = guard.tick, "snapshot saved to {}", path.display());
            }
        }
        Err(e) => {
            error!("snapshot serialize failed: {e}");
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Bridge smoke test
// ──────────────────────────────────────────────────────────────

/// Probe all bridge ports on startup (non-blocking).
async fn bridge_smoke_test(state: SharedState) {
    // Give services a moment to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    let bridges = [
        ("synthex-v2", 8092_u16),
        ("povm", 8125),
        ("reasoning-memory", 8130),
        ("vms", 8120),
        ("maintenance-engine", 8180),
    ];

    let mut reachable = Vec::new();
    let mut unreachable = Vec::new();

    for (name, port) in &bridges {
        let addr = format!("127.0.0.1:{port}");
        match tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => reachable.push(*name),
            _ => unreachable.push(*name),
        }
    }

    // Update staleness state
    {
        let mut s = state.write();
        s.prev_bridge_staleness.synthex_stale = unreachable.contains(&"synthex");
        s.prev_bridge_staleness.nexus_stale = unreachable.contains(&"nexus");
        s.prev_bridge_staleness.povm_stale = unreachable.contains(&"povm");
        s.prev_bridge_staleness.rm_stale = unreachable.contains(&"reasoning-memory");
        s.prev_bridge_staleness.vms_stale = unreachable.contains(&"vms");
        s.prev_bridge_staleness.me_stale = unreachable.contains(&"maintenance-engine");
    }

    if unreachable.is_empty() {
        info!(bridges = reachable.join(", "), "startup smoke test — all bridges reachable");
    } else {
        warn!(
            reachable = reachable.join(", "),
            unreachable = unreachable.join(", "),
            "startup smoke test — some bridges unreachable"
        );
    }
}

// ──────────────────────────────────────────────────────────────
// Tick loop
// ──────────────────────────────────────────────────────────────

/// Spawn the background tick loop with bridge polling and post-tick operations.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn spawn_tick_loop(
    state: SharedState,
    network: Arc<RwLock<CouplingNetwork>>,
    conductor: Arc<Conductor>,
    bridges: Arc<pane_vortex::m6_bridges::BridgeSet>,
    bus_state: Arc<RwLock<BusState>>,
    snap_path: PathBuf,
) {
    tokio::spawn(async move {
        let mut tick_count: u64 = 0;
        let mut suggestion_engine =
            pane_vortex::m7_coordination::m34_suggestions::SuggestionEngine::new();

        // SQLite persistence (supplements JSON snapshot)
        #[cfg(feature = "persistence")]
        let persistence = {
            match pane_vortex::m7_coordination::m36_persistence::PersistenceManager::new() {
                Ok(pm) => {
                    info!("sqlite persistence initialized");
                    Some(pm)
                }
                Err(e) => {
                    warn!("sqlite persistence init failed: {e}");
                    None
                }
            }
        };

        loop {
            let sphere_count = {
                let guard = state.read();
                guard.spheres.len()
            };

            // Adaptive tick interval: 1s busy (>3 spheres), 5s normal, 15s quiet (0 spheres)
            let sleep_secs = if sphere_count > 3 {
                1
            } else if sphere_count > 0 {
                m04_constants::TICK_INTERVAL_SECS
            } else {
                15
            };

            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;

            // ── Synchronous tick (under lock) ──
            let tick_result = {
                let mut app = state.write();
                let mut net = network.write();
                let result = tick_orchestrator(
                    &mut app,
                    &mut net,
                    &conductor,
                    Some(&bridges),
                );

                if tick_count % 100 == 0 {
                    info!(
                        tick = app.tick,
                        r = format!("{:.3}", result.order_parameter.r),
                        spheres = app.spheres.len(),
                        action = %result.decision.action,
                        ms = format!("{:.1}", result.total_ms),
                        bridge_ms = format!("{:.2}", result.phase_timings.bridge_ms),
                        "tick"
                    );
                }
                result
            };
            // AppState lock RELEASED here

            // ── Fire-and-forget bridge polls (inbound) ──
            spawn_bridge_polls(&bridges, tick_result.tick);

            // ── Fire-and-forget bridge posts (outbound) — E2 ──
            spawn_bridge_posts(
                &bridges,
                tick_result.tick,
                tick_result.order_parameter.r,
                tick_result.sphere_count,
            );

            // ── Publish field.tick event to IPC bus subscribers ──
            {
                let event = pane_vortex::m7_coordination::m30_bus_types::BusEvent::new(
                    "field.tick".to_owned(),
                    serde_json::json!({
                        "tick": tick_result.tick,
                        "r": tick_result.order_parameter.r,
                        "spheres": tick_result.sphere_count,
                        "action": format!("{:?}", tick_result.decision.action),
                    }),
                    tick_result.tick,
                );
                bus_state.write().publish_event(event);
            }

            // ── Prune stale tasks (GAP-G1 + GAP-G2) ──
            {
                let mut bus = bus_state.write();
                bus.prune_completed_tasks(3600.0);
                let requeued = bus.prune_stale_claims(300.0);
                drop(bus);
                if requeued > 0 {
                    eprintln!("[tick] requeued {requeued} stale claimed tasks");
                }
            }

            // ── Generate suggestions → BusState ──
            {
                let spheres_snap = state.read().spheres.clone();
                let suggestions =
                    suggestion_engine.generate(&tick_result.decision, &spheres_snap);
                let json_suggestions: Vec<serde_json::Value> = suggestions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "type": format!("{:?}", s.suggestion_type),
                            "target": s.target_sphere.as_str(),
                            "reason": s.reason,
                            "confidence": s.confidence,
                            "tick": s.tick,
                        })
                    })
                    .collect();
                if !json_suggestions.is_empty() {
                    let mut bus = bus_state.write();
                    for js in json_suggestions {
                        bus.add_suggestion(js);
                    }
                }
            }

            // ── SQLite persistence ──
            #[cfg(feature = "persistence")]
            if let Some(ref pm) = persistence {
                // Save decision event every tick
                if let Err(e) = pm.save_event(
                    &format!("{:?}", tick_result.decision.action),
                    &serde_json::to_string(&tick_result.field_state)
                        .unwrap_or_default(),
                    tick_result.tick,
                ) {
                    warn!(tick = tick_result.tick, error = %e, "persistence save_event failed");
                }
                // Save field snapshot every SNAPSHOT_INTERVAL ticks
                if tick_result.should_snapshot {
                    let k_mod = network.read().k_modulation;
                    if let Err(e) = pm.save_snapshot(
                        tick_result.tick,
                        tick_result.order_parameter.r,
                        tick_result.order_parameter.psi,
                        tick_result.sphere_count,
                        0,
                        k_mod,
                        "{}",
                    ) {
                        warn!(tick = tick_result.tick, error = %e, "persistence save_snapshot failed");
                    }
                }
            }

            // ── Periodic JSON snapshot ──
            tick_count += 1;
            if tick_count % m04_constants::SNAPSHOT_INTERVAL == 0 {
                save_snapshot(&state, &snap_path);
            }
        }
    });
}

// ──────────────────────────────────────────────────────────────
// Bridge polling (fire-and-forget)
// ──────────────────────────────────────────────────────────────

/// Spawn fire-and-forget bridge poll tasks for bridges that are due.
///
/// Each bridge uses synchronous `TcpStream`, so polls run in `spawn_blocking`
/// to avoid blocking the tokio async runtime.
fn spawn_bridge_polls(bridges: &Arc<pane_vortex::m6_bridges::BridgeSet>, tick: u64) {
    // Set last_poll_tick BEFORE spawning so the next tick's is_stale() check
    // sees a recent timestamp regardless of whether the async poll completes.
    // The poll itself sets stale=false on success or records failure on error.

    // SYNTHEX thermal poll (every 6 ticks)
    if bridges.synthex.should_poll(tick) {
        bridges.synthex.set_last_poll_tick(tick);
        let b = bridges.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = b.synthex.poll_thermal() {
                    warn!(tick, error = %e, "synthex thermal poll failed");
                }
            })
            .await;
        });
    }

    // Nexus/SAN-K7 strategy poll (every 12 ticks)
    if bridges.nexus.should_poll(tick) {
        bridges.nexus.set_last_poll_tick(tick);
        let b = bridges.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = b.nexus.poll_metrics() {
                    warn!(tick, error = %e, "nexus metrics poll failed");
                }
            })
            .await;
        });
    }

    // Maintenance Engine observer poll (every 12 ticks)
    if bridges.me.should_poll(tick) {
        bridges.me.set_last_poll_tick(tick);
        let b = bridges.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = b.me.poll_observer() {
                    warn!(tick, error = %e, "me observer poll failed");
                }
            })
            .await;
        });
    }
}

// ──────────────────────────────────────────────────────────────
// Bridge posts (outbound write-back) — E2
// ──────────────────────────────────────────────────────────────

/// Spawn fire-and-forget outbound writes to POVM, RM, and VMS bridges.
///
/// Each bridge checks `should_write(tick)` to throttle writes:
/// - POVM snapshot: every 12 ticks
/// - RM `field_state` record: every 60 ticks
/// - VMS `field_state` post: every 60 ticks
fn spawn_bridge_posts(
    bridges: &Arc<pane_vortex::m6_bridges::BridgeSet>,
    tick: u64,
    r: f64,
    sphere_count: usize,
) {
    // POVM snapshot (every 12 ticks)
    #[allow(clippy::cast_precision_loss)]
    if bridges.povm.should_write(tick) {
        bridges.povm.set_last_write_tick(tick);
        let b = bridges.clone();
        let sc_f = sphere_count as f64;
        let tk_f = tick as f64;
        let payload = serde_json::to_vec(&serde_json::json!({
            "content": format!("field_state tick={tick} r={r:.3} spheres={sphere_count}"),
            "intensity": r,
            "phi": 0.0,
            "theta": 0.0,
            "tensor": [r, sc_f, tk_f, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            "session_created": format!("pv2-tick-{tick}"),
        }))
        .unwrap_or_default();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                match b.povm.snapshot(&payload) {
                    Ok(()) => info!(tick, "povm snapshot posted"),
                    Err(e) => warn!(tick, error = %e, "povm snapshot failed"),
                }
            })
            .await;
        });
    }

    // RM field_state record (every 60 ticks) — TSV format
    if bridges.rm.should_poll(tick) {
        bridges.rm.set_last_poll_tick(tick);
        let b = bridges.clone();
        let content = format!("tick={tick} r={r:.3} spheres={sphere_count}");
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                let record = pane_vortex::m6_bridges::m26_rm_bridge::RmRecord::field_state(
                    content,
                    r,
                );
                match b.rm.post_record(&record) {
                    Ok(()) => info!(tick, "rm field_state posted"),
                    Err(e) => warn!(tick, error = %e, "rm post failed"),
                }
            })
            .await;
        });
    }

    // VMS field_state post (every 60 ticks)
    if bridges.vms.should_write(tick) {
        bridges.vms.set_last_write_tick(tick);
        let b = bridges.clone();
        let payload = serde_json::to_vec(&serde_json::json!({
            "tick": tick,
            "r": r,
            "spheres": sphere_count,
        }))
        .unwrap_or_default();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                match b.vms.post_field_state(&payload) {
                    Ok(()) => info!(tick, "vms field_state posted"),
                    Err(e) => warn!(tick, error = %e, "vms post failed"),
                }
            })
            .await;
        });
    }

    // POVM hydration read-back (every 12 ticks) — F1 (bidirectional flow)
    if bridges.povm.should_read(tick) {
        bridges.povm.set_last_read_tick(tick);
        let b = bridges.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || {
                match b.povm.hydrate_pathways() {
                    Ok(pathways) => {
                        info!(count = pathways.len(), "povm hydration: pathways loaded");
                    }
                    Err(e) => {
                        warn!(error = %e, "povm hydrate failed");
                        b.povm.record_failure();
                    }
                }
            })
            .await;
        });
    }
}

// ──────────────────────────────────────────────────────────────
// Bind with retry
// ──────────────────────────────────────────────────────────────

/// Bind TCP listener with `SO_REUSEADDR` and exponential-backoff retry.
async fn bind_with_retry(addr: &str) -> TcpListener {
    let mut delay = Duration::from_millis(m04_constants::BIND_INITIAL_DELAY_MS);

    for attempt in 1..=m04_constants::BIND_MAX_RETRIES {
        let socket = match socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        ) {
            Ok(s) => s,
            Err(e) => {
                error!(attempt, "failed to create socket: {e}");
                if attempt == m04_constants::BIND_MAX_RETRIES {
                    error!("exhausted bind attempts — exiting");
                    std::process::exit(1);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
                continue;
            }
        };

        socket.set_reuse_address(true).ok();
        socket.set_nonblocking(true).ok();

        let sock_addr: std::net::SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                error!("invalid bind address '{addr}': {e}");
                std::process::exit(1);
            }
        };

        if let Err(e) = socket.bind(&sock_addr.into()) {
            warn!(attempt, addr, "bind failed: {e} — retrying in {}ms", delay.as_millis());
            if attempt == m04_constants::BIND_MAX_RETRIES {
                error!("exhausted bind attempts for {addr} — exiting");
                std::process::exit(1);
            }
            tokio::time::sleep(delay).await;
            delay *= 2;
            continue;
        }

        if let Err(e) = socket.listen(128) {
            warn!(attempt, "listen(128) failed: {e} — retrying");
            if attempt == m04_constants::BIND_MAX_RETRIES {
                error!("exhausted listen attempts — exiting");
                std::process::exit(1);
            }
            tokio::time::sleep(delay).await;
            delay *= 2;
            continue;
        }

        let std_listener: std::net::TcpListener = socket.into();
        match TcpListener::from_std(std_listener) {
            Ok(l) => return l,
            Err(e) => {
                error!(attempt, "TcpListener::from_std failed: {e}");
                if attempt == m04_constants::BIND_MAX_RETRIES {
                    std::process::exit(1);
                }
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }

    // Unreachable but satisfies type checker
    error!("bind_with_retry: unreachable");
    std::process::exit(1);
}

// ──────────────────────────────────────────────────────────────
// Coupling network reconciliation
// ──────────────────────────────────────────────────────────────

/// Create a coupling network pre-populated with spheres from the restored snapshot.
///
/// Without this, a restart creates an empty `CouplingNetwork` even though `AppState`
/// has 50+ spheres — causing zero coupling edges until spheres re-register via API.
fn reconcile_coupling(
    state: &pane_vortex::m3_field::m15_app_state::SharedState,
) -> CouplingNetwork {
    let mut net = CouplingNetwork::new();
    let guard = state.read();
    for (id, sphere) in &guard.spheres {
        net.register(id.clone(), sphere.phase, sphere.frequency);
    }
    if !guard.spheres.is_empty() {
        info!(
            spheres = guard.spheres.len(),
            connections = net.connections.len(),
            "coupling network reconciled with snapshot spheres"
        );
    }
    net
}

// ──────────────────────────────────────────────────────────────
// Bridge tick seeding
// ──────────────────────────────────────────────────────────────

/// Seed bridge tick counters from restored snapshot to prevent transient staleness.
///
/// Without this, bridges start with `last_write_tick=0` after restart, causing
/// `is_stale()` to return true until the first write cycle fires.
fn seed_bridge_ticks(
    bridges: pane_vortex::m6_bridges::BridgeSet,
    state: &pane_vortex::m3_field::m15_app_state::SharedState,
) -> pane_vortex::m6_bridges::BridgeSet {
    let restored_tick = state.read().tick;
    if restored_tick > 0 {
        // Seed tick counters so is_stale() doesn't see a huge gap
        bridges.povm.set_last_write_tick(restored_tick);
        bridges.povm.set_last_read_tick(restored_tick);
        bridges.rm.set_last_poll_tick(restored_tick);
        bridges.vms.set_last_write_tick(restored_tick);
        bridges.vms.set_last_read_tick(restored_tick);
        // Trigger a write to clear the initial stale=true flag.
        // Must use valid payloads — POVM requires content+theta, VMS accepts any JSON.
        let seed_payload = serde_json::to_vec(&serde_json::json!({
            "content": format!("seed tick={restored_tick}"),
            "intensity": 0.0,
            "phi": 0.0,
            "theta": 0.0,
            "tensor": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }))
        .unwrap_or_default();
        if let Err(e) = bridges.povm.snapshot(&seed_payload) {
            warn!(error = %e, "seed POVM snapshot failed");
        }
        if let Err(e) = bridges.vms.post_field_state(&seed_payload) {
            warn!(error = %e, "seed VMS post failed");
        }
        let record = pane_vortex::m6_bridges::m26_rm_bridge::RmRecord::field_state(
            format!("seed tick={restored_tick}"),
            0.0,
        );
        if let Err(e) = bridges.rm.post_record(&record) {
            warn!(error = %e, "seed RM post failed");
        }
    }
    bridges
}

// ──────────────────────────────────────────────────────────────
// Diversity seeding
// ──────────────────────────────────────────────────────────────

/// Seed 3 diverse-frequency spheres if the field has fewer than 3.
///
/// Without frequency diversity, `auto_scale_k` falls back to K=1.5 (supercritical),
/// and all coupling weights converge to identical values (Session 073 convergence trap).
/// These seed spheres provide the frequency spread needed for healthy Kuramoto dynamics.
fn seed_diversity(
    state: &SharedState,
    network: &Arc<RwLock<CouplingNetwork>>,
) {
    const SEEDS: [(&str, &str, f64); 3] = [
        ("seed-diversity-1", "diversity-low",  0.18),
        ("seed-diversity-2", "diversity-mid",  0.26),
        ("seed-diversity-3", "diversity-high", 0.34),
    ];

    let count = state.read().spheres.len();
    if count >= 3 {
        info!(count, "field has >= 3 spheres — diversity seeding skipped");
        return;
    }

    let mut seeded = 0u32;
    for &(id, persona, freq) in &SEEDS {
        let pid = PaneId::new(id);
        if state.read().spheres.contains_key(&pid) {
            continue;
        }
        let Ok(sphere) = PaneSphere::new(pid.clone(), persona.into(), freq) else {
            warn!(id, "diversity seed sphere creation failed");
            continue;
        };
        let phase = sphere.phase;
        {
            let mut guard = state.write();
            guard.spheres.insert(pid.clone(), sphere);
            guard.state_changes += 1;
        }
        network.write().register(pid, phase, freq);
        seeded += 1;
    }
    if seeded > 0 {
        info!(seeded, "diversity seed spheres registered (0.18 / 0.26 / 0.34 Hz)");
    }
}

// ──────────────────────────────────────────────────────────────
// Main
// ──────────────────────────────────────────────────────────────

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() {
    init_tracing();

    // BUG-001b defense: refuse to start if another pane-vortex is already running
    // from the same binary. Lock auto-releases on Drop at the end of main.
    let _pidlock = match habitat_pidlock::PidLock::acquire("pane-vortex") {
        Ok(lock) => lock,
        Err(e) => {
            error!("pidlock acquire failed: {e}");
            std::process::exit(1);
        }
    };

    // Load configuration
    let config = match PvConfig::load() {
        Ok(c) => {
            info!(port = c.server.port, "configuration loaded");
            c
        }
        Err(e) => {
            // Fall back to defaults if config file missing
            warn!("config load failed ({e}), using defaults");
            PvConfig::default()
        }
    };

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config.server.port);

    // Restore state from snapshot
    let snap = snapshot_path();
    let state = restore_snapshot(&snap);

    // Create coupling network and reconcile with restored spheres
    let network = Arc::new(RwLock::new(reconcile_coupling(&state)));

    // Seed diverse-frequency spheres if field has fewer than 3
    seed_diversity(&state, &network);

    // Create conductor
    let conductor = Arc::new(Conductor::new());

    // Create bridge set and seed tick counters from restored state
    let bridges = Arc::new(seed_bridge_ticks(
        pane_vortex::m6_bridges::BridgeSet::from_config(&config.bridges),
        &state,
    ));
    info!("bridge set initialized (6 bridges + consent gate)");

    // Create shared bus state (used by tick loop, IPC listener, and API router)
    let bus_state = Arc::new(RwLock::new(BusState::new()));

    // Spawn bridge smoke test (non-blocking)
    tokio::spawn(bridge_smoke_test(state.clone()));

    // Spawn tick loop
    spawn_tick_loop(
        state.clone(),
        network.clone(),
        conductor,
        bridges,
        bus_state.clone(),
        snap.clone(),
    );

    // Spawn IPC bus listener with restart wrapper (Session 071 #5).
    // Previously fire-and-forget — if listener crashed, socket died silently
    // while HTTP stayed healthy. Now retries with exponential backoff.
    {
        let listener_state = state.clone();
        let listener_bus = bus_state.clone();
        tokio::spawn(async move {
            let mut attempt: u32 = 0;
            loop {
                attempt += 1;
                info!(attempt, "IPC bus listener starting");
                match start_bus_listener(listener_state.clone(), listener_bus.clone()).await {
                    Ok(()) => {
                        info!("IPC bus listener exited cleanly");
                        break;
                    }
                    Err(e) => {
                        let backoff = Duration::from_secs(2_u64.saturating_pow(attempt.min(5)));
                        error!(
                            attempt,
                            backoff_secs = backoff.as_secs(),
                            error = %e,
                            "IPC bus listener failed — restarting"
                        );
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        });
    }

    // Spawn signal handler for graceful shutdown
    let shutdown_state = state.clone();
    let shutdown_snap = snap;
    tokio::spawn(async move {
        let sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate());

        if let Ok(mut sigterm) = sigterm {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => info!("received SIGINT"),
                _ = sigterm.recv() => info!("received SIGTERM"),
            }
        } else {
            warn!("SIGTERM handler unavailable — using SIGINT only");
            tokio::signal::ctrl_c().await.ok();
            info!("received SIGINT");
        }

        info!("shutting down — saving snapshot");
        save_snapshot(&shutdown_state, &shutdown_snap);
        std::process::exit(0);
    });

    // Build API router
    #[cfg(feature = "api")]
    let app = {
        let ctx = pane_vortex::m2_services::m10_api_server::AppContext {
            state: state.clone(),
            network: network.clone(),
            bus: bus_state,
            cascade: std::sync::Arc::new(parking_lot::RwLock::new(
                pane_vortex::m7_coordination::m33_cascade::CascadeTracker::new(),
            )),
        };
        pane_vortex::m2_services::m10_api_server::build_router(ctx)
    };

    #[cfg(not(feature = "api"))]
    let app = {
        drop(bus_state);
        warn!("API feature disabled — no HTTP routes available");
        axum::Router::new()
    };

    // Bind and serve
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| config.server.bind_addr.clone());
    if bind_addr != "127.0.0.1" && bind_addr != "::1" {
        warn!("BIND_ADDR={bind_addr} — daemon exposed on network interface with NO authentication");
    }

    let addr = format!("{bind_addr}:{port}");
    let listener = bind_with_retry(&addr).await;

    info!("pane-vortex v2 daemon listening on {addr}");
    info!(
        tick_interval = m04_constants::TICK_INTERVAL_SECS,
        coupling_steps = m04_constants::COUPLING_STEPS_PER_TICK,
        "daemon configuration"
    );

    if let Err(e) = axum::serve(listener, app).await {
        error!("server error: {e}");
        std::process::exit(1);
    }
}
