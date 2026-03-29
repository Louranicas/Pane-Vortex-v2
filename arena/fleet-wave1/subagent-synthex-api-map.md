# SYNTHEX V3 API Complete Map (Subagent Discovery)

## Writable Endpoint Found!
**`POST /api/ingest`** — accepts arbitrary thermal JSON, returns `{"accepted": true}`

## Endpoint Map
| Endpoint | Method | Status | Purpose |
|----------|--------|--------|---------|
| `/api/health` | GET | 200 | Liveness |
| `/v3/thermal` | GET | 200 | Thermal state (temp, target, PID, heat sources) |
| `/v3/diagnostics` | GET | 200 | Probes (synergy CRITICAL at 0.5) |
| **`/api/ingest`** | **POST** | **200** | **Writable thermal injection** |
| `/v3/patterns` | GET | 404 | Not implemented |
| `/v3/cascade` | GET | 404 | Not implemented |
| All others | * | 404 | Not implemented |

## Injection Vector
Accepts: `{"heat_source_id":"HS-001","reading":0.5}`, `{"cascade_amplification":100}`, etc.
Always returns HTTP 200. Readings may feed into delayed computation.
