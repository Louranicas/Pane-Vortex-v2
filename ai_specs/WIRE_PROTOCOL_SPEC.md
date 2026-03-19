# Wire Protocol Specification

> NDJSON wire protocol for the PV IPC bus.
> All frames are single-line JSON objects terminated by `\n`.
> Schema: `.claude/schemas/bus_frame.schema.json`
> Spec: `IPC_BUS_SPEC.md` for bus architecture
> Plan: `MASTERPLAN.md` | Obsidian: `[[Pane-Vortex IPC Bus — Session 019b]]`

## Overview

The wire protocol uses Newline-Delimited JSON (NDJSON). Each frame is a JSON object
on a single line. The `"type"` field discriminates frame variants. All string fields
have maximum lengths enforced server-side. Unknown fields are ignored (forward compatibility).

## 1. Frame Types

### 1.1 Handshake (Client -> Server)

Sent immediately after connection. Must arrive within 5 seconds.

```json
{
  "type": "handshake",
  "id": "sphere-alpha-01",
  "version": "2.0"
}
```

| Field | Type | Max Length | Required | Description |
|-------|------|-----------|----------|-------------|
| type | string | — | YES | Must be `"handshake"` |
| id | string | 128 | YES | Sphere identifier (matches registration) |
| version | string | 16 | YES | Protocol version (currently `"2.0"`) |

### 1.2 Welcome (Server -> Client)

Sent in response to a valid handshake.

```json
{
  "type": "welcome",
  "server_version": "2.0.0",
  "tick": 42,
  "sphere_count": 3,
  "r": 0.847
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | string | YES | `"welcome"` |
| server_version | string | YES | Server binary version |
| tick | u64 | YES | Current tick number |
| sphere_count | usize | YES | Active sphere count |
| r | f64 | YES | Current order parameter |

### 1.3 Subscribe (Client -> Server)

Register interest in event patterns. Can be sent multiple times to add patterns.

```json
{
  "type": "subscribe",
  "patterns": ["field.*", "task.*", "cascade.*"]
}
```

| Field | Type | Max | Required | Description |
|-------|------|-----|----------|-------------|
| type | string | — | YES | `"subscribe"` |
| patterns | string[] | 20 patterns, 128 chars each | YES | Glob patterns for event matching |

### 1.4 Subscribed (Server -> Client)

Confirmation of subscription registration.

```json
{
  "type": "subscribed",
  "patterns": ["field.*", "task.*", "cascade.*"],
  "total_subscriptions": 3
}
```

### 1.5 Submit (Client -> Server)

Submit a task to the queue.

```json
{
  "type": "submit",
  "description": "Run cargo test on m16_coupling_network",
  "target": "any_idle",
  "target_type": "any_idle",
  "payload": "{\"cmd\":\"cargo test\",\"module\":\"m16\"}",
  "tags": ["test", "coupling"]
}
```

| Field | Type | Max Length | Required | Description |
|-------|------|-----------|----------|-------------|
| type | string | — | YES | `"submit"` |
| description | string | 1000 | YES | Human-readable task description |
| target | string | 128 | NO | Specific sphere or routing hint |
| target_type | string | 32 | YES | `specific`, `any_idle`, `field_driven`, `willing` |
| payload | string | 65536 | NO | JSON payload for the task |
| tags | string[] | 10 tags, 64 chars each | NO | Task tags for filtering |

### 1.6 Task Submitted (Server -> Client)

Confirmation of task submission with assigned ID.

```json
{
  "type": "task_submitted",
  "task_id": "task-a1b2c3d4",
  "status": "submitted",
  "submitted_at": "2026-03-19T14:30:00Z"
}
```

### 1.7 Event (Server -> Client)

Broadcast event matching client's subscriptions.

```json
{
  "type": "event",
  "event_type": "field.decision",
  "source": "tick_loop",
  "tick": 42,
  "data": {
    "action": "Stable",
    "r": 0.847,
    "k_mod": 1.02,
    "sphere_count": 3,
    "modulation_breakdown": {
      "conductor_k_mod": 1.0,
      "synthex_influence": 0.01,
      "nexus_influence": 0.01,
      "me_influence": 0.0,
      "consent_scale": 0.95,
      "effective_k": 2.34
    }
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | string | YES | `"event"` |
| event_type | string | YES | Dot-separated event type |
| source | string | NO | What generated this event |
| tick | u64 | NO | Tick when event was generated |
| data | object | YES | Event-specific payload |

### 1.8 Cascade Handoff (Client -> Server)

Dispatch a context cascade to another sphere.

```json
{
  "type": "cascade_handoff",
  "target": "sphere-beta-02",
  "brief": "Continue implementing m16_coupling_network.rs. Focus on step_inner(). See patterns/CONCURRENCY_PATTERNS.md for lock ordering.",
  "depth": 1
}
```

| Field | Type | Max Length | Required | Description |
|-------|------|-----------|----------|-------------|
| type | string | — | YES | `"cascade_handoff"` |
| target | string | 128 | YES | Target sphere ID |
| brief | string | 5000 | YES | Context brief (markdown) |
| depth | u32 | — | NO | Cascade depth (default 0, max 5) |

### 1.9 Cascade Ack (Client -> Server)

Acknowledge receipt of a cascade handoff.

```json
{
  "type": "cascade_ack",
  "source": "sphere-alpha-01",
  "status": "accepted"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | string | YES | `"cascade_ack"` |
| source | string | YES | Original sender sphere ID |
| status | string | YES | `"accepted"` or `"rejected"` |

### 1.10 Reject Cascade (Client -> Server)

Explicitly reject a cascade (NA-P-7: cascade rejection right).

```json
{
  "type": "reject_cascade",
  "source": "sphere-alpha-01",
  "reason": "Currently at capacity — 3 tasks in progress"
}
```

| Field | Type | Max Length | Required | Description |
|-------|------|-----------|----------|-------------|
| type | string | — | YES | `"reject_cascade"` |
| source | string | 128 | YES | Original sender sphere ID |
| reason | string | 500 | YES | Human-readable rejection reason |

### 1.11 Claim (Client -> Server)

Claim a task from the queue.

```json
{
  "type": "claim",
  "task_id": "task-a1b2c3d4"
}
```

### 1.12 Complete (Client -> Server)

Mark a task as completed.

```json
{
  "type": "complete",
  "task_id": "task-a1b2c3d4",
  "result": "{\"status\":\"pass\",\"tests\":35}"
}
```

### 1.13 Fail (Client -> Server)

Mark a task as failed.

```json
{
  "type": "fail",
  "task_id": "task-a1b2c3d4",
  "error": "cargo test failed: 2 failures in m16_coupling_network"
}
```

## 2. Serde Implementation

All frames use `#[serde(tag = "type")]` for the type discriminator:

```rust
/// m30_bus_types.rs
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
}
```

### 2.1 NDJSON Framing

```rust
// Writing a frame
fn write_frame<W: AsyncWrite>(writer: &mut W, frame: &BusFrame) -> Result<()> {
    let json = serde_json::to_string(frame)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

// Reading a frame
fn read_frame<R: AsyncBufRead>(reader: &mut R) -> Result<BusFrame> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 { return Err(PvError::ConnectionClosed); }
    if n > 65536 { return Err(PvError::FrameTooLarge); }
    let frame: BusFrame = serde_json::from_str(&line)?;
    Ok(frame)
}
```

## 3. Error Handling

### 3.1 Error Frames

On invalid input, the server sends an error frame and continues:

```json
{
  "type": "error",
  "code": "invalid_frame",
  "message": "Unknown frame type: query"
}
```

Error codes:
| Code | Meaning |
|------|---------|
| `invalid_frame` | Malformed JSON or unknown type |
| `handshake_required` | Frame sent before handshake |
| `handshake_timeout` | 5s elapsed without handshake |
| `task_not_found` | Task ID does not exist |
| `already_claimed` | Task already claimed by another sphere |
| `rate_limited` | Too many frames per second |
| `frame_too_large` | Line exceeds 65536 bytes |
| `cascade_rejected` | Target sphere rejected the cascade |

### 3.2 Connection Close

Either side can close the connection:
- Client: sends EOF (closes write half)
- Server: sends EOF on shutdown, allowing clients to drain
- On unexpected close: server cleans up subscriptions and marks client disconnected

## 4. Rate Limiting

| Resource | Limit | Per |
|----------|-------|-----|
| Frames | 100 | per second per connection |
| Cascades | 10 | per minute per connection |
| Subscriptions | 20 | per connection |
| Tasks submitted | 60 | per minute per connection |

## 5. Backward Compatibility

The `version` field in handshake allows protocol versioning:
- `"1.0"` — v1 protocol (basic frames, no reject_cascade, no consent)
- `"2.0"` — v2 protocol (full frame set, consent-aware routing)

Server supports both versions. v1 clients receive a subset of events.

## 6. Testing Strategy

| Test | Property |
|------|----------|
| Parse all 13 frame types | Roundtrip serialization/deserialization |
| Reject oversized frames | > 65536 bytes returns error |
| Handshake timeout | No handshake in 5s = disconnect |
| Glob matching | `field.*` matches `field.tick`, not `task.submitted` |
| NDJSON boundary | Multiple frames in one TCP read (line splitting) |
| Incomplete JSON | Partial line buffering until newline |
