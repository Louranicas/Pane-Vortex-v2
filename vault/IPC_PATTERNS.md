# IPC Patterns

> 10 IPC patterns for NDJSON framing, backpressure, reconnection, and socket management.
> Source: v1 ipc.rs, bus.rs, client.rs | Spec: WIRE_PROTOCOL_SPEC.md
> See also: ASYNC_PATTERNS.md for tokio socket patterns

## IP01: NDJSON Line Framing

```rust
// Write: one JSON object per line, terminated by \n
async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, frame: &BusFrame) -> PvResult<()> {
    let json = serde_json::to_string(frame)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

// Read: buffer lines, parse each as JSON
async fn read_frame<R: AsyncBufRead + Unpin>(reader: &mut R) -> PvResult<BusFrame> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 { return Err(PvError::ConnectionClosed); }
    if n > 65536 { return Err(PvError::FrameTooLarge); }
    let frame: BusFrame = serde_json::from_str(line.trim())?;
    Ok(frame)
}
```

Key properties:
- One complete JSON object per line (no multi-line JSON)
- Newline is the frame delimiter
- Line length capped at 65536 bytes (matches HTTP body limit)
- `trim()` handles trailing `\r\n` on Windows

## IP02: Handshake-First Protocol

```rust
async fn handle_new_connection(stream: UnixStream) -> PvResult<PaneId> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Read handshake with timeout
    let frame = tokio::time::timeout(
        Duration::from_secs(5),
        read_frame(&mut reader)
    ).await
        .map_err(|_| PvError::Bus("handshake timeout".into()))??;

    match frame {
        BusFrame::Handshake { id, version } => {
            validate_sphere_id(&id)?;
            // Send welcome
            write_frame(&mut writer, &BusFrame::Welcome {
                server_version: env!("CARGO_PKG_VERSION").into(),
                tick: current_tick(),
                sphere_count: sphere_count(),
                r: current_r(),
            }).await?;
            Ok(id)
        }
        _ => Err(PvError::Bus("expected handshake frame".into())),
    }
}
```

The first frame MUST be a handshake. Anything else is rejected.

## IP03: Glob Pattern Matching for Subscriptions

```rust
fn pattern_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == "*" { return true; }

    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    let event_parts: Vec<&str> = event_type.split('.').collect();

    if pattern_parts.len() != event_parts.len() { return false; }

    pattern_parts.iter().zip(event_parts.iter()).all(|(p, e)| *p == "*" || p == e)
}

// Examples:
// "field.*" matches "field.tick", "field.decision"
// "field.*" does NOT match "field.decision.urgent" (different depth)
// "*.tick" matches "field.tick"
// "*" matches everything
```

No recursive `**` support. Patterns match at the same depth only.

## IP04: Backpressure via Bounded Channel

```rust
let (event_tx, _) = broadcast::channel::<BusEvent>(256);

// Producer (tick loop): never blocks
let _ = event_tx.send(event); // returns Err if no receivers, OK to ignore

// Consumer (connection handler): detects lag
match event_rx.recv().await {
    Ok(event) => { write_frame(&mut writer, &event_to_frame(event)).await?; }
    Err(broadcast::error::RecvError::Lagged(n)) => {
        warn!("{sphere_id} lagged by {n} events");
        // Optionally: send a lag notification frame
    }
    Err(broadcast::error::RecvError::Closed) => { break; }
}
```

The producer (tick loop) NEVER blocks on event sending. Slow consumers miss events.

## IP05: Socket Cleanup on Startup

```rust
fn prepare_socket_path(path: &Path) -> PvResult<()> {
    // Remove stale socket from previous crash
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
```

Always clean up the socket file before binding. A stale socket from a crashed daemon
will prevent the new instance from starting.

## IP06: Per-Connection Event Filtering

```rust
struct ClientState {
    id: PaneId,
    subscriptions: Vec<String>,
    event_rx: broadcast::Receiver<BusEvent>,
}

impl ClientState {
    fn wants_event(&self, event_type: &str) -> bool {
        self.subscriptions.iter().any(|p| pattern_matches(p, event_type))
    }
}
```

Each connection maintains its own subscription list. Events are filtered per-connection
before writing to the socket, not at the broadcast level.

## IP07: Task Routing Logic

```rust
fn route_task(task: &BusTask, spheres: &HashMap<PaneId, PaneSphere>) -> Option<PaneId> {
    match task.target_type {
        TargetType::Specific => {
            task.target.as_ref().and_then(|t| spheres.get(t).map(|_| t.clone()))
        }
        TargetType::AnyIdle => {
            spheres.iter()
                .filter(|(_, s)| s.status == PaneStatus::Idle)
                .map(|(id, _)| id.clone())
                .next()
        }
        TargetType::FieldDriven => {
            None // Decision engine selects target
        }
        TargetType::Willing => {
            spheres.iter()
                .filter(|(_, s)| s.consent.accept_cascade && s.receptivity > 0.3)
                .min_by(|(_, a), (_, b)| a.total_steps.cmp(&b.total_steps))
                .map(|(id, _)| id.clone())
        }
    }
}
```

## IP08: Cascade Depth Limiting

```rust
async fn handle_cascade(frame: BusFrame::CascadeHandoff, sender: &PaneId) -> PvResult<()> {
    let depth = frame.depth.unwrap_or(0);
    if depth >= CASCADE_MAX_DEPTH {
        return Err(PvError::CascadeDepthExceeded(depth));
    }
    // Proceed with dispatch, incrementing depth
    dispatch_cascade(sender, &frame.target, &frame.brief, depth + 1).await
}
```

Maximum depth of 5 prevents infinite cascade chains.

## IP09: Rate Limiting Per Connection

```rust
struct RateLimiter {
    frame_count: u32,
    last_reset: Instant,
    cascade_count: u32,
    cascade_last_reset: Instant,
}

impl RateLimiter {
    fn check_frame(&mut self) -> PvResult<()> {
        if self.last_reset.elapsed() > Duration::from_secs(1) {
            self.frame_count = 0;
            self.last_reset = Instant::now();
        }
        self.frame_count += 1;
        if self.frame_count > 100 {
            return Err(PvError::Bus("rate limited: >100 frames/sec".into()));
        }
        Ok(())
    }

    fn check_cascade(&mut self) -> PvResult<()> {
        if self.cascade_last_reset.elapsed() > Duration::from_secs(60) {
            self.cascade_count = 0;
            self.cascade_last_reset = Instant::now();
        }
        self.cascade_count += 1;
        if self.cascade_count > CASCADE_RATE_LIMIT {
            return Err(PvError::Bus("rate limited: >10 cascades/min".into()));
        }
        Ok(())
    }
}
```

## IP10: Client Binary Connection Pattern

```rust
// pane-vortex-client: connect, handshake, execute, exit
async fn client_main(subcommand: Subcommand) -> PvResult<()> {
    let stream = UnixStream::connect(SOCKET_PATH).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Handshake
    write_frame(&mut writer, &BusFrame::Handshake {
        id: format!("client-{}", std::process::id()),
        version: "2.0".into(),
    }).await?;

    let welcome = read_frame(&mut reader).await?;
    // Verify welcome...

    // Execute subcommand
    match subcommand {
        Subcommand::Subscribe { patterns } => {
            write_frame(&mut writer, &BusFrame::Subscribe { patterns }).await?;
            loop {
                let event = read_frame(&mut reader).await?;
                println!("{}", serde_json::to_string(&event)?);
            }
        }
        Subcommand::Submit { description, target } => {
            write_frame(&mut writer, &BusFrame::Submit { description, target, .. }).await?;
            let response = read_frame(&mut reader).await?;
            println!("{}", serde_json::to_string(&response)?);
        }
        // ...
    }
}
```
