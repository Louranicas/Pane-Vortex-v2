# Tunnel Detection — Buoy Resonance

> Generated Session 044 by Claude-BETA analyzing src/field.rs:108-161 (12KB analysis)

## Geometry: Buoys on a Unit Sphere (S²)

Each sphere gets 3 buoys at 120° intervals on the equator:

| Buoy | θ (polar) | φ (azimuth) | Learning Rate | Point3D |
|------|-----------|-------------|---------------|---------|
| primary | π/2 | 0 | 0.005 | (1.0, 0.0, 0.0) |
| secondary | π/2 | 2π/3 | 0.002 | (-0.5, 0.866, 0.0) |
| tertiary | π/2 | 4π/3 | 0.001 | (-0.5, -0.866, 0.0) |

## How Tunnels Form

1. Buoys **drift** via learning: each tool use nudges the nearest buoy toward a semantic attractor
2. Two spheres doing **similar work** develop overlapping buoy positions
3. `detect_tunnels()` computes **buoy overlap** = dot product of normalized positions
4. Overlap > TUNNEL_THRESHOLD (0.8) + phase proximity < 0.8 rad → **tunnel detected**
5. Tunnels create **preferential routing** for cascade handoffs and task dispatch

## Key Constants
- `TUNNEL_THRESHOLD`: 0.8 rad phase proximity
- Buoy learning rates: 0.005 (primary), 0.002 (secondary), 0.001 (tertiary)
- 3 buoys per sphere, placed at 120° intervals

## Current State
- 100 tunnels across 19 spheres (from field/decision API)
- Tunnels are dense because many spheres share "primary" buoy labels

## Links
- [[Hebbian Learning Deep Dive]]
- [[Session 044 — Fleet Orchestration Pioneer]]
- [[KURAMOTO_FIELD_SPEC]]
