# IPC Bus Architecture Deep Dive

> Generated Session 044 by Claude-PV1-Sidecar analyzing V1 source code (src/bus.rs, src/ipc.rs, src/nexus_bus/)

## Two Bus Systems

### A. IPC Bus (src/bus.rs + src/ipc.rs)
Unix domain socket for Claude Code instances (sphere-to-sphere)
- Socket: `$XDG_RUNTIME_DIR/pane-vortex-bus.sock`
- Protocol: NDJSON over Unix socket
- Security: SO_PEERCRED UID check
- Capacity: max 200 simultaneous connections
- Channel: mpsc capacity 256, backpressure drops

### B. NexusBus (src/nexus_bus/)
TCP HTTP bridges for ULTRAPLATE services (service-to-PV)
- Raw TCP HTTP (no hyper/reqwest) with 3s timeout
- 5 bridges: CS-V7 (:8110/12t), Tool Library (:8105/24t), DevEnv (local/60t), VMS (:8120/12t), ME (:8080/12t)
- Health tracking: 3+ failures = unhealthy, recovery probe every 5th interval

## Message Flow

```
[Client/Sidecar] → Unix socket → SO_PEERCRED → Handshake(5s) → Message Loop
                                                                    ↓
ClientFrame:                                              ServerFrame:
  Ping/Subscribe/Unsubscribe                                Pong/Event
  TaskSubmit/TaskClaim/TaskComplete                         TaskAssigned/TaskBroadcast
  Request/Response                                          Response/Error
  CascadeHandoff/CascadeAck                                CascadeDelivered
```

## Lock Ordering (CRITICAL)

```
AppState → BusState → NexusBusState
```

**ALWAYS acquire AppState FIRST, then BusState. Never reverse.**

Special cascade pattern: `AppState.read → drop → BusState.read → drop → BusState.write → drop → AppState.write`

## V1 Compat Mechanisms

1. **Protocol version in handshake** — `protocol_version: 1` in HandshakeOk
2. **Response fallback routing** — unknown responses broadcast to all peers
3. **NDJSON forward-compat** — unknown fields/frames ignored by serde
4. **Cascade fallback briefs** — markdown files to `shared-context/tasks/` for non-bus consumers
5. **TaskTarget::Willing** — original broadcast model preserved alongside FieldDriven/Specific

## NexusBus Influence Pipeline

```
poll_due() → poll_single(BridgeId) → bridge::read()
  → transport::get_json() [raw TCP HTTP]
  → BridgeReading { k_adjustment ∈ [0.97, 1.03] }
  → apply_readings() → consent_gated_k_adjustment()
  → k_mod *= gated_adj → feeds Kuramoto coupling
```

## Key Source Files (V1)
| File | Purpose |
|------|---------|
| src/bus.rs | BusState, frame types, task queue, suggestions |
| src/ipc.rs | Socket listener, connection handler, frame dispatch |
| src/nexus_bus/mod.rs | poll_due(), apply_readings(), write_outbound() |
| src/nexus_bus/bridge.rs | BridgeId, BridgeReading, BridgeHealth |
| src/nexus_bus/transport.rs | Raw TCP HTTP get_json()/post_json() |

## Links
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[IPC_BUS_SPEC]]
- [[IPC_PATTERNS]]
- [[Consent Flow Analysis]]
