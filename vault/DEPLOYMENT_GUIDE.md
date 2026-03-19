---
title: "Pane-Vortex V2 — Deployment Guide"
date: 2026-03-19
tags: [deployment, devenv, production, pane-vortex-v2]
plan_ref: "MASTERPLAN.md"
obsidian:
  - "[[The Habitat — Integrated Master Plan V3]]"
  - "[[ULTRAPLATE Developer Environment]]"
  - "[[ULTRAPLATE Quick Start — Session 031]]"
---

# Pane-Vortex V2 — Deployment Guide

> DevEnv integration, binary deployment, PID tracking, health checks, and troubleshooting.
> PV is ULTRAPLATE Service ID `pane-vortex`, Port 8132, Batch 5.

---

## 1. ULTRAPLATE DevEnv Integration (PREFERRED)

PV runs as a managed service in the ULTRAPLATE developer environment. DevEnv handles:
- Process lifecycle (start/stop/restart)
- PID file tracking (`~/.local/share/devenv/pids/pane-vortex.pid`)
- Auto-restart on crash
- Log rotation
- Dependency ordering (Batch 5: after povm-engine + synthex)

### Configuration

PV is registered in `~/.config/devenv/devenv.toml`:

```toml
[services.pane-vortex]
name = "pane-vortex"
command = "/home/louranicas/.local/bin/pane-vortex"
port = 8132
health_path = "/health"
batch = 5
depends_on = ["povm-engine", "synthex"]
env = { PORT = "8132", RUST_LOG = "pane_vortex=info" }
```

### Commands

```bash
# Start (respects batch ordering)
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start

# Restart PV only
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart pane-vortex

# Stop PV only
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml stop pane-vortex

# Status of all services
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml status

# Health check of all services
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml health
```

### Dependency Batches

PV is Batch 5 — it starts last because it depends on services in earlier batches:

```
Batch 1 (no deps):  devops-engine, codesynthor-v7, povm-engine
Batch 2 (needs B1): synthex, san-k7, maintenance-engine, architect-agent, prometheus-swarm
Batch 3 (needs B2): nais, bash-engine, tool-maker
Batch 4 (needs B3): claude-context-manager, tool-library, reasoning-memory
Batch 5 (needs B4): vortex-memory-system, pane-vortex
```

---

## 2. Build and Deploy Binary

### Full Release Build

```bash
cd ~/claude-code-workspace/pane-vortex-v2

# Build release binary
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo build --release 2>&1 | tail -20

# Run quality gate BEFORE deploying
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo check 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo clippy -- -D warnings -W clippy::pedantic 2>&1 | tail -20 && \
CARGO_TARGET_DIR=/tmp/cargo-pv2 cargo test --lib --release 2>&1 | tail -30
```

### Deploy (CRITICAL — Follow This Order Exactly)

```bash
# 1. Kill FIRST (separate command — pkill exit 144 kills chained commands)
pkill -f "pane-vortex" || true

# 2. Wait for process to die
sleep 1

# 3. Copy binary (bypass cp alias with backslash)
\cp -f /tmp/cargo-pv2/release/pane-vortex bin/pane-vortex
\cp -f /tmp/cargo-pv2/release/pane-vortex ~/.local/bin/pane-vortex

# 4. Copy client binary
\cp -f /tmp/cargo-pv2/release/pane-vortex-client ~/.local/bin/pane-vortex-client

# 5. Start via devenv (PREFERRED)
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml restart pane-vortex
```

**DEPLOYMENT TRAP (recurred 3x in V1):** Never chain after pkill. `pkill -f x && cp` fails because exit code 144 from pkill kills the `&&` chain. Always use separate commands.

---

## 3. Manual Start (Legacy)

If DevEnv is unavailable:

```bash
# Start daemon (loopback only by default)
PORT=8132 RUST_LOG=pane_vortex=info nohup ~/.local/bin/pane-vortex > /tmp/pane-vortex.log 2>&1 &

# Allow external access
PORT=8132 BIND_ADDR=0.0.0.0 nohup ~/.local/bin/pane-vortex > /tmp/pane-vortex.log 2>&1 &

# Start IPC bus socket
# (The daemon starts the socket automatically at /run/user/1000/pane-vortex-bus.sock)
```

**WARNING:** Manual start does not provide PID tracking, auto-restart, or dependency ordering. Use DevEnv when possible.

---

## 4. Health Checks

### Single Service

```bash
curl -s http://localhost:8132/health | jq .
```

Expected response:
```json
{
  "status": "ok",
  "service": "pane-vortex",
  "version": "2.0.0",
  "uptime_secs": 42,
  "sphere_count": 0,
  "tick": 8
}
```

### All ULTRAPLATE Services (16 Active)

```bash
declare -A hpath=([8080]="/api/health" [8081]="/health" [8090]="/api/health" [8100]="/health" \
  [8101]="/health" [8102]="/health" [8103]="/health" [8104]="/health" [8105]="/health" \
  [8110]="/health" [8120]="/health" [8125]="/health" [8130]="/health" [8132]="/health" \
  [9001]="/health" [10001]="/health")
for port in "${!hpath[@]}"; do
  echo "Port $port: $(curl -s -o /dev/null -w '%{http_code}' "http://localhost:$port${hpath[$port]}" 2>/dev/null)"
done
```

### Key Diagnostic Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | Basic health + uptime + sphere count |
| `GET /spheres` | List all registered spheres |
| `GET /field/state` | Current r, k, decision, chimera |
| `GET /field/decision` | Current conductor decision with attribution |
| `GET /integration/matrix` | Cross-service integration status |
| `GET /nexus/metrics` | Nexus bridge strategy + confidence |

---

## 5. PID Tracking

### DevEnv PID Files

```bash
# Check PV PID file
cat ~/.local/share/devenv/pids/pane-vortex.pid

# Verify process is running
kill -0 $(cat ~/.local/share/devenv/pids/pane-vortex.pid) && echo "running" || echo "dead"
```

### Manual PID Check

```bash
# Find PV process
pgrep -f "pane-vortex" -a

# Check port 8132
ss -tlnp sport = :8132
```

---

## 6. Known Issue: `devenv stop` Does Not Kill Processes

The `devenv stop` command removes PID files but may leave processes alive on ports. If `devenv start` shows fewer than 16 services, kill rogue port occupants first:

```bash
for port in 8080 8081 8090 8100 8101 8102 8103 8104 8105 8110 8120 8125 8130 8132 9001 10001; do
  pid=$(ss -tlnp "sport = :$port" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | head -1)
  [[ -n "$pid" ]] && kill "$pid" 2>/dev/null
done
sleep 2
~/.local/bin/devenv -c ~/.config/devenv/devenv.toml start
```

---

## 7. Logging

### Structured Logging

PV uses `tracing` + `tracing-subscriber` with env-filter:

```bash
# Default: info level
RUST_LOG=pane_vortex=info

# Debug level for specific modules
RUST_LOG=pane_vortex::m7_coordination::m35_tick=debug,pane_vortex=info

# Trace level (very verbose)
RUST_LOG=pane_vortex=trace
```

### CRITICAL: Never Write to stdout in Daemon Mode

DevEnv pipes stdout. If the pipe breaks (e.g., DevEnv restarts), writing to stdout causes SIGPIPE, which kills the daemon (BUG-018 from V1). Always use file-based tracing:

```rust
// CORRECT: file-based subscriber
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)  // stderr, not stdout
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### Log Locations

| Source | Location |
|--------|----------|
| DevEnv managed | DevEnv log directory (check devenv config) |
| Manual start | `/tmp/pane-vortex.log` (from nohup redirect) |
| stderr | `/tmp/pane-vortex.log` (if redirected) |

---

## 8. Database Management

### Database Locations

| Database | Path | Tables |
|----------|------|--------|
| Field tracking | `data/field_tracking.db` | field_snapshots, sphere_history, coupling_history |
| Bus tracking | `data/bus_tracking.db` | bus_tasks, bus_events, event_subscriptions, cascade_events, task_tags, task_dependencies, proposals, votes, consent_declarations, data_manifests |

### Migrations

Migrations auto-apply on startup from `migrations/` directory:
- `001_field_tables.sql` — Field tracking tables
- `002_bus_tables.sql` — Bus and task tables
- `003_governance_tables.sql` — Governance tables (V3.4)

### Database Health Check

```bash
# Check integrity
sqlite3 data/field_tracking.db "PRAGMA integrity_check;"
sqlite3 data/bus_tracking.db "PRAGMA integrity_check;"

# Check WAL mode
sqlite3 data/field_tracking.db "PRAGMA journal_mode;"
# Expected: wal

# Check table schemas
sqlite3 -header -column data/field_tracking.db ".schema"
sqlite3 -header -column data/bus_tracking.db ".schema"
```

### Database Backup

```bash
# Online backup (safe with WAL mode)
sqlite3 data/field_tracking.db ".backup /tmp/field_tracking_backup.db"
sqlite3 data/bus_tracking.db ".backup /tmp/bus_tracking_backup.db"
```

---

## 9. Security

| Setting | Default | Production |
|---------|---------|-----------|
| Bind address | 127.0.0.1 (loopback) | Same (override with BIND_ADDR=0.0.0.0) |
| Body limit | 65KB | Same |
| Socket permissions | 0700 | Same |
| Sphere cap | 200 | Same |
| Tool name limit | 128 chars | Same |
| Cascade rate limit | 10/min/source | Same |

---

## 10. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Port 8132 occupied | Stale process from previous run | `ss -tlnp sport = :8132` then `kill PID` |
| DevEnv shows 0 services | Stale PID files | Run port-killing loop above |
| Daemon dies immediately | SIGPIPE from stdout | Use stderr or file-based tracing |
| Database locked | WAL not enabled | Delete DB and restart (migrations recreate) |
| Bridge connection refused | Dependency service not running | Check batch dependencies, start missing service |
| IPC socket permission denied | Wrong socket permissions | Check `/run/user/1000/` ownership |
| Binary not found | Not deployed to ~/.local/bin | Run deploy steps above |
| Old binary running | Forgot to kill before copy | Always kill first, then copy, then start |

---

## Cross-References

- **[QUICKSTART.md](QUICKSTART.md)** — Quick build and verify
- **[CLAUDE.md](../CLAUDE.md)** — Build commands and anti-patterns
- **[config/default.toml](../config/default.toml)** — All configuration options
- **[MASTERPLAN.md](../MASTERPLAN.md)** — V3 plan deployment targets
- **Obsidian:** `[[ULTRAPLATE Developer Environment]]`, `[[ULTRAPLATE Quick Start — Session 031]]`
