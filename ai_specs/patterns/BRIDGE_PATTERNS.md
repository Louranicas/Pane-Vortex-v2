# Bridge Patterns

> 8 bridge patterns for external service communication.
> Covers raw TCP HTTP, fire-and-forget, consent gate, timeout, TSV, and polling.
> Source: v1 synthex_bridge.rs, nexus_bridge.rs, me_bridge.rs, povm_bridge.rs, bridge.rs
> Modules: m22-m28 | See also: CONSENT_SPEC.md for consent gate details

## BP01: Raw TCP HTTP (No hyper Dependency)

All bridges use raw TCP streams with hand-crafted HTTP/1.1 to avoid adding a
heavy HTTP client dependency (hyper, reqwest).

```rust
async fn raw_http_get(addr: &str, path: &str) -> PvResult<String> {
    let timeout_dur = Duration::from_secs(3);
    let mut stream = tokio::time::timeout(
        timeout_dur,
        TcpStream::connect(addr)
    ).await
        .map_err(|_| PvError::Bridge(format!("connect timeout to {addr}")))??;

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await?;
    stream.shutdown().await?; // half-close: signal end of request

    let mut response = String::new();
    BufReader::new(stream).read_to_string(&mut response).await?;

    // Extract body after \r\n\r\n
    let body = response.split("\r\n\r\n").nth(1)
        .ok_or_else(|| PvError::Bridge("malformed HTTP response".into()))?;
    Ok(body.to_string())
}

async fn raw_http_post(addr: &str, path: &str, body: &str, content_type: &str) -> PvResult<String> {
    let timeout_dur = Duration::from_secs(3);
    let mut stream = tokio::time::timeout(
        timeout_dur,
        TcpStream::connect(addr)
    ).await
        .map_err(|_| PvError::Bridge(format!("connect timeout to {addr}")))??;

    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await?;
    stream.shutdown().await?;

    let mut response = String::new();
    BufReader::new(stream).read_to_string(&mut response).await?;
    let resp_body = response.split("\r\n\r\n").nth(1).unwrap_or("");
    Ok(resp_body.to_string())
}
```

Key properties:
- `Connection: close` for each request (no keep-alive)
- `shutdown()` for clean half-close (v1 bug: transport half-close missing)
- 3s timeout on connect
- Body extracted by splitting on `\r\n\r\n`

## BP02: Fire-and-Forget Write Pattern

```rust
/// Post field snapshot to POVM engine. Non-blocking, error-tolerant.
pub async fn post_field_snapshot(field: &FieldState) -> PvResult<()> {
    let body = serde_json::to_string(field)?;
    match raw_http_post(POVM_ADDR, "/pathway/ingest", &body, "application/json").await {
        Ok(_) => Ok(()),
        Err(e) => {
            debug!("POVM snapshot write failed (non-critical): {e}");
            Ok(()) // Swallow error — write failures are non-critical
        }
    }
}
```

Pattern: bridge writes always return Ok. Failures are logged at debug level and
swallowed. The daemon continues regardless of bridge health.

## BP03: Polling with SharedState Cache

```rust
pub type SharedThermalState = Arc<RwLock<Option<ThermalState>>>;

pub async fn poll_thermal(state: &SharedThermalState, tick: u64) {
    if tick % THERMAL_POLL_INTERVAL != 0 { return; } // poll every 6 ticks

    match fetch_thermal_from_synthex().await {
        Ok(thermal) => {
            *state.write().await = Some(thermal);
        }
        Err(e) => {
            debug!("SYNTHEX poll failed: {e}");
            // Keep stale data — do NOT clear the cache
        }
    }
}
```

Pattern: poll at fixed intervals, cache in SharedState. On failure, keep stale data.
The consumer checks `is_stale()` and weights stale data less.

## BP04: TSV for Reasoning Memory (NOT JSON)

```rust
/// Post to Reasoning Memory. CRITICAL: RM expects TAB-SEPARATED VALUES, not JSON.
pub async fn post_to_rm(category: &str, content: &str, confidence: f64, ttl: u64) -> PvResult<()> {
    let body = format!("{category}\tpane-vortex\t{confidence}\t{ttl}\t{content}");
    raw_http_post(RM_ADDR, "/put", &body, "text/tab-separated-values").await?;
    Ok(())
}
```

v1 trap: sending JSON to RM silently fails (RM parses TSV, not JSON).

## BP05: Consent Gate on Bridge Influence

```rust
/// Apply SYNTHEX thermal influence through the consent gate
pub fn apply_thermal_influence(
    thermal: &ThermalState,
    spheres: &HashMap<PaneId, PaneSphere>,
    budget: (f64, f64),
) -> f64 {
    let raw_adj = thermal.thermal_deviation() * 0.1; // scale to small adjustment
    consent_gated_k_adjustment(raw_adj, spheres, budget)
}
```

Every bridge that produces a k_adjustment MUST route through `consent_gated_k_adjustment()`.
Direct modification of `k_modulation` without consent is a consent violation.

## BP06: Stale Data Detection

```rust
impl ThermalState {
    pub fn is_stale(&self, now: f64) -> bool {
        now - self.fetched_at > 60.0 // stale after 2 poll intervals
    }
}

// In conductor: reduce influence of stale data
let thermal_weight = if thermal.is_stale(now_secs()) { 0.5 } else { 1.0 };
let thermal_adj = thermal.thermal_deviation() * 0.1 * thermal_weight;
```

Pattern: bridge data has a `fetched_at` timestamp. Consumers check freshness and
reduce weight for stale data rather than ignoring it entirely.

## BP07: Bridge Health Events

```rust
pub async fn check_bridge_health(
    thermal: &SharedThermalState,
    nexus: &SharedNexusState,
    me: &SharedMeState,
    bus: &SharedBusState,
) {
    let now = now_secs();
    let bridges = vec![
        ("synthex", thermal.read().await.as_ref().map_or(true, |t| t.is_stale(now))),
        ("nexus", nexus.read().await.as_ref().map_or(true, |n| n.is_stale(now))),
        ("me", me.read().await.as_ref().map_or(true, |m| m.is_stale(now))),
    ];

    for (name, stale) in &bridges {
        if *stale {
            let _ = bus.read().await.event_tx.send(BusEvent {
                event_type: format!("bridge.health"),
                data: serde_json::json!({ "bridge": name, "status": "stale" }),
                tick: None,
            });
        }
    }
}
```

Bridge health is monitored and reported via bus events.

## BP08: Bridge Address Resolution

```rust
fn resolve_addr(service: &str, default: &str) -> String {
    // Check environment variable first
    let env_key = format!("{}_ADDR", service.to_uppercase());
    if let Ok(addr) = std::env::var(&env_key) {
        return addr;
    }
    default.to_string()
}

// Usage:
const SYNTHEX_DEFAULT: &str = "127.0.0.1:8090";
let synthex_addr = resolve_addr("synthex", SYNTHEX_DEFAULT);
```

All bridge addresses are configurable via environment variables:
- `SYNTHEX_ADDR` (default: 127.0.0.1:8090)
- `NEXUS_ADDR` (default: 127.0.0.1:8100)
- `ME_ADDR` (default: 127.0.0.1:8080)
- `POVM_ADDR` (default: 127.0.0.1:8125)
- `RM_ADDR` (default: 127.0.0.1:8130)
- `VMS_ADDR` (default: 127.0.0.1:8120)
