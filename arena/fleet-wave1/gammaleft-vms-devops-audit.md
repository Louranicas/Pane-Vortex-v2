# WAVE-3 GAMMA-LEFT: VMS & DevOps Deep Audit

> **Agent:** GAMMA-LEFT | **Date:** 2026-03-21 | **Wave:** 3
> **Services audited:** VMS (8120), DevOps Engine (8081), CodeSynthor V7 (8110), NAIS (8101), Bash Engine (8102)

---

## 1. Vortex Memory System (VMS) — Port 8120

### Health

```json
{
  "r": 0.0,
  "coherent": false,
  "zone": "Incoherent",
  "sphere_count": 1,
  "total_memories": 0,
  "open_count": 0,
  "closed_count": 0,
  "morphogenic_cycle": 0,
  "fractal_depth_avg": 0.0,
  "version": "1.0.0"
}
```

### API Surface

| Endpoint | Status | Notes |
|----------|--------|-------|
| `/health` | 200 | Full state in health response |
| `/spheres` | 404 | Not exposed |
| `/memories` | 404 | Not exposed |
| `/field` | 200 (empty) | Returns empty body |
| `/vortex` | 404 | Not exposed |
| `/ovm` | 404 | Not exposed |
| `/state` | 404 | Not exposed |
| `/zone` | 404 | Not exposed |
| `/morphogenic` | 404 | Not exposed |

### Assessment: DEGRADED

- **r=0.0, zone=Incoherent** — field has zero coherence
- **total_memories=0** — no memories stored despite being operational
- **sphere_count=1** — single sphere registered, no multi-sphere dynamics
- **morphogenic_cycle=0** — zero cycles completed
- **API surface is minimal** — only `/health` returns data. All other endpoints 404. VMS appears to be running but functionally dormant.
- **CORS headers present** (access-control-allow-origin: *) — built for cross-service access but nothing connects to it

### Dependencies

- **Upstream:** POVM Engine (8125) — VMS owns port 8120 (sphere-vortex disabled to avoid collision)
- **Downstream:** Pane-Vortex (8132) — VMS is in Batch 5, depends on POVM Engine
- **Gap:** No evidence of active data flow between VMS and any other service

---

## 2. DevOps Engine — Port 8081

### Health

```json
{
  "status": "healthy",
  "fleet": { "active": 40, "agents": 40, "quorum_met": true },
  "subsystems": {
    "cache": "operational", "consensus": "operational",
    "hebbian": "operational", "mycelial": "operational",
    "pipeline": "operational", "tensor_memory": "operational"
  },
  "uptime_seconds": 0,
  "version": "2.0.0"
}
```

### API Surface

| Endpoint | Status | Response |
|----------|--------|----------|
| `/health` | 200 | Fleet + subsystems overview |
| `/status` | 200 | Full status with modules, fleet, memory, pipelines |
| `/agents` | 200 | 40 agents across 8 tiers (JSON object) |
| `/metrics` | 200 | Prometheus format (14 metrics) |
| `POST /pipeline` | Listed | Pipeline submission (not tested) |
| `GET /pipeline/{id}` | Listed | Pipeline lookup |

### Agent Tier Distribution (40 agents total)

| Tier | Count | Weight | Synergy |
|------|-------|--------|---------|
| Omega | 8 | 2.0 | 0.95 |
| Conductor | 7 | — | — |
| Scribe | 6 | — | — |
| Librarian | 5 | — | — |
| Optimizer | 5 | — | — |
| Validator | 5 | — | — |
| Integrator | 3 | — | — |
| Sentinel | 1 | — | — |

### Module Inventory (13 modules, all ready)

M1_hebbian, M2_mycelial, M3_tensor, M4_ml_optimizer, M5_weaver, M6_agent_swarm, M7_deployment, M8_integration, M9_autonomous_planning, M10_task_queue, M11_confidence_gate, M12_e2e_pipeline, M13_advanced_cache

### Assessment: HEALTHY but IDLE

- **All 40 agents active**, quorum met (27/40 PBFT)
- **0 pipelines** (active, completed, or failed) — engine has never executed work
- **hebbian_synapses=0, cached_patterns=0** — no learning has occurred
- **uptime_seconds=0** — counter may be broken or service just restarted
- **Prometheus metrics operational** — 14 gauges/counters exposed
- **Synergy at 0.93** — healthy but static (no pipeline activity to drive it)

### Dependencies

- **Batch 1** (no dependencies) — DevOps Engine starts first
- **Downstream consumers:** SAN-K7 (8100), SYNTHEX (8090), Maintenance Engine (8080)
- **Cross-service:** Shares Hebbian/tensor patterns with NAIS conceptually, but no runtime wiring observed

---

## 3. CodeSynthor V7 — Port 8110

### Health

```json
{
  "status": "healthy",
  "service": "codesynthor_v7",
  "version": "7.3.0",
  "uptime_secs": 232559,
  "requests": 61856,
  "modules": 62,
  "layers": 17,
  "synergy": 0.985,
  "loop": "M1<->M62 BIDIRECTIONAL",
  "ultraplate": true,
  "bulletproof": true
}
```

### API Surface

| Endpoint | Status | Response |
|----------|--------|----------|
| `/health` | 200 | Full service status (JSON) |
| `/status` | 200 | Detailed module/layer stats (JSON) |
| `/modules` | 200 | Plain text — service banner, not JSON |
| `/synergy` | 200 | Plain text — not JSON |

### Assessment: HEALTHY and ACTIVE

- **62 modules across 17 layers** — fully operational bidirectional loop (M1<->M62)
- **61,856 requests processed** — by far the most active service audited
- **Synergy at 0.985** — highest of all services (near-perfect)
- **Uptime 232,559s (~2.7 days)** — stable, no restarts
- **"bulletproof": true** — self-reported resilience flag
- **API inconsistency:** `/modules` and `/synergy` return plain text banners, not JSON. Only `/health` and `/status` are machine-parseable.

### Dependencies

- **Batch 1** (no dependencies) — starts independently
- **Upstream to SYNTHEX:** POVM pathway `nexus-bus:cs-v7 -> synthex` has weight 1.046 (strongest in entire graph) — CodeSynthor is SYNTHEX's primary feeder
- **No direct REST calls observed** to other services

---

## 4. NAIS (Neural Adaptive Intelligence) — Port 8101

### Health

```json
{
  "status": "healthy",
  "service": "nais",
  "uptime_secs": 232556,
  "requests": 61817
}
```

### API Surface

| Endpoint | Status | Response |
|----------|--------|----------|
| `/health` | 200 | Minimal JSON (4 fields) |
| `/status` | 200 | Plain text banner ("NAIS v1.0.0 - ULTRAPLATE Neural Adaptive Intelligence") |
| `/agents` | 200 | Same plain text banner |

### Assessment: HEALTHY but OPAQUE

- **61,817 requests** — nearly identical to CodeSynthor (61,856), suggesting they're polled at the same rate
- **Uptime 232,556s** — stable, matching CodeSynthor
- **No meaningful API beyond `/health`** — `/status` and `/agents` return identical text banners, not structured data
- **Minimal observability** — cannot inspect neural state, adaptation metrics, or agent topology

### Dependencies

- **Batch 3** (depends on SYNTHEX, SAN-K7)
- **No outbound REST calls observed**
- **Request count parity with CodeSynthor** suggests shared polling (likely from SYNTHEX or devenv health checks)

---

## 5. Bash Engine — Port 8102

### Health

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_secs": 232606
}
```

### API Surface

| Endpoint | Status | Response |
|----------|--------|----------|
| `/health` | 200 | Minimal JSON (3 fields) |
| `/status` | 404 | Not found |
| `/patterns` | 404 | Not found |
| `/safety` | 404 | Not found |
| `POST /validate` | 404 | Not found |

### Assessment: HEALTHY but MINIMAL

- **Only `/health` exists** — all other expected endpoints (status, patterns, safety, validate) return 404
- **Uptime 232,606s** — stable
- **No request counter exposed** — cannot measure activity
- **45 safety patterns** (per CLAUDE.md) exist in code but are not exposed via API
- **Most limited API surface of all audited services**

### Dependencies

- **Batch 3** (depends on SYNTHEX, SAN-K7)
- **No cross-service wiring observed**

---

## 6. Inter-Service Dependency Map

```
                    BATCH 1 (no deps)
                    ┌──────────────┐
                    │ DevOps (8081)│──── 40 agents, PBFT, 13 modules
                    │ CS-V7 (8110) │──── 62 modules, synergy 0.985
                    │ POVM  (8125) │──── 2,427 pathways, 42 memories
                    └──────┬───────┘
                           │
                    BATCH 2 (needs B1)
                    ┌──────┴───────┐
                    │ SYNTHEX(8090)│◄── CS-V7 strongest pathway (w=1.046)
                    │ SAN-K7(8100) │
                    │ ME    (8080) │
                    └──────┬───────┘
                           │
                    BATCH 3 (needs B2)
                    ┌──────┴───────┐
                    │ NAIS  (8101) │──── 61,817 reqs (polled w/ CS-V7)
                    │ Bash  (8102) │──── health-only API
                    │ ToolMk(8103) │
                    └──────┬───────┘
                           │
                    BATCH 4 (needs B3)
                    ┌──────┴───────┐
                    │ CCM   (8104) │
                    │ ToolLb(8105) │
                    │ RM    (8130) │
                    └──────┬───────┘
                           │
                    BATCH 5 (needs B4)
                    ┌──────┴───────┐
                    │ VMS   (8120) │──── INCOHERENT, 0 memories
                    │ PV    (8132) │──── 31 spheres, r=0.0
                    └──────────────┘
```

### Observed Cross-Service Connections

| From | To | Evidence | Strength |
|------|----|----------|----------|
| CS-V7 (8110) | SYNTHEX (8090) | POVM pathway w=1.046 | Strongest in graph |
| PV (8132) | POVM (8125) | povm_bridge.rs, field snapshots | Active (every 12 ticks) |
| PV (8132) | RM (8130) | bridge.rs, fire-and-forget TCP | Active |
| PV (8132) | SYNTHEX (8090) | synthex_bridge.rs, thermal k_adj | Active |
| nexus-bus | PV (8132) | POVM pathway w=1.020 (devenv-patterns) | Learned |
| DevOps (8081) | None observed | 0 pipelines, no outbound calls | Idle |
| VMS (8120) | None observed | Incoherent, no memories | Dormant |
| NAIS (8101) | None observed | Opaque API, no cross-refs | Unknown |
| Bash (8102) | None observed | Health-only API | Isolated |

### Key Findings

1. **VMS is functionally dead** — r=0.0, zero memories, zone Incoherent, minimal API. It occupies port 8120 but provides no value. Sphere-vortex was disabled to give VMS this port, yet VMS does nothing with it.

2. **DevOps Engine is idle despite full fleet** — 40 agents across 8 tiers, PBFT consensus ready, 13 modules loaded, but zero pipelines ever executed. All that capacity sits unused.

3. **NAIS and Bash Engine are black boxes** — health endpoints work but no structured API for inspection. NAIS returns text banners for all non-health routes. Bash Engine 404s everything except `/health`.

4. **CodeSynthor is the workhorse** — highest request count (61,856), highest synergy (0.985), strongest POVM pathway to SYNTHEX (w=1.046). It's the most active and best-connected service in this audit group.

5. **Batch 3 services (NAIS, Bash) have no observed downstream consumers** — they depend on Batch 2 services but nothing in Batch 4/5 appears to depend on them specifically.

6. **Request count parity** — NAIS (61,817) and CS-V7 (61,856) have nearly identical request counts, differing by only 39. This suggests a shared polling mechanism, likely devenv health checks at identical intervals.

---

GAMMALEFT-WAVE3-COMPLETE
