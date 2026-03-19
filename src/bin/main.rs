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
    m03_config::PvConfig,
    m04_constants,
};
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
            // Fallback to stderr (not stdout — stderr survives pipe breaks better)
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
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
        ("synthex", 8090_u16),
        ("nexus", 8100),
        ("povm", 8125),
        ("reasoning-memory", 8130),
        ("vms", 8120),
        ("maintenance-engine", 8080),
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

/// Spawn the background tick loop.
fn spawn_tick_loop(
    state: SharedState,
    network: Arc<RwLock<CouplingNetwork>>,
    conductor: Arc<Conductor>,
    snap_path: PathBuf,
) {
    tokio::spawn(async move {
        let mut tick_count: u64 = 0;

        loop {
            let interval = {
                let guard = state.read();
                guard.spheres.len()
            };

            // Adaptive tick interval: 1s busy (>3 spheres), 5s normal, 15s quiet (0 spheres)
            let sleep_secs = if interval > 3 {
                1
            } else if interval > 0 {
                m04_constants::TICK_INTERVAL_SECS
            } else {
                15
            };

            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;

            // Execute tick
            {
                let mut app = state.write();
                let mut net = network.write();
                let result = tick_orchestrator(&mut app, &mut net, &conductor);

                if tick_count % 100 == 0 {
                    info!(
                        tick = app.tick,
                        r = format!("{:.3}", result.order_parameter.r),
                        spheres = app.spheres.len(),
                        action = %result.decision.action,
                        ms = format!("{:.1}", result.total_ms),
                        "tick"
                    );
                }
            }

            // Periodic snapshot
            tick_count += 1;
            if tick_count % m04_constants::SNAPSHOT_INTERVAL == 0 {
                save_snapshot(&state, &snap_path);
            }
        }
    });
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

        socket.listen(128).ok();

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
// Main
// ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    init_tracing();

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

    // Create coupling network
    let network = Arc::new(RwLock::new(CouplingNetwork::new()));

    // Create conductor
    let conductor = Arc::new(Conductor::new());

    // Spawn bridge smoke test (non-blocking)
    tokio::spawn(bridge_smoke_test(state.clone()));

    // Spawn tick loop
    spawn_tick_loop(state.clone(), network.clone(), conductor, snap.clone());

    // Create shared bus state (used by both IPC listener and API router)
    let bus_state = Arc::new(RwLock::new(BusState::new()));

    // Spawn IPC bus listener
    {
        let listener_state = state.clone();
        let listener_bus = bus_state.clone();
        tokio::spawn(async move {
            if let Err(e) = start_bus_listener(listener_state, listener_bus).await {
                error!("IPC bus listener failed: {e}");
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
