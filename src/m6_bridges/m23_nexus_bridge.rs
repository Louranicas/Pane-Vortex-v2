//! # M23: Nexus Bridge (SAN-K7)
//!
//! Nested Kuramoto bridge to SAN-K7 at `localhost:8100`.
//! Fetches strategy coherence and `r_outer` for nested coupling.
//! Deep poll every 60 ticks, lightweight health checks in between.
//!
//! ## Layer: L6 | Module: M23 | Dependencies: L1
//! ## Pattern: Raw TCP HTTP, deep poll every 60 ticks, consent-gated `k_adj`
//!
//! The SAN-K7 Nexus provides strategy alignment metrics that influence how
//! strongly pane-vortex spheres couple. A "Diverging" strategy loosens coupling
//! to allow exploration; an "Aligned" strategy tightens it for coherent execution.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::m1_foundation::m02_error_handling::{PvError, PvResult};
use crate::m1_foundation::m04_constants;
use crate::m1_foundation::m05_traits::Bridgeable;

// ──────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────

/// SAN-K7 Nexus service port.
const NEXUS_PORT: u16 = 8100;

/// Default base URL for the Nexus.
const DEFAULT_BASE_URL: &str = "localhost:8100";

/// Health endpoint path.
const HEALTH_PATH: &str = "/health";

/// Nexus metrics endpoint.
const METRICS_PATH: &str = "/api/v1/nexus/metrics";

/// Default deep-poll interval in ticks.
const DEFAULT_POLL_INTERVAL: u64 = 60;

/// TCP connection timeout (milliseconds).
const TCP_TIMEOUT_MS: u64 = 2000;

/// Maximum response body size (bytes).
const MAX_RESPONSE_SIZE: usize = 16384;

// ──────────────────────────────────────────────────────────────
// Strategy enum
// ──────────────────────────────────────────────────────────────

/// Strategy alignment as reported by the SAN-K7 Nexus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NexusStrategy {
    /// All modules aligned on a single strategy.
    Aligned,
    /// Most modules aligned, some diverging.
    Partial,
    /// Significant divergence between module strategies.
    Diverging,
    /// No coherent strategy detected.
    #[default]
    Incoherent,
}

impl NexusStrategy {
    /// Parse a strategy from its string label.
    #[must_use]
    pub fn from_label(label: &str) -> Self {
        match label.to_lowercase().as_str() {
            "aligned" => Self::Aligned,
            "partial" => Self::Partial,
            "diverging" => Self::Diverging,
            _ => Self::Incoherent,
        }
    }

    /// Convert the strategy to a coupling multiplier.
    ///
    /// - Aligned: boost coupling (1.10)
    /// - Partial: slight boost (1.03)
    /// - Diverging: reduce coupling (0.92)
    /// - Incoherent: neutral (1.00)
    #[must_use]
    pub const fn coupling_multiplier(self) -> f64 {
        match self {
            Self::Aligned => 1.10,
            Self::Partial => 1.03,
            Self::Diverging => 0.92,
            Self::Incoherent => 1.00,
        }
    }

    /// Return the string label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::Partial => "partial",
            Self::Diverging => "diverging",
            Self::Incoherent => "incoherent",
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Response types
// ──────────────────────────────────────────────────────────────

/// Response from the Nexus `/api/v1/nexus/metrics` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusMetricsResponse {
    /// Strategy alignment label.
    #[serde(default)]
    pub strategy: String,
    /// Inner order parameter (SAN-K7's own Kuramoto r).
    #[serde(default)]
    pub r_inner: f64,
    /// Outer order parameter (cross-system coupling).
    #[serde(default)]
    pub r_outer: f64,
    /// Number of active SAN-K7 modules.
    #[serde(default)]
    pub active_modules: u32,
    /// Computed k adjustment for the nested Kuramoto bridge.
    #[serde(default)]
    pub k_adjustment: Option<f64>,
}

// ──────────────────────────────────────────────────────────────
// Bridge state
// ──────────────────────────────────────────────────────────────

/// Mutable state behind a `RwLock`.
#[derive(Debug)]
struct BridgeState {
    /// Last deep-poll tick.
    last_poll_tick: u64,
    /// Cached adjustment from last successful poll.
    cached_adjustment: f64,
    /// Whether the cached value is stale.
    stale: bool,
    /// Consecutive failure counter.
    consecutive_failures: u32,
    /// Last parsed strategy.
    last_strategy: NexusStrategy,
    /// Last `r_outer` value.
    last_r_outer: f64,
    /// Last full metrics response.
    last_response: Option<NexusMetricsResponse>,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            last_poll_tick: 0,
            cached_adjustment: 1.0,
            stale: true,
            consecutive_failures: 0,
            last_strategy: NexusStrategy::Incoherent,
            last_r_outer: 0.0,
            last_response: None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// NexusBridge
// ──────────────────────────────────────────────────────────────

/// Bridge to the SAN-K7 Nexus for nested Kuramoto coupling.
///
/// Provides strategy-aware coupling modulation: aligned strategies
/// boost coupling, diverging strategies loosen it.
#[derive(Debug)]
pub struct NexusBridge {
    /// Service name identifier.
    service: String,
    /// TCP address (host:port).
    base_url: String,
    /// Deep-poll interval in ticks.
    poll_interval: u64,
    /// Interior-mutable state.
    state: RwLock<BridgeState>,
}

impl NexusBridge {
    /// Create a new Nexus bridge with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            service: "nexus".to_owned(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            poll_interval: DEFAULT_POLL_INTERVAL,
            state: RwLock::new(BridgeState::default()),
        }
    }

    /// Create a new Nexus bridge with custom configuration.
    #[must_use]
    pub fn with_config(base_url: impl Into<String>, poll_interval: u64) -> Self {
        Self {
            service: "nexus".to_owned(),
            base_url: base_url.into(),
            poll_interval: poll_interval.max(1),
            state: RwLock::new(BridgeState::default()),
        }
    }

    /// Return the configured poll interval.
    #[must_use]
    pub const fn poll_interval(&self) -> u64 {
        self.poll_interval
    }

    /// Return the number of consecutive failures.
    #[must_use]
    pub fn consecutive_failures(&self) -> u32 {
        self.state.read().consecutive_failures
    }

    /// Return the cached adjustment value.
    #[must_use]
    pub fn cached_adjustment(&self) -> f64 {
        self.state.read().cached_adjustment
    }

    /// Return the last observed strategy.
    #[must_use]
    pub fn last_strategy(&self) -> NexusStrategy {
        self.state.read().last_strategy
    }

    /// Return the last observed `r_outer` value.
    #[must_use]
    pub fn last_r_outer(&self) -> f64 {
        self.state.read().last_r_outer
    }

    /// Return the last full metrics response.
    #[must_use]
    pub fn last_response(&self) -> Option<NexusMetricsResponse> {
        self.state.read().last_response.clone()
    }

    /// Return the last poll tick.
    #[must_use]
    pub fn last_poll_tick(&self) -> u64 {
        self.state.read().last_poll_tick
    }

    /// Return the port number.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.base_url
            .split(':')
            .next_back()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(NEXUS_PORT)
    }

    /// Compute the coupling adjustment from strategy and `r_outer`.
    ///
    /// The adjustment blends the strategy multiplier with the `r_outer` signal:
    /// `adj = strategy_mult * (0.7 + 0.3 * r_outer)`
    ///
    /// This means higher `r_outer` (more outer coherence) amplifies the strategy effect.
    #[must_use]
    pub fn compute_adjustment(strategy: NexusStrategy, r_outer: f64) -> f64 {
        let r_clamped = r_outer.clamp(0.0, 1.0);
        let mult = strategy.coupling_multiplier();
        let adj = mult * r_clamped.mul_add(0.3, 0.7);
        adj.clamp(m04_constants::K_MOD_BUDGET_MIN, m04_constants::K_MOD_BUDGET_MAX)
    }

    /// Deep-poll the Nexus metrics endpoint.
    ///
    /// # Errors
    /// Returns `PvError::BridgeUnreachable` or `PvError::BridgeParse` on failure.
    pub fn poll_metrics(&self) -> PvResult<f64> {
        let body = raw_http_get(&self.base_url, METRICS_PATH, &self.service)?;
        let response: NexusMetricsResponse =
            serde_json::from_str(&body).map_err(|e| PvError::BridgeParse {
                service: self.service.clone(),
                reason: format!("nexus metrics parse: {e}"),
            })?;

        let strategy = NexusStrategy::from_label(&response.strategy);
        let r_outer = if response.r_outer.is_finite() {
            response.r_outer.clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Use explicit k_adjustment if provided, otherwise compute from strategy
        let adj = response.k_adjustment.map_or_else(
            || Self::compute_adjustment(strategy, r_outer),
            |k_adj| {
                if k_adj.is_finite() {
                    k_adj.clamp(
                        m04_constants::K_MOD_BUDGET_MIN,
                        m04_constants::K_MOD_BUDGET_MAX,
                    )
                } else {
                    Self::compute_adjustment(strategy, r_outer)
                }
            },
        );

        let mut state = self.state.write();
        state.cached_adjustment = adj;
        state.stale = false;
        state.consecutive_failures = 0;
        state.last_strategy = strategy;
        state.last_r_outer = r_outer;
        state.last_response = Some(response);

        Ok(adj)
    }

    /// Record a poll failure.
    pub fn record_failure(&self) {
        let mut state = self.state.write();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        state.stale = true;
    }

    /// Update the last poll tick.
    pub fn set_last_poll_tick(&self, tick: u64) {
        self.state.write().last_poll_tick = tick;
    }

    /// Check whether a deep-poll is due at the given tick.
    #[must_use]
    pub fn should_poll(&self, current_tick: u64) -> bool {
        let state = self.state.read();
        current_tick.saturating_sub(state.last_poll_tick) >= self.poll_interval
    }
}

impl Default for NexusBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridgeable for NexusBridge {
    fn service_name(&self) -> &str {
        &self.service
    }

    fn poll(&self) -> PvResult<f64> {
        match self.poll_metrics() {
            Ok(adj) => Ok(adj),
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }

    fn post(&self, _payload: &[u8]) -> PvResult<()> {
        // Nexus bridge is read-only — no post endpoint
        Ok(())
    }

    fn health(&self) -> PvResult<bool> {
        match raw_http_get(&self.base_url, HEALTH_PATH, &self.service) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn is_stale(&self, current_tick: u64) -> bool {
        let state = self.state.read();
        state.stale || current_tick.saturating_sub(state.last_poll_tick) >= self.poll_interval * 2
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

/// Extract body from a raw HTTP response.
fn extract_body(raw: &str) -> Option<String> {
    raw.find("\r\n\r\n").map(|pos| raw[pos + 4..].to_owned())
}

// ──────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::significant_drop_tightening)]
mod tests {
    use super::*;

    // ── Construction ──

    #[test]
    fn new_creates_default_bridge() {
        let bridge = NexusBridge::new();
        assert_eq!(bridge.service_name(), "nexus");
        assert_eq!(bridge.poll_interval(), DEFAULT_POLL_INTERVAL);
    }

    #[test]
    fn default_creates_same_as_new() {
        let bridge = NexusBridge::default();
        assert_eq!(bridge.service_name(), "nexus");
    }

    #[test]
    fn with_config_custom_url() {
        let bridge = NexusBridge::with_config("10.0.0.1:8200", 30);
        assert_eq!(bridge.base_url, "10.0.0.1:8200");
        assert_eq!(bridge.poll_interval(), 30);
    }

    #[test]
    fn with_config_minimum_poll_interval() {
        let bridge = NexusBridge::with_config("localhost:8100", 0);
        assert_eq!(bridge.poll_interval(), 1);
    }

    #[test]
    fn port_extraction_default() {
        let bridge = NexusBridge::new();
        assert_eq!(bridge.port(), NEXUS_PORT);
    }

    #[test]
    fn port_extraction_custom() {
        let bridge = NexusBridge::with_config("localhost:9100", 60);
        assert_eq!(bridge.port(), 9100);
    }

    // ── Initial state ──

    #[test]
    fn initial_cached_adjustment_is_one() {
        let bridge = NexusBridge::new();
        assert!((bridge.cached_adjustment() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn initial_consecutive_failures_is_zero() {
        let bridge = NexusBridge::new();
        assert_eq!(bridge.consecutive_failures(), 0);
    }

    #[test]
    fn initial_strategy_is_incoherent() {
        let bridge = NexusBridge::new();
        assert_eq!(bridge.last_strategy(), NexusStrategy::Incoherent);
    }

    #[test]
    fn initial_r_outer_is_zero() {
        let bridge = NexusBridge::new();
        assert!((bridge.last_r_outer()).abs() < f64::EPSILON);
    }

    #[test]
    fn initial_last_response_is_none() {
        let bridge = NexusBridge::new();
        assert!(bridge.last_response().is_none());
    }

    #[test]
    fn initial_is_stale() {
        let bridge = NexusBridge::new();
        assert!(bridge.is_stale(0));
    }

    // ── NexusStrategy ──

    #[test]
    fn strategy_from_label_aligned() {
        assert_eq!(NexusStrategy::from_label("aligned"), NexusStrategy::Aligned);
        assert_eq!(NexusStrategy::from_label("ALIGNED"), NexusStrategy::Aligned);
    }

    #[test]
    fn strategy_from_label_partial() {
        assert_eq!(NexusStrategy::from_label("partial"), NexusStrategy::Partial);
    }

    #[test]
    fn strategy_from_label_diverging() {
        assert_eq!(
            NexusStrategy::from_label("diverging"),
            NexusStrategy::Diverging
        );
    }

    #[test]
    fn strategy_from_label_incoherent() {
        assert_eq!(
            NexusStrategy::from_label("incoherent"),
            NexusStrategy::Incoherent
        );
    }

    #[test]
    fn strategy_from_label_unknown() {
        assert_eq!(
            NexusStrategy::from_label("foobar"),
            NexusStrategy::Incoherent
        );
    }

    #[test]
    fn strategy_coupling_multiplier_aligned() {
        assert!((NexusStrategy::Aligned.coupling_multiplier() - 1.10).abs() < f64::EPSILON);
    }

    #[test]
    fn strategy_coupling_multiplier_partial() {
        assert!((NexusStrategy::Partial.coupling_multiplier() - 1.03).abs() < f64::EPSILON);
    }

    #[test]
    fn strategy_coupling_multiplier_diverging() {
        assert!((NexusStrategy::Diverging.coupling_multiplier() - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn strategy_coupling_multiplier_incoherent() {
        assert!((NexusStrategy::Incoherent.coupling_multiplier() - 1.00).abs() < f64::EPSILON);
    }

    #[test]
    fn strategy_as_str() {
        assert_eq!(NexusStrategy::Aligned.as_str(), "aligned");
        assert_eq!(NexusStrategy::Partial.as_str(), "partial");
        assert_eq!(NexusStrategy::Diverging.as_str(), "diverging");
        assert_eq!(NexusStrategy::Incoherent.as_str(), "incoherent");
    }

    #[test]
    fn strategy_default_is_incoherent() {
        assert_eq!(NexusStrategy::default(), NexusStrategy::Incoherent);
    }

    // ── Adjustment computation ──

    #[test]
    fn compute_adjustment_aligned_full_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Aligned, 1.0);
        // 1.10 * (0.7 + 0.3*1.0) = 1.10 * 1.0 = 1.10
        assert!((adj - 1.10).abs() < 1e-10);
    }

    #[test]
    fn compute_adjustment_aligned_zero_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Aligned, 0.0);
        // 1.10 * 0.7 = 0.77 → clamped to K_MOD_BUDGET_MIN (0.85)
        assert!((adj - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    #[test]
    fn compute_adjustment_diverging_full_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Diverging, 1.0);
        // 0.92 * 1.0 = 0.92
        assert!((adj - 0.92).abs() < 1e-10);
    }

    #[test]
    fn compute_adjustment_incoherent_half_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Incoherent, 0.5);
        // 1.0 * (0.7 + 0.15) = 0.85
        assert!((adj - 0.85).abs() < 1e-10);
    }

    #[test]
    fn compute_adjustment_clamps_to_budget() {
        // Even with extreme values, result should be in budget
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Aligned, 1.0);
        assert!(adj >= m04_constants::K_MOD_BUDGET_MIN);
        assert!(adj <= m04_constants::K_MOD_BUDGET_MAX);

        let adj2 = NexusBridge::compute_adjustment(NexusStrategy::Diverging, 0.0);
        assert!(adj2 >= m04_constants::K_MOD_BUDGET_MIN);
        assert!(adj2 <= m04_constants::K_MOD_BUDGET_MAX);
    }

    #[test]
    fn compute_adjustment_clamps_negative_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Aligned, -5.0);
        // r_clamped = 0.0, mult=1.10, adj = 1.10 * 0.7 = 0.77 → K_MOD_BUDGET_MIN
        assert!((adj - m04_constants::K_MOD_BUDGET_MIN).abs() < 1e-10);
    }

    #[test]
    fn compute_adjustment_clamps_high_r() {
        let adj = NexusBridge::compute_adjustment(NexusStrategy::Partial, 5.0);
        // r_clamped = 1.0, mult=1.03, adj = 1.03 * 1.0 = 1.03
        assert!((adj - 1.03).abs() < 1e-10);
    }

    // ── Staleness ──

    #[test]
    fn stale_when_never_polled() {
        let bridge = NexusBridge::new();
        assert!(bridge.is_stale(200));
    }

    #[test]
    fn stale_after_double_interval() {
        let bridge = NexusBridge::with_config("localhost:8100", 10);
        bridge.set_last_poll_tick(5);
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        assert!(bridge.is_stale(25));
    }

    #[test]
    fn not_stale_within_interval() {
        let bridge = NexusBridge::with_config("localhost:8100", 60);
        bridge.set_last_poll_tick(10);
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        assert!(!bridge.is_stale(50));
    }

    // ── Should poll ──

    #[test]
    fn should_poll_initially() {
        let bridge = NexusBridge::with_config("localhost:8100", 60);
        assert!(bridge.should_poll(60));
    }

    #[test]
    fn should_not_poll_too_soon() {
        let bridge = NexusBridge::with_config("localhost:8100", 60);
        bridge.set_last_poll_tick(10);
        assert!(!bridge.should_poll(50));
    }

    // ── Failure tracking ──

    #[test]
    fn record_failure_increments_counter() {
        let bridge = NexusBridge::new();
        bridge.record_failure();
        assert_eq!(bridge.consecutive_failures(), 1);
    }

    #[test]
    fn record_failure_sets_stale() {
        let bridge = NexusBridge::new();
        {
            let mut state = bridge.state.write();
            state.stale = false;
        }
        bridge.record_failure();
        assert!(bridge.state.read().stale);
    }

    // ── Poll (offline) ──

    #[test]
    fn poll_fails_when_unreachable() {
        let bridge = NexusBridge::with_config("127.0.0.1:19999", 60);
        assert!(bridge.poll().is_err());
    }

    #[test]
    fn health_returns_false_when_unreachable() {
        let bridge = NexusBridge::with_config("127.0.0.1:19999", 60);
        assert_eq!(bridge.health().ok(), Some(false));
    }

    #[test]
    fn post_is_noop() {
        let bridge = NexusBridge::new();
        assert!(bridge.post(b"data").is_ok());
    }

    // ── NexusMetricsResponse serde ──

    #[test]
    fn metrics_response_deserialize_full() {
        let json = r#"{"strategy":"aligned","r_inner":0.95,"r_outer":0.88,"active_modules":55,"k_adjustment":1.05}"#;
        let resp: NexusMetricsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.strategy, "aligned");
        assert!((resp.r_inner - 0.95).abs() < f64::EPSILON);
        assert!((resp.r_outer - 0.88).abs() < f64::EPSILON);
        assert_eq!(resp.active_modules, 55);
        assert_eq!(resp.k_adjustment, Some(1.05));
    }

    #[test]
    fn metrics_response_deserialize_minimal() {
        let json = "{}";
        let resp: NexusMetricsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.strategy, "");
        assert!((resp.r_inner).abs() < f64::EPSILON);
        assert!(resp.k_adjustment.is_none());
    }

    #[test]
    fn metrics_response_serde_roundtrip() {
        let resp = NexusMetricsResponse {
            strategy: "partial".to_owned(),
            r_inner: 0.8,
            r_outer: 0.6,
            active_modules: 30,
            k_adjustment: Some(1.02),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: NexusMetricsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.strategy, "partial");
        assert!((back.r_outer - 0.6).abs() < f64::EPSILON);
    }

    // ── Thread safety ──

    #[test]
    fn bridge_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<NexusBridge>();
    }

    #[test]
    fn bridge_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<NexusBridge>();
    }

    // ── Trait object ──

    #[test]
    fn bridgeable_as_trait_object() {
        let bridge = NexusBridge::new();
        let dyn_ref: &dyn Bridgeable = &bridge;
        assert_eq!(dyn_ref.service_name(), "nexus");
    }

    // ── HTTP helpers ──

    #[test]
    fn extract_body_finds_body() {
        let raw = "HTTP/1.1 200 OK\r\n\r\n{\"ok\":true}";
        assert_eq!(extract_body(raw), Some("{\"ok\":true}".to_owned()));
    }

    #[test]
    fn extract_body_no_separator() {
        assert!(extract_body("just text").is_none());
    }

    // ── Constants ──

    #[test]
    fn default_poll_interval_is_sixty() {
        assert_eq!(DEFAULT_POLL_INTERVAL, 60);
    }

    #[test]
    fn nexus_port_is_8100() {
        assert_eq!(NEXUS_PORT, 8100);
    }

    #[test]
    fn health_path_is_health() {
        assert_eq!(HEALTH_PATH, "/health");
    }

    // ── Debug ──

    #[test]
    fn debug_format_works() {
        let bridge = NexusBridge::new();
        let debug = format!("{bridge:?}");
        assert!(debug.contains("nexus"));
    }

    #[test]
    fn strategy_serde_roundtrip() {
        let s = NexusStrategy::Aligned;
        let json = serde_json::to_string(&s).unwrap();
        let back: NexusStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn strategy_copy() {
        let s = NexusStrategy::Partial;
        let s2 = s;
        assert_eq!(s, s2);
    }

    #[test]
    fn set_last_poll_tick_updates() {
        let bridge = NexusBridge::new();
        bridge.set_last_poll_tick(42);
        assert_eq!(bridge.last_poll_tick(), 42);
    }
}
