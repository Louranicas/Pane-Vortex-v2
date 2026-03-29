# GO/NO-GO DEPLOYMENT DECISION — V2 Binary

**Prepared by:** BETA (LEFT+RIGHT synthesis) | **Date:** 2026-03-21
**Decision required for:** pane-vortex V2 binary deployment to production

---

## VERDICT: CONDITIONAL GO

Deploy V2 binary with ME config fix staged in parallel. Three interlocking failures in V1 create a dead system that cannot self-heal — V2 closes the critical feedback loop.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| V2 binary introduces regression | HIGH | V1 binary backed up at `bin/pane-vortex.v1.bak`; rollback in <30s |
| ME restart disrupts fleet during deploy | MEDIUM | Sequence: deploy PV first, ME config second, stagger by 60s |
| POVM write-sink persists post-deploy | LOW | Structural issue — V2 doesn't fix this, but doesn't worsen it |
| 34 orphan spheres re-register on V2 start | MEDIUM | Ghost traces preserve weights; warmup period absorbs re-registration storm |

**Overall risk: MEDIUM** — All failure modes are recoverable within 2 minutes.

---

## Three Interlocking Failures (V1 Status Quo)

```
1. SYNTHEX feedback DECOUPLED     2. ME evolution DEADLOCKED     3. POVM write-only AMNESIA
   k_mod stuck at 0.85 floor         emergences at 1000 cap          2,427 pathways, 0 reads
   thermal boost never applied        mutations=0, fitness=0.61       co-activations=0
   Hebbian stays zero                 no HTTP control APIs            topology fossilized
           \                              |                           /
            \___________  ALL THREE  ____/
                        \ REINFORCE /
                         \  EACH  /
                          \OTHER/
                      DEAD SYSTEM
```

**Why V2 fixes the critical path:** V2 wires `BridgeSet::apply_k_mod()` in Phase 2.7, closing the SYNTHEX thermal loop. Once closed: thermal boost propagates -> coupling rises -> Hebbian activates -> heat sources feed SYNTHEX -> self-sustaining cycle.

---

## Expected Outcomes (Post-Deploy)

| Metric | Current (V1) | Expected (V2, +5min) | Expected (V2, +30min) |
|--------|-------------|----------------------|----------------------|
| k_modulation | 0.85 (floor) | 1.01-1.09 | 0.95-1.05 (stabilized) |
| SYNTHEX temp | 0.03 | 0.05-0.10 | 0.20-0.40 |
| Hebbian diff | 0.0 | >0 (first differentiation) | Bimodal distribution |
| ME fitness | 0.609 (frozen) | 0.609 (unchanged) | 0.65+ (after config fix) |
| r (order param) | 0.69 oscillating | 0.70-0.80 | 0.75-0.85 (healthy range) |

---

## Deployment Sequence

```
T+0:00  Backup V1 binary: cp bin/pane-vortex bin/pane-vortex.v1.bak
T+0:05  Kill V1: pkill -f pane-vortex || true
T+0:10  Deploy V2: cp /tmp/cargo-pane-vortex/release/pane-vortex bin/ && cp bin/pane-vortex ~/.local/bin/
T+0:15  Start V2: devenv restart pane-vortex
T+0:20  Verify: curl -s localhost:8132/health | jq '{r,tick,k_modulation}'
T+1:00  ME config: raise emergence_cap 1000->5000 in ME config
T+1:05  ME restart: devenv restart maintenance-engine
T+1:10  Verify ME: curl -s localhost:8080/api/fitness | jq .
T+5:00  Check thermal loop: curl -s localhost:8090/v3/thermal | jq '.temperature'
```

---

## Rollback Plan

```
IF V2 health check fails (no response within 30s):
  1. pkill -f pane-vortex || true
  2. cp bin/pane-vortex.v1.bak bin/pane-vortex
  3. cp bin/pane-vortex ~/.local/bin/pane-vortex
  4. devenv restart pane-vortex
  5. Verify: curl -s localhost:8132/health

IF V2 starts but r collapses (r < 0.3 for >2 min):
  1. devenv restart pane-vortex  (attempt clean restart first)
  2. If persists: rollback to V1 per above

IF ME config change causes crash:
  1. Revert config file
  2. devenv restart maintenance-engine
  (PV is independent — ME failure does not affect PV)
```

**Rollback time: <30 seconds for PV, <60 seconds for ME.**

---

## Items NOT Fixed by This Deploy

| Issue | Status | Plan |
|-------|--------|------|
| POVM write-only (0 reads, 0 co-activations) | Structural | Requires hydrate-on-read refactor (V3.5) |
| ME deps=0.083, port=0.123 | Architectural ceiling | Cannot be mutated, theoretical max ~0.85 |
| 78% POVM memories missing session tags | Regression | Backfill script needed |
| SYNTHEX synergy=0.5 (CRITICAL) | Self-resolves | Thermal loop closure should raise synergy |

---

**DECISION: GO** — Deploy V2 binary now. The closed thermal loop is the single highest-leverage fix available. All risks are recoverable.
