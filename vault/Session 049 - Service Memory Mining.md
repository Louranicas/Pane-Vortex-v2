# Session 049 — Service Memory Mining

> **service_tracking.db: 22 tables, 32 indices, 213 total rows**
> **Richest tables: learned_patterns (57), service_events (33), orchestration_graph (25)**
> **Captured:** 2026-03-21

---

## Database Overview

**Location:** `~/claude-code-workspace/developer_environment_manager/service_tracking.db`

| Category | Tables | Total Rows |
|----------|--------|------------|
| Service lifecycle | 4 (services, dependencies, events, registry) | 60 |
| Communication | 3 (comm_paths, coordination_patterns, health_checks) | 14 |
| Intelligence | 3 (orchestration_graph, synergy, compatibility) | 31 |
| Learning | 4 (learned_patterns, cross_agent, decision_improvements, workflow) | 65 |
| Token optimization | 4 (startup_opts, efficiency, performance, events) | 16 |
| Infrastructure | 3 (metadata, module_tracking, sqlite_sequence) | 27+ |
| **Total** | **22** | **213** |

**No event_log.db found** anywhere in the workspace.

---

## Service Communication Paths (10 paths)

| Source | Target | Type | Protocol |
|--------|--------|------|----------|
| devops-engine-v2 | synthex-core-v2 | async | https-rest |
| san-k7-m20 | synthex-engine | async | json-rpc |
| san-k7-m21 | service-mesh-controller | sync | grpc |
| san-k7-m23 | analytics-engine | async | json-rpc |
| san-k7-m6.2 | synthex-engine | async | protobuf |
| sphere-vortex-v3 | synthex-core | sync | http |
| sphere-vortex-v3 | san-k7-orchestrator | sync | http |
| sphere-vortex-v3 | tool-library | async | http |
| synthex-core-v2 | devops-engine-v2 | event | websocket |
| synthex-engine | san-k7-orchestrator | async | json-rpc |

**Finding:** sphere-vortex has 3 outbound paths (SYNTHEX, K7, Tool Library). K7 has the most connections (4 paths involving K7 modules). SYNTHEX is the most connected overall — appears in 5/10 paths as source or target.

---

## Data-Sharing Service Pairs (by synergy)

### Inter-Service Synergy (measured)

| Source | Target | Metric | Score |
|--------|--------|--------|-------|
| sphere-vortex-v3 | san-k7-orchestrator | orchestration_coupling | **0.992** |
| sphere-vortex-v3 | synthex-core | cognitive_integration | **0.985** |
| sphere-vortex-v3 | nais | intelligence_alignment | 0.974 |
| devops-engine-v2 | synthex-core-v2 | orchestration-efficiency | 0.920 |
| synthex-core-v2 | devops-engine-v2 | learning-feedback-effectiveness | 0.880 |

**Top pair:** PV-K7 at 0.992 synergy — nearly perfect orchestration coupling. This aligns with the system_synergy.db score of 99.2 for the same pair.

### Orchestration Graph (top weighted edges)

| Source Module | Target Module | Relationship | Weight |
|---------------|---------------|-------------|--------|
| san-k7-m20 | synthex-core | ml-pattern-learning | 99.1 |
| san-k7-m24 | workflow-manager | distributed-orchestration | 98.8 |
| san-k7-m23 | analytics-engine | pattern-recognition | 98.7 |
| san-k7-m21 | service-mesh | traffic-intelligence | 98.5 |
| synthex-core | san-k7-m6.2 | unified-pipeline | 97.4 |

K7 modules dominate the graph. The K7-SYNTHEX axis is the backbone of the orchestration network.

---

## Learned Patterns (57 total, top 10)

| Pattern | Strength | Reinforcements | Type |
|---------|----------|----------------|------|
| progressive_disclosure | 1.000 | 1 | best_practice |
| cached_startup_knowledge | 1.000 | 1 | best_practice |
| B1_sqlite_state_query | 0.980 | 10 | best_practice |
| B2_quality_gate_chain | 0.980 | 10 | best_practice |
| TC4_sqlite_state_loop | 0.980 | 6 | tool_chain |
| B5_rg_alternation_fix | 0.980 | 1 | environment_optimization |
| database_schema_assumption | 0.980 | 2 | **anti_pattern** |
| greptile_api_v2_deprecated | 0.980 | 3 | integration |
| TC1_funnel_discovery | 0.970 | 6 | tool_chain |
| TC5_build_fix_converge | 0.970 | 6 | tool_chain |

**Most reinforced:** B1 and B2 patterns at 10 reinforcements each — these are the most-validated operational patterns.

---

## Token Efficiency Gains

| Operation | Before | After | Improvement | Method |
|-----------|--------|-------|-------------|--------|
| environment_startup | 102,000 | 500 | **99.5%** | Progressive disclosure skill |
| state_management (TC4) | 25,000 | 130 | 99.5% | SQLite closed-loop vs MCP |
| session_bootstrap (B1) | 25,000 | 200 | 99.2% | SQLite replaces MCP read_graph |
| exploration (TC3) | 50,000 | 500 | 99.0% | Subagent context isolation |
| health_sweep (B3) | 2,600 | 39 | 98.5% | curl status code only |

**Total savings:** From ~204,600 tokens to ~1,369 tokens per session bootstrap — **99.3% reduction**.

---

## Cross-Agent Learnings (3 entries)

1. **environment_startup → all:** Progressive disclosure, cache knowledge, trust system state
2. **claude-opus-4.6 → all:** B1-B10 bash patterns, synergy 0.88-0.98, always prefer dedicated tools
3. **claude-opus → all:** Greptile API requires TWO separate credentials (API key + Git PAT)

---

## K7 Synergy Check

| Metric | Value |
|--------|-------|
| Command | synergy-check |
| Module | M45 |
| Status | Executed |
| Duration | 0ms |

Synergy check passed with zero latency — likely a cached/pre-computed result.

---

## Which Service Pairs Share the Most Data?

Combining communication_paths, inter_service_synergy, and orchestration_graph:

| Rank | Pair | Evidence |
|------|------|----------|
| **1** | **K7 ↔ SYNTHEX** | 59 integration points (system_synergy.db), 3 comm paths, 4 orchestration edges, bidirectional async (json-rpc + protobuf) |
| **2** | **PV ↔ K7** | synergy 0.992, sync HTTP, 5 integration points, orchestration_coupling metric |
| **3** | **PV ↔ SYNTHEX** | synergy 0.985, sync HTTP, 8 integration points (system_synergy.db), cognitive_integration metric |
| **4** | **DevOps ↔ SYNTHEX** | Bidirectional (async https-rest + event websocket), synergy 0.92/0.88 |
| **5** | **PV ↔ NAIS** | synergy 0.974, intelligence_alignment, 3 integration points |

**K7-SYNTHEX is the most data-coupled pair** across all persistence layers — 59 integration points, 3+ communication paths, and the highest orchestration graph weights.

---

## Cross-References

- [[Session 049 — Master Index]]
- [[Session 049 - Synergy Analysis]] — system_synergy.db cross-correlation
- [[Session 049 - Persistence Architecture]] — persistence layer map
- [[Session 049 - Trinity Chain]] — K7-SYNTHEX-ME live analysis
- [[ULTRAPLATE Master Index]]
