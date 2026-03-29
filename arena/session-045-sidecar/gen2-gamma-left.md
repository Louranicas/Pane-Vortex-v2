# GAMMA-LEFT Gen2 Report — Field History Mining

> **Agent:** GAMMA-LEFT | **Date:** 2026-03-21 | **Source:** `~/.local/share/pane-vortex/field_tracking.db`

## Field Decision History (from snapshots)

```
decision_action | cnt
----------------|----
Stable          |  37
FreshFleet      |  36
```

**Total snapshots:** 73

## Analysis

- **Stable (37, 50.7%):** Field in equilibrium — no intervention needed.
- **FreshFleet (36, 49.3%):** Spheres registered but none had worked yet — startup/re-registration state.
- **Missing states:** Zero occurrences of `HasBlockedAgents`, `NeedsCoherence`, `NeedsDivergence`, `IdleFleet`, or `Recovering` in the snapshot history — despite the live field currently reporting `HasBlockedAgents` with r=0.895 and 100 tunnels.

## Implication

The snapshot DB captures only quiescent states. Active decision states (`HasBlockedAgents`, `NeedsCoherence`, `NeedsDivergence`) are transient and resolve between the 60-tick snapshot interval. The current live `HasBlockedAgents` state is invisible to historical analysis — a gap in observability.

GAMMA-LEFT REPORTING: field history mined
