# Session 049 — Database Census

**Date:** 2026-03-21

## DevEnv Manager DBs (SYNTHEX ecosystem)

| Database | Rows | Tables |
|----------|------|--------|
| agent_deployment.db | 63 | haiku_agents |
| flow_state.db | 30 | flow_sessions, flow_success_patterns |
| flow_tensor_memory.db | 28 | deployment_flows, flow_patterns, tensor_streams, depth_transitions, skill_adaptations |
| hebbian_pulse.db | 95 | config, decay_audit_log, phase1_patterns |
| module_5_1_metrics.db | 14 | api_metrics, test_summary |
| service_tracking.db | 18 | service_events, services |
| synthex_tracking.db | 224 | 8 tables (sync, metrics, endpoints, phases, health, services, metadata, websocket) |
| tensor_memory.db | 49 | 9 tables (tensor_memory, neural_pathways, tensors, pca_projections, saturn_encodings, etc.) |
| system_synergy.db | 0 | (empty) |
| **Subtotal** | **521** | |

Additional empty/schema-only DBs: compliance_tracking, v3_homeostasis, synthex.db (data variant)

## Pane-Vortex DBs

| Database | Rows | Tables |
|----------|------|--------|
| bus_tracking.db | 3,782 | bus_events, bus_tasks, cascade_events, event_subscriptions, task_dependencies, task_tags, schema_versions |
| field_tracking.db | 23,632 | field_snapshots, coupling_history, sphere_history, executor_tasks, schema_versions |
| povm_data.db | 40 | memories, pathways, field_snapshots, sessions |
| backup-v1/ (2 DBs) | — | Same schema as above (pre-deploy backup) |
| **Subtotal** | **27,454** | |

## Total Ecosystem

| Category | DBs | Rows | Dominant |
|----------|-----|------|----------|
| DevEnv/SYNTHEX | 9+ active | 521 | synthex_tracking (224) |
| Pane-Vortex | 3 active | 27,454 | field_tracking (23,632) |
| **Total** | **12+ active** | **27,975** | |

## K7 Module Status

- **45/45 modules healthy** (M1–M45)
- 0 degraded, 0 unhealthy

---
*Cross-refs:* [[ULTRAPLATE Master Index]], [[Session 049 — Master Index]]
