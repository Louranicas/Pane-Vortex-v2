# Error Handling Patterns

> 10 error handling patterns for pane-vortex v2.
> Covers PvError, the ? operator, From impls, context enrichment, and axum integration.
> Source: v1 error handling across 22 modules
> See also: `RUST_CORE_PATTERNS.md` P01 (no unwrap), P16 (error context)

## EP01: Unified Error Enum

All errors flow through a single enum. No ad-hoc error types.

```rust
#[derive(Debug)]
pub enum PvError {
    InvalidInput(String),
    SphereNotFound(String),
    SphereAlreadyRegistered(String),
    SphereCapReached,
    TaskNotFound(String),
    TaskAlreadyClaimed(String),
    CascadeRejected(String),
    CascadeDepthExceeded(u32),
    ConsentViolation(String),
    Database(String),
    Config(String),
    Bridge(String),
    Bus(String),
    ConnectionClosed,
    FrameTooLarge,
    Io(std::io::Error),
    Serde(serde_json::Error),
    Internal(String),
}

pub type PvResult<T> = Result<T, PvError>;
```

## EP02: The ? Operator for Propagation

```rust
pub fn load_config() -> PvResult<PvConfig> {
    let content = std::fs::read_to_string("config/default.toml")?; // io::Error -> PvError::Io
    let config: PvConfig = toml::from_str(&content)?;               // toml::Error -> PvError
    Ok(config)
}
```

The `?` operator converts via `From` implementations. Every external error type
used in the codebase has a `From` impl.

## EP03: From Implementations for External Errors

```rust
impl From<std::io::Error> for PvError {
    fn from(e: std::io::Error) -> Self {
        PvError::Io(e)
    }
}

impl From<serde_json::Error> for PvError {
    fn from(e: serde_json::Error) -> Self {
        PvError::Serde(e)
    }
}

#[cfg(feature = "persistence")]
impl From<rusqlite::Error> for PvError {
    fn from(e: rusqlite::Error) -> Self {
        PvError::Database(e.to_string())
    }
}

impl From<figment::Error> for PvError {
    fn from(e: figment::Error) -> Self {
        PvError::Config(e.to_string())
    }
}
```

## EP04: Context-Rich Error Messages

```rust
// WRONG — no context
Err(PvError::SphereNotFound("not found".into()))

// CORRECT — include the ID
Err(PvError::SphereNotFound(format!("sphere '{sphere_id}' not registered")))

// CORRECT — include the operation
Err(PvError::Database(format!("failed to persist field snapshot at tick {tick}: {e}")))
```

## EP05: Axum IntoResponse Mapping

```rust
impl axum::response::IntoResponse for PvError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            PvError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            PvError::SphereNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            PvError::SphereAlreadyRegistered(msg) => (StatusCode::CONFLICT, msg.clone()),
            PvError::SphereCapReached => (StatusCode::TOO_MANY_REQUESTS, "sphere cap reached".into()),
            PvError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            PvError::TaskAlreadyClaimed(msg) => (StatusCode::CONFLICT, msg.clone()),
            PvError::ConsentViolation(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            PvError::CascadeRejected(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into()),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}
```

Note: Internal errors expose a generic message, not the actual error (security).

## EP06: Error-Tolerant Bridge Operations

```rust
// Bridges must never crash the daemon
pub async fn poll_thermal(state: &SharedThermalState) {
    match fetch_thermal().await {
        Ok(thermal) => {
            *state.write().await = Some(thermal);
        }
        Err(e) => {
            debug!("SYNTHEX thermal poll failed: {e}");
            // Do NOT clear state — keep stale data until refresh
        }
    }
}
```

Pattern: on bridge failure, log and keep stale data. Never propagate bridge errors upward.

## EP07: Validation at the Boundary

```rust
// Validate ONCE at the API/bus boundary, then trust internally
async fn register_sphere(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<RegisterRequest>,
) -> PvResult<impl IntoResponse> {
    validate_sphere_id(&id)?;                    // validate ID
    let phase = validate_phase(body.phase)?;     // validate phase
    let persona = truncate_string(&body.persona, 64); // truncate string

    // Internal functions trust these values are clean
    let mut app = state.write().await;
    app.register_sphere(id, persona, phase)?;
    Ok(StatusCode::CREATED)
}
```

Do not re-validate inside internal functions. Validate at the boundary (API handler, bus frame handler).

## EP08: Fallible Constructors

```rust
impl PaneSphere {
    pub fn new(id: PaneId, persona: String, phase: Phase, frequency: Frequency) -> PvResult<Self> {
        if !phase.is_finite() {
            return Err(PvError::InvalidInput("phase must be finite".into()));
        }
        if !frequency.is_finite() {
            return Err(PvError::InvalidInput("frequency must be finite".into()));
        }
        Ok(Self {
            id,
            persona,
            phase: phase.rem_euclid(TAU),
            frequency: frequency.clamp(FREQUENCY_MIN, FREQUENCY_MAX),
            // ...
        })
    }
}
```

Constructors that take user input return `PvResult`, not bare `Self`.

## EP09: Debug vs Error Logging

```rust
// ERROR: something unexpected that needs attention
error!("failed to persist snapshot: {e}");

// WARN: recoverable situation
warn!("SYNTHEX bridge stale (last poll 60s ago)");

// INFO: normal lifecycle events
info!(tick = 42, r = 0.847, "tick complete");

// DEBUG: diagnostic detail (not shown in production)
debug!("coupling step: r changed from {old} to {new}");
```

Rule: bridge failures are `debug!` (expected during startup). State corruption is `error!`.

## EP10: Never Panic in Production

```rust
// WRONG
assert!(phase.is_finite()); // panics in production

// CORRECT
if !phase.is_finite() {
    return Err(PvError::InvalidInput("phase not finite".into()));
}
```

`assert!` and `panic!` are for tests only. Production code uses `Result`.
Enforced by `[lints.clippy] panic = "deny"` in Cargo.toml.
