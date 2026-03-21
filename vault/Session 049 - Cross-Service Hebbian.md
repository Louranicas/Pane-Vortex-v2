# Session 049 — Cross-Service Hebbian Co-Activation

**Date:** 2026-03-21

## Bus Event Co-Activation

Only 1 event type in bus ring buffer:
| Type | Count |
|------|-------|
| field.tick | 50 |

The bus event buffer (capped at 50) only contains tick events. No sphere.registered, task.submitted, or cascade.dispatched events visible — they've been rotated out by the continuous tick stream.

## Cross-Service Pathways Created

Posted 3 service-pair co-activation pathways to POVM:

| Pre (source) | Post (target) | Weight |
|-------------|--------------|--------|
| pane-vortex | maintenance-engine | 0.85 |
| pane-vortex | synthex | 0.85 |
| pane-vortex | povm-engine | 0.85 |

All accepted. These represent the 3 primary bridge relationships.

## K7 Co-Activation Pattern Search

K7 found 10 pattern matches for "co-activation" across L1-L4 layers in 11D tensor space. M2 module handles pattern matching.

## Which Services Fire Together?

Based on bridge architecture and observed data:

| Pair | Evidence | Coupling Strength |
|------|----------|------------------|
| PV ↔ SYNTHEX | Strongest bridge (0.82 synergy, Session 047) | High (but thermal loop disconnected) |
| PV ↔ POVM | Fast poll tier (6s), 82 memories | Medium (write-only, BUG-034) |
| PV ↔ ME | Standard poll (30s), fitness 0.619 | Medium (observer data flows) |
| PV ↔ RM | Every heartbeat, 52 pv2 entries | High (most active write channel) |
| PV ↔ K7 | Nexus combined_effect 1.02 | Medium (pattern queries) |
| SYNTHEX ↔ K7 | Strongest POVM pathway (1.046) | High (tensor shared space) |

---
*Cross-refs:* [[Session 049 - Observability Cluster]], [[Session 049 — Master Index]]
