# Session 049 — STDP Weight Evolution

**Date:** 2026-03-21

## PV Coupling Weights

| Weight | Count | % |
|--------|-------|---|
| 0.09 (baseline) | 3,770 | 99.7% |
| 0.60 (fleet clique) | 12 | 0.3% |

Two weight classes only. Binary: either baseline or fully strengthened.

## POVM Pathway Weights (top 20)

| Pre | Post | Weight |
|-----|------|--------|
| nexus-bus:cs-v7 | synthex | **1.046** |
| nexus-bus:devenv-patterns | pane-vortex | **1.020** |
| operator-028 | alpha-left | 1.000 |
| 6:left | 4:top-right | 1.000 |
| nexus-bus:vms-read | pane-vortex | 1.000 |
| orchestrator:550010 | 4:top-right | 1.000 |
| (10 more numeric pairs) | ... | 1.000 |

## PV vs POVM Heavyweight Comparison

| PV Heavyweights (0.60) | In POVM? |
|------------------------|----------|
| orchestrator-044 ↔ fleet-alpha | No |
| orchestrator-044 ↔ fleet-beta-1 | No |
| orchestrator-044 ↔ fleet-gamma-1 | No |
| fleet-alpha ↔ fleet-beta-1 | No |
| fleet-alpha ↔ fleet-gamma-1 | No |
| fleet-beta-1 ↔ fleet-gamma-1 | No |

**No overlap.** PV coupling heavyweights (fleet clique) are completely absent from POVM pathways. The two Hebbian systems track different relationships:
- **PV coupling:** sphere-to-sphere phase co-activation
- **POVM pathways:** service-to-service and pane-to-pane operational co-occurrence

## SYNTHEX Thermal Breakthrough

Injected: `{"source":"stdp-tracker","type":"hebbian","data":{"weight_classes":2,"max_weight":0.6}}`

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Temperature | 0.030 | **0.0475** | **+0.0175** |
| PID output | -0.335 | -0.326 | +0.009 |
| Hebbian reading | 0.0 | 0.0 | none |

**First observed thermal change!** The `type: "hebbian"` ingest event raised temperature from 0.03 to 0.0475 (+58%). PID output shifted accordingly. However, the Hebbian heat source (HS-001) still reads 0.0 — the temperature change came through a different pathway (possibly direct thermal injection rather than heat source routing).

Temperature sustained at 0.0475 across subsequent checks — not a transient spike.

## STDP Learning Velocity

| System | Learning Rate | Evidence |
|--------|-------------|----------|
| PV coupling | Very slow | 12/3782 edges differentiated (~0.3%) over 110K ticks |
| POVM pathways | Moderate | 2436 pathways, strongest at 1.046 (4.6% above baseline) |
| hebbian_pulse.db | Zero | 0 neural_pathways (never wired) |
| SYNTHEX thermal | Responsive | Temperature moved on hebbian-type ingest (+58%) |

---
*Cross-refs:* [[Session 049 - Hebbian Pulse Analysis]], [[Session 049 - SYNTHEX Feedback Loop]], [[Session 049 — Master Index]]
