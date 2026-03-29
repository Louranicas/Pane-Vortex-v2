# Hebbian Operational Topology — Session 045

## Emergent Operational Coupling (hebbian_pulse.db)

90 pathways. The strongest reveal which tools/patterns fire together most:

### Top Pathways by Activation Count

| Source → Target | Strength | Activations | Meaning |
|-----------------|----------|-------------|---------|
| B2_quality_gate → B5_output_filter | 1.0 | 202 | Quality gate always pipes through output filter |
| DevOps → SYNTHEX | 0.98 | 150 | DevOps tasks trigger SYNTHEX brain |
| Bash_Engine → NAIS | 0.99 | 142 | Bash commands activate neural adaptive |
| B1_sqlite → B5_output_filter | 1.0 | 102 | SQLite queries always filtered |
| B8_verification → B2_quality_gate | 0.99 | 92 | Verification triggers quality gate |
| TC4_sqlite_read → TC4_sqlite_write | 1.0 | 82 | Read-modify-write pattern |
| B3_health_check → B7_process_lifecycle | 0.98 | 82 | Health check triggers process mgmt |
| TC2_parallel_fanout → conditional_action | 1.0 | 63 | Fan-out leads to conditional execution |
| TC5_bash_build → TC5_read_offset | 1.0 | 52 | Build triggers read-at-offset |
| TC5_edit_fix → TC5_bash_verify | 1.0 | 52 | Edit triggers build verification |

### Operational Clusters

```
Cluster 1: Quality Pipeline (most active)
  B2_quality_gate ──202──→ B5_output_filter
  B8_verification ──92───→ B2_quality_gate
  TC5_edit_fix ──52──→ TC5_bash_verify
  Total activations: 346

Cluster 2: Service Integration
  DevOps ──150──→ SYNTHEX
  Bash_Engine ──142──→ NAIS
  Total activations: 292

Cluster 3: Data Pipeline
  B1_sqlite ──102──→ B5_output_filter
  TC4_sqlite_read ──82──→ TC4_sqlite_write
  B3_health_check ──82──→ B7_process_lifecycle
  Total activations: 266

Cluster 4: Tool Chains
  TC1_funnel_discovery ──42──→ native_read_tool
  TC2_parallel_fanout ──63──→ conditional_action
  TC3_subagent_isolation ──22──→ bash_verification
  Total activations: 127
```

### B11 Discovery

A new pattern `B11_heredoc_batch → B1_sqlite` exists with 3 activations and
strength 0.98. This represents the heredoc-based batch commit pattern connecting
to SQLite state queries — likely from git commit workflows.

## Pulse Events (34 total)

The hebbian pulse system fires on significant state transitions. 34 events
recorded, 32 decay audits. 5 pulses in the hebbian_pulses table.
0 neural_pathways (the neural_pathways table is unused — the actual learning
happens through hebbian_pathways).

## POVM vs Hebbian Topology Comparison

| Feature | POVM Pathways | Hebbian Pulse |
|---------|--------------|---------------|
| Count | 2,427 | 90 |
| Scope | Cross-sphere coupling | Cross-tool coupling |
| Weight range | 0.15–1.05 | 0.0–1.0 |
| Activation tracking | No | Yes (count per edge) |
| Decay | STDP (LTP/LTD) | Audit-logged decay |
| Hub topology | beta-left, GAMMA-synth | B2→B5 (quality pipeline) |
| Bimodal | Yes (90% default, 2.8% crystal) | No (continuous) |

POVM tracks sphere-to-sphere coupling. Hebbian tracks pattern-to-pattern coupling.
They operate at different levels but converge in structure — both show hubs with
dense connections and periphery with sparse links.
