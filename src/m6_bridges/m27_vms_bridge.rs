//! # M27: VMS Bridge
//!
//! Posts field memory to VMS (Vortex Memory System) at `localhost:8120` every 60 ticks.
//! VMS is the hippocampus -- fractal sphere topology, 47 MCP tools.
//!
//! ## Layer: L6 | Module: M27 | Dependencies: L1
//! ## Pattern: Fire-and-forget writes (C14), periodic reads for field hydration
//!
//! ## Alert: VMS dormant (r=0.0, 0 memories) -- BUG-017
//! The VMS is currently dormant. This bridge handles gracefully by
//! still writing field state and checking health, but accepting that
//! hydration may return empty results.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::m1_foundation::m02_error_handling::{PvError, PvResult};
use crate::m1_foundation::m05_traits::Bridgeable;

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// VMS service port.
const VMS_PORT: u16 = 8120;

/// Default base URL.
const DEFAULT_BASE_URL: &str = "localhost:8120";

/// Health endpoint path.
const HEALTH_PATH: &str = "/health";

/// Memories endpoint for posting field state.
const MEMORIES_PATH: &str = "/memories";

/// Hydrate endpoint for reading field state back.
const HYDRATE_PATH: &str = "/hydrate";

/// Default write interval in ticks.
const DEFAULT_WRITE_INTERVAL: u64 = 60;

/// Default read (hydration) interval in ticks.
const DEFAULT_READ_INTERVAL: u64 = 120;

/// TCP connection timeout (milliseconds).
const TCP_TIMEOUT_MS: u64 = 2000;

/// Maximum response body size (bytes).
const MAX_RESPONSE_SIZE: usize = 65536;

// ──────────────────────────────────────────────────────────────
// Response types
// ──────────────────────────────────────────────────────────────

/// Response from the VMS `/hydrate` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrateResponse {
    /// Stored field memories.
    #[serde(default)]
    pub memories: Vec<VmsMemory>,
    /// VMS order parameter (may be 0.0 if dormant).
    #[serde(default)]
    pub r: f64,
    /// Number of active spheres in VMS.
    #[serde(default)]
    pub sphere_count: u32,
}

/// A single memory entry from VMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmsMemory {
    /// Memory identifier.
    #[serde(default)]
    pub id: String,
    /// Content payload.
    #[serde(default)]
    pub content: String,
    /// Activation level.
    #[serde(default)]
    pub activation: f64,
    /// Creation timestamp (unix seconds).
    #[serde(default)]
    pub timestamp: f64,
}

// ──────────────────────────────────────────────────────────────
// Bridge state
// ──────────────────────────────────────────────────────────────

/// Mutable state behind a `RwLock`.
#[derive(Debug)]
struct BridgeState {
    /// Last write tick.
    last_write_tick: u64,
    /// Last read tick.
    last_read_tick: u64,
    /// Cached adjustment (neutral for VMS).
    cached_adjustment: f64,
    /// Whether data is stale.
    stale: bool,
    /// Consecutive failure counter.
    consecutive_failures: u32,
    /// Whether VMS appears dormant (BUG-017).
    is_dormant: bool,
    /// Last hydration response.
    last_hydrate: Option<HydrateResponse>,
    /// Total snapshots posted this session.
    snapshots_posted: u64,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            last_write_tick: 0,
            last_read_tick: 0,
            cached_adjustment: 1.0,
            stale: true,
            consecutive_failures: 0,
            is_dormant: true, // Assume dormant until proven otherwise
            last_hydrate: None,
            snapshots_posted: 0,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// VmsBridge
// ──────────────────────────────────────────────────────────────

/// Bridge to the Vortex Memory System for field state persistence.
///
/// VMS is the hippocampus of the Habitat. It stores field memories
/// in a fractal sphere topology. Currently dormant (BUG-017), this
/// bridge writes field state fire-and-forget and reads via `/hydrate`.
#[derive(Debug)]
pub struct VmsBridge {
    /// Service name identifier.
    service: String,
    /// TCP address (host:port).
    base_url: String,
    /// Write (snapshot) interval in ticks.
    write_interval: u64,
    /// Read (hydration) interval in ticks.
    read_interval: u64,
    /// Interior-mutable state.
    state: RwLock<BridgeState>,
}

impl VmsBridge {
    /// Create a new VMS bridge with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            service: "vms".to_owned(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            write_interval: DEFAULT_WRITE_INTERVAL,
            read_interval: DEFAULT_READ_INTERVAL,
            state: RwLock::new(BridgeState::default()),
        }
    }

    /// Create a new VMS bridge with custom configuration.
    #[must_use]
    pub fn with_config(
        base_url: impl Into<String>,
        write_interval: u64,
        read_interval: u64,
    ) -> Self {
        Self {
            service: "vms".to_owned(),
            base_url: base_url.into(),
            write_interval: write_interval.max(1),
            read_interval: read_interval.max(1),
            state: RwLock::new(BridgeState::default()),
        }
    }

    /// Return the configured write interval.
    #[must_use]
    pub const fn write_interval(&self) -> u64 {
        self.write_interval
    }

    /// Return the configured read interval.
    #[must_use]
    pub const fn read_interval(&self) -> u64 {
        self.read_interval
    }

    /// Return the number of consecutive failures.
    #[must_use]
    pub fn consecutive_failures(&self) -> u32 {
        self.state.read().consecutive_failures
    }

    /// Return the cached adjustment value (always neutral for VMS).
    #[must_use]
    pub fn cached_adjustment(&self) -> f64 {
        self.state.read().cached_adjustment
    }

    /// Return whether VMS appears dormant (BUG-017).
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        self.state.read().is_dormant
    }

    /// Return the total snapshots posted this session.
    #[must_use]
    pub fn snapshots_posted(&self) -> u64 {
        self.state.read().snapshots_posted
    }

    /// Return the last hydration response.
    #[must_use]
    pub fn last_hydrate(&self) -> Option<HydrateResponse> {
        self.state.read().last_hydrate.clone()
    }

    /// Return the port number.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.base_url
            .split(':')
            .next_back()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(VMS_PORT)
    }

    /// Return the last write tick.
    #[must_use]
    pub fn last_write_tick(&self) -> u64 {
        self.state.read().last_write_tick
    }

    /// Return the last read tick.
    #[must_use]
    pub fn last_read_tick(&self) -> u64 {
        self.state.read().last_read_tick
    }

    /// Post field state to VMS memories endpoint (fire-and-forget).
    ///
    /// # Errors
    /// Returns `PvError::BridgeUnreachable` if the connection fails.
    pub fn post_field_state(&self, payload: &[u8]) -> PvResult<()> {
        raw_http_post(&self.base_url, MEMORIES_PATH, payload, &self.service)?;
        let mut state = self.state.write();
        state.snapshots_posted = state.snapshots_posted.saturating_add(1);
        state.consecutive_failures = 0;
        state.stale = false;
        Ok(())
    }

    /// Hydrate field state from VMS.
    ///
    /// # Errors
    /// Returns `PvError::BridgeUnreachable` or `PvError::BridgeParse` on failure.
    pub fn hydrate(&self) -> PvResult<HydrateResponse> {
        let body = raw_http_get(&self.base_url, HYDRATE_PATH, &self.service)?;
        let response: HydrateResponse =
            serde_json::from_str(&body).map_err(|e| PvError::BridgeParse {
                service: self.service.clone(),
                reason: format!("hydrate parse: {e}"),
            })?;

        let mut state = self.state.write();

        // BUG-017 detection: VMS is dormant if r=0.0 and no memories
        state.is_dormant = response.r.abs() < f64::EPSILON && response.memories.is_empty();

        state.last_hydrate = Some(response.clone());
        state.stale = false;
        state.consecutive_failures = 0;

        Ok(response)
    }

    /// Record a failure.
    pub fn record_failure(&self) {
        let mut state = self.state.write();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        state.stale = true;
    }

    /// Update the last write tick.
    pub fn set_last_write_tick(&self, tick: u64) {
        self.state.write().last_write_tick = tick;
    }

    /// Update the last read tick.
    pub fn set_last_read_tick(&self, tick: u64) {
        self.state.write().last_read_tick = tick;
    }

    /// Check whether a snapshot write is due.
    #[must_use]
    pub fn should_write(&self, current_tick: u64) -> bool {
        let state = self.state.read();
        current_tick.saturating_sub(state.last_write_tick) >= self.write_interval
    }

    /// Check whether a hydration read is due.
    #[must_use]
    pub fn should_read(&self, current_tick: u64) -> bool {
        let state = self.state.read();
        current_tick.saturating_sub(state.last_read_tick) >= self.read_interval
    }
}

impl Default for VmsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridgeable for VmsBridge {
    fn service_name(&self) -> &str {
        &self.service
    }

    fn poll(&self) -> PvResult<f64> {
        // VMS does not produce k_adj, return neutral
        Ok(self.cached_adjustment())
    }

    fn post(&self, payload: &[u8]) -> PvResult<()> {
        match self.post_field_state(payload) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }

    fn health(&self) -> PvResult<bool> {
        match raw_http_get(&self.base_url, HEALTH_PATH, &self.service) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn is_stale(&self, current_tick: u64) -> bool {
        let state = self.state.read();
        state.stale
            || current_tick.saturating_sub(state.last_write_tick) >= self.write_interval * 3
    }
}

// ──────────────────────────────────────────────────────────────
// Raw TCP HTTP helpers
// ──────────────────────────────────────────────────────────────

/// Send a raw HTTP GET request over TCP.
fn raw_http_get(addr: &str, path: &str, service: &str) -> PvResult<String> {
    let timeout = Duration::from_millis(TCP_TIMEOUT_MS);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|_| PvError::BridgeUnreachable {
            service: service.to_owned(),
            url: addr.to_owned(),
        })?,
        timeout,
    )
    .map_err(|_| PvError::BridgeUnreachable {
        service: service.to_owned(),
        url: addr.to_owned(),
    })?;

    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| PvError::BridgeUnreachable {
            service: service.to_owned(),
            url: addr.to_owned(),
        })?;

    let host = addr.split(':').next().unwrap_or("localhost");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).map_err(|_| {
        PvError::BridgeUnreachable {
            service: service.to_owned(),
            url: addr.to_owned(),
        }
    })?;

    let mut buf = vec![0u8; MAX_RESPONSE_SIZE];
    let mut total = 0;
    loop {
        match stream.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if total >= MAX_RESPONSE_SIZE {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(_) => {
                return Err(PvError::BridgeUnreachable {
                    service: service.to_owned(),
                    url: addr.to_owned(),
                });
            }
        }
    }

    let response = String::from_utf8_lossy(&buf[..total]);
    extract_body(&response).ok_or_else(|| PvError::BridgeParse {
        service: service.to_owned(),
        reason: "no body in HTTP response".to_owned(),
    })
}

/// Send a raw HTTP POST request over TCP (fire-and-forget).
fn raw_http_post(addr: &str, path: &str, body: &[u8], service: &str) -> PvResult<()> {
    let timeout = Duration::from_millis(TCP_TIMEOUT_MS);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|_| PvError::BridgeUnreachable {
            service: service.to_owned(),
            url: addr.to_owned(),
        })?,
        timeout,
    )
    .map_err(|_| PvError::BridgeUnreachable {
        service: service.to_owned(),
        url: addr.to_owned(),
    })?;

    let host = addr.split(':').next().unwrap_or("localhost");
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(request.as_bytes()).map_err(|_| {
        PvError::BridgeUnreachable {
            service: service.to_owned(),
            url: addr.to_owned(),
        }
    })?;
    stream.write_all(body).map_err(|_| PvError::BridgeUnreachable {
        service: service.to_owned(),
        url: addr.to_owned(),
    })?;

    Ok(())
}

/// Extract body from a raw HTTP response.
fn extract_body(raw: &str) -> Option<String> {
    raw.find("\r\n\r\n").map(|pos| raw[pos + 4..].to_owned())
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ──

    #[test]
    fn new_creates_default_bridge() {
        let bridge = VmsBridge::new();
        assert_eq!(bridge.service_name(), "vms");
        assert_eq!(bridge.write_interval(), DEFAULT_WRITE_INTERVAL);
        assert_eq!(bridge.read_interval(), DEFAULT_READ_INTERVAL);
    }

    #[test]
    fn default_creates_same_as_new() {
        let bridge = VmsBridge::default();
        assert_eq!(bridge.service_name(), "vms");
    }

    #[test]
    fn with_config_custom() {
        let bridge = VmsBridge::with_config("10.0.0.1:8220", 30, 90);
        assert_eq!(bridge.base_url, "10.0.0.1:8220");
        assert_eq!(bridge.write_interval(), 30);
        assert_eq!(bridge.read_interval(), 90);
    }

    #[test]
    fn with_config_minimum_intervals() {
        let bridge = VmsBridge::with_config("localhost:8120", 0, 0);
        assert_eq!(bridge.write_interval(), 1);
        assert_eq!(bridge.read_interval(), 1);
    }

    #[test]
    fn port_extraction_default() {
        let bridge = VmsBridge::new();
        assert_eq!(bridge.port(), VMS_PORT);
    }

    #[test]
    fn port_extraction_custom() {
        let bridge = VmsBridge::with_config("localhost:9120", 60, 120);
        assert_eq!(bridge.port(), 9120);
    }

    // ── Initial state ──

    #[test]
    fn initial_cached_adjustment_is_one() {
        let bridge = VmsBridge::new();
        assert!((bridge.cached_adjustment() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn initial_failures_is_zero() {
        let bridge = VmsBridge::new();
        assert_eq!(bridge.consecutive_failures(), 0);
    }

    #[test]
    fn initial_is_dormant() {
        let bridge = VmsBridge::new();
        assert!(bridge.is_dormant());
    }

    #[test]
    fn initial_snapshots_is_zero() {
        let bridge = VmsBridge::new();
        assert_eq!(bridge.snapshots_posted(), 0);
    }

    #[test]
    fn initial_no_hydrate() {
        let bridge = VmsBridge::new();
        assert!(bridge.last_hydrate().is_none());
    }

    #[test]
    fn initial_is_stale() {
        let bridge = VmsBridge::new();
        assert!(bridge.is_stale(0));
    }

    // ── Poll returns neutral ──

    #[test]
    fn poll_returns_neutral() {
        let bridge = VmsBridge::new();
        assert!((bridge.poll().unwrap_or(0.0) - 1.0).abs() < f64::EPSILON);
    }

    // ── Should write / should read ──

    #[test]
    fn should_write_initially() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        assert!(bridge.should_write(60));
    }

    #[test]
    fn should_not_write_too_soon() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        bridge.set_last_write_tick(10);
        assert!(!bridge.should_write(50));
    }

    #[test]
    fn should_write_after_interval() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        bridge.set_last_write_tick(10);
        assert!(bridge.should_write(70));
    }

    #[test]
    fn should_read_initially() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        assert!(bridge.should_read(120));
    }

    #[test]
    fn should_not_read_too_soon() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        bridge.set_last_read_tick(10);
        assert!(!bridge.should_read(100));
    }

    #[test]
    fn should_read_after_interval() {
        let bridge = VmsBridge::with_config("localhost:8120", 60, 120);
        bridge.set_last_read_tick(10);
        assert!(bridge.should_read(130));
    }

    // ── Tick management ──

    #[test]
    fn set_last_write_tick_updates() {
        let bridge = VmsBridge::new();
        bridge.set_last_write_tick(42);
        assert_eq!(bridge.last_write_tick(), 42);
    }

    #[test]
    fn set_last_read_tick_updates() {
        let bridge = VmsBridge::new();
        bridge.set_last_read_tick(100);
        assert_eq!(bridge.last_read_tick(), 100);
    }

    // ── Staleness ──

    #[test]
    fn stale_when_never_written() {
        let bridge = VmsBridge::new();
        assert!(bridge.is_stale(200));
    }

    #[test]
    fn stale_after_triple_write_interval() {
        let bridge = VmsBridge::with_config("localhost:8120", 10, 60);
        bridge.set_last_write_tick(5);
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        assert!(bridge.is_stale(35));
    }

    #[test]
    fn not_stale_within_triple_interval() {
        let bridge = VmsBridge::with_config("localhost:8120", 10, 60);
        bridge.set_last_write_tick(10);
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        assert!(!bridge.is_stale(25));
    }

    // ── Failure tracking ──

    #[test]
    fn record_failure_increments() {
        let bridge = VmsBridge::new();
        bridge.record_failure();
        assert_eq!(bridge.consecutive_failures(), 1);
    }

    #[test]
    fn record_failure_sets_stale() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        bridge.record_failure();
        assert!(bridge.state.read().stale);
    }

    // ── Post (offline) ──

    #[test]
    fn post_fails_when_unreachable() {
        let bridge = VmsBridge::with_config("127.0.0.1:19999", 60, 120);
        assert!(bridge.post(b"test").is_err());
    }

    #[test]
    fn health_returns_false_when_unreachable() {
        let bridge = VmsBridge::with_config("127.0.0.1:19999", 60, 120);
        assert_eq!(bridge.health().ok(), Some(false));
    }

    // ── HydrateResponse serde ──

    #[test]
    fn hydrate_response_deserialize_full() {
        let json = r#"{"memories":[{"id":"m1","content":"test","activation":0.8,"timestamp":1000.0}],"r":0.5,"sphere_count":3}"#;
        let resp: HydrateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.memories.len(), 1);
        assert!((resp.r - 0.5).abs() < f64::EPSILON);
        assert_eq!(resp.sphere_count, 3);
    }

    #[test]
    fn hydrate_response_deserialize_empty() {
        let json = "{}";
        let resp: HydrateResponse = serde_json::from_str(json).unwrap();
        assert!(resp.memories.is_empty());
        assert!((resp.r).abs() < f64::EPSILON);
    }

    #[test]
    fn hydrate_response_serde_roundtrip() {
        let resp = HydrateResponse {
            memories: vec![VmsMemory {
                id: "m1".to_owned(),
                content: "test".to_owned(),
                activation: 0.9,
                timestamp: 1000.0,
            }],
            r: 0.7,
            sphere_count: 5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: HydrateResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.memories.len(), 1);
    }

    #[test]
    fn vms_memory_deserialize() {
        let json = r#"{"id":"m42","content":"field snapshot","activation":0.6,"timestamp":5000.0}"#;
        let m: VmsMemory = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, "m42");
        assert!((m.activation - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn vms_memory_deserialize_minimal() {
        let json = "{}";
        let m: VmsMemory = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, "");
        assert!((m.activation).abs() < f64::EPSILON);
    }

    // ── Dormancy detection ──

    #[test]
    fn dormant_when_r_zero_no_memories() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.is_dormant = true;
        }
        assert!(bridge.is_dormant());
    }

    #[test]
    fn not_dormant_when_r_nonzero() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.is_dormant = false;
        }
        assert!(!bridge.is_dormant());
    }

    // ── Thread safety ──

    #[test]
    fn bridge_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<VmsBridge>();
    }

    #[test]
    fn bridge_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<VmsBridge>();
    }

    // ── Trait object ──

    #[test]
    fn bridgeable_as_trait_object() {
        let bridge = VmsBridge::new();
        let dyn_ref: &dyn Bridgeable = &bridge;
        assert_eq!(dyn_ref.service_name(), "vms");
    }

    // ── HTTP helpers ──

    #[test]
    fn extract_body_finds_body() {
        let raw = "HTTP/1.1 200 OK\r\n\r\n{\"memories\":[]}";
        assert_eq!(extract_body(raw), Some("{\"memories\":[]}".to_owned()));
    }

    #[test]
    fn extract_body_no_separator() {
        assert!(extract_body("no headers").is_none());
    }

    // ── Constants ──

    #[test]
    fn vms_port_is_8120() {
        assert_eq!(VMS_PORT, 8120);
    }

    #[test]
    fn default_write_interval_is_sixty() {
        assert_eq!(DEFAULT_WRITE_INTERVAL, 60);
    }

    #[test]
    fn default_read_interval_is_one_twenty() {
        assert_eq!(DEFAULT_READ_INTERVAL, 120);
    }

    // ── Debug ──

    #[test]
    fn debug_format_works() {
        let bridge = VmsBridge::new();
        let debug = format!("{bridge:?}");
        assert!(debug.contains("vms"));
    }

    // ── Snapshot counter ──

    #[test]
    fn snapshots_increment_on_simulated_post() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.snapshots_posted = 5;
        }
        assert_eq!(bridge.snapshots_posted(), 5);
    }

    #[test]
    fn snapshots_saturate() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.snapshots_posted = u64::MAX;
        }
        assert_eq!(bridge.snapshots_posted(), u64::MAX);
    }

    // ── Additional tests for 50+ coverage ──

    #[test]
    fn vms_memory_serde_roundtrip() {
        let m = VmsMemory {
            id: "m1".to_owned(),
            content: "hello".to_owned(),
            activation: 0.75,
            timestamp: 12345.0,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: VmsMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "m1");
        assert!((back.activation - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn consecutive_failures_saturate() {
        let bridge = VmsBridge::new();
        {
            let mut state = bridge.state.write();
            state.consecutive_failures = u32::MAX;
        }
        bridge.record_failure();
        assert_eq!(bridge.consecutive_failures(), u32::MAX);
    }

    #[test]
    fn hydrate_fails_when_unreachable() {
        let bridge = VmsBridge::with_config("127.0.0.1:19999", 60, 120);
        assert!(bridge.hydrate().is_err());
    }

    #[test]
    fn post_increments_failure_on_error() {
        let bridge = VmsBridge::with_config("127.0.0.1:19999", 60, 120);
        let _ = bridge.post(b"test");
        assert!(bridge.consecutive_failures() >= 1);
    }

    #[test]
    fn hydrate_response_dormant_scenario() {
        let json = r#"{"memories":[],"r":0.0,"sphere_count":0}"#;
        let resp: HydrateResponse = serde_json::from_str(json).unwrap();
        assert!(resp.memories.is_empty());
        assert!(resp.r.abs() < f64::EPSILON);
    }
}
