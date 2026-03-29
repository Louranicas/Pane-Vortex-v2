# Nexus Performance Baseline — Hebbian Task

> **Instance:** PV2-MAIN | Command tab | 2026-03-21
> **Target:** SAN-K7 Orchestrator :8100 | **Tick:** ~73,500
> **Method:** 4 iterations (1 detailed + 3 timing-only) per command

---

## 1. Command Metadata (Iteration 0 — Full Capture)

| Command | Module | Route | Status | Exec (ms) | Curl (ms) |
|---------|--------|-------|--------|-----------|-----------|
| service-health | M6 | static | Completed | 0 | 0.365 |
| synergy-check | M45 | static | Completed | 0 | 0.262 |
| best-practice | M44 | static | Completed | 0 | 0.215 |
| deploy-swarm | M40 | static | Completed | 0 | 0.474 |
| memory-consolidate | M2 | static | Completed | 0 | 0.231 |
| lint | M45 | static | Completed | 0 | 0.252 |
| compliance | M45 | static | Completed | 0 | 0.494 |
| build | M45 | static | Completed | 0 | 0.424 |
| pattern-search | M2 | static | Completed | 0 | 0.387 |
| module-status | M45 | static | Completed | 0 | 0.232 |

**All 10 commands:** 0ms internal execution (static routes), sub-millisecond curl round-trip.

---

## 2. Latency Distribution (4 Iterations)

| Command | Iter 0 | Iter 1 | Iter 2 | Iter 3 | Mean | Min | Max | Stdev |
|---------|--------|--------|--------|--------|------|-----|-----|-------|
| service-health | 0.365 | 0.550 | 0.258 | 0.935 | 0.527 | 0.258 | 0.935 | 0.256 |
| synergy-check | 0.262 | 0.376 | 0.382 | 0.498 | 0.380 | 0.262 | 0.498 | 0.084 |
| best-practice | 0.215 | 0.254 | 0.239 | 0.243 | 0.238 | 0.215 | 0.254 | 0.014 |
| deploy-swarm | 0.474 | 0.251 | 0.501 | 0.263 | 0.372 | 0.251 | 0.501 | 0.118 |
| memory-consolidate | 0.231 | 0.220 | 0.219 | 0.250 | 0.230 | 0.219 | 0.250 | 0.013 |
| lint | 0.252 | 0.466 | 0.224 | 0.211 | 0.288 | 0.211 | 0.466 | 0.103 |
| compliance | 0.494 | 0.260 | 0.254 | 0.235 | 0.311 | 0.235 | 0.494 | 0.109 |
| build | 0.424 | 0.334 | 0.368 | 0.445 | 0.393 | 0.334 | 0.445 | 0.043 |
| pattern-search | 0.387 | 0.208 | 0.242 | 0.273 | 0.278 | 0.208 | 0.387 | 0.067 |
| module-status | 0.232 | 0.217 | 0.402 | 0.237 | 0.272 | 0.217 | 0.402 | 0.076 |

---

## 3. Performance Tiers

### Sorted by Mean Latency

```
Command              Mean(ms)   Tier          Heatmap
────────────────────────────────────────────────────────────
memory-consolidate   0.230      ULTRA-FAST    █
best-practice        0.238      ULTRA-FAST    █
module-status        0.272      FAST          █▏
pattern-search       0.278      FAST          █▏
lint                 0.288      FAST          █▎
compliance           0.311      FAST          █▍
deploy-swarm         0.372      NORMAL        █▋
synergy-check        0.380      NORMAL        █▋
build                0.393      NORMAL        █▊
service-health       0.527      VARIABLE      ██▍
                     |    |    |    |    |    |
                     0   0.2  0.4  0.6  0.8  1.0 ms
```

| Tier | Range | Commands | Count |
|------|-------|----------|-------|
| ULTRA-FAST | < 0.25ms | memory-consolidate, best-practice | 2 |
| FAST | 0.25-0.32ms | module-status, pattern-search, lint, compliance | 4 |
| NORMAL | 0.32-0.40ms | deploy-swarm, synergy-check, build | 3 |
| VARIABLE | > 0.40ms | service-health | 1 |

---

## 4. Variance Analysis

| Command | Stdev (ms) | CV (%) | Stability |
|---------|-----------|--------|-----------|
| memory-consolidate | 0.013 | 5.7% | **Rock-solid** |
| best-practice | 0.014 | 5.9% | **Rock-solid** |
| build | 0.043 | 10.9% | Stable |
| pattern-search | 0.067 | 24.1% | Moderate |
| module-status | 0.076 | 27.9% | Moderate |
| synergy-check | 0.084 | 22.1% | Moderate |
| lint | 0.103 | 35.8% | Variable |
| compliance | 0.109 | 35.0% | Variable |
| deploy-swarm | 0.118 | 31.7% | Variable |
| **service-health** | **0.256** | **48.6%** | **Unstable** |

### Stability Buckets

```
Rock-solid (CV < 10%):  memory-consolidate (5.7%), best-practice (5.9%)
Stable (CV 10-20%):     build (10.9%)
Moderate (CV 20-30%):   pattern-search (24.1%), synergy-check (22.1%), module-status (27.9%)
Variable (CV 30-40%):   lint (35.8%), compliance (35.0%), deploy-swarm (31.7%)
Unstable (CV > 40%):    service-health (48.6%)
```

**service-health** has the widest spread: 0.258ms to 0.935ms across iterations. This is likely because M6 does a real service enumeration (querying 11 services) while other commands return canned static data. The 0.935ms outlier in iter3 suggests occasional GC pressure or thread contention in the M6 health aggregator.

---

## 5. Module Performance Map

| Module | Commands | Mean (ms) | Interpretation |
|--------|----------|-----------|----------------|
| **M2** | memory-consolidate, pattern-search | 0.254 | Fastest module — tensor engine is lean |
| **M6** | service-health | 0.527 | Slowest — does real service enumeration |
| **M40** | deploy-swarm | 0.372 | Swarm dispatch — moderate |
| **M44** | best-practice | 0.238 | Oracle — very fast static lookup |
| **M45** | synergy-check, lint, compliance, build, module-status | 0.329 | Workhorse module — 5 commands, consistent |

### Module Load Distribution

```
M45   █████████████████████████████████████████████████  5 commands (50%)
M2    ██████████████████████                             2 commands (20%)
M6    ███████████                                        1 command  (10%)
M40   ███████████                                        1 command  (10%)
M44   ███████████                                        1 command  (10%)
```

M45 handles half of all Nexus commands. Despite the load, it maintains 0.329ms mean — evidence of well-designed static routing.

---

## 6. Aggregate Statistics

| Metric | Value |
|--------|-------|
| **Total commands executed** | 40 (10 × 4 iterations) |
| **Total wall time** | ~13ms |
| **Mean per-command** | 0.329ms |
| **Median per-command** | 0.261ms |
| **P95 (estimated)** | 0.520ms |
| **P99 (estimated)** | 0.935ms |
| **Min observed** | 0.208ms (pattern-search, iter1) |
| **Max observed** | 0.935ms (service-health, iter3) |
| **All success** | 40/40 (100%) |
| **Internal exec (all)** | 0ms (static routes) |

### Throughput Estimate

```
Mean per-command:    0.329ms
Sequential 10-cmd:  ~3.3ms
Theoretical max:    ~3,000 commands/sec (sequential)
With parallelism:   ~10,000+ commands/sec (estimated)
```

---

## 7. Baseline Reference Card

This table serves as the performance baseline for post-V2 comparison:

| Command | Baseline Mean | Baseline P95 | Module | Route | Notes |
|---------|-------------|-------------|--------|-------|-------|
| service-health | 0.527ms | 0.935ms | M6 | static | Real enumeration — expect variance |
| synergy-check | 0.380ms | 0.498ms | M45 | static | V2 may add live synergy calc |
| best-practice | 0.238ms | 0.254ms | M44 | static | Most stable command |
| deploy-swarm | 0.372ms | 0.501ms | M40 | static | V2 may add real dispatch |
| memory-consolidate | 0.230ms | 0.250ms | M2 | static | Most consistent command |
| lint | 0.288ms | 0.466ms | M45 | static | High variance — GC sensitive |
| compliance | 0.311ms | 0.494ms | M45 | static | V2 may add live checks |
| build | 0.393ms | 0.445ms | M45 | static | Stable |
| pattern-search | 0.278ms | 0.387ms | M2 | static | Query param ignored (canned) |
| module-status | 0.272ms | 0.402ms | M45 | static | V2 may add real module polling |

**Post-V2 expectation:** Commands currently on static routes may switch to dynamic routing, increasing latency from ~0.3ms to ~1-5ms as they perform real work. The tradeoff is actual data instead of canned responses.

---

PV2MAIN-HEBBIAN-COMPLETE
