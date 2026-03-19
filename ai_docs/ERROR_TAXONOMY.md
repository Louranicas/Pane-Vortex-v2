---
title: "Pane-Vortex V2 — Error Taxonomy"
date: 2026-03-19
tags: [errors, taxonomy, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian: "[[The Habitat — Integrated Master Plan V3]]"
---

# Pane-Vortex V2 — Error Taxonomy

> PvError enum variants mapped to modules, categories, and recovery strategies.
> Defined in `m02_error_handling`. Used everywhere via `PvResult<T>`.

---

## Error Categories

PvError is a 6-variant enum covering all failure modes:

```rust
pub enum PvError {
    Field(FieldError),
    Bridge(BridgeError),
    Bus(BusError),
    Persistence(PersistenceError),
    Validation(ValidationError),
    Config(ConfigError),
}
```

---

## 1. Field Errors

Failures in the Kuramoto field dynamics (L3, L4, L5).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `PhaseNaN` | m11, m16 | Phase update produced NaN | Reset to 0.0, log warning |
| `FrequencyOutOfRange` | m11, m06 | Frequency outside [0.001, 10.0] | Clamp to bounds |
| `SphereNotFound(PaneId)` | m11, m15 | Sphere ID not in registry | Return 404 to caller |
| `SphereLimitExceeded` | m15 | Registration exceeds SPHERE_CAP (200) | Reject registration |
| `DuplicateSphere(PaneId)` | m15 | Sphere already registered | Deregister first, then re-register |
| `ChimeraComputationFailed` | m13 | Phase sort or gap computation failed | Return empty ChimeraResult |
| `TunnelDetectionFailed` | m12 | Phase diff computation overflowed | Skip tunnel detection this tick |
| `MemoryOverflow` | m21 | Memory count exceeded MAX+50 before prune | Force immediate prune |
| `InvalidPhaseRegion` | m14 | Semantic phase region not in [0, TAU) | Reject steer message |
| `GhostNotFound(PaneId)` | m12 | Ghost trace ID not in deque | Return 404 |
| `WarmupActive` | m15 | Operation attempted during warmup | Return Recovering decision |

### Field Error Design Principles

- **NaN is never propagated.** Any NaN is caught by `is_finite()` guard (pattern P08) and replaced with a safe default.
- **Out-of-range values are clamped, not rejected.** Frequency, phase, and strength all clamp to valid ranges.
- **Sphere operations fail explicitly.** Registration, deregistration, and status updates return clear errors on failure.

---

## 2. Bridge Errors

Failures in external service communication (L6).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `ConnectionRefused(service)` | m22-m27 | Target service not running | Log, skip this poll cycle |
| `ConnectionTimeout(service)` | m22-m27 | TCP connect timed out | Log, skip this poll cycle |
| `ReadTimeout(service)` | m22-m27 | Response read timed out | Log, use cached value |
| `InvalidResponse(service)` | m22-m27 | JSON parse failed on response body | Log, skip this poll cycle |
| `HttpError(service, code)` | m22-m27 | Non-200 HTTP status | Log, skip this poll cycle |
| `ConsentDenied(sphere_id)` | m28 | Sphere opted out of modulation | Zero out k_adjustment for this sphere |
| `KModBudgetExceeded` | m28 | Total k_mod outside [0.85, 1.15] | Clamp to budget bounds |
| `RmFormatError` | m26 | TSV formatting failed | Log, skip post |

### Bridge Error Design Principles

- **All bridge errors are non-fatal.** The tick loop continues regardless of bridge failures.
- **Fire-and-forget writes never block.** Write failures are logged but don't affect field dynamics.
- **Read failures use cached values.** Last successful poll result is retained until the next successful poll.
- **Consent denial is not an error condition.** It is the expected behavior when a sphere has opted out.

---

## 3. Bus Errors

Failures in the IPC bus and task queue (L7).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `SocketBindFailed(path)` | m29 | Unix socket already exists or permission denied | Remove stale socket, retry |
| `SocketAcceptFailed` | m29 | Error accepting new connection | Log, continue listening |
| `ConnectionDropped(sphere_id)` | m29 | Client disconnected unexpectedly | Remove from connections map |
| `FrameParseError(line)` | m30 | NDJSON line not valid JSON or missing fields | Send Error frame back to client |
| `UnknownFrameType(type_)` | m30 | Frame type not in recognized set | Send Error frame back to client |
| `TaskNotFound(task_id)` | m30 | Task ID not in queue | Return 404 to caller |
| `InvalidTransition(from, to)` | m30 | Task status transition not allowed | Return error to caller |
| `TaskAlreadyClaimed(task_id)` | m30 | Task already claimed by another sphere | Return conflict to caller |
| `CascadeRateLimited(source)` | m33 | Cascade rate exceeded limit (10/min) | Return rate limit error |
| `CascadeRejected(target)` | m33 | Target sphere rejected the cascade | Re-route to next eligible |
| `SubscriptionInvalid(pattern)` | m29 | Glob pattern malformed | Send Error frame back to client |
| `EventBufferFull` | m29 | mpsc channel capacity (256) exhausted | Drop oldest event, log warning |
| `HandshakeRequired` | m29 | Frame received before handshake | Send Error frame, disconnect |

### Bus Error Design Principles

- **Connection errors are isolated.** One bad connection does not affect others.
- **Parse errors respond with Error frames.** The client receives a structured error, not a disconnect.
- **Rate limits are per-source.** One sphere being rate-limited does not affect others.
- **Handshake is mandatory.** All frames before handshake are rejected.

---

## 4. Persistence Errors

Failures in SQLite operations (L7 m36).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `DatabaseOpenFailed(path)` | m36 | File not found or permissions | Create directory, retry |
| `MigrationFailed(migration)` | m36 | SQL migration error | Log error, abort startup |
| `InsertFailed(table)` | m36 | Insert query failed | Log, skip this write |
| `QueryFailed(table)` | m36 | Select query failed | Log, return None |
| `WalBusy` | m36 | WAL busy timeout exceeded | Retry with longer timeout |
| `IntegrityCheckFailed(db)` | m36 | PRAGMA integrity_check failed | Log critical, continue read-only |
| `SnapshotCorrupted` | m36 | Snapshot data failed validation | Skip restore, start fresh |

### Persistence Error Design Principles

- **Write failures are non-fatal.** Missing a snapshot is acceptable; crashing is not.
- **Migration failures abort startup.** Schema must be correct before the daemon runs.
- **WAL mode is mandatory.** All connections open with `PRAGMA journal_mode=WAL` (pattern P15).
- **Integrity checks run at startup.** Corrupt databases are flagged but the daemon continues.

---

## 5. Validation Errors

Input validation failures (L1 m06).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `PhaseOutOfRange(value)` | m06 | Phase not in [0, TAU) | Wrap with rem_euclid(TAU) |
| `FrequencyOutOfRange(value)` | m06 | Frequency outside [0.001, 10.0] | Clamp to bounds |
| `SphereIdTooLong(len)` | m06 | Sphere ID exceeds 128 chars | Truncate or reject |
| `SphereIdEmpty` | m06 | Empty sphere ID string | Reject with 400 |
| `StringTooLong(field, len)` | m06 | String field exceeds max length | Truncate to limit |
| `NotFinite(field)` | m06 | Value is NaN or infinite | Replace with default |
| `KModOutOfRange(value)` | m06 | k_mod outside allowed range | Clamp to budget |
| `InvalidJson(detail)` | m10 | Request body JSON parse error | Return 400 with detail |
| `BodyTooLarge(size)` | m10 | Request body exceeds 65KB limit | Return 413 |

### Validation Error Design Principles

- **Validate at the boundary.** All inputs validated in API handlers or bus frame parsing.
- **Prefer clamping over rejection.** For numeric values, clamp to valid range. For strings, truncate.
- **Return structured errors.** API responses include error category, field name, and helpful message.

---

## 6. Config Errors

Configuration loading failures (L1 m03).

| Variant | Module | Cause | Recovery |
|---------|--------|-------|----------|
| `FileNotFound(path)` | m03 | Config TOML file missing | Use compiled defaults |
| `ParseError(detail)` | m03 | TOML syntax error | Abort startup with clear message |
| `InvalidValue(field, reason)` | m03 | Value outside acceptable range | Abort startup with suggestion |
| `MissingField(field)` | m03 | Required field not present | Use default if available |
| `EnvVarConflict(var)` | m03 | Environment variable overrides conflict | Log warning, env takes precedence |

### Config Error Design Principles

- **Defaults are always available.** Config file missing = use defaults. Config partially missing = merge with defaults.
- **Invalid config aborts startup.** Bad config should never produce a running daemon with wrong behavior.
- **Environment variables win.** `PV2_` prefixed env vars override both default.toml and production.toml.

---

## Error Conversion Table

| Source Type | PvError Variant | Via |
|-------------|----------------|-----|
| `std::io::Error` | `Bridge(ConnectionRefused)` or `Bus(SocketBindFailed)` | `impl From<std::io::Error>` |
| `serde_json::Error` | `Validation(InvalidJson)` or `Bridge(InvalidResponse)` | `impl From<serde_json::Error>` |
| `rusqlite::Error` | `Persistence(*)` | `impl From<rusqlite::Error>` |
| `toml::de::Error` | `Config(ParseError)` | `impl From<toml::de::Error>` |
| `figment::Error` | `Config(*)` | `impl From<figment::Error>` |
| `uuid::Error` | `Validation(InvalidJson)` | `impl From<uuid::Error>` |

---

## HTTP Status Code Mapping

| PvError Category | HTTP Status | When |
|-----------------|-------------|------|
| `Validation(*)` | 400 Bad Request | Invalid input from client |
| `Field(SphereNotFound)` | 404 Not Found | Unknown sphere ID |
| `Field(GhostNotFound)` | 404 Not Found | Unknown ghost ID |
| `Bus(TaskNotFound)` | 404 Not Found | Unknown task ID |
| `Field(SphereLimitExceeded)` | 429 Too Many Requests | Sphere cap reached |
| `Bus(CascadeRateLimited)` | 429 Too Many Requests | Rate limit exceeded |
| `Bus(TaskAlreadyClaimed)` | 409 Conflict | Task already claimed |
| `Validation(BodyTooLarge)` | 413 Payload Too Large | Body > 65KB |
| `Persistence(*)` | 500 Internal Server Error | Database failure |
| `Config(*)` | 500 Internal Server Error | Config issue at runtime |

---

## Cross-References

- **[CODE_MODULE_MAP.md](CODE_MODULE_MAP.md)** — Module-level type definitions
- **[modules/L1_FOUNDATION.md](modules/L1_FOUNDATION.md)** — m02_error_handling details
- **[.claude/patterns.json](../.claude/patterns.json)** — Pattern P03 (error propagation), P08 (NaN guard)
- **[.claude/anti_patterns.json](../.claude/anti_patterns.json)** — AP01 (unwrap in production)
- **Obsidian:** `[[Pane-Vortex — Fleet Coordination Daemon]]` (V1 error handling)
