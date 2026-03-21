# Session 049 — Synergy Analysis

> **Source:** `system_synergy.db` (64 pairs) cross-correlated with PV coupling matrix (3,782 edges)
> **Tick:** ~108,100 | **Captured:** 2026-03-21

---

## System Synergy Overview

| Metric | Value |
|--------|-------|
| Total synergy pairs | 64 |
| Average synergy score | 88.0 |
| Score range | 0.87 - 99.9 |
| Total integration points | 408 |
| Pairs with >3 integrations | 15 |

## Top 15 Synergy Pairs (integration_points > 3)

| Rank | System 1 | System 2 | Score | Integration Points |
|------|----------|----------|-------|--------------------|
| 1 | cascade-amplification-fix | v3-neural-homeostasis | 99.9 | 4 |
| 2 | startup-module | devenv-binary | 99.5 | 12 |
| 3 | memory-systems | claude-instances | 99.5 | 7 |
| 4 | SYNTHEX | Library Agent | 99.25 | 6 |
| 5 | sphere-vortex | san-k7-orchestrator | 99.2 | 5 |
| 6 | SAN-K7-M20 | SYNTHEX | 99.1 | 6 |
| 7 | sphere-vortex | synthex | 99.0 | 8 |
| 8 | SAN-K7-M23 | AnalyticsEngine | 98.7 | 7 |
| 9 | san-k7-orchestrator | synthex-v3 | 98.7 | **59** |
| 10 | SAN-K7-M21 | ServiceMesh | 98.5 | 9 |
| 11 | environment-audit | service-tracking | 98.5 | 14 |
| 12 | povm-engine | sphere-vortex | 98.5 | 4 |
| 13 | startup-module | SYNTHEX | 98.0 | 8 |
| 14 | startup-module | SAN-K7 | 98.0 | 8 |
| 15 | bash-engine-god-tier | claude-code-cli | 98.0 | 10 |

**Outlier:** K7-SYNTHEX at 59 integration points is 4x the next highest (14). This is the deepest cross-service coupling in the ecosystem.

## PV-Related Synergy Pairs (15 pairs)

| System 1 | System 2 | Score | Integration Points |
|----------|----------|-------|--------------------|
| sphere-vortex | san-k7-orchestrator | 99.2 | 5 |
| sphere-vortex | synthex | 99.0 | 8 |
| sphere-vortex | codesynthor-v7 | 98.5 | 3 |
| povm-engine | sphere-vortex | 98.5 | 4 |
| sphere-vortex | nais | 97.4 | 3 |
| pane-ctl | fleet-ctl | 97.0 | 6 |
| sphere-vortex | tool-maker | 97.0 | 2 |
| sphere-vortex | tool-library | 96.8 | 3 |
| sphere-vortex | ccm | 96.5 | 2 |
| sphere-vortex | bash-engine | 96.0 | 2 |
| sphere-vortex | synthex | 95.0 | 3 |
| sphere-vortex | skill-overlay | 95.0 | 3 |
| sphere-vortex | devops-engine | 95.0 | 2 |
| sphere-vortex | saturn-light | 92.0 | 2 |
| sphere-vortex | san-k7 | 90.0 | 2 |

**PV's strongest partners:** SYNTHEX (99.0, 8 integration points) and K7 (99.2, 5 points).

## PV Coupling Matrix State

| Metric | Value |
|--------|-------|
| Total edges | 3,782 |
| Unique weights | 2 (0.09 baseline, 0.6 Hebbian) |
| Strengthened edges | 12 (0.32%) |

### Hebbian K4 Clique (weight=0.6)

All 12 directed edges between: `fleet-alpha`, `fleet-beta-1`, `fleet-gamma-1`, `orchestrator-044`

## Cross-Correlation: Synergy DB vs PV Coupling

### Alignment

| Synergy Pair | DB Score | PV Bridge? | Coupling Weight |
|-------------|----------|------------|-----------------|
| PV - SYNTHEX | 99.0 | m22_synthex_bridge | 0.09 (baseline) |
| PV - K7/Nexus | 99.2 | m23_nexus_bridge | 0.09 (baseline) |
| PV - POVM | 98.5 | m25_povm_bridge | 0.09 (baseline) |
| PV - ME | (not in DB) | m24_me_bridge | 0.09 (baseline) |
| PV - RM | (not in DB) | m26_rm_bridge | 0.09 (baseline) |
| PV - VMS | (not in DB) | m27_vms_bridge | 0.09 (baseline) |

### Key Finding: Synergy-Coupling Mismatch

The synergy DB records high scores (98.5-99.2) for PV's service partnerships, but the PV coupling matrix shows **zero differentiation** on these service-to-service edges. All service bridges sit at baseline 0.09.

The only Hebbian-strengthened edges (0.6) are between **fleet coordination spheres** (fleet-alpha, fleet-beta-1, fleet-gamma-1, orchestrator-044) — a **process-level** cluster, not a **service-level** one.

This reveals two distinct coupling layers:

1. **Service synergy** (system_synergy.db) — measures architectural integration depth. PV-SYNTHEX at 99.0/8pts reflects 8 code-level touch points (thermal bridge, phase injection, diagnostic polling, etc.)

2. **Runtime coupling** (PV coupling matrix) — measures temporal co-activation via Hebbian STDP. Fleet instances that coordinated together got strengthened. Services don't register as spheres, so they can't form Hebbian bonds.

### Gap: No Service-Level Hebbian Learning

The PV coupling matrix only tracks **sphere-to-sphere** edges. Service bridges (SYNTHEX, K7, POVM, ME, RM, VMS) operate outside the Kuramoto field — they're fire-and-forget HTTP, not oscillating spheres. This means:

- High-synergy service pairs (K7-SYNTHEX at 59 integration points) have **no runtime feedback** into the coupling field
- Bridge health affects conductor decisions but doesn't modulate coupling weights
- The synergy DB is a static architectural map; PV coupling is a dynamic runtime map

## Recommendations

1. **Bridge synergy injection** — When bridge polls succeed, inject a small coupling signal between the requesting sphere and a virtual "service sphere". This would let Hebbian STDP learn which services are co-active with which fleet members.

2. **Synergy-weighted K initialization** — Use synergy scores to seed initial coupling weights instead of uniform 0.09. PV-SYNTHEX (99.0) could start at 0.15, while PV-saturn-light (92.0) starts at 0.10.

3. **ME bridge gap** — PV-ME synergy is missing from the DB despite having a live bridge (m24). Add the pair: score ~97, integration_points ~4 (observer polling, fitness-to-k_mod, RALPH feedback, EventBus).

4. **K7-SYNTHEX deep coupling** — 59 integration points is extraordinary. Consider a dedicated monitoring dashboard for this pair since degradation here would cascade widely.

5. **Prune stale synergy entries** — Some pairs reference deprecated components (Library Agent is disabled, saturn-light status unknown). Audit for freshness.

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - Post-Deploy Coupling]] — coupling matrix deep dive
- [[Session 049 - Cascade Synthesis]] — ecosystem score analysis
- [[ULTRAPLATE Master Index]] — service topology
- [[Vortex Sphere Brain-Body Architecture]] — coupling field design
