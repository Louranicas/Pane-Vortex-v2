# Session 049 — System Architecture Schematics

**Date:** 2026-03-21 | **All 16/16 services verified live**

## 1. ULTRAPLATE Service Topology (16 Active Services)

```mermaid
graph TB
    subgraph "Batch 1 (no deps)"
        DE[DevOps Engine<br/>:8081]
        CS[CodeSynthor V7<br/>:8110]
        POVM[POVM Engine<br/>:8125]
    end

    subgraph "Batch 2 (needs B1)"
        SX[SYNTHEX<br/>:8090]
        K7[SAN-K7<br/>:8100]
        ME[Maintenance Engine<br/>:8080]
        AA[Architect Agent<br/>:9001]
        PS[Prometheus Swarm<br/>:10001]
    end

    subgraph "Batch 3 (needs B2)"
        NAIS[NAIS<br/>:8101]
        BE[Bash Engine<br/>:8102]
        TM[Tool Maker<br/>:8103]
    end

    subgraph "Batch 4 (needs B3)"
        CCM[Context Manager<br/>:8104]
        TL[Tool Library<br/>:8105]
        RM[Reasoning Memory<br/>:8130]
    end

    subgraph "Batch 5 (needs B4)"
        VMS[Vortex Memory<br/>:8120]
        PV[Pane-Vortex<br/>:8132]
    end

    DE --> SX
    CS --> K7
    POVM --> VMS
    POVM --> PV
    SX --> PV
    K7 --> NAIS
    ME --> CCM
    NAIS --> CCM
    BE --> TL
    TM --> TL
    CCM --> VMS
    RM --> PV
```

## 2. PV2 8-Layer Module Dependency Graph

```mermaid
graph BT
    L1[L1 Foundation<br/>m01-m06<br/>3.4K LOC, 183 tests]
    L2[L2 Services<br/>m07-m10<br/>4.6K LOC, 95 tests]
    L3[L3 Field<br/>m11-m15<br/>4.2K LOC, 211 tests]
    L4[L4 Coupling<br/>m16-m18<br/>1.7K LOC, 94 tests]
    L5[L5 Learning<br/>m19-m21<br/>1.4K LOC, 77 tests]
    L6[L6 Bridges<br/>m22-m28<br/>6.8K LOC, 390 tests]
    L7[L7 Coordination<br/>m29-m36<br/>7.5K LOC, 311 tests]
    L8[L8 Governance<br/>m37-m41<br/>2.1K LOC, 108 tests]

    L2 --> L1
    L3 --> L1
    L4 --> L3
    L5 --> L4
    L6 --> L5
    L6 --> L2
    L7 --> L6
    L8 --> L7

    style L1 fill:#4a9,stroke:#333
    style L6 fill:#f96,stroke:#333
    style L7 fill:#f96,stroke:#333
```

## 3. Bridge Data Flow (6 Bridges + Consent Gate)

```mermaid
graph LR
    PV[PV2 Tick Loop<br/>:8132]

    subgraph "Bridge Poll (read, every N ticks)"
        SX_R[SYNTHEX<br/>:8090<br/>/v3/thermal<br/>Standard 30s]
        NX_R[Nexus K7<br/>:8100<br/>/api/v1/nexus<br/>Standard 30s]
        ME_R[ME<br/>:8080<br/>/api/observer<br/>Standard 30s]
        POVM_R[POVM<br/>:8125<br/>/hydrate<br/>Fast 6s]
        RM_R[RM<br/>:8130<br/>/search<br/>Standard 30s]
        VMS_R[VMS<br/>:8120<br/>/health<br/>Standard 30s]
    end

    subgraph "Bridge Post (write, on events)"
        SX_W[SYNTHEX<br/>/api/ingest]
        NX_W[Nexus<br/>/api/v1/nexus/command]
        ME_W[ME<br/>/api/events]
        POVM_W[POVM<br/>/memories]
        RM_W[RM<br/>POST /put TSV]
        VMS_W[VMS<br/>/memories]
    end

    subgraph "Consent Gate (m28)"
        CG[consent_gated_k_adjustment<br/>Per-sphere opt-out]
    end

    PV -->|poll| SX_R
    PV -->|poll| NX_R
    PV -->|poll| ME_R
    PV -->|poll| POVM_R
    PV -->|poll| RM_R
    PV -->|poll| VMS_R

    SX_R -->|combined_effect| CG
    NX_R -->|combined_effect| CG
    ME_R -->|combined_effect| CG
    CG -->|k_mod| PV

    PV -->|post| SX_W
    PV -->|post| NX_W
    PV -->|post| ME_W
    PV -->|post| POVM_W
    PV -->|post| RM_W
```

### Poll Tiers

| Tier | Interval | Services |
|------|----------|----------|
| Fast (6s) | Every tick | POVM |
| Standard (30s) | Every 6 ticks | ME, RM, VMS, SYNTHEX |
| Nexus | Every 6 ticks | K7 (combined with poll) |

### Write Triggers

| Bridge | Trigger | Direction |
|--------|---------|-----------|
| SYNTHEX | Tick thermal data | PV → SX |
| Nexus | Pattern events | PV → K7 |
| ME | Field state events | PV → ME |
| POVM | Memory crystallisation | PV → POVM |
| RM | Heartbeats + tasks | PV → RM (TSV!) |

---
*Cross-refs:* [[ULTRAPLATE Master Index]], [[Session 049 — Full Remediation Deployed]]
