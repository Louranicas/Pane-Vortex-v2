# Cross-Database Architecture

## 6 Database Paradigms

| Paradigm | Example | Pattern |
|----------|---------|---------|
| WAL SQLite | PV field_tracking.db | High-write snapshots |
| Tracking DB | service_tracking.db | Append-only events |
| Tensor Memory | tensor_memory.db | 11D tensor encoding |
| Hebbian Pulse | hebbian_pulse.db | Pathway strength + LTP/LTD |
| Synergy Scoring | system_synergy.db | Cross-service scores |
| TSV Flat File | Reasoning Memory | Category\tAgent\tConf\tTTL\tContent |

## Key Databases by Service

| Service | Database | Key Data |
|---------|----------|----------|
| PV | field_tracking.db | field_snapshots, sphere_history, coupling |
| PV | bus_tracking.db | bus_tasks, bus_events, cascade_events |
| SYNTHEX | synthex.db, v3_homeostasis.db, hebbian_pulse.db, flow_tensor_memory.db | Core state, thermal PID, neural, tensor |
| DevEnv | service_tracking.db, system_synergy.db, episodic_memory.db | Health history, synergy, sessions |
| Orchestrator | code.db, tensor_memory.db, performance.db | Modules, SAN-K7 tensors, benchmarks |
| POVM | povm_data.db | 36 memories, 2425 pathways |
| RM | TSV flat file | 3400+ entries |

## Cross-DB Query Patterns

```bash
# Synergy scores
sqlite3 -header -column ~/claude-code-workspace/developer_environment_manager/system_synergy.db \
  "SELECT system_1, system_2, ROUND(synergy_score,2), integration_points FROM system_synergy WHERE integration_points > 5 ORDER BY integration_points DESC;"

# SAN-K7 ↔ SYNTHEX: 59 integration points (highest)
# SYNTHEX ↔ DevOps: 10 points, 97.3% synergy
# Swarm ↔ RM: 12 points, 98.0% synergy
```

## Known Issues
- 166 databases total, 360.6 MB — 20-30% are empty schemas
- hebbian_pulse.db has 0 neural_pathways, only 5 pulses
- field_tracking.db is at pane-vortex/data/ NOT ~/.local/share/
- ME EventBus has 275K events but subscriber_count=0 (cosmetic — uses polling)
