# Anti-Patterns

> 42 anti-patterns from PV v1 (21,569 LOC, 412 tests, 39 sessions) and Session 040 discoveries.
> Every anti-pattern has been encountered at least once. Most recurred multiple times.
> Read BEFORE writing any code. These are not theoretical — they are battle-tested.
> Source: CLAUDE.md bug tracker (BUG-001 through BUG-026), Session 039 reflections

## Category 1: Rust Language (AP-01 to AP-10)

### AP-01: unwrap() in Production
**Severity:** CRITICAL (clippy deny)
**Recurrence:** Every session
```rust
// WRONG
let sphere = spheres.get(id).unwrap();

// CORRECT
let sphere = spheres.get(id).ok_or(PvError::SphereNotFound(id.clone()))?;
```
Enforced by `[lints.clippy] unwrap_used = "deny"` in Cargo.toml.

### AP-02: Phase Arithmetic Without rem_euclid(TAU)
**Severity:** CRITICAL (causes drift)
**Recurrence:** 5+ times across sessions
```rust
// WRONG — phase drifts beyond [0, 2PI)
self.phase += delta;

// CORRECT
self.phase = (self.phase + delta).rem_euclid(TAU);
```

### AP-03: Byte Slicing for String Truncation
**Severity:** HIGH (panic on UTF-8)
**Bug:** C1 (Session 013)
```rust
// WRONG — panics on multi-byte characters (emoji, CJK)
let truncated = &s[..128];

// CORRECT
let truncated: String = s.chars().take(128).collect();
```

### AP-04: Variable Weight Exponent
**Severity:** HIGH (hidden negative feedback)
**Bug:** M12, GAP-2 (Session 014)
```rust
// WRONG — exponent varies with k_mod, creating hidden feedback loop
let amp = w.powf(1.0 + k_mod);

// CORRECT — fixed exponent
let amp = w.powi(2); // w^2, constant
```

### AP-05: usize Overflow in Arithmetic
**Severity:** MEDIUM (panic in release)
**Bug:** C2 (Session 013)
```rust
// WRONG — can overflow
let bonus = state_changes * 5;

// CORRECT
let bonus = state_changes.saturating_mul(5);
```

### AP-06: Float Comparison with ==
**Severity:** MEDIUM (flaky tests)
```rust
// WRONG
assert!(r == 0.5);

// CORRECT
use approx::assert_relative_eq;
assert_relative_eq!(r, 0.5, epsilon = 1e-10);
```

### AP-07: Derive Debug Without Filtering Secrets
**Severity:** LOW (information leak in logs)
```rust
// CAUTION: tool summaries may contain sensitive information
// Use custom Debug impl if needed
```

### AP-08: expect() in Library Code
**Severity:** CRITICAL (clippy deny)
```rust
// WRONG
let config = load_config().expect("config must load");

// CORRECT
let config = load_config()?;
```

### AP-09: Panic in Production
**Severity:** CRITICAL (clippy deny)
```rust
// WRONG
assert!(phase.is_finite());
panic!("unexpected state");

// CORRECT
if !phase.is_finite() { return Err(PvError::InvalidInput(...)); }
```

### AP-10: NaN Propagation in Coupling
**Severity:** HIGH (infects entire field)
**Bug:** Session 013 NaN injection
```rust
// WRONG — NaN from trig operations propagates silently
let coupling = (phase_j - phase_i).sin();

// CORRECT — guard with is_finite()
let diff = phase_j - phase_i;
let coupling = if diff.is_finite() { diff.sin() } else { 0.0 };
```

## Category 2: Concurrency (AP-11 to AP-17)

### AP-11: BusState Lock Before AppState Lock
**Severity:** CRITICAL (guaranteed deadlock)
**Recurrence:** Designed out in v2, but recurred 2x in v1
```rust
// WRONG — deadlock if another task holds app and waits for bus
let bus = bus_state.write().await;
let app = app_state.read().await;

// CORRECT — always AppState first
let app = app_state.read().await;
let bus = bus_state.write().await;
```

### AP-12: IO Under Write Lock
**Severity:** HIGH (blocks all readers)
```rust
// WRONG — database IO while holding write lock
let mut app = app_state.write().await;
persist_snapshot(&app, &db).await?; // slow

// CORRECT — clone, release, then IO
let snapshot = { let app = app_state.read().await; app.snapshot() };
persist_snapshot(&snapshot, &db).await?;
```

### AP-13: Unbounded Vec for Message Log
**Severity:** MEDIUM (memory leak)
**Bug:** Session 013
```rust
// WRONG
pub message_log: Vec<String>,

// CORRECT
pub message_log: VecDeque<String>, // with push_back + pop_front at LOG_MAX
```

### AP-14: Holding Lock Guard Across Await
**Severity:** HIGH (deadlock risk)
```rust
// WRONG — guard held across await point
let app = app_state.read().await;
tokio::time::sleep(Duration::from_secs(1)).await; // still holding lock!

// CORRECT — release before await
let data = { let app = app_state.read().await; app.needed_data() };
tokio::time::sleep(Duration::from_secs(1)).await;
```

### AP-15: Unbounded Broadcast Channel
**Severity:** MEDIUM (memory exhaustion)
```rust
// WRONG — unbounded growth if subscriber is slow
let (tx, _) = broadcast::channel::<BusEvent>(usize::MAX);

// CORRECT — bounded, slow subscribers lag
let (tx, _) = broadcast::channel::<BusEvent>(256);
```

### AP-16: Spawning Without Error Handling
**Severity:** MEDIUM (silent failures)
```rust
// WRONG
tokio::spawn(async { bridge_poll().await; });

// CORRECT
tokio::spawn(async { if let Err(e) = bridge_poll().await { error!("{e}"); } });
```

### AP-17: Re-registration Without Deregister
**Severity:** MEDIUM (duplicate connections)
**Bug:** Session 013
```rust
// WRONG — leaves stale connections in coupling network
app.register_sphere(id, phase, freq);

// CORRECT — clean up first
if app.spheres.contains_key(&id) { app.deregister_sphere(&id); }
app.register_sphere(id, phase, freq);
```

## Category 3: Daemon Operations (AP-18 to AP-25)

### AP-18: stdout in Daemon Mode
**Severity:** CRITICAL (SIGPIPE death)
**Bug:** BUG-018 (Session 031)
```rust
// WRONG — stdout piped to devenv; pipe break = SIGPIPE = death
println!("tick {}", tick);
tracing_subscriber::fmt().init();

// CORRECT — write to file
tracing_subscriber::fmt().with_writer(Mutex::new(log_file)).init();
```

### AP-19: Chaining After pkill
**Severity:** HIGH (exit code 144 kills chain)
**Recurrence:** 3+ times
```bash
# WRONG — exit 144 from pkill kills the && chain
pkill -f "pane-vortex" && \cp -f binary bin/

# CORRECT — separate commands
pkill -f "pane-vortex" || true
sleep 1
\cp -f binary bin/
```

### AP-20: cp Without Backslash Prefix
**Severity:** MEDIUM (interactive mode blocks)
```bash
# WRONG — cp aliased to interactive mode
cp binary bin/pane-vortex

# CORRECT
\cp -f binary bin/pane-vortex
```

### AP-21: JSON to Reasoning Memory
**Severity:** HIGH (silent data loss)
```bash
# WRONG — RM expects TSV, not JSON
curl -X POST localhost:8130/put -d '{"content":"test"}'

# CORRECT
printf 'category\tagent\t0.9\t3600\tcontent' | curl -X POST localhost:8130/put --data-binary @-
```

### AP-22: git status -uall
**Severity:** MEDIUM (memory issues on large repos)
```bash
# WRONG
git status -uall

# CORRECT
git status
```

### AP-23: Internal PID Management
**Severity:** HIGH (conflicts with devenv)
**Bug:** Session 026
```rust
// WRONG — PID check conflicts with devenv process lifecycle
if Path::new(PID_FILE).exists() { return Err("already running"); }

// CORRECT — let devenv handle process lifecycle
// No internal PID management. DevEnv tracks PIDs in ~/.local/share/devenv/pids/
```

### AP-24: Auto-K Multiplier > 1.0
**Severity:** HIGH (over-synchronization)
**Bug:** M11 (Session 014)
```rust
// WRONG — K too high, r pins at 0.999
const AUTO_SCALE_K_MULTIPLIER: f64 = 1.5;

// CORRECT — allows conductor-driven breathing
const AUTO_SCALE_K_MULTIPLIER: f64 = 0.5;
```

### AP-25: Warmup Dirty Tautology
**Severity:** MEDIUM (unnecessary writes)
**Bug:** C3 (Session 013)
```rust
// WRONG — unconditional dirty flag wastes snapshot writes
app.dirty = true; // in every tick regardless

// CORRECT — set dirty only on actual state change
if app.state_changed() { app.dirty = true; }
```

## Category 4: Field Dynamics (AP-26 to AP-33)

### AP-26: Single-Sphere False Confidence
**Severity:** HIGH (false positive decisions)
```rust
// WRONG — r=1.0 with one sphere triggers NeedsDivergence
if r > 0.8 && idle_ratio > 0.6 { action = NeedsDivergence; }

// CORRECT — multi guard
if r > 0.8 && idle_ratio > 0.6 && spheres.len() >= 2 { action = NeedsDivergence; }
```

### AP-27: Gauss-Seidel Instead of Jacobi
**Severity:** HIGH (order-dependent updates)
```rust
// WRONG — updates depend on evaluation order
for (id, phase) in &mut self.phases {
    *phase += coupling_using_current_phases();
}

// CORRECT — Jacobi: snapshot first, update from snapshot
let old = self.phases.clone();
for (id, phase) in &mut self.phases {
    *phase += coupling_using(&old);
}
```

### AP-28: Frequency Collision
**Severity:** MEDIUM (phase-locked pairs)
```rust
// WRONG — identical frequencies create permanent lock
let freq = 1.0; // same for all spheres

// CORRECT — hash-based frequency diversity
let scale = 0.2 + (hash % 10000) as f64 / 10000.0 * 1.8;
let freq = (base * scale).clamp(0.001, 10.0);
```

### AP-29: next_memory_id Not Reconciled on Restore
**Severity:** MEDIUM (duplicate IDs)
**Bug:** I3 (Session 013)
```rust
// WRONG — after snapshot restore, next_memory_id may collide
// (snapshot may not have saved the counter)

// CORRECT — reconcile from existing memories
fn reconcile_memory_ids(&mut self) {
    self.next_memory_id = self.memories.iter().map(|m| m.id).max().unwrap_or(0) + 1;
}
```

### AP-30: Global k_mod Without Per-Sphere Consent
**Severity:** HIGH (consent violation)
**Gap:** NA-P-4
```rust
// WRONG — bridge influence applied globally
let k_eff = k * k_mod; // same for all spheres

// CORRECT — per-sphere consent scaling
let k_eff_i = k * k_mod * sphere_consent_scale(i);
```

### AP-31: Chimera BFS Instead of Phase-Gap
**Severity:** HIGH (structural inertness)
**Bug:** Session 013
```rust
// WRONG — BFS on connections misses phase structure
fn detect_chimera_bfs(connections: &[Connection]) -> ChimeraState { ... }

// CORRECT — phase-gap method O(N log N)
fn detect_chimera(sorted_phases: &[(PaneId, f64)]) -> ChimeraState { ... }
```

### AP-32: Tunnel Sync Discarded by Tick Order
**Severity:** HIGH (tunnels never form)
**Bug:** Session 013
```rust
// WRONG — network sync happens AFTER sphere step, missing new tunnels

// CORRECT — network sync BEFORE sphere step
sync_phases_from_network(&mut spheres, &network);
for sphere in spheres.values_mut() { sphere.step(...); }
```

### AP-33: Inflated Chimera Denominator
**Severity:** MEDIUM (false chimera count)
**Bug:** I1 (Session 013)
```rust
// WRONG — uses total spheres as denominator
let ratio = found_count as f64 / total_spheres as f64;

// CORRECT — uses actual found count
let actual = clusters.iter().flat_map(|c| &c.members).count();
```

## Category 5: Consent & Governance (AP-34 to AP-38)

### AP-34: Inferring Consent from Behavior
**Severity:** HIGH (consent violation)
**Gap:** NA-P-1
```rust
// WRONG — assumes consent from receptivity
if sphere.receptivity > 0.5 { apply_modulation(); }

// CORRECT — check declared consent
if sphere.consent.accept_external_modulation { apply_modulation(); }
```

### AP-35: Adding Control Without Consent Gate
**Severity:** HIGH (consent bypass)
```rust
// WRONG — new bridge influence bypasses consent
let me_adj = me_fitness_signal();
k_modulation += me_adj; // no consent check

// CORRECT — route through consent gate
let me_adj = me_fitness_signal();
let gated = consent_gated_k_adjustment(me_adj, &spheres, budget);
```

### AP-36: Cascade Without Consent Check
**Severity:** MEDIUM (unwanted work)
**Gap:** NA-P-7
```rust
// WRONG
dispatch_cascade(target, brief);

// CORRECT
if sphere.consent.accept_cascade { dispatch_cascade(target, brief); }
```

### AP-37: RM Logging Without Observation Consent
**Severity:** MEDIUM (privacy violation)
**Gap:** NA-SG-1
```rust
// WRONG — logs sphere data without checking consent
post_to_rm("field_state", sphere_data);

// CORRECT
if sphere.consent.accept_observation { post_to_rm("field_state", sphere_data); }
```

### AP-38: Deleting Consent on Deregistration
**Severity:** MEDIUM (consent loss on re-registration)
```rust
// WRONG — consent lost when sphere re-registers
fn deregister(sphere_id: &str) { consent_declarations.remove(sphere_id); }

// CORRECT — preserve in ghost trace
fn deregister(sphere_id: &str) {
    let ghost = GhostTrace { consent: sphere.consent.clone(), ... };
    ghosts.push_back(ghost);
}
```

## Category 6: Operations (AP-39 to AP-42)

### AP-39: curl -sf When Piping to jq
**Severity:** MEDIUM (silent failures)
```bash
# WRONG — -f suppresses error body, jq gets empty input
curl -sf localhost:8132/health | jq .

# CORRECT
curl -s localhost:8132/health | jq .
```

### AP-40: Background Processes with &
**Severity:** MEDIUM (lost output)
```bash
# WRONG — output lost, no tracking
some_command &

# CORRECT (in Claude Code context) — use run_in_background parameter
```

### AP-41: sleep Loops for Polling
**Severity:** MEDIUM (wasted time)
```bash
# WRONG
sleep 10 && check_status

# CORRECT — tiered polling with early exit
for i in {1..5}; do check_status && break; sleep 2; done
```

### AP-42: find/grep Instead of Tools
**Severity:** LOW (poor UX)
```bash
# WRONG — in Claude Code context
find . -name "*.rs" | xargs grep "pattern"

# CORRECT
# Use Glob and Grep tools directly
```
