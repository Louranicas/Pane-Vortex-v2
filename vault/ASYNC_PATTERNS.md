# Async Patterns

> 12 async patterns for tokio-based daemon development.
> Covers shutdown, cancellation, lock ordering, task spawning, timeouts.
> Source: v1 main.rs, ipc.rs + BUG-018 (SIGPIPE)
> See also: CONCURRENCY_PATTERNS.md for Arc/RwLock patterns.

## AP01: Graceful Shutdown via Signal Handler

```rust
let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
    .expect("SIGTERM handler");

tokio::select! {
    _ = sigterm.recv() => {
        info!("SIGTERM received, shutting down");
        shutdown(app_state, bus_state, dbs).await;
    }
    _ = ctrl_c() => {
        info!("Ctrl+C received");
        shutdown(app_state, bus_state, dbs).await;
    }
}
```

Always handle SIGTERM (sent by devenv on stop) AND Ctrl+C (sent by user).

## AP02: File-Based Tracing (Never stdout in Daemons)

```rust
let log_file = std::fs::OpenOptions::new()
    .create(true).append(true)
    .open("/tmp/pane-vortex.log");

tracing_subscriber::fmt()
    .with_writer(std::sync::Mutex::new(log_file?))
    .with_ansi(false)
    .init();
```

When started by devenv, stdout is piped. If devenv exits, SIGPIPE kills the daemon (BUG-018).

## AP03: Timeout on External Operations

```rust
use tokio::time::timeout;

let result = timeout(Duration::from_secs(3), TcpStream::connect(addr)).await;

match result {
    Ok(Ok(stream)) => { /* connected */ }
    Ok(Err(e)) => { warn!("connection error: {e}"); }
    Err(_) => { warn!("connection timeout"); }
}
```

All bridge operations have 3s timeouts.

## AP04: Select for Multi-Source Event Loop

```rust
loop {
    tokio::select! {
        line = reader.read_line(&mut buf) => {
            handle_client_frame(&buf).await?;
            buf.clear();
        }
        event = event_rx.recv() => {
            if pattern_matches(&subscriptions, &event.event_type) {
                write_frame(&mut writer, &BusFrame::Event { .. }).await?;
            }
        }
        _ = shutdown_rx.changed() => { break; }
    }
}
```

## AP05: Spawn with Error Logging

```rust
tokio::spawn(async move {
    if let Err(e) = risky_operation().await {
        error!("background task failed: {e}");
    }
});
```

Never silently drop errors from spawned tasks.

## AP06: Tick Loop with Fixed Interval

```rust
let mut interval = tokio::time::interval(Duration::from_secs(TICK_INTERVAL_SECS));
interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

loop {
    interval.tick().await;
    if let Err(e) = tick_once(&app_state, &bus_state, &dbs).await {
        error!("tick error: {e}");
    }
}
```

MissedTickBehavior::Delay prevents burst-catching if a tick takes longer than the interval.

## AP07: Lock-Then-Drop Pattern

```rust
let decision = {
    let app = app_state.read().await;
    compute_decision(&app)
};

let bus = bus_state.write().await;
bus.event_tx.send(decision_event);
```

Never hold AppState lock while acquiring BusState lock in the same scope.

## AP08: Fire-and-Forget Bridge Writes

```rust
tokio::spawn(async move {
    if let Err(e) = post_field_to_povm(&field_snapshot).await {
        debug!("POVM bridge write failed (non-critical): {e}");
    }
});
```

Bridge writes are spawned as independent tasks. Failures logged at debug level.

## AP09: Bounded Broadcast Channel

```rust
let (event_tx, _) = broadcast::channel::<BusEvent>(EVENT_BUFFER_SIZE);

match event_rx.recv().await {
    Ok(event) => { /* process */ }
    Err(broadcast::error::RecvError::Lagged(n)) => {
        warn!("subscriber lagged by {n} events");
    }
    Err(broadcast::error::RecvError::Closed) => { break; }
}
```

Slow subscribers miss events rather than causing unbounded memory growth.

## AP10: Connection Cleanup with Drop Guard

```rust
struct ConnectionGuard {
    sphere_id: PaneId,
    bus_state: SharedBusState,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let bus = self.bus_state.clone();
        let id = self.sphere_id.clone();
        tokio::spawn(async move {
            let mut state = bus.write().await;
            state.connected_clients.remove(&id);
            state.subscriptions.remove(&id);
        });
    }
}
```

Ensures client state is cleaned up even on panic or unexpected disconnection.

## AP11: Exponential Backoff for Port Binding

```rust
let mut delay = Duration::from_millis(BIND_INITIAL_DELAY_MS);
for attempt in 0..BIND_MAX_RETRIES {
    match TcpListener::bind(addr).await {
        Ok(listener) => return Ok(listener),
        Err(e) if attempt < BIND_MAX_RETRIES - 1 => {
            warn!("bind attempt {attempt} failed: {e}, retrying in {delay:?}");
            tokio::time::sleep(delay).await;
            delay *= 2;
        }
        Err(e) => return Err(e.into()),
    }
}
```

Handles stale processes from unclean shutdowns.

## AP12: Watch Channel for Configuration Hot-Reload

```rust
let (config_tx, config_rx) = watch::channel(initial_config);

// Governance: update config when proposal approved
config_tx.send(new_config).ok();

// Tick loop: read latest config
let config = config_rx.borrow().clone();
```

watch::channel provides single-producer, multi-consumer with latest-value semantics.
