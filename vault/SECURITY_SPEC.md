# Security Specification

> Security hardening measures for pane-vortex v2.
> Covers network binding, resource caps, input validation, socket permissions, NaN guards.
> Source: v1 audit Session 016c + Session 039 discoveries
> Plan: `MASTERPLAN.md` V3.3 | Obsidian: `[[Pane-Vortex — Fleet Coordination Daemon]]`

## Overview

Pane-vortex is a local-only daemon for fleet coordination. The security model assumes
a trusted local user but defends against:
1. Accidental external exposure (loopback binding)
2. Resource exhaustion (sphere caps, memory limits)
3. Input injection (string truncation, NaN guards)
4. Unauthorized socket access (file permissions)
5. Data sovereignty violations (consent gates)

## 1. Network Binding

### 1.1 Loopback-Only Default

```rust
/// m10_api_server.rs
let bind_addr = config.server.bind_addr; // "127.0.0.1" from default.toml
```

**Default:** `127.0.0.1:8132` (loopback only).
**Override:** `BIND_ADDR=0.0.0.0` environment variable (for container deployments).
**NEVER** bind to 0.0.0.0 in production without explicit intent.

### 1.2 Port Selection

Port 8132 is assigned to pane-vortex in the ULTRAPLATE service registry.
The binary retries binding up to 5 times with exponential backoff (500ms, 1s, 2s, 4s, 8s)
to handle stale processes from unclean shutdowns.

## 2. Resource Caps

### 2.1 Sphere Cap

```rust
const SPHERE_CAP: usize = 200;
```

Prevents O(N^2) memory exhaustion from coupling connections. At 200 spheres,
the connection matrix has 200*199 = 39,800 entries. Each Connection is ~128 bytes,
so maximum coupling memory is ~5MB.

**Enforcement:** POST /sphere/{id}/register returns HTTP 429 when cap reached.

### 2.2 Memory Cap Per Sphere

```rust
const MEMORY_MAX_COUNT: usize = 500;
```

Each SphereMemory is ~200 bytes. 500 memories = ~100KB per sphere.
Maximum: 200 spheres * 500 memories = 20MB.

**Enforcement:** Amortised batch prune at MEMORY_MAX_COUNT + 50 (550), triggered
every MEMORY_PRUNE_INTERVAL (200) steps.

### 2.3 Bus Connection Limit

```rust
const MAX_BUS_CONNECTIONS: usize = 50;
```

Unix socket connections are capped at 50. Each connection holds a ~64KB read buffer.
Maximum bus memory: 50 * 64KB = 3.2MB.

### 2.4 Event Buffer

```rust
const EVENT_BUFFER_SIZE: usize = 256;
```

Bounded broadcast channel for bus events. Slow subscribers miss events rather than
causing unbounded memory growth. `broadcast::Receiver::recv()` returns `Lagged(n)`
when events are dropped.

### 2.5 Request Body Limit

```rust
const BODY_LIMIT_BYTES: usize = 65_536; // 65KB
```

Enforced by axum `DefaultBodyLimit`. Prevents large payload attacks.

### 2.6 Ghost Trace Limit

```rust
const GHOST_MAX: usize = 20;
```

Only the 20 most recent ghost traces are retained. Oldest evicted on new deregistration.

### 2.7 Cascade Depth Limit

```rust
const CASCADE_MAX_DEPTH: u32 = 5;
```

Prevents infinite cascade chains. Cascades at depth >= 5 are rejected.

### 2.8 Cascade Rate Limit

```rust
const CASCADE_RATE_LIMIT: usize = 10; // per minute per connection
```

Prevents cascade storms.

## 3. Input Validation

### 3.1 String Truncation

All user-provided strings are truncated to prevent unbounded storage:

| Field | Max Length | Enforcement |
|-------|-----------|-------------|
| sphere_id | 128 chars | Reject if exceeds |
| persona | 64 chars | Truncate |
| last_tool | 128 chars | Truncate (v1 fix: was unbounded) |
| summary | 500 chars | Truncate |
| description (task) | 1000 chars | Truncate |
| brief (cascade) | 5000 chars | Truncate |
| reason (reject) | 500 chars | Truncate |
| pattern (subscription) | 128 chars | Reject if exceeds |
| payload (task) | 65536 chars | Reject if exceeds (matches body limit) |

**Implementation pattern:**
```rust
/// m06_validation.rs
pub fn truncate_string(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
```

**CRITICAL:** Use `.chars().take(n)` NOT byte slicing. Byte slicing panics on
multi-byte UTF-8 (v1 C1 bug fix).

### 3.2 NaN Guards

All floating-point inputs are validated before storage:

```rust
/// m06_validation.rs
pub fn validate_phase(phase: f64) -> PvResult<f64> {
    if !phase.is_finite() {
        return Err(PvError::InvalidInput("phase must be finite".into()));
    }
    Ok(phase.rem_euclid(TAU))
}

pub fn validate_frequency(freq: f64) -> PvResult<f64> {
    if !freq.is_finite() {
        return Err(PvError::InvalidInput("frequency must be finite".into()));
    }
    Ok(freq.clamp(FREQUENCY_MIN, FREQUENCY_MAX))
}

pub fn validate_weight(w: f64) -> PvResult<f64> {
    if !w.is_finite() {
        return Err(PvError::InvalidInput("weight must be finite".into()));
    }
    Ok(w.clamp(0.0, 1.0))
}
```

**Where applied:**
- POST /sphere/{id}/phase — phase validation
- POST /sphere/{id}/steer — strength validation
- POST /coupling/inject — weight validation
- Coupling step inner loop — all trig results checked
- Bridge responses — all parsed floats checked

### 3.3 Sphere ID Validation

```rust
pub fn validate_sphere_id(id: &str) -> PvResult<()> {
    if id.is_empty() || id.len() > 128 {
        return Err(PvError::InvalidInput("sphere ID must be 1-128 chars".into()));
    }
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err(PvError::InvalidInput("sphere ID contains invalid chars".into()));
    }
    Ok(())
}
```

## 4. Socket Permissions

### 4.1 IPC Bus Socket

```rust
const SOCKET_PERMISSIONS: u32 = 0o700;
```

The Unix domain socket at `/run/user/1000/pane-vortex-bus.sock` is created with
permissions `0700` (owner read/write/execute only). No group or world access.

```rust
fn set_socket_permissions(path: &Path) -> PvResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(SOCKET_PERMISSIONS);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}
```

### 4.2 Snapshot Directory

```rust
fn create_snapshot_dir(dir: &Path) -> PvResult<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}
```

The snapshot directory at `~/.local/share/pane-vortex/` contains serialized sphere
data including tool summaries. Restricted to owner-only (0700).

### 4.3 Database Files

Database files in `data/` inherit the directory permissions. No special enforcement
beyond default umask.

## 5. Consent as Security

The consent gate (m28_consent_gate.rs) is both a sovereignty mechanism and a
security boundary:

### 5.1 What Consent Controls

| Control | Without Consent | With Consent |
|---------|----------------|--------------|
| External k_mod | Applied globally | Per-sphere max_k_adjustment |
| Phase injection | Full strength | Scaled by receptivity * consent |
| Cascade dispatch | Sent to any sphere | Only to accept_cascade == true |
| Observation | All spheres | Only accept_observation == true |
| Nvim monitoring | All spheres | Only accept_nvim_monitoring == true |
| Hebbian learning | All spheres | Opt-out via opt_out_hebbian |
| Cross-activation | All spheres | Opt-out via opt_out_cross_activation |

### 5.2 Default Consent

All consent flags default to `true` (opt-in). A sphere must actively change its
consent declaration to restrict what the field can do to it.

### 5.3 Consent Persistence

Consent declarations are persisted in `data/bus_tracking.db` table `consent_declarations`.
They survive daemon restarts and are loaded during sphere re-registration.

## 6. Denial of Service Mitigation

### 6.1 Tick Loop Protection

The tick loop runs on a fixed 5s interval. Even if coupling computation takes 4.9s,
the next tick waits until the full interval. This prevents runaway compute from
starving API handlers.

### 6.2 EmergencyCoherence Cap

When HasBlockedAgents triggers EmergencyCoherence, the targets vector is capped at 50:

```rust
let targets = blocked_spheres.into_iter().take(50).collect();
```

Prevents O(N) phase injection when many spheres are blocked.

### 6.3 Event Flood Protection

Bus events are generated at most once per tick per event type. Field events are
rate-limited to the tick interval (5s). Bridge events are rate-limited to their
poll intervals (30-60s).

## 7. Daemon Security

### 7.1 Signal Handling

```rust
// SIGTERM: graceful shutdown with state persistence
tokio::signal::unix::signal(SignalKind::terminate())
    .expect("SIGTERM handler")
    .recv().await;
// Persist state before exit
```

### 7.2 SIGPIPE Protection

**CRITICAL (BUG-018):** Never write to stdout in daemon mode. Stdout is piped to
devenv; if devenv exits, the pipe breaks and writes trigger SIGPIPE -> immediate death.

```rust
// CORRECT: Write to file
tracing_subscriber::fmt()
    .with_writer(std::sync::Mutex::new(log_file))
    .init();

// WRONG: Write to stdout (SIGPIPE death)
tracing_subscriber::fmt().init();
```

### 7.3 PID File

No internal PID management. The devenv process manager handles PID tracking,
process lifecycle, and restart-on-crash. Attempting internal PID management caused
conflicts in v1 (Session 026).

## 8. Testing Strategy

| Test | Property |
|------|----------|
| Loopback verification | Connection from non-loopback rejected |
| Sphere cap enforcement | 201st registration returns 429 |
| Memory cap enforcement | 551st memory triggers prune |
| String truncation | 129-char tool name truncated to 128 |
| NaN rejection | NaN phase returns 400 |
| Socket permissions | Socket file mode is 0700 |
| Body limit | 66KB request returns 413 |
| Cascade depth limit | depth=6 cascade rejected |
| UTF-8 safety | Multi-byte emoji in tool name truncated correctly |

## 9. Anti-Patterns

- **AP-1:** Binding to 0.0.0.0 in default config
- **AP-2:** Byte slicing for string truncation (panics on UTF-8)
- **AP-3:** Using `unwrap()` in input validation paths
- **AP-4:** Trusting client-provided sphere_id without validation
- **AP-5:** Writing tracing to stdout in daemon mode (SIGPIPE)
- **AP-6:** Internal PID management conflicting with devenv
- **AP-7:** Unbounded Vec for ghost traces or messages
