# IPC Bus Wire Protocol

Socket: `/run/user/1000/pane-vortex-bus.sock` | NDJSON | 0700

## Wire Format

```
CLIENT → SERVER: {"type":"handshake","id":"sphere-id","version":"1.0"}
SERVER → CLIENT: {"type":"welcome","sphere_id":"sphere-id","tick":N,"r":0.0}

CLIENT → SERVER: {"type":"subscribe","patterns":["field.*","sphere.*"]}
SERVER → CLIENT: {"type":"subscribed","patterns":["field.*","sphere.*"]}

CLIENT → SERVER: {"type":"submit","description":"task","target":"any_idle"}
SERVER → CLIENT: {"type":"task_submitted","task_id":"uuid"}

SERVER → CLIENT: {"type":"event","event_type":"field.tick","data":{...}}
```

## V1 Compat Layer (m29_ipc_bus.rs)
V1 sends `{"type":"handshake","sphere_id":"..."}` — flat JSON
V2 uses serde-tagged `{"Handshake":{"pane_id":"...","version":"2.0"}}`
V2 detects V1 format via JSON `type` field fallback and responds with:
`{"type":"HandshakeOk","tick":N,"peer_count":N,"r":0.0,"protocol_version":1}`

## 18 Event Types
`field.tick` `field.decision` `sphere.registered` `sphere.connected`
`sphere.disconnected` `sphere.deregistered` `task.submitted` `task.claimed`
`task.completed` `task.failed` `cascade.dispatched` `cascade.ack`
`field.suggestion` `evolution.pattern` `bridge.synthex` `bridge.nexus`
`bridge.me` `conductor.action`

## Task Targets
| Target | Routing |
|--------|---------|
| specific | Named sphere ID |
| any_idle | First idle sphere |
| field_driven | Conductor picks via chimera routing |
| willing | Opt-in (respects opt_out_cross_activation) |

## Client Binary
```bash
PANE_VORTEX_ID="id" pane-vortex-client connect|subscribe|submit|cascade|disconnect
```

## Swarm Sidecar Bridge
WASM → /tmp/swarm-commands.pipe (FIFO) → swarm-sidecar → Unix socket → PV bus
PV bus → swarm-sidecar → /tmp/swarm-events.jsonl (ring file) → WASM
