# Concurrency Patterns

> 12 concurrency patterns for Arc/RwLock, channel capacity, and spawn safety.
> Covers the shared state model, lock ordering, reader/writer patterns.
> Source: v1 state.rs, api.rs, main.rs | Bug: deadlock recurred 2x in v1
> See also: `ASYNC_PATTERNS.md` for tokio-specific patterns.

## CP01: Arc<RwLock<T>> for Shared State

```rust
pub type SharedState = Arc<RwLock<AppState>>;
pub type SharedBusState = Arc<RwLock<BusState>>;
pub type SharedThermalState = Arc<RwLock<Option<ThermalState>>>;
```

Pattern: wrap in `Arc` for shared ownership, `RwLock` for interior mutability.
Use `tokio::sync::RwLock` (not `std::sync::RwLock`) for async compatibility.

## CP02: Lock Ordering — AppState Before BusState

```rust
// RULE: When acquiring both locks, ALWAYS this order:
// 1. AppState (read or write)
// 2. BusState (read or write)

// CORRECT
async fn tick_with_events(app: &SharedState, bus: &SharedBusState) {
    let mut app_guard = app.write().await;
    // ... mutate app state ...
    let decision = compute_decision(&app_guard);
    drop(app_guard); // drop BEFORE acquiring bus lock
    let bus_guard = bus.write().await;
    bus_guard.event_tx.send(decision);
}

// WRONG — inverted order, will deadlock
async fn bad_pattern(app: &SharedState, bus: &SharedBusState) {
    let bus_guard = bus.write().await;
    let app_guard = app.read().await; // DEADLOCK if another task holds app and waits for bus
}
```

Document every function that acquires both locks in `MODULE_MATRIX.md`.

## CP03: Minimize Lock Hold Time

```rust
// WRONG — holds write lock during IO
let mut app = app_state.write().await;
persist_snapshot(&app, &db).await?; // slow IO under write lock
drop(app);

// CORRECT — clone needed data, release lock, then IO
let snapshot = {
    let app = app_state.read().await;
    app.clone_snapshot_data()
}; // lock released
persist_snapshot(&snapshot, &db).await?;
```

Never do IO (disk, network) while holding a write lock.

## CP04: Read Lock for Computation

```rust
// Use read locks when possible — allows concurrent readers
let field = {
    let app = app_state.read().await; // read lock — other reads proceed
    FieldState::compute(&app.spheres, &app.network, app.tick)
}; // lock released
```

Rule: if you don't need to mutate, use `.read()`.

## CP05: Write Lock for Mutations

```rust
// Write lock blocks all other access — minimize scope
{
    let mut app = app_state.write().await;
    app.tick += 1;
    app.dirty = true;
} // write lock released immediately
```

## CP06: Clone Before Cross-Task Communication

```rust
// WRONG — sending reference across task boundary
let app = app_state.read().await;
tokio::spawn(async move {
    use_field(&app); // ERROR: app_guard is not Send
});

// CORRECT — clone the data you need
let field_data = {
    let app = app_state.read().await;
    app.snapshot_for_bridge()
};
tokio::spawn(async move {
    post_to_povm(&field_data).await;
});
```

## CP07: Atomic for Simple Counters

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

fn increment_tick() -> u64 {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed)
}
```

Use atomics for simple counters that don't need lock consistency with other state.
Used in bridge modules for last_poll tracking.

## CP08: OnceLock for Runtime Configuration

```rust
use std::sync::OnceLock;

static K_MOD_BOUNDS: OnceLock<(f64, f64)> = OnceLock::new();

pub fn init_k_mod_bounds(min: f64, max: f64) {
    K_MOD_BOUNDS.set((min, max)).ok();
}

pub fn k_mod_min() -> f64 {
    K_MOD_BOUNDS.get().map_or(K_MOD_MIN, |&(min, _)| min)
}
```

For values set once at startup and read many times.

## CP09: Channel Capacity Sizing

```rust
// Bounded channel: 256 events
let (tx, _) = broadcast::channel::<BusEvent>(256);

// Unbounded ONLY when consumer is guaranteed faster than producer
let (tx, rx) = mpsc::unbounded_channel::<ShutdownSignal>();
```

| Channel | Capacity | Rationale |
|---------|----------|-----------|
| Event broadcast | 256 | Slow subscribers lag, not block |
| Task submission | 100 | Backpressure via bounded |
| Shutdown signal | 1 (watch) | Latest value only |

## CP10: Spawn Safety — Send + 'static Bounds

```rust
// All data passed to tokio::spawn must be Send + 'static
// This means:
// - No references to stack data
// - No &self (must clone or Arc)
// - No MutexGuard / RwLockGuard across await points

// CORRECT
let state = app_state.clone(); // Arc clone is cheap
tokio::spawn(async move {
    let app = state.read().await;
    // ...
});
```

## CP11: Prevent Writer Starvation

```rust
// tokio::sync::RwLock is writer-preferring by default
// But if read-heavy workload starves writers, consider:
// 1. Batch reads (read once, compute multiple results)
// 2. Use parking_lot::RwLock with fair scheduling
// 3. Keep write critical sections very short

// Pattern: batch API reads
async fn batch_field_response(app: &SharedState) -> FieldBatch {
    let app = app.read().await;
    FieldBatch {
        r: app.network.order_parameter(),
        chimera: ChimeraState::detect(&app.network),
        field: FieldState::compute(&app.spheres, &app.network, app.tick),
        decision: compute_decision(&app),
    }
}
```

## CP12: Deadlock Detection in Tests

```rust
#[tokio::test]
async fn no_deadlock_concurrent_tick_and_api() {
    let app = shared_test_state();
    let bus = shared_test_bus();

    let tick_handle = tokio::spawn({
        let app = app.clone();
        let bus = bus.clone();
        async move {
            for _ in 0..100 {
                tick_once(&app, &bus).await.unwrap();
            }
        }
    });

    let api_handle = tokio::spawn({
        let app = app.clone();
        async move {
            for _ in 0..100 {
                let _ = app.read().await;
            }
        }
    });

    // Both should complete without deadlock
    let (r1, r2) = tokio::join!(tick_handle, api_handle);
    r1.unwrap();
    r2.unwrap();
}
```

Stress test concurrent access patterns to catch deadlocks.
