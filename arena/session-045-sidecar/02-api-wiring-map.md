# PV V2 API Surface + Intelligence Router Wiring — Session 045

## API Routes (18 GET + POST confirmed live)

### Core (3)
| Route | Keys | Description |
|-------|------|-------------|
| GET /health | fleet_mode, k, k_modulation, r, spheres, status, tick, warmup_remaining | Main health check |
| GET /spheres | spheres[] | All registered spheres |
| GET /ghosts | ghosts[] | Ghost traces |

### Field (8)
| Route | Keys | Description |
|-------|------|-------------|
| GET /field | chimera, harmonics, order_parameter, sphere_count, tick, total_memories, tunnels | Full field snapshot |
| GET /field/r | psi, r | Order parameter |
| GET /field/decision | action, blocked/idle/working_spheres, r, r_trend, routing, tunnel_count | Decision engine |
| GET /field/decisions | decisions[] | Decision audit trail |
| GET /field/chimera | desync_clusters, is_chimera, sync_clusters | Chimera detection |
| GET /field/tunnels | count, tunnels[] | Active buoy tunnels |
| GET /field/k | auto_k, k, k_modulation | Coupling K state |
| GET /field/spectrum | l0_monopole, l1_dipole, l2_quadrupole | Harmonic decomposition |

### Bus (6)
| Route | Keys | Description |
|-------|------|-------------|
| GET /bus/info | cascade_count, events, subscribers, tasks | Bus summary |
| GET /bus/tasks | tasks[] | Pending tasks |
| GET /bus/events | events[] | Recent events (last 50) |
| GET /bus/cascades | cascades[] | Pending cascade handoffs |
| GET /bus/suggestions | suggestions[], total_generated | Field-driven suggestions |
| POST /bus/submit | task_id, status | Submit task via HTTP |
| POST /bus/cascade | cascade_index, source, target, status | Cascade handoff |

### Bridges (1)
| Route | Keys | Description |
|-------|------|-------------|
| GET /bridges/health | synthex_stale, nexus_stale, me_stale, povm_stale, rm_stale, vms_stale | Bridge freshness |

### Governance (V2 code, NOT live on V1 daemon — 404s)
| Route | Method | Status |
|-------|--------|--------|
| /field/propose | POST | 404 (V2 only) |
| /field/proposals | GET | 404 (V2 only) |
| /sphere/{id}/vote/{proposal_id} | POST | 404 (V2 only) |
| /sphere/{id}/consent | GET | 404 (V2 only) |
| /sphere/{id}/data-manifest | GET | 404 (V2 only) |

## Intelligence Router Architecture

```
                    ┌─────────────────────────┐
                    │  IntelligenceRouter      │
                    │  (swarm-orchestrator)    │
                    └──────┬──────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         ┌────────┐  ┌────────┐  ┌────────────┐
         │  POVM  │  │   RM   │  │   Nexus    │
         │:8125   │  │:8130   │  │:8100       │
         └────────┘  └────────┘  └────────────┘
              │            │            │
         pathways     context     r_outer
         weight>0.5   TSV parse   strategy
              │            │      alignment
              ▼            ▼            ▼
         ┌─────────────────────────────────┐
         │  compute_route_score()          │
         │  = 0.3×hebbian                  │
         │  + 0.3×receptivity              │
         │  + 0.2×strategy_confidence      │
         │  + 0.2×historical_success       │
         └─────────────────────────────────┘
              │
              ▼
         dispatch to highest-scoring sphere
```

## Cross-Service Data Flow

```
SYNTHEX (:8090) ──thermal_adj──▷ ConsentGate ──combined──▷ k_modulation
Nexus   (:8100) ──strategy_adj──▷ ConsentGate              │
ME      (:8080) ──health_adj────▷ ConsentGate              ▼
POVM    (:8125) ──(GAP-3: now)──▷ ConsentGate        CouplingNetwork
RM      (:8130) ──(GAP-3: now)──▷ ConsentGate              │
VMS     (:8120) ──(GAP-3: now)──▷ ConsentGate              ▼
                                                     Kuramoto step()
                                                     → phase updates
                                                     → field state
                                                     → FieldDecision
                                                           │
                                                           ▼
                                                     Conductor (PI)
                                                     → k_mod adjustment
                                                     → multiplicative
                                                       composition
```

## Event Types (from ring file analysis)

| Type | Count | Description |
|------|-------|-------------|
| field.tick | 257 | Periodic field state snapshot |
| field.suggestion | 140 | Field-driven work suggestions |
| field.decision | 4 | Decision state changes |
| field.cascade | 4 | Cascade handoff events |
