//! # Fleet Verify — Habitat Fleet Instance Verification
//!
//! Reliable fleet status via PV sphere API, IPC bus info, sidecar probe,
//! and bridge staleness checks. Replaces fragile screen-dump approach.
//!
//! ## Usage
//! ```text
//! fleet-verify          # formatted text output
//! fleet-verify --json   # machine-readable JSON
//! ```
//!
//! ## Data Sources
//! 1. PV sphere API (`/spheres`) — authoritative sphere state
//! 2. PV bus info (`/bus/info`) — subscriber + task + event counts
//! 3. Sidecar PID + ring file — WASM bridge health
//! 4. Bridge health (`/bridges/health`) — staleness flags
//! 5. PV health (`/health`) — field state composite

use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// PV daemon address.
const PV_ADDR: &str = "127.0.0.1:8132";

/// HTTP timeout for raw TCP requests.
const HTTP_TIMEOUT_MS: u64 = 3000;

/// Sidecar event ring file.
const SIDECAR_RING: &str = "/tmp/swarm-events.jsonl";

/// Maximum HTTP response buffer.
const MAX_RESPONSE: usize = 65536;

// ──────────────────────────────────────────────────────────────
// API Response Types — every field is consumed
// ──────────────────────────────────────────────────────────────

/// PV `/health` response.
#[derive(Debug, Deserialize)]
struct HealthResponse {
    r: f64,
    tick: u64,
    /// Used for cross-validation against sphere census.
    spheres: usize,
    #[serde(default)]
    k_modulation: f64,
    #[serde(default)]
    status: String,
}

impl HealthResponse {
    /// Cross-validate sphere count against census.
    const fn sphere_mismatch(&self, census_total: usize) -> bool {
        self.spheres != census_total
    }
}

/// PV `/spheres` response.
#[derive(Debug, Deserialize)]
struct SpheresResponse {
    spheres: Vec<Sphere>,
}

/// Individual sphere from `/spheres`.
#[derive(Debug, Deserialize)]
struct Sphere {
    id: String,
    status: String,
    #[serde(default)]
    persona: String,
}

impl Sphere {
    /// Whether this sphere is a fleet-tagged worker.
    fn is_fleet(&self) -> bool {
        self.id.starts_with("4:")
            || self.id.starts_with("5:")
            || self.id.starts_with("6:")
            || self.id.starts_with("fleet-")
            || self.persona.starts_with("fleet")
    }
}

/// PV `/bus/info` response.
#[derive(Debug, Deserialize)]
struct BusInfo {
    #[serde(default)]
    subscribers: usize,
    #[serde(default)]
    tasks: usize,
    #[serde(default)]
    events: usize,
    #[serde(default)]
    cascade_count: usize,
}

/// Bridge staleness as named entries for iteration.
///
/// Parsed from the PV `/bridges/health` JSON using a `HashMap` to avoid
/// a struct with 6 bools (clippy pedantic `struct_excessive_bools`).
struct BridgeHealth {
    entries: Vec<(&'static str, bool)>,
}

/// Bridge names in the API response (mapped from `*_stale` keys).
const BRIDGE_NAMES: [&str; 6] = ["synthex", "nexus", "me", "povm", "rm", "vms"];

impl BridgeHealth {
    /// Parse from JSON body using a `HashMap`.
    fn from_json(body: &str) -> Option<Self> {
        let map: std::collections::HashMap<String, bool> =
            serde_json::from_str(body).ok()?;
        let entries = BRIDGE_NAMES
            .iter()
            .map(|name| {
                let key = format!("{name}_stale");
                let stale = map.get(&key).copied().unwrap_or(true);
                (*name, stale)
            })
            .collect();
        Some(Self { entries })
    }

    /// All bridges assumed stale (fallback when PV unreachable).
    fn all_stale() -> Self {
        Self {
            entries: BRIDGE_NAMES.iter().map(|name| (*name, true)).collect(),
        }
    }

    /// Count stale bridges.
    fn stale_count(&self) -> usize {
        self.entries.iter().filter(|(_, stale)| *stale).count()
    }

    /// List names of stale bridges.
    fn stale_names(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|(_, stale)| *stale)
            .map(|(name, _)| *name)
            .collect()
    }
}

// ──────────────────────────────────────────────────────────────
// Output — every field is serialized
// ──────────────────────────────────────────────────────────────

/// Composite fleet verification result.
#[derive(Debug, Serialize)]
struct FleetStatus {
    spheres: usize,
    working: usize,
    idle: usize,
    blocked: usize,
    fleet_workers: usize,
    subscribers: usize,
    pending_tasks: usize,
    bus_events: usize,
    cascades: usize,
    sidecar: String,
    sidecar_events: usize,
    stale_bridges: usize,
    stale_names: Vec<String>,
    confidence: usize,
    pv_r: f64,
    pv_tick: u64,
    pv_k_mod: f64,
    pv_status: String,
}

// ──────────────────────────────────────────────────────────────
// HTTP Client (raw TCP — same pattern as PV bridges)
// ──────────────────────────────────────────────────────────────

/// Send raw HTTP GET and return response body.
fn http_get(path: &str) -> Option<String> {
    let timeout = Duration::from_millis(HTTP_TIMEOUT_MS);
    let mut stream = TcpStream::connect_timeout(
        &PV_ADDR.parse().ok()?,
        timeout,
    )
    .ok()?;
    stream.set_read_timeout(Some(timeout)).ok()?;

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    );
    std::io::Write::write_all(&mut stream, request.as_bytes()).ok()?;

    let mut buf = vec![0u8; MAX_RESPONSE];
    let mut total = 0;
    loop {
        match stream.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if total >= buf.len() {
                    break;
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(_) => break,
        }
    }

    let response = String::from_utf8_lossy(&buf[..total]);
    response
        .find("\r\n\r\n")
        .map(|pos| response[pos + 4..].trim_end_matches('\0').to_owned())
}

// ──────────────────────────────────────────────────────────────
// Probes
// ──────────────────────────────────────────────────────────────

/// Sphere census from PV API.
struct SphereCensus {
    total: usize,
    working: usize,
    idle: usize,
    blocked: usize,
    fleet: usize,
}

/// Probe PV sphere API for a full census.
fn probe_spheres() -> SphereCensus {
    let empty = SphereCensus {
        total: 0,
        working: 0,
        idle: 0,
        blocked: 0,
        fleet: 0,
    };
    let Some(body) = http_get("/spheres") else {
        return empty;
    };
    let Ok(resp) = serde_json::from_str::<SpheresResponse>(&body) else {
        return empty;
    };

    SphereCensus {
        total: resp.spheres.len(),
        working: resp.spheres.iter().filter(|s| s.status == "Working").count(),
        idle: resp.spheres.iter().filter(|s| s.status == "Idle").count(),
        blocked: resp.spheres.iter().filter(|s| s.status == "Blocked").count(),
        fleet: resp.spheres.iter().filter(|s| s.is_fleet()).count(),
    }
}

/// Probe PV bus info.
fn probe_bus() -> BusInfo {
    http_get("/bus/info")
        .and_then(|b| serde_json::from_str(&b).ok())
        .unwrap_or(BusInfo {
            subscribers: 0,
            tasks: 0,
            events: 0,
            cascade_count: 0,
        })
}

/// Probe bridge health, returning the parsed struct.
fn probe_bridges() -> BridgeHealth {
    http_get("/bridges/health")
        .and_then(|b| BridgeHealth::from_json(&b))
        .unwrap_or_else(BridgeHealth::all_stale)
}

/// Probe PV health.
fn probe_health() -> HealthResponse {
    http_get("/health")
        .and_then(|b| serde_json::from_str(&b).ok())
        .unwrap_or_else(|| HealthResponse {
            r: 0.0,
            tick: 0,
            spheres: 0,
            k_modulation: 0.0,
            status: "unreachable".to_owned(),
        })
}

/// Probe sidecar status and ring file size.
fn probe_sidecar() -> (String, usize) {
    let pid_exists = std::process::Command::new("pgrep")
        .args(["-x", "swarm-sidecar"])
        .output()
        .is_ok_and(|o| o.status.success());

    let status = if pid_exists { "UP" } else { "DOWN" };

    let ring_lines = std::fs::read_to_string(SIDECAR_RING)
        .map(|c| c.lines().count())
        .unwrap_or(0);

    (status.to_owned(), ring_lines)
}

/// Compute confidence score (0-100).
const fn compute_confidence(census: &SphereCensus, bus: &BusInfo, sidecar_up: bool, stale: usize) -> usize {
    let mut score: usize = 0;
    if census.total > 0 {
        score += 30;
    }
    if census.working > 0 {
        score += 25;
    }
    if bus.subscribers > 0 {
        score += 20;
    }
    if sidecar_up {
        score += 15;
    }
    if stale == 0 {
        score += 10;
    }
    score
}

// ──────────────────────────────────────────────────────────────
// Display
// ──────────────────────────────────────────────────────────────

/// Print formatted text output.
fn print_text(status: &FleetStatus) {
    let bar = "\u{2550}".repeat(46);
    println!("\u{2554}{bar}\u{2557}");
    println!("\u{2551}  FLEET VERIFY (Rust)                       \u{2551}");
    println!("\u{2560}{bar}\u{2563}");
    println!(
        "\u{2551} Spheres:    {} total ({} working, {} idle, {} blocked)",
        status.spheres, status.working, status.idle, status.blocked
    );
    println!("\u{2551} Fleet:      {} fleet-tagged spheres", status.fleet_workers);
    println!(
        "\u{2551} Bus:        {} subs, {} tasks, {} events, {} cascades",
        status.subscribers, status.pending_tasks, status.bus_events, status.cascades
    );
    println!(
        "\u{2551} Sidecar:    {} ({} ring events)",
        status.sidecar, status.sidecar_events
    );
    if status.stale_names.is_empty() {
        println!("\u{2551} Bridges:    0/6 stale");
    } else {
        println!(
            "\u{2551} Bridges:    {}/6 stale [{}]",
            status.stale_bridges,
            status.stale_names.join(", ")
        );
    }
    println!(
        "\u{2551} Field:      r={:.3} k_mod={:.3} tick={} ({})",
        status.pv_r, status.pv_k_mod, status.pv_tick, status.pv_status
    );
    println!("\u{2551} Confidence: {}/100", status.confidence);
    println!("\u{255a}{bar}\u{255d}");
}

// ──────────────────────────────────────────────────────────────
// Main
// ──────────────────────────────────────────────────────────────

fn main() {
    let json_mode = std::env::args().any(|a| a == "--json");

    let census = probe_spheres();
    let bus = probe_bus();
    let bridges = probe_bridges();
    let health = probe_health();
    let (sidecar_status, sidecar_events) = probe_sidecar();

    let stale = bridges.stale_count();
    let conf = compute_confidence(&census, &bus, &sidecar_status == "UP", stale);

    let status = FleetStatus {
        spheres: census.total,
        working: census.working,
        idle: census.idle,
        blocked: census.blocked,
        fleet_workers: census.fleet,
        subscribers: bus.subscribers,
        pending_tasks: bus.tasks,
        bus_events: bus.events,
        cascades: bus.cascade_count,
        sidecar: sidecar_status,
        sidecar_events,
        stale_bridges: stale,
        stale_names: bridges.stale_names().iter().map(|s| (*s).to_owned()).collect(),
        confidence: conf,
        pv_r: health.r,
        pv_tick: health.tick,
        pv_k_mod: health.k_modulation,
        pv_status: health.status.clone(),
    };

    if json_mode {
        match serde_json::to_string_pretty(&status) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("JSON serialize error: {e}"),
        }
    } else {
        print_text(&status);
    }

    // Cross-validation: sphere count from /health vs /spheres
    if health.sphere_mismatch(census.total) {
        eprintln!(
            "WARN: sphere count mismatch — /health reports {} but /spheres has {}",
            health.spheres, census.total
        );
    }
}
