# V1 API Route Map

> Generated Session 044 by Claude-PV1-Sidecar analyzing src/api.rs (18KB analysis)
> 81 routes: 64 GET + 17 POST (73 base + 8 evolution feature-gated)

## Field Endpoints (11 GET)
| Route | Description |
|-------|-------------|
| `GET /health` | tick, spheres, r, k, fleet_mode |
| `GET /field` | Full FieldState (chimera, harmonics, tunnels) |
| `GET /field/r` | Order parameter r + psi |
| `GET /field/spectrum` | Harmonic spectrum (l0, l1, l2) |
| `GET /field/chimera` | Chimera detection (clusters, gaps) |
| `GET /field/tunnels` | Buoy-overlap tunnels |
| `GET /field/decision` | Current action, pressures, routing |
| `GET /field/decisions` | Historical decision log |
| `GET /field/k` | Coupling K, k_modulation, auto_k |
| `GET /field/warmup` | Warmup remaining ticks |
| `GET /field/history` | SQLite field snapshots |

## Sphere Endpoints (9 GET + 12 POST)
| Route | Description |
|-------|-------------|
| `GET /spheres` | All sphere summaries |
| `GET /sphere/{id}` | Single sphere detail |
| `POST /sphere/{id}/register` | Register (persona, freq) + ghost inheritance |
| `POST /sphere/{id}/deregister` | Deregister → GhostTrace (NA-28) |
| `POST /sphere/{id}/memory` | Record tool usage → semantic phase inject |
| `POST /sphere/{id}/phase` | Manual phase/frequency update |
| `POST /sphere/{id}/status` | Update status (Working/Idle/Blocked/Complete) |
| `POST /sphere/{id}/steer` | Consent-gated phase steering |
| `GET /sphere/{id}/recall` | Memory recall (zone/limit/near_phase/near_buoy) |
| `GET /sphere/{id}/neighbors` | Coupling neighborhood (weights, phase diffs) |

## Key Discovery
V2 scaffold routes (`/synthex/thermal`, `/nexus/metrics`) return 404 on V1 daemon. The V1 API is at `/field/*`, `/sphere/*`, `/bus/*`, `/ghosts`, `/conductor/*`, `/network/*`.

Full route map: `/tmp/arena/v1-api-routes.txt` (18KB)

## Links
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[IPC Bus Architecture Deep Dive]]
- [[API_SPEC]]
