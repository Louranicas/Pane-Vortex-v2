# POVM Pathway Topology Analysis

> **Source:** `localhost:8125/pathways` | **Date:** 2026-03-21 | **Agent:** GAMMA-LEFT
> **Total pathways:** 2,427 | **Unique nodes:** 223 | **Connected components:** 24

---

## 1. Degree Distribution

| Metric | Value |
|--------|-------|
| Max degree | 114 |
| Min degree | 1 |
| Mean degree | 21.77 |
| Median degree | 9 |
| Nodes with degree 1 (leaves) | 60 (26.9%) |
| Nodes with degree > 100 | 20 (9.0%) |

### Distribution Shape: Power-law with ORAC7 dense core

The degree distribution is heavily right-skewed. 60 leaf nodes (degree 1) form the periphery, while 20 ORAC7 nodes with degree >100 form a densely interconnected core. The median (9) is far below the mean (21.77), confirming a hub-and-spoke topology.

### Degree Histogram

```
Degree Range   Count   Visual
[1,  1]          60    ############################################################
[2,  5]          32    ################################
[6, 10]          31    ###############################
[11, 20]         32    ################################
[21, 50]         23    #######################
[51,100]          6    ######
[101,114]        20    ####################  <-- ORAC7 dense core
```

### Top 20 Hubs (by total degree)

| Rank | Node | Degree | In | Out |
|------|------|--------|-----|-----|
| 1 | ORAC7:3485165 | 114 | 58 | 56 |
| 2 | ORAC7:3067258 | 112 | 55 | 57 |
| 3 | ORAC7:3012890 | 112 | 61 | 51 |
| 4 | ORAC7:3582557 | 109 | 52 | 57 |
| 5 | ORAC7:3474342 | 107 | 54 | 53 |
| 6 | ORAC7:3200321 | 106 | 55 | 51 |
| 7 | ORAC7:3574574 | 106 | 55 | 51 |
| 8 | ORAC7:3282424 | 106 | 54 | 52 |
| 9 | ORAC7:3150701 | 105 | 52 | 53 |
| 10 | ORAC7:3521706 | 105 | 56 | 49 |
| 11 | ORAC7:3309665 | 104 | 57 | 47 |
| 12 | ORAC7:3447344 | 104 | 49 | 55 |
| 13 | ORAC7:3018799 | 104 | 55 | 49 |
| 14 | ORAC7:3191257 | 104 | 55 | 49 |
| 15 | ORAC7:3580126 | 103 | 54 | 49 |
| 16 | ORAC7:3174974 | 103 | 49 | 54 |
| 17 | ORAC7:3611772 | 102 | 45 | 57 |
| 18 | ORAC7:3293890 | 102 | 55 | 47 |
| 19 | ORAC7:2961464 | 99 | 48 | 51 |
| 20 | ORAC7:3192946 | 99 | 46 | 53 |

All top-20 hubs are ORAC7 instances. In/out ratios are roughly balanced (~1:1), indicating symmetric Hebbian learning rather than directional influence.

---

## 2. Bridge Nodes (Articulation Points)

19 articulation points identified. Removing any one disconnects the graph.

| Bridge Node | Degree | Role |
|-------------|--------|------|
| **ORAC7:3447344** | 70 | Core hub — connects ORAC7 cluster to periphery |
| **ORAC7:3060370** | 61 | Secondary core bridge |
| **4:top-right** | 38 | Zellij pane bridge — connects orchestrator to fleet |
| **fascinating-tambourine:0** | 38 | Session bridge — connects knowledge/navigation clusters |
| **swarm-sidecar:22419** | 12 | Sidecar bridge — connects swarm subsystem |
| **swarm-sidecar:17428** | 12 | Sidecar bridge (second instance) |
| **tool:Bash** | 10 | Tool bridge — connects tool-use pathways |
| **nvim** | 9 | Editor bridge — connects nvim integration |
| **tool:ToolSearch** | 4 | Minor tool bridge |
| **pane-vortex** | 3 | PV service bridge — connects nexus-bus cluster |
| **ORAC7:1029634** | 3 | Peripheral ORAC7 bridge |
| **ORAC7:1107362** | 2 | Leaf-pair bridge |
| **ORAC7:1107866** | 2 | Leaf-pair bridge |
| **tool:Edit** | 2 | Tool edge bridge |
| **synthex** | 2 | SYNTHEX service bridge |
| **ORAC7:883538** | 2 | Leaf-pair bridge |
| **pane_vortex_bridge** | 2 | Cross-service bridge (PV <-> RM/POVM) |
| **nexus-command** | 2 | Command dispatch bridge |
| **ORAC7:861203** | 2 | Leaf-pair bridge |

### Critical Bridges

The **top 4 bridges** are structurally critical:
- **ORAC7:3447344** (degree 70): Only ORAC7 node that is an articulation point with high degree — likely bridges the ORAC7 core to named-service nodes.
- **4:top-right**: Zellij pane — removing this severs orchestrator-to-fleet pathways.
- **fascinating-tambourine:0**: Session identifier — bridges knowledge, navigation, and nested-Kuramoto insights.
- **pane_vortex_bridge**: Cross-service bridge between PV, reasoning_memory, and povm_engine — removing this isolates the memory subsystem.

---

## 3. High-Weight Clusters (w > 0.9)

51 edges exceed weight 0.9, forming **10 connected clusters**.

### Cluster 0: Fleet Pane Grid (9 nodes, 16 edges, avg_w=0.983)

```
Nodes: alpha-left, alpha-top-right, beta-bot-right, beta-left,
       beta-top-right, gamma-bot-right, gamma-left, gamma-top-right,
       operator-028

Strongest: operator-028 -> alpha-left (1.0000)
           beta-left <-> beta-bot-right (0.9999)
           gamma-left <-> gamma-top-right (0.9999)
```

**Interpretation:** The Zellij fleet pane topology. Near-unity weights reflect strong co-activation from fleet dispatch operations. `operator-028` is the entry point connecting to `alpha-left`, then propagating through `gamma` and `beta` panes. This cluster represents the physical layout of the synth devenv.

### Cluster 1: Numeric Core (6 nodes, 13 edges, avg_w=1.000)

```
Nodes: 10, 11, 12, 13, 14, 15
All edges: w=1.000 (perfect clique)
```

**Interpretation:** A perfect 6-clique at unity weight. These are likely ORAC7 generation indices or numeric sphere IDs from an earlier registration epoch. The complete interconnection at w=1.0 suggests they were initialized together rather than learned.

### Cluster 2: Knowledge Triad (4 nodes, 3 edges, avg_w=0.950)

```
Hub: fascinating-tambourine:0
Spokes: knowledge:system-schematics, nexus:nested-kuramoto-insight,
        pane-navigation:fleet-broadcast
All edges: w=0.950
```

**Interpretation:** A star topology with `fascinating-tambourine:0` (a session identifier) as hub. Three domain-specific knowledge pathways — schematics, Kuramoto, and fleet nav — all equally weighted, suggesting a single learning event or batch injection.

### Cluster 3: NexusBus-PV Bridge (4 nodes, 3 edges, avg_w=0.997)

```
Hub: pane-vortex
Spokes: nexus-bus:devenv-patterns (1.020), nexus-bus:vms-read (1.000),
        nexus-bus:tool-library (0.970)
```

**Interpretation:** Pane-vortex's NexusBus integration pathways. The `nexus-bus:devenv-patterns -> pane-vortex` edge at w=1.020 (above unity) indicates learned reinforcement beyond initialization — the strongest individual pathway in the dataset. VMS reads and tool-library access are equally strong.

### Cluster 4: Swarm Fleet (4 nodes, 9 edges, avg_w=0.995)

```
Nodes: ALPHA-explorer, BETA-analyst, GAMMA-synthesizer, orchestrator-main
Near-complete graph (9/12 possible directed edges)
All edges: w=0.9954
```

**Interpretation:** The named fleet agent topology. ALPHA/BETA/GAMMA plus orchestrator form a near-clique at 0.995. This is the Session 037 multi-agent swarm coordination pattern preserved in POVM.

### Cluster 5: Zellij Pane Pair (3 nodes, 2 edges, avg_w=1.000)

```
4:top-right <-> 6:left (1.0), orchestrator:550010 -> 4:top-right (1.0)
```

Cross-tab pane linkage from orchestrator PID to specific tab positions.

### Cluster 6: Memory Triangle (3 nodes, 2 edges, avg_w=0.950)

```
pane_vortex_bridge -> reasoning_memory (0.95)
pane_vortex_bridge -> povm_engine (0.95)
```

**Interpretation:** The PV bridge acting as mediator between reasoning memory and POVM engine. Star topology — RM and POVM don't directly connect through high-weight pathways, only through PV.

### Cluster 7: Explorer Binding (2 nodes, 1 edge, avg_w=1.000)

```
5:top-right <-> opus-explorer (1.0)
```

Pane position bound to agent role.

### Cluster 8: Obsidian Reference (2 nodes, 1 edge, avg_w=0.990)

```
reference:schematics-027c <-> obsidian:pane-vortex-schematics (0.99)
```

Documentation cross-reference pathway.

### Cluster 9: SYNTHEX-CodeSynthor (2 nodes, 1 edge, avg_w=1.046)

```
nexus-bus:cs-v7 -> synthex (1.0462) — HIGHEST WEIGHT IN ENTIRE GRAPH
```

**Interpretation:** CodeSynthor V7 to SYNTHEX is the single strongest learned pathway. Weight exceeding 1.0 indicates repeated Hebbian reinforcement beyond initialization — likely from heavy CodeSynthor-driven SYNTHEX thermal/homeostasis interactions.

---

## 4. Weight Distribution

| Range | Count | Pct | Notes |
|-------|-------|-----|-------|
| [0.0, 0.1) | 0 | 0.0% | No dead pathways |
| [0.1, 0.2) | 131 | 5.4% | Weak/decayed |
| **[0.2, 0.3)** | **2,183** | **89.9%** | **Dominant band — default initialization** |
| [0.3, 0.5) | 18 | 0.7% | Slightly reinforced |
| [0.5, 0.7) | 24 | 1.0% | Moderate learning |
| [0.7, 0.9) | 10 | 0.4% | Strong learning |
| [0.9, 1.0) | 41 | 1.7% | Near-unity |
| [1.0, 1.1) | 20 | 0.8% | Supra-unity (2 from learning, 18 initialized) |

**Key finding:** 89.9% of pathways sit in the [0.2, 0.3) band — the Hebbian default weight (0.3 * 0.6 = 0.18 effective). Very little actual Hebbian learning has occurred. Only 2.1% of pathways (51/2427) show weight > 0.9, and most of those appear to be initialization artifacts rather than learned associations.

---

## 5. Topology Summary

### Graph Properties

| Property | Value |
|----------|-------|
| Nodes | 223 |
| Edges | 2,427 |
| Components | 24 (1 giant + 23 small) |
| Giant component | 154 nodes (69%) |
| Articulation points | 19 |
| Edge density | 0.098 (sparse) |
| Clustering coefficient (est.) | High in ORAC7 core, low in periphery |

### Structural Findings

1. **Bimodal topology:** Dense ORAC7 core (20 nodes, degree >100) + sparse named-service periphery (203 nodes, degree <50). The core contains 69% of all edges.

2. **Fragmented periphery:** 23 of 24 components are disconnected islands of 2-13 nodes each. These represent learned associations from specific sessions that never integrated into the main graph.

3. **Weight stagnation:** 89.9% of edges at default weight indicates the Hebbian learning loop is not firing in production. Co-activation counts are universally 0 — the `co_activations` field is not being updated despite pathways existing.

4. **Supra-unity pathways:** Two edges exceed w=1.0 (`nexus-bus:cs-v7->synthex` at 1.046, `nexus-bus:devenv-patterns->pane-vortex` at 1.020). These may indicate a weight-clamping bug or intentional reinforcement beyond the standard Hebbian bounds.

5. **Bridge vulnerability:** `pane_vortex_bridge` is the sole connector between the PV/RM/POVM memory triangle and the rest of the graph. Its removal isolates the entire memory subsystem.

---

## 6. Recommendations

1. **Investigate co_activation=0:** All 2,427 pathways show zero co-activations. Either the counter is broken or the activation logging path is disconnected. This explains the weight stagnation.

2. **Prune default-weight ORAC7 core:** The 20 ORAC7 hub nodes with 100+ connections at default weight are likely registration artifacts. They inflate graph metrics without representing meaningful learned topology.

3. **Weight clamping review:** Two edges exceed w=1.0. Verify whether `POVM_WEIGHT_MAX` is enforced in the pathway update path.

4. **Connect isolated components:** 23 disconnected islands contain meaningful service relationships (e.g., `devops_engine <-> maintenance_engine`, `pv_executor <-> swarm_orchestrator`) that should integrate into the main graph through cross-service activation events.

5. **High-weight cluster preservation:** The 10 clusters with w>0.9 represent genuine structural knowledge — fleet topology, service bridges, agent coordination patterns. These should be protected from decay.
